use super::{Anchor, Description, WikiLink};
use crate::StrictEq;
use derive_more::{Constructor, Display, From};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, path::Path};

/// Represents a link to a file or directory in another wiki
#[derive(
    Clone, Debug, Display, From, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub enum InterWikiLink<'a> {
    Indexed(IndexedInterWikiLink<'a>),
    Named(NamedInterWikiLink<'a>),
}

impl InterWikiLink<'_> {
    pub fn to_borrowed(&self) -> InterWikiLink {
        match self {
            Self::Indexed(x) => InterWikiLink::from(x.to_borrowed()),
            Self::Named(x) => InterWikiLink::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> InterWikiLink<'static> {
        match self {
            Self::Indexed(x) => InterWikiLink::from(x.into_owned()),
            Self::Named(x) => InterWikiLink::from(x.into_owned()),
        }
    }
}

impl<'a> InterWikiLink<'a> {
    /// Returns the index associated with this interwiki link if it is an
    /// indexed interwiki link
    pub fn index(&self) -> Option<u32> {
        match self {
            Self::Indexed(x) => Some(x.index),
            _ => None,
        }
    }

    /// Returns the name associated with this interwiki link if it is a
    /// named interwiki link
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Named(x) => Some(&x.name),
            _ => None,
        }
    }

    pub fn link(&self) -> &WikiLink<'a> {
        match self {
            Self::Indexed(x) => &x.link,
            Self::Named(x) => &x.link,
        }
    }

    pub fn path(&self) -> &Path {
        &self.link().path
    }

    pub fn description(&self) -> Option<&Description> {
        self.link().description.as_ref()
    }

    pub fn anchor(&self) -> Option<&Anchor> {
        self.link().anchor.as_ref()
    }
}

impl<'a> StrictEq for InterWikiLink<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Represents a link to a file or directory in another wiki specified by
/// an index that maps to the g:vimwiki_list
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct IndexedInterWikiLink<'a> {
    pub index: u32,
    pub link: WikiLink<'a>,
}

impl IndexedInterWikiLink<'_> {
    pub fn to_borrowed(&self) -> IndexedInterWikiLink {
        IndexedInterWikiLink {
            index: self.index,
            link: self.link.to_borrowed(),
        }
    }

    pub fn into_owned(self) -> IndexedInterWikiLink<'static> {
        IndexedInterWikiLink {
            index: self.index,
            link: self.link.into_owned(),
        }
    }
}

impl<'a> fmt::Display for IndexedInterWikiLink<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.link)
    }
}

impl<'a> From<(u32, WikiLink<'a>)> for IndexedInterWikiLink<'a> {
    fn from((index, link): (u32, WikiLink<'a>)) -> Self {
        Self::new(index, link)
    }
}

impl<'a> StrictEq for IndexedInterWikiLink<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Represents a link to a file or directory in another wiki specified by
/// a name that maps to the name key in g:vimwiki_list
#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct NamedInterWikiLink<'a> {
    pub name: Cow<'a, str>,
    pub link: WikiLink<'a>,
}

impl NamedInterWikiLink<'_> {
    pub fn to_borrowed(&self) -> NamedInterWikiLink {
        use self::Cow::*;

        NamedInterWikiLink {
            name: Cow::Borrowed(match &self.name {
                Borrowed(x) => *x,
                Owned(x) => x.as_str(),
            }),
            link: self.link.to_borrowed(),
        }
    }

    pub fn into_owned(self) -> NamedInterWikiLink<'static> {
        NamedInterWikiLink {
            name: Cow::from(self.name.into_owned()),
            link: self.link.into_owned(),
        }
    }
}

impl<'a> fmt::Display for NamedInterWikiLink<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.link)
    }
}

impl<'a> From<(String, WikiLink<'a>)> for NamedInterWikiLink<'a> {
    fn from((name, link): (String, WikiLink<'a>)) -> Self {
        Self::new(Cow::from(name), link)
    }
}

impl<'a> From<(&'a str, WikiLink<'a>)> for NamedInterWikiLink<'a> {
    fn from((name, link): (&'a str, WikiLink<'a>)) -> Self {
        Self::new(Cow::from(name), link)
    }
}

impl<'a> StrictEq for NamedInterWikiLink<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
