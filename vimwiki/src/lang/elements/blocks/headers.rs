use super::{Element, InlineElementContainer};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Header {
    pub level: usize,
    pub content: InlineElementContainer,
    pub centered: bool,
}

impl Element for Header {}

impl Header {
    /// Represents the smallest a header's level can be
    pub const MIN_LEVEL: usize = 1;

    /// Represents teh largest a header's level can be
    pub const MAX_LEVEL: usize = 6;
}
