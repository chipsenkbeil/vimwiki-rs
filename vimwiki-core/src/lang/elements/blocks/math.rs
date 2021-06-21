use crate::StrictEq;
use derive_more::{Constructor, Index, IndexMut, IntoIterator};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, fmt, iter::FromIterator};

#[derive(
    Constructor,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
pub struct MathBlock<'a> {
    /// Represents the lines of text contained within the math block
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub lines: Vec<Cow<'a, str>>,

    /// Represents the environment associated with the math block if it has one
    pub environment: Option<Cow<'a, str>>,
}

impl<'a> MathBlock<'a> {
    /// Constructs a math block with the provided lines using no environment
    pub fn from_lines<I: IntoIterator<Item = L>, L: Into<Cow<'a, str>>>(
        iter: I,
    ) -> Self {
        Self {
            lines: iter.into_iter().map(Into::into).collect(),
            environment: None,
        }
    }
}

impl MathBlock<'_> {
    pub fn to_borrowed(&self) -> MathBlock {
        use self::Cow::*;

        MathBlock {
            lines: self
                .lines
                .iter()
                .map(|x| {
                    Cow::Borrowed(match x {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    })
                })
                .collect(),
            environment: self.environment.as_ref().map(|x| {
                Cow::Borrowed(match &x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            }),
        }
    }

    pub fn into_owned(self) -> MathBlock<'static> {
        MathBlock {
            lines: self
                .lines
                .into_iter()
                .map(|x| Cow::from(x.into_owned()))
                .collect(),
            environment: self.environment.map(|x| Cow::from(x.into_owned())),
        }
    }
}

impl<'a> fmt::Display for MathBlock<'a> {
    /// Writes out the math block by writing out each of its lines, separated
    /// by line feed
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

impl<'a> FromIterator<&'a str> for MathBlock<'a> {
    /// Produces a new math block using the given iterator as the
    /// math block's lines
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::from_lines(iter)
    }
}

impl FromIterator<String> for MathBlock<'static> {
    /// Produces a new math block using the given iterator as the
    /// math block's lines
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::from_lines(iter)
    }
}

impl<'a> FromIterator<Cow<'a, str>> for MathBlock<'a> {
    /// Produces a new math block using the given iterator as the
    /// math block's lines
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> Self {
        Self::from_lines(iter)
    }
}

impl<'a> StrictEq for MathBlock<'a> {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
