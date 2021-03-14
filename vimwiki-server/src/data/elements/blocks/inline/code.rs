use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

/// Represents a single document inline code
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct CodeInline {
    /// The segment of the document this inline code covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The raw code
    code: String,
}

impl<'a> TryFrom<Located<v::CodeInline<'a>>> for CodeInline {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::CodeInline<'a>>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .code(le.into_inner().code.to_string())
                .finish_and_commit(),
        )
    }
}
