use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::{Constructor, Index, IndexMut, IntoIterator};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Constructor,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Index,
    IndexMut,
    IntoIterator,
    Serialize,
    Deserialize,
)]
pub struct Header<'a> {
    /// Represents the content contained within the header
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub content: InlineElementContainer<'a>,

    /// Represents the level of the header (1, 2, 3, etc)
    pub level: usize,

    /// Represents whether or not the header is centered
    pub centered: bool,
}

impl<'a> Header<'a> {
    /// Represents the smallest a header's level can be
    pub const MIN_LEVEL: usize = 1;

    /// Represents teh largest a header's level can be
    pub const MAX_LEVEL: usize = 6;
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

impl<'a> fmt::Display for Header<'a> {
    /// Writes out the header by writing out its content using the underlying
    /// display impl
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
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
