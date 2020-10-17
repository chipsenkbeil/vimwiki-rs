use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum Comment<'a> {
    Line(LineComment<'a>),
    MultiLine(MultiLineComment<'a>),
}

impl Comment<'_> {
    pub fn as_borrowed(&self) -> Comment {
        match self {
            Self::Line(x) => Comment::from(x.as_borrowed()),
            Self::MultiLine(x) => Comment::from(x.as_borrowed()),
        }
    }

    pub fn into_owned(self) -> Comment<'static> {
        match self {
            Self::Line(x) => Comment::from(x.into_owned()),
            Self::MultiLine(x) => Comment::from(x.into_owned()),
        }
    }
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
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

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct MultiLineComment<'a>(pub Vec<Cow<'a, str>>);

impl MultiLineComment<'_> {
    pub fn as_borrowed(&self) -> MultiLineComment {
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
        let inner = self.0.iter().map(|x| Cow::from(x.into_owned())).collect();

        MultiLineComment(inner)
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
