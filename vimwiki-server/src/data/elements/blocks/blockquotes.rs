use crate::data::{ConvertToDatabaseError, Region};

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
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Blockquote<'a>>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
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
