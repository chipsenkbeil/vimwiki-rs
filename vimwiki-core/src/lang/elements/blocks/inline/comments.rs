use crate::StrictEq;
use derive_more::{
    AsRef, Constructor, Display, From, Index, IndexMut, IntoIterator, IsVariant,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, iter::FromIterator};

#[derive(
    Clone,
    Debug,
    Display,
    Hash,
    From,
    Eq,
    PartialEq,
    IsVariant,
    Serialize,
    Deserialize,
)]
pub enum Comment<'a> {
    Line(LineComment<'a>),
    MultiLine(MultiLineComment<'a>),
}

impl Comment<'_> {
    pub fn to_borrowed(&self) -> Comment {
        match self {
            Self::Line(x) => Comment::from(x.as_borrowed()),
            Self::MultiLine(x) => Comment::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> Comment<'static> {
        match self {
            Self::Line(x) => Comment::from(x.into_owned()),
            Self::MultiLine(x) => Comment::from(x.into_owned()),
        }
    }
}

impl<'a> StrictEq for Comment<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(
    AsRef,
    Clone,
    Constructor,
    Debug,
    Display,
    From,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
#[display(fmt = "{}", "_0.trim()")]
pub struct LineComment<'a>(Cow<'a, str>);

impl<'a> LineComment<'a> {
    /// Returns comment's text as a [`str`]
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl LineComment<'_> {
    pub fn as_borrowed(&self) -> LineComment {
        use self::Cow::*;

        let inner = match &self.0 {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        };

        LineComment(Cow::Borrowed(inner))
    }

    pub fn into_owned(self) -> LineComment<'static> {
        LineComment(Cow::from(self.0.into_owned()))
    }
}

impl<'a> From<&'a str> for LineComment<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::Borrowed(s))
    }
}

impl<'a> From<String> for LineComment<'a> {
    fn from(s: String) -> Self {
        Self::new(Cow::Owned(s))
    }
}

impl<'a> StrictEq for LineComment<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(
    AsRef,
    Clone,
    Constructor,
    Debug,
    Display,
    Eq,
    PartialEq,
    Hash,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
#[display(fmt = "{}", "_0.join(\"\n\")")]
#[into_iterator(owned, ref, ref_mut)]
pub struct MultiLineComment<'a>(Vec<Cow<'a, str>>);

impl<'a> MultiLineComment<'a> {
    /// Returns total lines available
    pub fn line_cnt(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over lines
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(AsRef::as_ref)
    }
}

impl MultiLineComment<'_> {
    pub fn to_borrowed(&self) -> MultiLineComment {
        use self::Cow::*;

        let inner = self
            .0
            .iter()
            .map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            })
            .collect();

        MultiLineComment(inner)
    }

    pub fn into_owned(self) -> MultiLineComment<'static> {
        let inner = self
            .0
            .into_iter()
            .map(|x| Cow::from(x.into_owned()))
            .collect();

        MultiLineComment(inner)
    }
}

impl<'a> From<&'a str> for MultiLineComment<'a> {
    fn from(s: &'a str) -> Self {
        std::iter::once(s).collect()
    }
}

impl<'a> From<String> for MultiLineComment<'a> {
    fn from(s: String) -> Self {
        std::iter::once(s).collect()
    }
}

impl<'a> FromIterator<&'a str> for MultiLineComment<'a> {
    /// Produces a multiline comment using the given iterator as the
    /// comment's lines
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        iter.into_iter().map(Cow::Borrowed).collect()
    }
}

impl FromIterator<String> for MultiLineComment<'static> {
    /// Produces a multiline comment using the given iterator as the
    /// comment's lines
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        iter.into_iter().map(Cow::Owned).collect()
    }
}

impl<'a> FromIterator<Cow<'a, str>> for MultiLineComment<'a> {
    /// Produces a multiline comment using the given iterator as the
    /// comment's lines
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> StrictEq for MultiLineComment<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
