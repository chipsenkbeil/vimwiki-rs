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

impl CodeInline<'_> {
    pub fn as_borrowed(&self) -> CodeInline {
        use self::Cow::*;

        let code = Cow::Borrowed(match &self.code {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        });

        CodeInline { code }
    }

    pub fn into_owned(self) -> CodeInline<'static> {
        let code = Cow::from(self.code.into_owned());

        CodeInline { code }
    }
}

impl<'a> From<&'a str> for CodeInline<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::from(s))
    }
}
