use super::Region;
use vimwiki::{elements, LC};

/// Represents a single document divider
#[derive(async_graphql::SimpleObject)]
pub struct Divider {
    /// The segment of the document this divider covers
    region: Region,
}

impl From<LC<elements::Divider>> for Divider {
    fn from(lc: LC<elements::Divider>) -> Self {
        let region = Region::from(lc.region);
        Self { region }
    }
}
