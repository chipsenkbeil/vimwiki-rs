use crate::data::{ConvertToDatabaseError, Date, Region};
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum Placeholder {
    Title(PlaceholderTitle),
    NoHtml(PlaceholderNoHtml),
    Template(PlaceholderTemplate),
    Date(PlaceholderDate),
    Other(PlaceholderOther),
}

impl<'a> TryFrom<Located<v::Placeholder<'a>>> for Placeholder {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Placeholder<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        match le.into_inner() {
            v::Placeholder::Title(title) => ConvertToDatabaseError::wrap(
                PlaceholderTitle::build()
                    .region(region)
                    .title(title.to_string())
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::NoHtml => ConvertToDatabaseError::wrap(
                PlaceholderNoHtml::build()
                    .region(region)
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::Template(template) => ConvertToDatabaseError::wrap(
                PlaceholderTemplate::build()
                    .region(region)
                    .template(template.to_string())
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::Date(date) => ConvertToDatabaseError::wrap(
                PlaceholderDate::build()
                    .region(region)
                    .date(Date::from(date))
                    .finish_and_commit(),
            )
            .map(Self::from),
            v::Placeholder::Other { name, value } => {
                ConvertToDatabaseError::wrap(
                    PlaceholderOther::build()
                        .region(region)
                        .name(name.to_string())
                        .value(value.to_string())
                        .finish_and_commit(),
                )
                .map(Self::from)
            }
        }
    }
}

/// Represents a single document title placeholder
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct PlaceholderTitle {
    /// The segment of the document this placeholder covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The title associated with this placeholder
    title: String,
}

/// Represents a single document nohtml placeholder
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct PlaceholderNoHtml {
    /// The segment of the document this placeholder covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
}

/// Represents a single document template placeholder
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct PlaceholderTemplate {
    /// The segment of the document this placeholder covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The template associated with this placeholder
    template: String,
}

/// Represents a single document date placeholder
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct PlaceholderDate {
    /// The segment of the document this placeholder covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The date associated with this placeholder
    #[ent(field, ext(async_graphql(filter_untyped)))]
    date: Date,
}

/// Represents a single document other placeholder
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct PlaceholderOther {
    /// The segment of the document this placeholder covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The name associated with this placeholder
    name: String,

    /// The value associated with this placeholder
    value: String,
}
