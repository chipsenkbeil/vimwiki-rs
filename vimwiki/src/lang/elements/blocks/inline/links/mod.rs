use crate::StrictEq;
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
    TransclusionLink(Box<TransclusionLink<'a>>),
}

impl Description<'_> {
    pub fn to_borrowed(&self) -> Description {
        use self::Cow::*;

        match self {
            Self::Text(ref x) => Description::from(Cow::Borrowed(match x {
                Borrowed(x) => *x,
                Owned(x) => x.as_str(),
            })),
            Self::TransclusionLink(ref x) => Description::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> Description<'static> {
        match self {
            Self::Text(x) => Description::from(Cow::from(x.into_owned())),
            Self::TransclusionLink(x) => Description::from(x.into_owned()),
        }
    }
}

impl<'a> Description<'a> {
    pub fn into_uri(self) -> Option<URI<'a>> {
        match self {
            Self::TransclusionLink(x) => Some((*x).uri),
            _ => None,
        }
    }
}

impl<'a> From<&'a str> for Description<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(Cow::from(s))
    }
}

impl<'a> From<URI<'a>> for Description<'a> {
    fn from(uri: URI<'a>) -> Self {
        Self::from(TransclusionLink::from(uri))
    }
}

impl<'a> From<TransclusionLink<'a>> for Description<'a> {
    fn from(link: TransclusionLink<'a>) -> Self {
        Self::TransclusionLink(Box::new(link))
    }
}

impl<'a> StrictEq for Description<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
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

impl Anchor<'_> {
    pub fn to_borrowed(&self) -> Anchor {
        use self::Cow::*;

        let elements = self
            .elements
            .iter()
            .map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            })
            .collect();

        Anchor { elements }
    }

    pub fn into_owned(self) -> Anchor<'static> {
        let elements = self
            .elements
            .into_iter()
            .map(|x| Cow::from(x.into_owned()))
            .collect();

        Anchor { elements }
    }
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

impl<'a> StrictEq for Anchor<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
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

impl Link<'_> {
    pub fn to_borrowed(&self) -> Link {
        match self {
            Self::Wiki(x) => Link::from(x.to_borrowed()),
            Self::InterWiki(x) => Link::from(x.to_borrowed()),
            Self::Diary(x) => Link::from(x.to_borrowed()),
            Self::Raw(x) => Link::from(x.to_borrowed()),
            Self::ExternalFile(x) => Link::from(x.to_borrowed()),
            Self::Transclusion(x) => Link::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> Link<'static> {
        match self {
            Self::Wiki(x) => Link::from(x.into_owned()),
            Self::InterWiki(x) => Link::from(x.into_owned()),
            Self::Diary(x) => Link::from(x.into_owned()),
            Self::Raw(x) => Link::from(x.into_owned()),
            Self::ExternalFile(x) => Link::from(x.into_owned()),
            Self::Transclusion(x) => Link::from(x.into_owned()),
        }
    }
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

impl<'a> StrictEq for Link<'a> {
    /// Performs strict_eq check on matching inner variants
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Wiki(x), Self::Wiki(y)) => x.strict_eq(y),
            (Self::InterWiki(x), Self::InterWiki(y)) => x.strict_eq(y),
            (Self::Diary(x), Self::Diary(y)) => x.strict_eq(y),
            (Self::Raw(x), Self::Raw(y)) => x.strict_eq(y),
            (Self::ExternalFile(x), Self::ExternalFile(y)) => x.strict_eq(y),
            (Self::Transclusion(x), Self::Transclusion(y)) => x.strict_eq(y),
            _ => false,
        }
    }
}

/// Helper function to borrow a `URI` similar to our other approaches as the
/// functionality is not available directly in the `uriparse` crate
fn uri_to_borrowed<'a>(uri: &'a URI<'a>) -> URI<'a> {
    let scheme = uri.scheme().as_borrowed();
    let authority = uri.authority().map(|x| x.as_borrowed());
    let query = uri.query().map(|x| x.as_borrowed());
    let fragment = uri.fragment().map(|x| x.as_borrowed());

    // NOTE: Requires an allocation of a new Vec of borrowed elements
    let path = uri.path().to_borrowed();

    URI::from_parts(scheme, authority, path, query, fragment)
        .expect("URI failed to borrow itself")
}
