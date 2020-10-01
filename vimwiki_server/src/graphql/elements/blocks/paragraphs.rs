use super::{InlineElement, Region};
use vimwiki::{elements, LC};

#[derive(async_graphql::SimpleObject)]
pub struct Paragraph {
    region: Region,
    elements: Vec<InlineElement>,
}

impl From<LC<elements::Paragraph>> for Paragraph {
    fn from(mut lc: LC<elements::Paragraph>) -> Self {
        Self {
            region: Region::from(lc.region),
            elements: lc
                .element
                .content
                .elements
                .drain(..)
                .map(InlineElement::from)
                .collect(),
        }
    }
}
