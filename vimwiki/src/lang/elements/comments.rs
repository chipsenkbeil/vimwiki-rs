use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum Comment<'a> {
    Line(LineComment<'a>),
    MultiLine(MultiLineComment<'a>),
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct LineComment<'a>(pub Cow<'a, str>);

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
