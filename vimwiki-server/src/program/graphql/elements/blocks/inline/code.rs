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
    fn from(lc: LE<elements::CodeInline>) -> Self {
        Self {
            region: Region::from(lc.region),
            code: lc.element.code,
        }
    }
}
