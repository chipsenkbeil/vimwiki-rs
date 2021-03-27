use crate::data::{GraphqlDatabaseError, Region};

use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Blockquote {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
    lines: Vec<String>,
}

impl<'a> TryFrom<Located<v::Blockquote<'a>>> for Blockquote {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Blockquote<'a>>) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .lines(
                    le.into_inner()
                        .lines
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                )
                .finish_and_commit(),
        )
    }
}
