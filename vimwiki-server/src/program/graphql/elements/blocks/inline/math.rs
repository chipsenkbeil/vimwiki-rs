use super::Region;
use vimwiki::{elements, Located};

/// Represents a single document inline math formula
#[derive(async_graphql::SimpleObject, Debug)]
pub struct MathInline {
    /// The segment of the document this inline math covers
    region: Region,

    /// The raw formula
    formula: String,
}

impl<'a> From<Located<elements::MathInline<'a>>> for MathInline {
    fn from(le: Located<elements::MathInline<'a>>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            formula: le.into_inner().formula.to_string(),
        }
    }
}
