use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
    TryInto,
};
use serde::{Deserialize, Serialize};
use url::Url;

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
    Clone, Debug, From, TryInto, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum Description {
    Text(String),
    URL(Url),
}

impl From<&str> for Description {
    fn from(s: &str) -> Self {
        Self::from(s.to_string())
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
pub struct Anchor {
    components: Vec<String>,
}

impl From<String> for Anchor {
    fn from(s: String) -> Self {
        Self::new(vec![s])
    }
}

impl From<&str> for Anchor {
    fn from(s: &str) -> Self {
        Self::new(vec![s.to_string()])
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Link {
    Wiki(WikiLink),
    InterWiki(InterWikiLink),
    Diary(DiaryLink),
    Raw(RawLink),
    ExternalFile(ExternalFileLink),
    Transclusion(TransclusionLink),
}

impl Link {
    pub fn description(&self) -> Option<&Description> {
        match self {
            Self::Wiki(x) => x.description.as_ref(),
            Self::InterWiki(x) => x.link().description.as_ref(),
            Self::Diary(x) => x.description.as_ref(),
            Self::Raw(_) => None,
            Self::ExternalFile(x) => x.description.as_ref(),
            Self::Transclusion(x) => x.description.as_ref(),
        }
    }

    pub fn anchor(&self) -> Option<&Anchor> {
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
