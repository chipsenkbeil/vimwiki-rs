use crate::data::{GraphqlDatabaseError, Region};
use derive_more::Display;
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document comment
#[simple_ent]
#[derive(async_graphql::Union, Debug, Display)]
pub enum Comment {
    Line(LineComment),
    MultiLine(MultiLineComment),
}

impl<'a> TryFrom<Located<v::Comment<'a>>> for Comment {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Comment<'a>>) -> Result<Self, Self::Error> {
        let region = le.region();
        Ok(match le.into_inner() {
            v::Comment::Line(x) => {
                Self::Line(LineComment::try_from(Located::new(x, region))?)
            }
            v::Comment::MultiLine(x) => Self::MultiLine(
                MultiLineComment::try_from(Located::new(x, region))?,
            ),
        })
    }
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

impl fmt::Display for LineComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.line())
    }
}

impl<'a> TryFrom<Located<v::LineComment<'a>>> for LineComment {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::LineComment<'a>>) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
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

impl fmt::Display for MultiLineComment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self.lines().iter() {
            write!(f, "{}", line)?;
        }
        Ok(())
    }
}

impl<'a> TryFrom<Located<v::MultiLineComment<'a>>> for MultiLineComment {
    type Error = GraphqlDatabaseError;

    fn try_from(
        le: Located<v::MultiLineComment<'a>>,
    ) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner().0.iter().map(ToString::to_string).collect(),
                )
                .finish_and_commit(),
        )
    }
}
