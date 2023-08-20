mod formatting;
pub mod markdown;

use crate::markdown::{DocumentElement, Paragraph};

#[must_use]
pub fn parse(text: &str) -> Vec<Box<dyn DocumentElement>> {
    let mut elements: Vec<Box<dyn DocumentElement>> = vec![];

    for paragraph in text.split("\n\n") {
        elements.push(Box::new(Paragraph::new(paragraph)));
    }

    elements
}

#[must_use]
pub fn render(elements: Vec<Box<dyn DocumentElement>>) -> String {
    let mut output = String::new();

    for element in elements {
        output += &(element.render() + "\n\n");
    }

    output
}
