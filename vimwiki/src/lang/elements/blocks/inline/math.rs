use crate::StrictEq;
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
pub struct MathInline<'a> {
    pub formula: Cow<'a, str>,
}

impl MathInline<'_> {
    pub fn as_borrowed(&self) -> MathInline {
        use self::Cow::*;

        let formula = Cow::Borrowed(match &self.formula {
            Borrowed(x) => *x,
            Owned(x) => x.as_str(),
        });

        MathInline { formula }
    }

    pub fn into_owned(self) -> MathInline<'static> {
        let formula = Cow::from(self.formula.into_owned());

        MathInline { formula }
    }
}

impl<'a> From<&'a str> for MathInline<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(Cow::from(s))
    }
}

impl<'a> StrictEq for MathInline<'a> {
    /// Same as PartialEq
    #[inline]
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}
