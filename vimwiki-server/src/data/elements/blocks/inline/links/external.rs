use crate::data::{GraphqlDatabaseError, Description, Region};
use entity::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document link to an external file
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ExternalFileLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// Scheme associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    scheme: ExternalFileLinkScheme,

    /// Path to the local file
    path: String,

    /// Optional description associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    description: Option<Description>,
}

impl fmt::Display for ExternalFileLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> TryFrom<Located<v::ExternalFileLink<'a>>> for ExternalFileLink {
    type Error = GraphqlDatabaseError;

    fn try_from(
        le: Located<v::ExternalFileLink<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .scheme(ExternalFileLinkScheme::from(element.scheme))
                .path(element.path.to_string_lossy().to_string())
                .description(element.description.map(Description::from))
                .finish_and_commit(),
        )
    }
}

/// Represents the scheme associated with an external file link
#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub enum ExternalFileLinkScheme {
    Local,
    File,
    Absolute,
}

impl From<v::ExternalFileLinkScheme> for ExternalFileLinkScheme {
    fn from(s: v::ExternalFileLinkScheme) -> Self {
        match s {
            v::ExternalFileLinkScheme::Local => Self::Local,
            v::ExternalFileLinkScheme::File => Self::File,
            v::ExternalFileLinkScheme::Absolute => Self::Absolute,
        }
    }
}

impl ValueLike for ExternalFileLinkScheme {
    fn into_value(self) -> Value {
        match self {
            Self::Local => Value::from("local"),
            Self::File => Value::from("file"),
            Self::Absolute => Value::from("absolute"),
        }
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => match x.as_str() {
                "local" => Ok(Self::Local),
                "file" => Ok(Self::File),
                "absolute" => Ok(Self::Absolute),
                _ => Err(Value::Text(x)),
            },
            x => Err(x),
        }
    }
}
