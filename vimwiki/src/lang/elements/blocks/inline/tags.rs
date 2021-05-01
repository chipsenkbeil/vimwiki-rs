use crate::StrictEq;
use derive_more::{
    Constructor, Deref, DerefMut, Display, From, Index, IndexMut, Into,
    IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

/// Represents a sequence of one or more tags
///
/// In vimwiki, :my-tag: would become
///
/// Tags([ Tag(my-tag) ])
///
/// Similarly, :my-tag-1:my-tag-2: would become
///
/// Tags([ Tag(my-tag-1), Tag(my-tag-2) ])
///
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
pub struct Tags<'a>(pub Vec<Tag<'a>>);

impl Tags<'_> {
    pub fn to_borrowed(&self) -> Tags {
        let inner = self.0.iter().map(Tag::as_borrowed).collect();

        Tags(inner)
    }

    pub fn into_owned(self) -> Tags<'static> {
        let inner = self.0.into_iter().map(Tag::into_owned).collect();

        Tags(inner)
    }
}

impl<'a> fmt::Display for Tags<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for tag in self.0.iter() {
            write!(f, ":{}", tag.0)?;
        }
        write!(f, ":")
    }
}

impl<'a> From<Tag<'a>> for Tags<'a> {
    fn from(tag: Tag<'a>) -> Self {
        Self::new(vec![tag])
    }
}

impl From<String> for Tags<'static> {
    fn from(s: String) -> Self {
        Self::from(Tag::from(s))
    }
}

impl<'a> From<&'a str> for Tags<'a> {
    fn from(s: &'a str) -> Self {
        Self::from(Tag::from(s))
    }
}

impl<'a> StrictEq for Tags<'a> {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// Represents a single tag
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    Display,
    From,
    Into,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
pub struct Tag<'a>(pub Cow<'a, str>);

impl Tag<'_> {
    pub fn as_borrowed(&self) -> Tag {
        use self::Cow::*;

        let inner = Cow::Borrowed(match &self.0 {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        });

        Tag(inner)
    }

    pub fn into_owned(self) -> Tag<'static> {
        let inner = Cow::from(self.0.into_owned());

        Tag(inner)
    }
}

impl<'a> Tag<'a> {
    pub fn as_str(&self) -> &'a str {
        use self::Cow::*;

        match self.0 {
            Borrowed(x) => x,
            Owned(x) => x.as_str(),
        }
    }
}

impl<'a> From<&'a str> for Tag<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::from(s))
    }
}

impl From<String> for Tag<'static> {
    fn from(s: String) -> Self {
        Self::new(Cow::from(s))
    }
}

impl<'a> StrictEq for Tag<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
