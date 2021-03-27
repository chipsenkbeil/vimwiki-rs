use crate::data::{GraphqlDatabaseError, Region};
use entity::*;
use std::{convert::TryFrom, fmt};
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

impl fmt::Display for CodeInline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl<'a> TryFrom<Located<v::CodeInline<'a>>> for CodeInline {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::CodeInline<'a>>) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .code(le.into_inner().code.to_string())
                .finish_and_commit(),
        )
    }
}
