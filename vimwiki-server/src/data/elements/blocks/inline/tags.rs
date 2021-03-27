use crate::data::{GraphqlDatabaseError, Region};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document inline set of tags
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Tags {
    /// The segment of the document this inline set of tags covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The set of tag names
    names: Vec<String>,
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.names().join(":"))
    }
}

impl<'a> TryFrom<Located<v::Tags<'a>>> for Tags {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Tags<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .names(
                    le.into_inner().0.iter().map(ToString::to_string).collect(),
                )
                .finish_and_commit(),
        )
    }
}
