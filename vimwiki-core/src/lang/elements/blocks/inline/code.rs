use crate::StrictEq;
use derive_more::{AsRef, Constructor, Display, Into};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    AsRef,
    Constructor,
    Clone,
    Debug,
    Display,
    Into,
    Eq,
    PartialEq,
    Hash,
    Serialize,
    Deserialize,
)]
#[as_ref(forward)]
pub struct CodeInline<'a>(Cow<'a, str>);

impl<'a> CodeInline<'a> {
    /// Extracts a string slice containing the entire code snippet
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::borrow::Cow;
    /// # use vimwiki_core::CodeInline;
    /// let code = CodeInline::new(Cow::Borrowed("some code"));
    /// assert_eq!(code.as_str(), "some code");
    /// ```
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl CodeInline<'_> {
    pub fn as_borrowed(&self) -> CodeInline {
        use self::Cow::*;

        let code = Cow::Borrowed(match &self.0 {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        });

        CodeInline::new(code)
    }

    pub fn into_owned(self) -> CodeInline<'static> {
        let code = Cow::from(self.0.into_owned());

        CodeInline::new(code)
    }
}

impl<'a> From<&'a str> for CodeInline<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::Borrowed(s))
    }
}

impl From<String> for CodeInline<'static> {
    fn from(s: String) -> Self {
        Self::new(Cow::Owned(s))
    }
}

impl<'a> StrictEq for CodeInline<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
