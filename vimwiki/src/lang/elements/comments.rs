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

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct MultiLineComment<'a>(pub Vec<Cow<'a, str>>);
