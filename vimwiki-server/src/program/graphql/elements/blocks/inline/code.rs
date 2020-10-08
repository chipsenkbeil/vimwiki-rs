use super::Region;
use vimwiki::{elements, LE};

/// Represents a single document inline code
#[derive(async_graphql::SimpleObject, Debug)]
pub struct CodeInline {
    /// The segment of the document this inline code covers
    region: Region,

    /// The raw code
    code: String,
}

impl From<LE<elements::CodeInline>> for CodeInline {
    fn from(le: LE<elements::CodeInline>) -> Self {
        Self {
            region: Region::from(le.region),
            code: le.element.code,
        }
    }
}
