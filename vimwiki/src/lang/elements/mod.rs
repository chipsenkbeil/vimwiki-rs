use super::utils::LE;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

mod blocks;
pub use blocks::*;
mod comments;
pub use comments::*;

/// Represents a full page containing different elements
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page {
    /// Comprised of the elements within a page
    pub elements: Vec<LE<BlockElement>>,

    /// Comprised of the comments within a page
    pub comments: Vec<LE<Comment>>,
}

/// Represents an arbitrary element for a language
pub trait Element {}
