use crate::data::{Anchor, ConvertToDatabaseError, Description, Region};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document wiki link
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct WikiLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// Whether or not the link connects to a directory
    is_dir: bool,

    /// Whether or not the link is just an anchor to a location
    /// within the current document
    is_local_anchor: bool,

    /// The path the link connects to
    path: String,

    /// Optional description associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    description: Option<Description>,

    /// Optional anchor associated with the link
    #[ent(field, ext(async_graphql(filter_untyped)))]
    anchor: Option<Anchor>,
}

impl fmt::Display for WikiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> TryFrom<Located<v::WikiLink<'a>>> for WikiLink {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::WikiLink<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();
        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .is_dir(element.is_path_dir())
                .is_local_anchor(element.is_local_anchor())
                .path(element.path.to_string_lossy().to_string())
                .description(element.description.map(Description::from))
                .anchor(element.anchor.map(Anchor::from))
                .finish_and_commit(),
        )
    }
}
