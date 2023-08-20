use crate::formatting::Format;

pub trait DocumentElement {
    fn render(&self) -> String;
}

pub struct Paragraph {
    render_slices: Vec<(String, Format)>,
}

impl Paragraph {
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut render_slices = Vec::new();
        let mut current_format = Format::new();

        let mut current_slice_start = 0_usize;

        let mut char_indices = text.char_indices();

        while let Some((char_index, current_char)) = char_indices.next() {
            match current_char {
                '\\' => {
                    // '\': character escape
                    let (next_char_index, _) = char_indices
                        .next()
                        // doing this allows the handling of the case of EOL after a backslash without any special cases
                        // it does it by tricking the final slice pusher outside the loop into thinking
                        // that there is nothing more to add (otherwise it would add the backslash at the end)
                        .unwrap_or((char_index + 1, '\\'));
                    #[allow(clippy::indexing_slicing)]
                    render_slices.push((
                        text[current_slice_start..char_index].to_owned(),
                        current_format,
                    ));
                    current_slice_start = next_char_index;
                }
                '\n' => {
                    // '\n': newline (replace with space)
                    #[allow(clippy::indexing_slicing)]
                    let slice = text[current_slice_start..char_index].to_owned() + " ";
                    render_slices.push((slice, current_format));
                    current_slice_start = char_index + 1;
                }
                '*' => {
                    // bold or italic
                    // both cases require the current slice to be pushed
                    #[allow(clippy::indexing_slicing)]
                    render_slices.push((
                        text[current_slice_start..char_index].to_owned(),
                        current_format,
                    ));
                    if let Some((next_char_index, '*')) = char_indices.next() {
                        // '**': toggle the bold format
                        current_slice_start = next_char_index + 1; // leapfrog the second asterisk
                        current_format.toggle_bold();
                    } else {
                        // '*': toggle the italic format
                        current_slice_start = char_index + 1; // leapfrog the asterisk
                        current_format.toggle_italic();
                    }
                }
                '~' => {
                    // strikethrough or just a tilde
                    if let Some((next_char_index, '~')) = char_indices.next() {
                        // '~~': toggle the strikethrough format
                        #[allow(clippy::indexing_slicing)]
                        render_slices.push((
                            text[current_slice_start..char_index].to_owned(),
                            current_format,
                        ));
                        current_slice_start = next_char_index + 1; // leapfrog the second tilde
                        current_format.toggle_strikethrough();
                    }
                }
                _other_char => (),
            }
        }

        // push the final slice to the render components, provided that there is something to push
        // in the first place
        if current_slice_start != text.len() {
            #[allow(clippy::indexing_slicing)]
            render_slices.push((text[current_slice_start..].to_owned(), current_format));
        }

        // remove any empty slices
        render_slices.retain(|elem| !elem.0.is_empty());

        Self { render_slices }
    }
}

