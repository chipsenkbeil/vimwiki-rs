use crate::data::{GraphqlDatabaseError, Region};
use entity::*;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt, str::FromStr};
use vimwiki::{elements as v, Located};

/// Represents special keywords that have unique syntax highlighting
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Keyword {
    /// The segment of the document this keyword covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The type of keyword
    #[ent(field, ext(async_graphql(filter_untyped)))]
    ty: KeywordType,
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ty)
    }
}

impl TryFrom<Located<v::Keyword>> for Keyword {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Keyword>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .ty(KeywordType::from(le.into_inner()))
                .finish_and_commit(),
        )
    }
}

/// Represents type of special keywords that have unique syntax highlighting
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
pub enum KeywordType {
    TODO,
    DONE,
    STARTED,
    FIXME,
    FIXED,
    XXX,
}

impl fmt::Display for KeywordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::TODO => "todo",
                Self::DONE => "done",
                Self::STARTED => "started",
                Self::FIXME => "fixme",
                Self::FIXED => "fixed",
                Self::XXX => "xxx",
            }
        )
    }
}

impl FromStr for KeywordType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "todo" => Ok(Self::TODO),
            "done" => Ok(Self::DONE),
            "started" => Ok(Self::STARTED),
            "fixme" => Ok(Self::FIXME),
            "fixed" => Ok(Self::FIXED),
            "xxx" => Ok(Self::XXX),
            _ => Err(()),
        }
    }
}

impl From<v::Keyword> for KeywordType {
    fn from(k: v::Keyword) -> Self {
        match k {
            v::Keyword::TODO => KeywordType::TODO,
            v::Keyword::DONE => KeywordType::DONE,
            v::Keyword::STARTED => KeywordType::STARTED,
            v::Keyword::FIXME => KeywordType::FIXME,
            v::Keyword::FIXED => KeywordType::FIXED,
            v::Keyword::XXX => KeywordType::XXX,
        }
    }
}

impl ValueLike for KeywordType {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}
