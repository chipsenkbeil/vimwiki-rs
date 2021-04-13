use entity::*;
use serde::{Deserialize, Serialize};
use vimwiki::elements as v;

/// Represents a segment of a document marked by a byte offset and length
#[derive(
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
}

impl From<v::Region> for Region {
    fn from(region: v::Region) -> Self {
        Self {
            offset: region.offset(),
            len: region.len(),
        }
    }
}