impl DocumentElement for Paragraph {
    fn render(&self) -> String {
        let mut render = String::new();
        let mut previous_format = Format::new();

        for (slice, format) in &self.render_slices {
            render += &(format.get_codes_for_format_change(previous_format) + slice);
            previous_format = *format;
        }
        // close up any hanging formatting
        render += &Format::new().get_codes_for_format_change(previous_format);

        render
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod paragraph_parsing {
        use super::*;

        #[test]
        fn escaped_character_mid_paragraph() {
            let paragraph = Paragraph::new(r"lorem ipsum \\dolor sit amet");
            assert_eq!(
                vec![
                    ("lorem ipsum ".to_owned(), Format::new()),
                    (r"\dolor sit amet".to_owned(), Format::new())
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn backslash_at_the_end_of_a_paragraph() {
            let paragraph = Paragraph::new(r"lorem ipsum\");
            assert_eq!(
                vec![("lorem ipsum".to_owned(), Format::new())],
                paragraph.render_slices
            );
        }

        #[test]
        fn newline_becomes_space() {
            let paragraph = Paragraph::new("lorem\nipsum");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new())
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn bold_at_start_of_paragraph() {
            let paragraph = Paragraph::new(r"**lorem** ipsum");
            assert_eq!(
                vec![
                    ("lorem".to_owned(), Format::new().set_bold()),
                    (" ipsum".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn bold_in_the_middle_of_paragraph() {
            let paragraph = Paragraph::new("lorem **ipsum** dolor");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_bold()),
                    (" dolor".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn bold_at_end_of_paragraph() {
            let paragraph = Paragraph::new("lorem **ipsum**");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_bold()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn italic_at_start_of_paragraph() {
            let paragraph = Paragraph::new("*lorem* ipsum");
            assert_eq!(
                vec![
                    ("lorem".to_owned(), Format::new().set_italic()),
                    (" ipsum".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn italic_in_the_middle_of_paragraph() {
            let paragraph = Paragraph::new("lorem *ipsum* dolor");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_italic()),
                    (" dolor".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn italic_at_end_of_paragraph() {
            let paragraph = Paragraph::new("lorem *ipsum*");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_italic()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn italic_with_asterisks_surrounded_by_spaces() {
            let paragraph = Paragraph::new("lorem * ipsum * dolor");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    (" ipsum ".to_owned(), Format::new().set_italic()),
                    (" dolor".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn strikethrough_at_start_of_paragraph() {
            let paragraph = Paragraph::new(r"~~lorem~~ ipsum");
            assert_eq!(
                vec![
                    ("lorem".to_owned(), Format::new().set_strikethrough()),
                    (" ipsum".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn strikethrough_in_the_middle_of_paragraph() {
            let paragraph = Paragraph::new(r"lorem ~~ipsum~~ dolor");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_strikethrough()),
                    (" dolor".to_owned(), Format::new()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn strikethrough_at_end_of_paragraph() {
            let paragraph = Paragraph::new(r"lorem ~~ipsum~~");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_strikethrough()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn just_a_ilde() {
            let paragraph = Paragraph::new("~lorem~ ipsum ~ dolor ~sit amet~");
            assert_eq!(
                vec![("~lorem~ ipsum ~ dolor ~sit amet~".to_owned(), Format::new()),],
                paragraph.render_slices
            );
        }

        #[test]
        fn two_overlapping_formats() {
            let paragraph = Paragraph::new(r"**lorem *ipsum** dolor*");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new().set_bold()),
                    ("ipsum".to_owned(), Format::new().set_bold().set_italic()),
                    (" dolor".to_owned(), Format::new().set_italic()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn enclosed_formats() {
            let paragraph = Paragraph::new(r"**lorem *ipsum* dolor**");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new().set_bold()),
                    ("ipsum".to_owned(), Format::new().set_bold().set_italic()),
                    (" dolor".to_owned(), Format::new().set_bold()),
                ],
                paragraph.render_slices
            );
        }

        #[test]
        fn enclosed_and_overlapping_formats() {
            let paragraph = Paragraph::new(r"**lorem *ipsum* ~~dolor** sit amet~~");
            assert_eq!(
                vec![
                    ("lorem ".to_owned(), Format::new().set_bold()),
                    ("ipsum".to_owned(), Format::new().set_bold().set_italic()),
                    (" ".to_owned(), Format::new().set_bold()),
                    (
                        "dolor".to_owned(),
                        Format::new().set_bold().set_strikethrough()
                    ),
                    (" sit amet".to_owned(), Format::new().set_strikethrough()),
                ],
                paragraph.render_slices
            );
        }
    }

    mod paragraph_rendering {
        use super::*;

        #[test]
        fn escaped_character_mid_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ipsum ".to_owned(), Format::new()),
                    (r"\dolor sit amet".to_owned(), Format::new()),
                ],
            };
            assert_eq!(
                r"lorem ipsum \dolor sit amet".to_owned(),
                paragraph.render(),
            );
        }

        #[test]
        fn bold_at_start_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem".to_owned(), Format::new().set_bold()),
                    (" ipsum".to_owned(), Format::new()),
                ],
            };
            assert_eq!("\x1b[1mlorem\x1b[22m ipsum".to_owned(), paragraph.render());
        }

        #[test]
        fn bold_in_the_middle_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_bold()),
                    (" dolor".to_owned(), Format::new()),
                ],
            };
            assert_eq!(
                "lorem \x1b[1mipsum\x1b[22m dolor".to_owned(),
                paragraph.render()
            );
        }

        #[test]
        fn bold_at_end_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_bold()),
                ],
            };
            assert_eq!("lorem \x1b[1mipsum\x1b[22m".to_owned(), paragraph.render());
        }

        #[test]
        fn italic_at_start_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem".to_owned(), Format::new().set_italic()),
                    (" ipsum".to_owned(), Format::new()),
                ],
            };
            assert_eq!("\x1b[3mlorem\x1b[23m ipsum".to_owned(), paragraph.render());
        }

        #[test]
        fn italic_in_the_middle_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_italic()),
                    (" dolor".to_owned(), Format::new()),
                ],
            };
            assert_eq!(
                "lorem \x1b[3mipsum\x1b[23m dolor".to_owned(),
                paragraph.render()
            );
        }

        #[test]
        fn italic_at_end_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_italic()),
                ],
            };
            assert_eq!("lorem \x1b[3mipsum\x1b[23m".to_owned(), paragraph.render());
        }

        #[test]
        fn italic_with_asterisks_surrounded_by_spaces() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    (" ipsum ".to_owned(), Format::new().set_italic()),
                    (" dolor".to_owned(), Format::new()),
                ],
            };
            assert_eq!(
                "lorem \x1b[3m ipsum \x1b[23m dolor".to_owned(),
                paragraph.render()
            );
        }

        #[test]
        fn strikethrough_at_start_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem".to_owned(), Format::new().set_strikethrough()),
                    (" ipsum".to_owned(), Format::new()),
                ],
            };
            assert_eq!("\x1b[9mlorem\x1b[29m ipsum".to_owned(), paragraph.render());
        }

        #[test]
        fn strikethrough_in_the_middle_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_strikethrough()),
                    (" dolor".to_owned(), Format::new()),
                ],
            };
            assert_eq!(
                "lorem \x1b[9mipsum\x1b[29m dolor".to_owned(),
                paragraph.render()
            );
        }

        #[test]
        fn strikethrough_at_end_of_paragraph() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new()),
                    ("ipsum".to_owned(), Format::new().set_strikethrough()),
                ],
            };
            assert_eq!("lorem \x1b[9mipsum\x1b[29m".to_owned(), paragraph.render());
        }

        #[test]
        fn two_overlapping_formats() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new().set_bold()),
                    ("ipsum".to_owned(), Format::new().set_bold().set_italic()),
                    (" dolor".to_owned(), Format::new().set_italic()),
                ],
            };
            assert_eq!(
                "\x1b[1mlorem \x1b[3mipsum\x1b[22m dolor\x1b[23m".to_owned(),
                paragraph.render()
            );
        }

        #[test]
        fn enclosed_formats() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new().set_bold()),
                    ("ipsum".to_owned(), Format::new().set_bold().set_italic()),
                    (" dolor".to_owned(), Format::new().set_bold()),
                ],
            };
            assert_eq!(
                "\x1b[1mlorem \x1b[3mipsum\x1b[23m dolor\x1b[22m".to_owned(),
                paragraph.render()
            );
        }

        #[test]
        fn enclosed_and_overlapping_formats() {
            let paragraph = Paragraph {
                render_slices: vec![
                    ("lorem ".to_owned(), Format::new().set_bold()),
                    ("ipsum".to_owned(), Format::new().set_bold().set_italic()),
                    (" ".to_owned(), Format::new().set_bold()),
                    (
                        "dolor".to_owned(),
                        Format::new().set_bold().set_strikethrough(),
                    ),
                    (" sit amet".to_owned(), Format::new().set_strikethrough()),
                ],
            };
            assert_eq!(
                "\x1b[1mlorem \x1b[3mipsum\x1b[23m \x1b[9mdolor\x1b[22m sit amet\x1b[29m"
                    .to_owned(),
                paragraph.render()
            );
        }
    }
}
