use crate::data::{GraphqlDatabaseError, Description, Region, Uri};
use entity::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document transclusion link
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct TransclusionLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The URI representing the link's content to pull in
    #[ent(field, ext(async_graphql(filter_untyped)))]
    uri: Uri,

    /// Optional description associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    description: Option<Description>,

    /// Additional properties associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    properties: Vec<Property>,
}

impl fmt::Display for TransclusionLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.uri()),
        }
    }
}

impl<'a> TryFrom<Located<v::TransclusionLink<'a>>> for TransclusionLink {
    type Error = GraphqlDatabaseError;

    fn try_from(
        le: Located<v::TransclusionLink<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .uri(Uri::from(element.uri))
                .description(element.description.map(Description::from))
                .properties(
                    element
                        .properties
                        .into_iter()
                        .map(|(key, value)| Property {
                            key: key.to_string(),
                            value: value.to_string(),
                        })
                        .collect(),
                )
                .finish_and_commit(),
        )
    }
}

#[derive(
    async_graphql::SimpleObject, Clone, Debug, Serialize, Deserialize, ValueLike,
)]
pub struct Property {
    key: String,
    value: String,
}
