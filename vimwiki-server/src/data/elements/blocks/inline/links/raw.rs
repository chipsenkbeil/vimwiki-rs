use crate::data::{GraphqlDatabaseError, Region, Uri};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document link formed from a raw URI
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct RawLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The URI representing the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    uri: Uri,
}

impl fmt::Display for RawLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri())
    }
}

impl<'a> TryFrom<Located<v::RawLink<'a>>> for RawLink {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::RawLink<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .uri(Uri::from(le.into_inner().uri))
                .finish_and_commit(),
        )
    }
}
