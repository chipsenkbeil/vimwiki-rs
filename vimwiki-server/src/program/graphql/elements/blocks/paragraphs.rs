use super::{InlineElement, Region};
use vimwiki::{elements, Located};

#[derive(async_graphql::SimpleObject, Debug)]
pub struct Paragraph {
    region: Region,
    elements: Vec<InlineElement>,
}

impl<'a> From<Located<elements::Paragraph<'a>>> for Paragraph {
    fn from(le: Located<elements::Paragraph<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            elements: element
                .content
                .elements
                .into_iter()
                .map(InlineElement::from)
                .collect(),
        }
    }
}
