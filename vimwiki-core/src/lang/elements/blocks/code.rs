use crate::StrictEq;
use derive_more::{Constructor, Index, IndexMut, IntoIterator};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fmt, iter::FromIterator};

#[derive(
    Constructor,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
pub struct CodeBlock<'a> {
    /// Represents the language associated with the code block if it has one
    pub language: Option<Cow<'a, str>>,

    /// Represents metadata associated with the code block in the form of
    /// key/value pairs
    pub metadata: HashMap<Cow<'a, str>, Cow<'a, str>>,

    /// Represents the lines of text contained within the code block
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub lines: Vec<Cow<'a, str>>,
}

impl<'a> CodeBlock<'a> {
    /// Constructs a code block with the provided lines using no language or metadata
    pub fn from_lines<I: IntoIterator<Item = L>, L: Into<Cow<'a, str>>>(
        iter: I,
    ) -> Self {
        Self {
            language: None,
            metadata: HashMap::new(),
            lines: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl CodeBlock<'_> {
    pub fn to_borrowed(&self) -> CodeBlock {
        use self::Cow::*;

        CodeBlock {
            language: self.language.as_ref().map(|x| {
                Cow::Borrowed(match x {
                    Borrowed(x) => *x,
                    Owned(x) => x.as_str(),
                })
            }),
            metadata: self
                .metadata
                .iter()
                .map(|(key, value)| {
                    let key = Cow::Borrowed(match key {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    });
                    let value = Cow::Borrowed(match value {
                        Borrowed(x) => *x,
                        Owned(x) => x.as_str(),
                    });

                    (key, value)
                })
                .collect(),
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
        }
    }

    pub fn into_owned(self) -> CodeBlock<'static> {
        CodeBlock {
            language: self.language.map(|x| Cow::from(x.into_owned())),
            metadata: self
                .metadata
                .into_iter()
                .map(|(key, value)| {
                    (Cow::from(key.into_owned()), Cow::from(value.into_owned()))
                })
                .collect(),
            lines: self
                .lines
                .into_iter()
                .map(|x| Cow::from(x.into_owned()))
                .collect(),
        }
    }
}

impl<'a> fmt::Display for CodeBlock<'a> {
    /// Writes out the code block by writing out each of its lines, separated
    /// by line feed
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

impl<'a> FromIterator<&'a str> for CodeBlock<'a> {
    /// Produces a new code block using the given iterator as the
    /// code block's lines
    fn from_iter<I: IntoIterator<Item = &'a str>>(iter: I) -> Self {
        Self::from_lines(iter)
    }
}

impl FromIterator<String> for CodeBlock<'static> {
    /// Produces a new code block using the given iterator as the
    /// code block's lines
    fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
        Self::from_lines(iter)
    }
}

impl<'a> FromIterator<Cow<'a, str>> for CodeBlock<'a> {
    /// Produces a new code block using the given iterator as the
    /// code block's lines
    fn from_iter<I: IntoIterator<Item = Cow<'a, str>>>(iter: I) -> Self {
        Self::from_lines(iter)
    }
}

impl<'a> StrictEq for CodeBlock<'a> {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
