use super::Region;
use vimwiki::{components, LC};

/// Represents a single document divider
#[derive(async_graphql::SimpleObject)]
pub struct Divider {
    /// The segment of the document this divider covers
    region: Region,
}

impl From<LC<components::Divider>> for Divider {
    fn from(lc: LC<components::Divider>) -> Self {
        let region = Region::from(lc.region);
        Self { region }
    }
}
