use super::{InlineElement, InlineElementContainer, Located};
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

    pub fn to_children(&'a self) -> Vec<Located<InlineElement<'a>>> {
        self.content.to_children()
    }
}
