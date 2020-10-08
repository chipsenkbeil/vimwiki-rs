use super::{InlineElement, Region};
use vimwiki::{elements, LE};

#[derive(async_graphql::SimpleObject, Debug)]
pub struct Paragraph {
    region: Region,
    elements: Vec<InlineElement>,
}

impl From<LE<elements::Paragraph>> for Paragraph {
    fn from(mut le: LE<elements::Paragraph>) -> Self {
        Self {
            region: Region::from(le.region),
            elements: le
                .element
                .content
                .elements
                .drain(..)
                .map(InlineElement::from)
                .collect(),
        }
    }
}
