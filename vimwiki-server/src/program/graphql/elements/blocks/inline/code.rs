use super::Region;
use vimwiki::{elements, Located};

/// Represents a single document inline code
#[derive(async_graphql::SimpleObject, Debug)]
pub struct CodeInline {
    /// The segment of the document this inline code covers
    region: Region,

    /// The raw code
    code: String,
}

impl<'a> From<Located<elements::CodeInline<'a>>> for CodeInline {
    fn from(le: Located<elements::CodeInline<'a>>) -> Self {
        let region = Region::from(le.region());
        Self {
            region,
            code: le.into_inner().code.to_string(),
        }
    }
}
