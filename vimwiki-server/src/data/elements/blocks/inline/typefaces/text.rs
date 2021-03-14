use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents raw text within a single document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Text {
    /// The segment of the document this text covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The text content
    content: String,
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl<'a> TryFrom<Located<v::Text<'a>>> for Text {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Text<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .content(le.into_inner().to_string())
                .finish_and_commit(),
        )
    }
}
