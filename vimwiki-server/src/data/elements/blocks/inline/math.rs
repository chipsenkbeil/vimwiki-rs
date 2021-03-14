use crate::data::{ConvertToDatabaseError, Region};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document inline math formula
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct MathInline {
    /// The segment of the document this inline math covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The raw formula
    formula: String,
}

impl fmt::Display for MathInline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formula())
    }
}

impl<'a> TryFrom<Located<v::MathInline<'a>>> for MathInline {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::MathInline<'a>>) -> Result<Self, Self::Error> {
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(Region::from(le.region()))
                .formula(le.into_inner().formula.to_string())
                .finish_and_commit(),
        )
    }
}
