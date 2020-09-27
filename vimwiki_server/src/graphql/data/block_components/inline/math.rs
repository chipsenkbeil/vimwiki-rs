use super::Region;
use vimwiki::{components, LC};

/// Represents a single document inline math formula
#[derive(async_graphql::SimpleObject)]
pub struct MathInline {
    /// The segment of the document this inline math covers
    region: Region,

    /// The raw formula
    formula: String,
}

impl From<LC<components::MathInline>> for MathInline {
    fn from(lc: LC<components::MathInline>) -> Self {
        Self {
            region: Region::from(lc.region),
            formula: lc.component.formula,
        }
    }
}
