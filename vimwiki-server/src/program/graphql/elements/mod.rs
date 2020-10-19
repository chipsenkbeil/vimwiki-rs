mod blocks;
pub use blocks::*;
mod utils;
pub use utils::*;

use vimwiki::{elements, ElementTree, ElementTreeNode, Located};

/// Represents a single document page
#[derive(Debug)]
pub struct Page {
    /// The elements contained within the page
    elements_and_trees: Vec<(BlockElement, ElementTree<'static>)>,
}

#[async_graphql::Object]
impl Page {
    /// Returns all elements in a page
    async fn elements(&self) -> Vec<&BlockElement> {
        self.elements_and_trees.iter().map(|(e, _)| e).collect()
    }

    async fn element_at_offset(&self, offset: i32) -> Option<Element> {
        self.elements_and_trees
            .iter()
            .find_map(|(_, tree)| tree.find_at_offset(offset as usize))
            .map(ElementTreeNode::clone)
            .map(ElementTreeNode::into_inner)
            .map(Element::from)
    }
}

impl<'a> From<vimwiki::elements::Page<'a>> for Page {
    fn from(page: vimwiki::elements::Page<'a>) -> Self {
        let elements_and_trees = page
            .elements
            .into_iter()
            .map(|e| {
                let e2 = e.clone();
                (
                    BlockElement::from(e),
                    ElementTree::from(e2.map(elements::Element::from))
                        .into_owned(),
                )
            })
            .collect();

        Self { elements_and_trees }
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
        }
    }
}
