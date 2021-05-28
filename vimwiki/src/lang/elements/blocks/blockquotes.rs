use crate::StrictEq;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Blockquote<'a> {
    pub lines: Vec<Cow<'a, str>>,
}

impl<'a> Blockquote<'a> {
    /// Returns lines within the blockquote
    pub fn lines(&self) -> &[Cow<'a, str>] {
        &self.lines
    }

    /// Returns an iterator over slices of lines where each item is a slice
    /// of lines representing a group of lines
    pub fn line_groups(&self) -> impl Iterator<Item = &[Cow<'a, str>]> {
        self.lines
            .split(|line| line.is_empty())
            .filter(|lines| !lines.is_empty())
    }
}

impl Blockquote<'_> {
    pub fn to_borrowed(&self) -> Blockquote {
        use self::Cow::*;

        let lines = self
            .lines
            .iter()
            .map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            })
            .collect();

        Blockquote { lines }
    }

    pub fn into_owned(self) -> Blockquote<'static> {
        let lines = self
            .lines
            .into_iter()
            .map(|x| Cow::from(x.into_owned()))
            .collect();

        Blockquote { lines }
    }
}

impl<'a> StrictEq for Blockquote<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
