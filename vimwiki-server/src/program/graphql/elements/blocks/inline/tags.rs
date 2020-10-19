use super::Region;
use vimwiki::{elements, Located};

/// Represents a single document inline set of tags
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Tags {
    /// The segment of the document this inline set of tags covers
    region: Region,

    /// The set of tag names
    names: Vec<String>,
}

impl<'a> From<Located<elements::Tags<'a>>> for Tags {
    fn from(le: Located<elements::Tags<'a>>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            names: le.into_inner().0.iter().map(ToString::to_string).collect(),
        }
    }
}
