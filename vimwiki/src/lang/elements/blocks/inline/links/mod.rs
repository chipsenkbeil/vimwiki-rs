use derive_more::{
    Constructor, Deref, DerefMut, Display, From, Index, IndexMut, Into,
    IntoIterator, TryInto,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};
use uriparse::URI;

mod diary;
pub use diary::DiaryLink;

mod external;
pub use external::{ExternalFileLink, ExternalFileLinkScheme};

mod interwiki;
pub use interwiki::{IndexedInterWikiLink, InterWikiLink, NamedInterWikiLink};

mod raw;
pub use raw::RawLink;

mod transclusion;
pub use transclusion::TransclusionLink;

mod wiki;
pub use wiki::WikiLink;

/// Represents a description for a link
#[derive(
    Clone,
    Debug,
    Display,
    From,
    TryInto,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum Description<'a> {
    Text(Cow<'a, str>),
    URI(URI<'a>),
}

impl<'a> From<&'a str> for Description<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(Cow::from(s))
    }
}

/// Represents an anchor
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    From,
    Index,
    IndexMut,
    Into,
    IntoIterator,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Anchor<'a> {
    pub elements: Vec<Cow<'a, str>>,
}

impl<'a> fmt::Display for Anchor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.elements.is_empty() {
            Ok(())
        } else {
            write!(f, "#{}", self.elements.join("#"))
        }
    }
}

impl From<String> for Anchor<'static> {
    fn from(s: String) -> Self {
        Self::new(vec![Cow::from(s)])
    }
}

impl<'a> From<&'a str> for Anchor<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(vec![Cow::from(s)])
    }
}

#[derive(
    Clone, Debug, Display, From, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum Link<'a> {
    Wiki(WikiLink<'a>),
    InterWiki(InterWikiLink<'a>),
    Diary(DiaryLink<'a>),
    Raw(RawLink<'a>),
    ExternalFile(ExternalFileLink<'a>),
    Transclusion(TransclusionLink<'a>),
}

impl<'a> Link<'a> {
    pub fn description(&self) -> Option<&Description<'a>> {
        match self {
            Self::Wiki(x) => x.description.as_ref(),
            Self::InterWiki(x) => x.link().description.as_ref(),
            Self::Diary(x) => x.description.as_ref(),
            Self::Raw(_) => None,
            Self::ExternalFile(x) => x.description.as_ref(),
            Self::Transclusion(x) => x.description.as_ref(),
        }
    }

    pub fn anchor(&self) -> Option<&Anchor<'a>> {
        match self {
            Self::Wiki(x) => x.anchor.as_ref(),
            Self::InterWiki(x) => x.link().anchor.as_ref(),
            Self::Diary(x) => x.anchor.as_ref(),
            Self::Raw(_) => None,
            Self::ExternalFile(_) => None,
            Self::Transclusion(_) => None,
        }
    }
}
