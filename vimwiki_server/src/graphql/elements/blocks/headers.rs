use super::Region;
use vimwiki::{elements, LC};

/// Represents a single document comment
#[derive(async_graphql::SimpleObject)]
pub struct Header {
    /// The segment of the document this header covers
    region: Region,

    /// The level of the header (ranging 1 to 6)
    level: i32,

    /// The text within the header
    text: String,

    /// Whether or not the header is centered
    centered: bool,
}

impl From<LC<elements::Header>> for Header {
    fn from(lc: LC<elements::Header>) -> Self {
        let region = Region::from(lc.region);
        Self {
            region,
            level: lc.element.level as i32,
            text: lc.element.text,
            centered: lc.element.centered,
        }
    }
}
