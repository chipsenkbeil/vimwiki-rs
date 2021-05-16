use crate::StrictEq;
use derive_more::{
    Constructor, Deref, DerefMut, From, Index, IndexMut, Into, IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt};

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
pub struct Anchor<'a>(Vec<Cow<'a, str>>);

impl Anchor<'_> {
    pub fn to_borrowed(&self) -> Anchor {
        use self::Cow::*;

        let elements = self
            .iter()
            .map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            })
            .collect();

        Anchor::new(elements)
    }

    pub fn into_owned(self) -> Anchor<'static> {
        let elements = self
            .into_iter()
            .map(|x| Cow::from(x.into_owned()))
            .collect();

        Anchor::new(elements)
    }
}

impl<'a> fmt::Display for Anchor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            Ok(())
        } else {
            write!(f, "#{}", self.join("#"))
        }
    }
}

impl<'a> std::iter::FromIterator<&'a str> for Anchor<'a> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::new(iter.into_iter().map(Cow::from).collect())
    }
}

impl std::iter::FromIterator<String> for Anchor<'static> {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::new(iter.into_iter().map(Cow::from).collect())
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
