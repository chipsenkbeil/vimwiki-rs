use crate::data::{Anchor, ConvertToDatabaseError, Description, Region};
use entity::*;
use std::{convert::TryFrom, fmt};
use vimwiki::{elements as v, Located};

/// Represents a single document wiki link within another wiki
/// referenced by index
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct IndexedInterWikiLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The index of the wiki this link is associated with
    index: i32,

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

impl fmt::Display for IndexedInterWikiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> TryFrom<Located<v::IndexedInterWikiLink<'a>>>
    for IndexedInterWikiLink
{
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::IndexedInterWikiLink<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .index(element.index as i32)
                .is_dir(element.link.is_path_dir())
                .is_local_anchor(element.link.is_local_anchor())
                .path(element.link.path.to_string_lossy().to_string())
                .description(element.link.description.map(Description::from))
                .anchor(element.link.anchor.map(Anchor::from))
                .finish_and_commit(),
        )
    }
}

/// Represents a single document wiki link within another wiki
/// referenced by name
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct NamedInterWikiLink {
    /// The segment of the document this link covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The name of the wiki this link is associated with
    name: String,

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

impl fmt::Display for NamedInterWikiLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.description().as_ref() {
            Some(desc) => write!(f, "{}", desc),
            None => write!(f, "{}", self.path()),
        }
    }
}

impl<'a> TryFrom<Located<v::NamedInterWikiLink<'a>>> for NamedInterWikiLink {
    type Error = ConvertToDatabaseError;

    fn try_from(
        le: Located<v::NamedInterWikiLink<'a>>,
    ) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();

        ConvertToDatabaseError::wrap(
            Self::build()
                .region(region)
                .name(element.name.to_string())
                .is_dir(element.link.is_path_dir())
                .is_local_anchor(element.link.is_local_anchor())
                .path(element.link.path.to_string_lossy().to_string())
                .description(element.link.description.map(Description::from))
                .anchor(element.link.anchor.map(Anchor::from))
                .finish_and_commit(),
        )
    }
}
