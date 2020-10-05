use super::Region;
use vimwiki::{elements, LE};

/// Represents a single document divider
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Divider {
    /// The segment of the document this divider covers
    region: Region,
}

impl From<LE<elements::Divider>> for Divider {
    fn from(lc: LE<elements::Divider>) -> Self {
        let region = Region::from(lc.region);
        Self { region }
    }
}
