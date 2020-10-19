use super::{InlineElement, Region};
use vimwiki::{elements, Located};

#[derive(Debug)]
pub struct Header {
    region: Region,
    level: i32,
    content: elements::InlineElementContainer<'static>,
    centered: bool,
}

/// Represents a single document comment
#[async_graphql::Object]
impl Header {
    /// The segment of the document this header covers
    async fn region(&self) -> &Region {
        &self.region
    }

    /// The level of the header (ranging 1 to 6)
    async fn level(&self) -> i32 {
        self.level
    }

    /// The content within the header as individual elements
    async fn content_elements(&self) -> Vec<InlineElement> {
        self.content
            .elements
            .iter()
            .map(|e| InlineElement::from(e.clone()))
            .collect()
    }

    /// The content within the header as it would be read by humans
    /// without frills
    async fn content(&self) -> String {
        self.content.to_string()
    }

    /// Whether or not the header is centered
    async fn centered(&self) -> bool {
        self.centered
    }
}

impl<'a> From<Located<elements::Header<'a>>> for Header {
    fn from(le: Located<elements::Header<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            level: element.level as i32,
            content: element.content.into_owned(),
            centered: element.centered,
        }
    }
}
