use crate::data::{FromVimwikiElement, GraphqlDatabaseError};
use derive_more::Display;
use entity::*;
use vimwiki::{elements as v, Located};

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

mod external;
pub use external::*;

mod transclusion;
pub use transclusion::*;

#[simple_ent]
#[derive(async_graphql::Union, Debug, Display)]
pub enum Link {
    Wiki(WikiLink),
    IndexedInterWiki(IndexedInterWikiLink),
    NamedInterWiki(NamedInterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    ExternalFile(ExternalFileLink),
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
            Self::ExternalFile(x) => x.page_id(),
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
            Self::ExternalFile(x) => x.parent_id(),
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
        Ok(match element.into_inner() {
            v::Link::Wiki(x) => Self::Wiki(WikiLink::from_vimwiki_element(
                page_id,
                parent_id,
                Located::new(x, region),
            )?),
            v::Link::InterWiki(v::InterWikiLink::Indexed(x)) => {
                Self::IndexedInterWiki(
                    IndexedInterWikiLink::from_vimwiki_element(
                        page_id,
                        parent_id,
                        Located::new(x, region),
                    )?,
                )
            }
            v::Link::InterWiki(v::InterWikiLink::Named(x)) => {
                Self::NamedInterWiki(NamedInterWikiLink::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::Link::Diary(x) => Self::Diary(DiaryLink::from_vimwiki_element(
                page_id,
                parent_id,
                Located::new(x, region),
            )?),
            v::Link::Raw(x) => Self::Raw(RawLink::from_vimwiki_element(
                page_id,
                parent_id,
                Located::new(x, region),
            )?),
            v::Link::ExternalFile(x) => {
                Self::ExternalFile(ExternalFileLink::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::Link::Transclusion(x) => {
                Self::Transclusion(TransclusionLink::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}
