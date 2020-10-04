use super::{InlineElement, Region};
use vimwiki::{elements, LE};

#[derive(Debug)]
pub struct Header {
    region: Region,
    level: i32,
    content: elements::InlineElementContainer,
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

impl From<LE<elements::Header>> for Header {
    fn from(le: LE<elements::Header>) -> Self {
        let region = Region::from(le.region);
        Self {
            region,
            level: le.element.level as i32,
            content: le.element.content,
            centered: le.element.centered,
        }
    }
}
