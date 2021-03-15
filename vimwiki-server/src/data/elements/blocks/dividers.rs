use crate::data::{ConvertToDatabaseError, Region};

use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Divider {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
}

impl TryFrom<Located<v::Divider>> for Divider {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Divider>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .finish_and_commit(),
        )
    }
}
