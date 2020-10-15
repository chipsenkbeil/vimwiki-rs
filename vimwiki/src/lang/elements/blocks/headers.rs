use super::InlineElementContainer;
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

impl<'a> Header<'a> {
    /// Represents the smallest a header's level can be
    pub const MIN_LEVEL: usize = 1;

    /// Represents teh largest a header's level can be
    pub const MAX_LEVEL: usize = 6;
}
