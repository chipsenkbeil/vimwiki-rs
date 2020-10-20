use super::Element;
use std::{iter, sync::Arc};
use vimwiki::elements;

#[derive(Debug)]
pub struct ElementTree(Arc<elements::ElementTree<'static>>);

impl ElementTree {
    pub fn element_at_offset(&self, offset: i32) -> Option<ElementTreeNode> {
        self.0
            .find_at_offset(offset as usize)
            .map(|x| wrap_node(&self.0, x))
    }
}

impl<'a> From<elements::ElementTree<'a>> for ElementTree {
    fn from(tree: elements::ElementTree<'a>) -> Self {
        Self(Arc::new(tree.into_owned()))
    }
}

#[derive(Debug)]
pub struct ElementTreeNode {
    tree: Arc<elements::ElementTree<'static>>,
    node: elements::ElementTreeNode<'static>,
}

#[async_graphql::Object]
impl ElementTreeNode {
    /// True if this node is at the root of a document
    async fn is_root(&self) -> bool {
        self.node.is_root()
    }

    /// True if this node has no children
    async fn is_leaf(&self) -> bool {
        self.node.is_leaf()
    }

    /// Represents the root of the current node
    async fn root(&self) -> ElementTreeNode {
        wrap_node(&self.tree, self.tree.root())
    }

    /// Represents the parent of the current node
    async fn parent(&self) -> Option<ElementTreeNode> {
        self.tree
            .parent(&self.node)
            .map(|x| wrap_node(&self.tree, x))
    }

    /// Represents the children of the current node
    async fn children(&self) -> Vec<ElementTreeNode> {
        self.tree
            .children(&self.node)
            .into_iter()
            .map(|x| wrap_node(&self.tree, x))
            .collect()
    }

    /// Represents all of the ancestors of the current node
    async fn ancestors(&self) -> Vec<ElementTreeNode> {
        self.tree
            .ancestors(&self.node)
            .map(|x| wrap_node(&self.tree, x))
            .collect()
    }

    /// Represents all of the descendants of the current node
    async fn descendants(&self) -> Vec<ElementTreeNode> {
        self.tree
            .descendants(&self.node)
            .map(|x| wrap_node(&self.tree, x))
            .collect()
    }

    /// Represents the element contained within the node
    async fn element(&self) -> Element {
        Element::from(self.node.clone().into_inner())
    }
}

fn wrap_node<'a>(
    tree: &Arc<elements::ElementTree<'static>>,
    node: &elements::ElementTreeNode<'a>,
) -> ElementTreeNode {
    ElementTreeNode {
        tree: Arc::clone(tree),
        node: node.clone().into_owned(),
    }
}
