use vimwiki::{components, LC};

mod blocks;
pub use blocks::*;
mod comments;
pub use comments::*;
mod utils;
pub use utils::*;

/// Represents a single document page
#[derive(async_graphql::SimpleObject)]
pub struct Page {
    /// The components contained within the page
    components: Vec<BlockComponent>,

    /// The comments contained within the page
    comments: Vec<Comment>,

    /// The area where the page resides
    region: Region,
}

impl From<LC<components::Page>> for Page {
    fn from(mut lc: LC<components::Page>) -> Self {
        let components = lc
            .component
            .components
            .drain(..)
            .map(BlockComponent::from)
            .collect();
        let comments =
            lc.component.comments.drain(..).map(Comment::from).collect();
        let region = Region::from(lc.region);

        Self {
            components,
            comments,
            region,
        }
    }
}
