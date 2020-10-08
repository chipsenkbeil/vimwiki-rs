use super::Region;
use vimwiki::{elements, LE};

/// Represents a single document inline set of tags
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Tags {
    /// The segment of the document this inline set of tags covers
    region: Region,

    /// The set of tag names
    names: Vec<String>,
}

impl From<LE<elements::Tags>> for Tags {
    fn from(mut lc: LE<elements::Tags>) -> Self {
        Self {
            region: Region::from(lc.region),
            names: lc.element.0.drain(..).map(|x| x.0).collect(),
        }
    }
}