use crate::data::{Anchor, ConvertToDatabaseError, Date, Description, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

/// Represents a single document link to a diary entry
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct DiaryLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// Date of diary entry
    #[ent(field, ext(async_graphql(filter_untyped)))]
    date: Date,

    /// Optional description associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    description: Option<Description>,

    /// Optional anchor associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    anchor: Option<Anchor>,
}

impl<'a> TryFrom<Located<v::DiaryLink<'a>>> for DiaryLink {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::DiaryLink<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .date(Date::from(element.date))
                .description(element.description.map(Description::from))
                .anchor(element.anchor.map(Anchor::from))
                .finish_and_commit(),
        )
    }
}
