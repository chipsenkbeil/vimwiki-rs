use crate::StrictEq;
use derive_more::{AsRef, Constructor, Display};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(
    AsRef,
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
#[as_ref(forward)]
pub struct MathInline<'a>(
    /// Represents the text contained within the inline math snippet
    Cow<'a, str>,
);

impl<'a> MathInline<'a> {
    /// Extracts a string slice containing the entire math snippet
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use std::borrow::Cow;
    /// # use vimwiki_core::MathInline;
    /// let math = MathInline::new(Cow::Borrowed("some math"));
    /// assert_eq!(math.as_str(), "some math");
    /// ```
    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl MathInline<'_> {
    pub fn as_borrowed(&self) -> MathInline {
        use self::Cow::*;

        let formula = Cow::Borrowed(match &self.0 {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        });

        MathInline::new(formula)
    }

    pub fn into_owned(self) -> MathInline<'static> {
        let formula = Cow::Owned(self.0.into_owned());

        MathInline::new(formula)
    }
}

impl<'a> From<&'a str> for MathInline<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::Borrowed(s))
    }
}

impl From<String> for MathInline<'static> {
    fn from(s: String) -> Self {
        Self::new(Cow::Owned(s))
    }
}

impl<'a> StrictEq for MathInline<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
