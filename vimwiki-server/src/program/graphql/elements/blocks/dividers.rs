use super::Region;
use vimwiki::{elements, Located};

/// Represents a single document divider
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Divider {
    /// The segment of the document this divider covers
    region: Region,
}

impl From<Located<elements::Divider>> for Divider {
    fn from(le: Located<elements::Divider>) -> Self {
        let region = Region::from(le.region());
        Self { region }
    }
}
