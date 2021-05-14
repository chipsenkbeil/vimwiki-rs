use entity::*;
use serde::{Deserialize, Serialize};
use vimwiki as v;

/// Represents a segment of a document marked by a byte offset and length
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    async_graphql::SimpleObject,
    Serialize,
    Deserialize,
    ValueLike,
)]
pub struct Region {
    /// The byte offset within a file where this element begins
    offset: usize,

    /// The byte length of this element within a file
    len: usize,

    /// The depth of the element within a series of elements with zero
    /// being a top-level element
    depth: u16,
}

impl From<v::Region> for Region {
    fn from(region: v::Region) -> Self {
        Self {
            offset: region.offset(),
            len: region.len(),
            depth: region.depth(),
        }
    }
}
