use bitflags::bitflags;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Format(FormatFlags);

impl Format {
    pub const fn new() -> Self {
        Self(FormatFlags::empty())
    }

    /// Returns the start and end codes required to bring about the required terminal formatting
    /// change. Returns the end codes to terminate any discontinued formatting, followed by the
    /// start codes to bring about the new formatting, all in one `String`
    pub fn get_codes_for_format_change(self, previous_format: Self) -> String {
        let new_format_flags = self.0.difference(previous_format.0);
        let discontinued_format_flags = previous_format.0.difference(self.0);

        Self(discontinued_format_flags).get_end_codes() + &Self(new_format_flags).get_start_codes()
    }

    pub fn toggle_bold(&mut self) {
        self.0.toggle(FormatFlags::BOLD);
    }

    pub fn toggle_italic(&mut self) {
        self.0.toggle(FormatFlags::ITALIC);
    }

    pub fn toggle_strikethrough(&mut self) {
        self.0.toggle(FormatFlags::STRIKETHROUGH);
    }

    fn get_start_codes(self) -> String {
        let mut codes = String::new();
        if self.0.contains(FormatFlags::BOLD) {
            codes += "\x1b[1m";
        }
        if self.0.contains(FormatFlags::ITALIC) {
            codes += "\x1b[3m";
        }
        if self.0.contains(FormatFlags::STRIKETHROUGH) {
            codes += "\x1b[9m";
        }
        codes
    }

    fn get_end_codes(self) -> String {
        let mut codes = String::new();
        if self.0.contains(FormatFlags::BOLD) {
            codes += "\x1b[22m";
        }
        if self.0.contains(FormatFlags::ITALIC) {
            codes += "\x1b[23m";
        }
        if self.0.contains(FormatFlags::STRIKETHROUGH) {
            codes += "\x1b[29m";
        }
        codes
    }

    // these three methods are just used for test formulation, hence the 'allow dead code' annotations

    #[allow(dead_code)]
    pub(crate) fn set_bold(mut self) -> Self {
        self.0.set(FormatFlags::BOLD, true);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn set_italic(mut self) -> Self {
        self.0.set(FormatFlags::ITALIC, true);
        self
    }

    #[allow(dead_code)]
    pub(crate) fn set_strikethrough(mut self) -> Self {
        self.0.set(FormatFlags::STRIKETHROUGH, true);
        self
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct FormatFlags: u8 {
        const BOLD = 1 << 0;
        const ITALIC = 1 << 1;
        const STRIKETHROUGH = 1 << 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod start_and_end_codes {
        use super::*;

        #[test]
        fn blank_format_returns_empty_string_for_start_codes() {
            assert_eq!(String::new(), Format::new().get_start_codes());
        }

        #[test]
        fn check_bold_start_code() {
            let mut format = Format::new();
            format.toggle_bold();
            assert_eq!("\x1b[1m".to_owned(), format.get_start_codes());
        }

        #[test]
        fn check_italic_start_code() {
            let mut format = Format::new();
            format.toggle_italic();
            assert_eq!("\x1b[3m".to_owned(), format.get_start_codes());
        }

        #[test]
        fn check_strikethrough_start_code() {
            let mut format = Format::new();
            format.toggle_strikethrough();
            assert_eq!("\x1b[9m".to_owned(), format.get_start_codes());
        }

        #[test]
        fn blank_format_returns_empty_string_for_end_codes() {
            assert_eq!(String::new(), Format::new().get_end_codes());
        }

        #[test]
        fn check_bold_end_code() {
            let mut format = Format::new();
            format.toggle_bold();
            assert_eq!("\x1b[22m".to_owned(), format.get_end_codes());
        }

        #[test]
        fn check_italic_end_code() {
            let mut format = Format::new();
            format.toggle_italic();
            assert_eq!("\x1b[23m".to_owned(), format.get_end_codes());
        }

        #[test]
        fn check_strikethrough_end_code() {
            let mut format = Format::new();
            format.toggle_strikethrough();
            assert_eq!("\x1b[29m".to_owned(), format.get_end_codes());
        }

        #[test]
        fn combination_of_formats_yeilds_correct_start_codes() {
            let mut format = Format::new();
            format.toggle_strikethrough();
            format.toggle_italic();
            let start_codes = format.get_start_codes();
            assert!(
                start_codes.contains("\x1b[9m"),
                "Start codes returned did not include a strikethrough start code"
            );
            assert!(
                start_codes.contains("\x1b[3m"),
                "Start codes returned did not include an italic start code"
            );
            assert!(
                !start_codes.contains("\x1b[1m"),
                "Start codes returned contained a bold start code when they should not have"
            );
        }
    }

    mod format_change_codes {
        use super::*;

        #[test]
        fn empty_previous_format() {
            let current_format = Format::new().set_bold().set_strikethrough();
            let codes = current_format.get_codes_for_format_change(Format::new());
            assert_eq!(current_format.get_start_codes(), codes,);
        }

        #[test]
        fn empty_current_format() {
            let previous_format = Format::new().set_bold().set_italic();
            let codes = Format::new().get_codes_for_format_change(previous_format);
            assert_eq!(previous_format.get_end_codes(), codes);
        }

        #[test]
        fn no_change_in_format() {
            let format = Format::new().set_italic();
            let codes = format.get_codes_for_format_change(format);
            assert_eq!(String::new(), codes);
        }

        #[test]
        fn both_formats_empty() {
            assert_eq!(
                String::new(),
                Format::new().get_codes_for_format_change(Format::new())
            );
        }

        #[test]
        fn no_format_overlap() {
            let previous_format = Format::new().set_italic();
            let current_format = Format::new().set_bold();
            let codes = current_format.get_codes_for_format_change(previous_format);
            assert_eq!(
                previous_format.get_end_codes() + &current_format.get_start_codes(),
                codes
            );
        }

        #[test]
        fn some_format_overlap_only_removal() {
            let previous_format = Format::new().set_bold().set_strikethrough();
            let current_format = Format::new().set_strikethrough();
            let codes = current_format.get_codes_for_format_change(previous_format);
            assert_eq!(Format::new().set_bold().get_end_codes(), codes);
        }

        #[test]
        fn some_format_overlap_only_addition() {
            let previous_format = Format::new().set_strikethrough();
            let current_format = Format::new().set_bold().set_strikethrough();
            let codes = current_format.get_codes_for_format_change(previous_format);
            assert_eq!(Format::new().set_bold().get_start_codes(), codes);
        }

        #[test]
        fn some_format_overlap_both_addition_and_removal() {
            let previous_format = Format::new().set_bold().set_italic();
            let current_format = Format::new().set_bold().set_strikethrough();
            let codes = current_format.get_codes_for_format_change(previous_format);
            assert_eq!(
                Format::new().set_italic().get_end_codes()
                    + &Format::new().set_strikethrough().get_start_codes(),
                codes
            );
        }
    }
}
