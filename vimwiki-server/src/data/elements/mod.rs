mod blocks;
pub use blocks::*;

mod utils;
pub use utils::*;

use derive_more::Display;
use entity::*;
use std::convert::TryFrom;

#[derive(Display)]
pub enum ConvertToDatabaseError {
    Database(DatabaseError),
    Builder(Box<dyn std::error::Error>),
}

impl ConvertToDatabaseError {
    pub fn wrap<T, E: std::error::Error + 'static>(
        x: Result<Result<T, DatabaseError>, E>,
    ) -> Result<T, Self> {
        match Self::try_from(x) {
            Ok(x) => Err(x),
            Err(x) => Ok(x),
        }
    }
}

impl std::error::Error for ConvertToDatabaseError {}

impl<T, E> TryFrom<Result<Result<T, DatabaseError>, E>>
    for ConvertToDatabaseError
where
    E: std::error::Error + 'static,
{
    type Error = T;

    fn try_from(
        x: Result<Result<T, DatabaseError>, E>,
    ) -> Result<Self, Self::Error> {
        match x {
            Ok(x) => match x {
                Ok(x) => Err(x),
                Err(x) => Ok(ConvertToDatabaseError::Database(x)),
            },
            Err(x) => Ok(ConvertToDatabaseError::Builder(Box::new(x))),
        }
    }
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Page {
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    elements: Vec<BlockElement>,
}

#[simple_ent]
#[derive(async_graphql::Union)]
pub enum Element {
    #[graphql(flatten)]
    Block(BlockElement),

    #[graphql(flatten)]
    Inline(InlineElement),

    #[graphql(flatten)]
    InlineBlock(InlineBlockElement),
}
