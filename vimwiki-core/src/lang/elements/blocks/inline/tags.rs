use crate::StrictEq;
use derive_more::{
    AsRef, Constructor, Deref, DerefMut, Display, From, Index, IndexMut, Into,
    IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, iter::FromIterator};

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
    AsRef,
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
#[as_ref(forward)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Tags<'a>(
    /// Represents the tags contained within the tag set
    Vec<Tag<'a>>,
);

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
    /// Extracts a string slice containing the entire tag
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::borrow::Cow;
    /// # use vimwiki_core::Tag;
    /// let tag = Tag::new(Cow::Borrowed("my-tag"));
    /// assert_eq!(tag.as_str(), "my-tag");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for tag in self.0.iter() {
            write!(f, ":{}", tag.0)?;
        }
        write!(f, ":")
    }
}

impl<'a> From<Tag<'a>> for Tags<'a> {
    fn from(tag: Tag<'a>) -> Self {
        std::iter::once(tag).collect()
    }
}

impl From<String> for Tags<'static> {
    fn from(s: String) -> Self {
        std::iter::once(s).collect()
    }
}

impl<'a> From<&'a str> for Tags<'a> {
    fn from(s: &'a str) -> Self {
        std::iter::once(s).collect()
    }
}

impl<'a> FromIterator<&'a str> for Tags<'a> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::new(iter.into_iter().map(Tag::from).collect())
    }
}

impl FromIterator<String> for Tags<'static> {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::new(iter.into_iter().map(Tag::from).collect())
    }
}

impl<'a> FromIterator<Cow<'a, str>> for Tags<'a> {
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> Self {
        Self::new(iter.into_iter().map(Tag::from).collect())
    }
}

impl<'a> FromIterator<Tag<'a>> for Tags<'a> {
    fn from_iter<I: IntoIterator<Item = Tag<'a>>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
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
    AsRef,
    Constructor,
    Clone,
    Debug,
    Display,
    From,
    Into,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
pub struct Tag<'a>(Cow<'a, str>);

impl<'a> Tag<'a> {
    /// Extracts a string slice containing the entire tag
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::borrow::Cow;
    /// # use vimwiki_core::Tag;
    /// let tag = Tag::new(Cow::Borrowed("my-tag"));
    /// assert_eq!(tag.as_str(), "my-tag");
    /// ```
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

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
