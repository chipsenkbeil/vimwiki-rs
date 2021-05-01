use crate::StrictEq;
use derive_more::{Constructor, Display, From};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    Clone, Debug, Display, Hash, From, Eq, PartialEq, Serialize, Deserialize,
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
    Constructor,
    Clone,
    Debug,
    Display,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{}", "_0.trim()")]
pub struct LineComment<'a>(pub Cow<'a, str>);

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

impl<'a> LineComment<'a> {
    pub fn as_str(&self) -> &'a str {
        use self::Cow::*;

        match self.0 {
            Borrowed(x) => x,
            Owned(x) => x.as_str(),
        }
    }
}

impl<'a> From<&'a str> for LineComment<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::from(s))
    }
}

impl<'a> From<String> for LineComment<'a> {
    fn from(s: String) -> Self {
        Self::new(Cow::from(s))
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
    Constructor,
    Clone,
    Debug,
    Display,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{}", "_0.join(\"\n\")")]
pub struct MultiLineComment<'a>(pub Vec<Cow<'a, str>>);

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

impl<'a> MultiLineComment<'a> {
    pub fn as_lines(&self) -> &[Cow<'a, str>] {
        &self.0
    }
}

impl<'a> From<&'a str> for MultiLineComment<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(vec![Cow::from(s)])
    }
}

impl<'a> From<String> for MultiLineComment<'a> {
    fn from(s: String) -> Self {
        Self::new(vec![Cow::from(s)])
    }
}

impl<'a> StrictEq for MultiLineComment<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
