mod blocks;
pub use blocks::*;
mod node;
pub use node::*;
mod utils;
pub use utils::*;

use derive_more::Constructor;
use std::sync::Arc;
use vimwiki::{elements, Located};

/// Represents a single document page
#[derive(Constructor, Clone, Debug)]
pub struct Page {
    forest: Arc<elements::ElementForest<'static>>,
}

#[async_graphql::Object]
impl Page {
    /// Returns all root-level elements in a page
    async fn root_elements(&self) -> Vec<BlockElement> {
        self.forest
            .roots()
            .filter_map(|root| {
                root.as_inner()
                    .as_ref()
                    .map(elements::Element::to_borrowed)
                    .map(elements::Element::into_block_element)
                    .transpose()
            })
            .map(BlockElement::from)
            .collect()
    }

    /// Returns element in page with specified id as traversable node
    async fn node<'a>(&'a self, id: i32) -> Option<ElementNode<'a>> {
        self.forest
            .find_tree_and_node_by_id(id as usize)
            .map(ElementNode::from)
    }

    /// Returns all elements in a page as traversable nodes
    async fn nodes<'a>(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .trees_and_nodes()
            .map(ElementNode::from)
            .collect()
    }

    /// Returns all tags in a page as traversable nodes
    async fn tags<'a>(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .trees_and_nodes()
            .filter(|(_, node)| {
                node.as_element()
                    .as_inline_element()
                    .map(|e| matches!(e, elements::InlineElement::Tags(_)))
                    .unwrap_or_default()
            })
            .map(ElementNode::from)
            .collect()
    }

    /// Finds the element at the specified offset as a traversable node
    async fn node_at_offset<'a>(
        &'a self,
        offset: i32,
    ) -> Option<ElementNode<'a>> {
        self.forest
            .find_tree_and_node_at_offset(offset as usize)
            .map(ElementNode::from)
    }
}

/// Represents some element in a document page
#[derive(async_graphql::Union, Debug)]
pub enum Element {
    #[graphql(flatten)]
    Block(BlockElement),

    #[graphql(flatten)]
    Inline(InlineElement),

    #[graphql(flatten)]
    InlineBlock(InlineBlockElement),
}

impl<'a> From<Located<elements::Element<'a>>> for Element {
    fn from(located: Located<elements::Element<'a>>) -> Self {
        let region = located.region();
        match located.into_inner() {
            elements::Element::Block(x) => {
                Element::from(BlockElement::from(Located::new(x, region)))
            }
            elements::Element::Inline(x) => {
                Element::from(InlineElement::from(Located::new(x, region)))
            }
            elements::Element::InlineBlock(x) => {
                Element::from(InlineBlockElement::from(Located::new(x, region)))
            }
        }
    }
}

/// Represents some inline block element in a document page
#[derive(async_graphql::Union, Debug)]
pub enum InlineBlockElement {
    ListItem(ListItem),
    Term(Term),
    Definition(Definition),
}

impl<'a> From<Located<elements::InlineBlockElement<'a>>>
    for InlineBlockElement
{
    fn from(located: Located<elements::InlineBlockElement<'a>>) -> Self {
        let region = located.region();
        match located.into_inner() {
            elements::InlineBlockElement::ListItem(x) => {
                InlineBlockElement::from(ListItem::from(Located::new(
                    x, region,
                )))
            }
            elements::InlineBlockElement::Term(x) => {
                InlineBlockElement::from(Term::from(Located::new(x, region)))
            }
            elements::InlineBlockElement::Definition(x) => {
                InlineBlockElement::from(Definition::from(Located::new(
                    x, region,
                )))
            }
        }
    }
}
