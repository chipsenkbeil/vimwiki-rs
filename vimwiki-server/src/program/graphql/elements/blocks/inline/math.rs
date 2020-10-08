use super::Region;
use vimwiki::{elements, LE};

/// Represents a single document inline math formula
#[derive(async_graphql::SimpleObject, Debug)]
pub struct MathInline {
    /// The segment of the document this inline math covers
    region: Region,

    /// The raw formula
    formula: String,
}

impl From<LE<elements::MathInline>> for MathInline {
    fn from(le: LE<elements::MathInline>) -> Self {
        Self {
            region: Region::from(le.region),
            formula: le.element.formula,
        }
    }
}
