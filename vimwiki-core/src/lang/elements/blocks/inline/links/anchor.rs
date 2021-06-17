use crate::StrictEq;
use derive_more::{
    Constructor, Deref, DerefMut, Index, IndexMut, IntoIterator,
};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, convert::TryFrom, fmt};
use uriparse::{Fragment, FragmentError};

/// Represents an anchor
#[derive(
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    Eq,
    PartialEq,
    Hash,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Anchor<'a>(
    /// Represents the individual parts of the anchor
    Vec<Cow<'a, str>>,
);

impl Anchor<'_> {
    pub fn to_borrowed(&self) -> Anchor {
        use self::Cow::*;

        let elements = self
            .into_iter()
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

impl<'a> Anchor<'a> {
    /// Produces an encoded URI fragment in the form of #my%23fragment
    /// if the anchor has any elements, otherwise yields an empty string
    pub fn to_encoded_uri_fragment(&self) -> String {
        use std::fmt::Write;
        let mut fragment = String::new();
        if !self.is_empty() {
            write!(&mut fragment, "#{}", self.0.join("%23")).expect(
                "Anchor encoded_uri_fragment returned error unexpectedly",
            );
        }
        fragment
    }

    // NOTE: Cannot use FromStr due to conflicting lifetimes of impl and trait
    //       method's input str
    /// Parses a uri fragment in the form of `#one#two#three` with a starting
    /// pound sign
    pub fn from_uri_fragment(s: &'a str) -> Option<Self> {
        let mut it = s.split('#');

        if let Some(piece) = it.next() {
            if piece.is_empty() {
                return Some(it.collect());
            }
        }

        None
    }
}

impl<'a> fmt::Display for Anchor<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            Ok(())
        } else {
            write!(f, "#{}", self.0.join("#"))
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

impl<'a> TryFrom<Anchor<'a>> for Fragment<'static> {
    type Error = FragmentError;

    /// Consumes an anchor, producing a newly-allocated fragment
    fn try_from(anchor: Anchor<'a>) -> Result<Self, Self::Error> {
        Fragment::try_from(anchor.0.join("-").as_str())
            .map(Fragment::into_owned)
    }
}

impl<'a> StrictEq for Anchor<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
