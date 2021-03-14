use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

/// Represents a single document comment
#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum Comment {
    Line(LineComment),
    MultiLine(MultiLineComment),
}

/// Represents a comment on a single line of a document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct LineComment {
    /// The segment of the document this comment covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The line of content contained within this comment
    line: String,
}

impl<'a> TryFrom<Located<v::LineComment<'a>>> for LineComment {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::LineComment<'a>>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .line(le.into_inner().0.to_string())
                .finish_and_commit(),
        )
    }
}

/// Represents a comment that can potentially cross multiple lines of a document
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct MultiLineComment {
    /// The segment of the document this comment covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The lines of content contained within this comment
    lines: Vec<String>,
}

impl<'a> TryFrom<Located<v::MultiLineComment<'a>>> for MultiLineComment {
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::MultiLineComment<'a>>,
    ) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner().0.iter().map(ToString::to_string).collect(),
                )
                .finish_and_commit(),
        )
    }
}
