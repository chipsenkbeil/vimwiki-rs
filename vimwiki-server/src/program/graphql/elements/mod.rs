use vimwiki::elements::{self, Located};

mod blocks;
pub use blocks::*;
mod comments;
pub use comments::*;
mod utils;
pub use utils::*;

/// Represents a single document page
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Page {
    /// The elements contained within the page
    elements: Vec<BlockElement>,

    /// The comments contained within the page
    comments: Vec<Comment>,

    /// The area where the page resides
    region: Region,
}

impl From<Located<elements::Page>> for Page {
    fn from(mut le: Located<elements::Page>) -> Self {
        let elements = le
            .element
            .elements
            .drain(..)
            .map(BlockElement::from)
            .collect();
        let comments =
            le.element.comments.drain(..).map(Comment::from).collect();
        let region = Region::from(le.region);

        Self {
            elements,
            comments,
            region,
        }
    }
}

/// Represents some element in a document page
#[derive(async_graphql::Union, Debug)]
pub enum Element {
    #[graphql(flatten)]
    Block(BlockElement),

    #[graphql(flatten)]
    Inline(InlineElement),
}
