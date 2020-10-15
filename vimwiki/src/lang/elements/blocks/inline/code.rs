use derive_more::{Constructor, Display};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

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
pub struct CodeInline<'a> {
    pub code: Cow<'a, str>,
}

impl<'a> From<&'a str> for CodeInline<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::from(s))
    }
}
