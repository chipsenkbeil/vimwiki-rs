use crate::data::{FromVimwikiElement, GraphqlDatabaseError};
use derive_more::Display;
use entity::*;
use entity_async_graphql::*;
use vimwiki::{self as v, Located};

mod common;
pub use common::*;

mod wiki;
pub use wiki::*;

mod interwiki;
pub use interwiki::*;

mod diary;
pub use diary::*;

mod raw;
pub use raw::*;

mod transclusion;
pub use transclusion::*;

#[gql_ent]
#[derive(Debug, Display)]
pub enum Link {
    Wiki(WikiLink),
    IndexedInterWiki(IndexedInterWikiLink),
    NamedInterWiki(NamedInterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    Transclusion(TransclusionLink),
}

impl Link {
    pub fn page_id(&self) -> Id {
        match self {
            Self::Wiki(x) => x.page_id(),
            Self::IndexedInterWiki(x) => x.page_id(),
            Self::NamedInterWiki(x) => x.page_id(),
            Self::Diary(x) => x.page_id(),
            Self::Raw(x) => x.page_id(),
            Self::Transclusion(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Wiki(x) => x.parent_id(),
            Self::IndexedInterWiki(x) => x.parent_id(),
            Self::NamedInterWiki(x) => x.parent_id(),
            Self::Diary(x) => x.parent_id(),
            Self::Raw(x) => x.parent_id(),
            Self::Transclusion(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for Link {
    type Element = Located<v::Link<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.as_inner() {
            v::Link::Wiki { .. } => Self::Wiki(WikiLink::from_vimwiki_element(
                page_id, parent_id, element,
            )?),
            v::Link::IndexedInterWiki { .. } => Self::IndexedInterWiki(
                IndexedInterWikiLink::from_vimwiki_element(
                    page_id, parent_id, element,
                )?,
            ),
            v::Link::NamedInterWiki { .. } => {
                Self::NamedInterWiki(NamedInterWikiLink::from_vimwiki_element(
                    page_id, parent_id, element,
                )?)
            }
            v::Link::Diary { .. } => Self::Diary(
                DiaryLink::from_vimwiki_element(page_id, parent_id, element)?,
            ),
            v::Link::Raw { .. } => Self::Raw(RawLink::from_vimwiki_element(
                page_id, parent_id, element,
            )?),
            v::Link::Transclusion { .. } => {
                Self::Transclusion(TransclusionLink::from_vimwiki_element(
                    page_id, parent_id, element,
                )?)
            }
        })
    }
}
