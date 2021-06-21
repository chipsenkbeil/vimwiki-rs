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
pub struct Blockquote<'a> {
    /// Represents the lines of text contained within the blockquote include
    /// potential blank lines
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub lines: Vec<Cow<'a, str>>,
}

impl<'a> Blockquote<'a> {
    /// Returns total line groups available
    pub fn line_group_cnt(&self) -> usize {
        self.line_groups().count()
    }

    /// Returns an iterator over slices of lines where each item is a slice
    /// of lines representing a group of lines
    pub fn line_groups(&self) -> impl Iterator<Item = &[Cow<'a, str>]> {
        self.lines
            .split(|line| line.is_empty())
            .filter(|lines| !lines.is_empty())
    }

    /// Returns an iterator over mutable slices of lines where each item is a slice
    /// of lines representing a group of lines
    pub fn mut_line_groups(&self) -> impl Iterator<Item = &[Cow<'a, str>]> {
        self.lines
            .split(|line| line.is_empty())
            .filter(|lines| !lines.is_empty())
    }
}

impl Blockquote<'_> {
    pub fn to_borrowed(&self) -> Blockquote {
        use self::Cow::*;

        self.lines
            .iter()
            .map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            })
            .collect()
    }

    pub fn into_owned(self) -> Blockquote<'static> {
        self.into_iter()
            .map(|x| Cow::from(x.into_owned()))
            .collect()
    }
}

impl<'a> fmt::Display for Blockquote<'a> {
    /// Writes out the blockquote by writing out each of its lines
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in self {
            writeln!(f, "{}", line)?;
        }

        Ok(())
    }
}

impl<'a> FromIterator<&'a str> for Blockquote<'a> {
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self {
            lines: iter.into_iter().map(Cow::Borrowed).collect(),
        }
    }
}

impl FromIterator<String> for Blockquote<'static> {
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self {
            lines: iter.into_iter().map(Cow::Owned).collect(),
        }
    }
}

impl<'a> FromIterator<Cow<'a, str>> for Blockquote<'a> {
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> Self {
        Self {
            lines: iter.into_iter().collect(),
        }
    }
}

impl<'a> StrictEq for Blockquote<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
