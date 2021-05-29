use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header<'a> {
    pub level: usize,
    pub content: InlineElementContainer<'a>,
    pub centered: bool,
}

impl Header<'_> {
    pub fn to_borrowed(&self) -> Header {
        Header {
            level: self.level,
            content: self.content.to_borrowed(),
            centered: self.centered,
        }
    }

    pub fn into_owned(self) -> Header<'static> {
        Header {
            level: self.level,
            content: self.content.into_owned(),
            centered: self.centered,
        }
    }
}

impl<'a> Header<'a> {
    /// Represents the smallest a header's level can be
    pub const MIN_LEVEL: usize = 1;

    /// Represents teh largest a header's level can be
    pub const MAX_LEVEL: usize = 6;
}

impl<'a> IntoChildren for Header<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.content.into_children()
    }
}

impl<'a> StrictEq for Header<'a> {
    /// Performs strict_eq on level, centered status, and content
    fn strict_eq(&self, other: &Self) -> bool {
        self.level == other.level
            && self.centered == other.centered
            && self.content.strict_eq(&other.content)
    }
}
