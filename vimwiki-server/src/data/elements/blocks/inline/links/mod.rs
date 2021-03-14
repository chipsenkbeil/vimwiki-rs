use crate::data::ConvertToDatabaseError;
use derive_more::Display;
use entity::*;
use std::convert::TryFrom;
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

impl<'a> TryFrom<Located<v::Link<'a>>> for Link {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Link<'a>>) -> Result<Self, Self::Error> {
        let region = le.region();
        Ok(match le.into_inner() {
            v::Link::Wiki(x) => {
                Self::Wiki(WikiLink::try_from(Located::new(x, region))?)
            }
            v::Link::InterWiki(v::InterWikiLink::Indexed(x)) => {
                Self::IndexedInterWiki(IndexedInterWikiLink::try_from(
                    Located::new(x, region),
                )?)
            }
            v::Link::InterWiki(v::InterWikiLink::Named(x)) => {
                Self::NamedInterWiki(NamedInterWikiLink::try_from(
                    Located::new(x, region),
                )?)
            }
            v::Link::Diary(x) => {
                Self::Diary(DiaryLink::try_from(Located::new(x, region))?)
            }
            v::Link::Raw(x) => {
                Self::Raw(RawLink::try_from(Located::new(x, region))?)
            }
            v::Link::ExternalFile(x) => Self::ExternalFile(
                ExternalFileLink::try_from(Located::new(x, region))?,
            ),
            v::Link::Transclusion(x) => Self::Transclusion(
                TransclusionLink::try_from(Located::new(x, region))?,
            ),
        })
    }
}
