use crate::StrictEq;
use derive_more::{AsRef, Display, Index, IndexMut, IntoIterator};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, iter::FromIterator};

#[derive(
    AsRef,
    Clone,
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
pub struct Blockquote<'a>(Vec<Cow<'a, str>>);

impl<'a> Blockquote<'a> {
    pub fn new<T: Into<Cow<'a, str>>, I: IntoIterator<Item = T>>(
        iter: I,
    ) -> Self {
        iter.into_iter().map(Into::into).collect()
    }

    /// Returns total lines available
    pub fn line_cnt(&self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over lines
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(AsRef::as_ref)
    }

    /// Returns an iterator over slices of lines where each item is a slice
    /// of lines representing a group of lines
    pub fn line_groups(&self) -> impl Iterator<Item = &[Cow<'a, str>]> {
        self.0
            .split(|line| line.is_empty())
            .filter(|lines| !lines.is_empty())
    }
}

impl Blockquote<'_> {
    pub fn to_borrowed(&self) -> Blockquote {
        use self::Cow::*;

        let lines = self.0.iter().map(|x| {
            Cow::Borrowed(match x {
                Borrowed(x) => *x,
                Owned(x) => x.as_str(),
            })
        });

        Blockquote::new(lines)
    }

    pub fn into_owned(self) -> Blockquote<'static> {
        let lines = self.into_iter().map(|x| Cow::from(x.into_owned()));
        Blockquote::new(lines)
    }
}

impl<'a> FromIterator<&'a str> for Blockquote<'a> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl FromIterator<String> for Blockquote<'static> {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl<'a> FromIterator<Cow<'a, str>> for Blockquote<'a> {
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl<'a> StrictEq for Blockquote<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
