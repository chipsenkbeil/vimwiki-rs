use crate::data::{GraphqlDatabaseError, Region};
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
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Divider>) -> Result<Self, Self::Error> {
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .finish_and_commit(),
        )
    }
}
