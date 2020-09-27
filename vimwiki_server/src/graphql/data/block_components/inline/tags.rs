use super::Region;
use vimwiki::{components, LC};

/// Represents a single document inline set of tags
#[derive(async_graphql::SimpleObject)]
pub struct Tags {
    /// The segment of the document this inline set of tags covers
    region: Region,

    /// The set of tag names
    names: Vec<String>,
}

impl From<LC<components::Tags>> for Tags {
    fn from(mut lc: LC<components::Tags>) -> Self {
        Self {
            region: Region::from(lc.region),
            names: lc.component.0.drain(..).map(|x| x.0).collect(),
        }
    }
}
