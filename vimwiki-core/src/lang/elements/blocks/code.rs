use crate::StrictEq;
use derive_more::{Constructor, Index, IndexMut, IntoIterator};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, iter::FromIterator};

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
    lang: Option<Cow<'a, str>>,
    metadata: HashMap<Cow<'a, str>, Cow<'a, str>>,

    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    lines: Vec<Cow<'a, str>>,
}

impl<'a> CodeBlock<'a> {
    /// Returns reference to the code block's language, if it has one
    pub fn language(&self) -> Option<&str> {
        self.lang.as_deref()
    }

    /// Returns iterator over the metadata of the code block in the form of
    /// key-value pairs
    pub fn metadata(&self) -> impl Iterator<Item = (&str, &str)> {
        self.metadata.iter().map(|(k, v)| (k.as_ref(), v.as_ref()))
    }

    /// Returns total lines contained within code block
    pub fn line_cnt(&self) -> usize {
        self.lines.len()
    }

    /// Returns iterator over the lines contained within the code block
    pub fn lines(&self) -> impl Iterator<Item = &str> {
        self.lines.iter().map(AsRef::as_ref)
    }

    /// Constructs a code block with the provided lines using no language or metadata
    pub fn from_lines<I: IntoIterator<Item = L>, L: Into<Cow<'a, str>>>(
        iter: I,
    ) -> Self {
        Self {
            lang: None,
            metadata: HashMap::new(),
            lines: iter.into_iter().map(Into::into).collect(),
        }
    }
}

impl CodeBlock<'_> {
    pub fn to_borrowed(&self) -> CodeBlock {
        use self::Cow::*;

        CodeBlock {
            lang: self.lang.as_ref().map(|x| {
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
            lang: self.lang.map(|x| Cow::from(x.into_owned())),
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
