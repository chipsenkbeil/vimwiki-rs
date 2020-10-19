mod blocks;
pub use blocks::*;
mod utils;
pub use utils::*;

/// Represents a single document page
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Page {
    /// The elements contained within the page
    elements: Vec<BlockElement>,
}

impl<'a> From<vimwiki::elements::Page<'a>> for Page {
    fn from(page: vimwiki::elements::Page<'a>) -> Self {
        let elements =
            page.elements.into_iter().map(BlockElement::from).collect();

        Self { elements }
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
