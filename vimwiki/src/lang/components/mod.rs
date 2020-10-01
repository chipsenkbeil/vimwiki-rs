use super::utils::LC;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

mod block_components;
pub use block_components::*;
mod comments;
pub use comments::*;

/// Represents a full page containing different components
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page {
    /// Comprised of the components within a page
    pub components: Vec<LC<BlockComponent>>,

    /// Comprised of the comments within a page
    pub comments: Vec<LC<Comment>>,
}
