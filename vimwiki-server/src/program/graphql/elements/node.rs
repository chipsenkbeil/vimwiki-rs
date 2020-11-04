use super::{Element, Region};
use derive_more::From;
use vimwiki::elements;

#[derive(Debug, From)]
pub struct ElementNode<'a> {
    tree: &'a elements::ElementTree<'a>,
    node: &'a elements::ElementNode<'a>,
}

#[async_graphql::Object]
impl<'a> ElementNode<'a> {
    /// Represents the id of the current node
    async fn id(&self) -> i32 {
        self.node.id() as i32
    }

    /// True if this node is at the root of a document
    async fn is_root(&self) -> bool {
        self.node.is_root()
    }

    /// True if this node has no children
    async fn is_leaf(&self) -> bool {
        self.node.is_leaf()
    }

    /// Represents the root of the current node
    async fn root(&self) -> ElementNode<'a> {
        ElementNode::from((self.tree, self.tree.root()))
    }

    /// Represents the parent of the current node
    async fn parent(&self) -> Option<ElementNode<'a>> {
        self.tree
            .parent(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
    }

    /// Represents the children of the current node
    async fn children(&self) -> Vec<ElementNode<'a>> {
        self.tree
            .children(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .collect()
    }

    /// Represents the siblings of the current node
    async fn siblings(&self) -> Vec<ElementNode<'a>> {
        self.tree
            .siblings(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .collect()
    }

    /// Represents the siblings before the current node
    async fn siblings_before(&self) -> Vec<ElementNode<'a>> {
        self.tree
            .siblings_before(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .collect()
    }

    /// Represents the previous sibling just before the current node
    async fn prev_sibling(&self) -> Option<ElementNode<'a>> {
        self.tree
            .siblings_before(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .last()
    }

    /// Represents the siblings after the current node
    async fn siblings_after(&self) -> Vec<ElementNode<'a>> {
        self.tree
            .siblings_after(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .collect()
    }

    /// Represents the next sibling just after the current node
    async fn next_sibling(&self) -> Option<ElementNode<'a>> {
        self.tree
            .siblings_after(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .next()
    }

    /// Represents all of the ancestors of the current node
    async fn ancestors(&self) -> Vec<ElementNode<'a>> {
        self.tree
            .ancestors(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .collect()
    }

    /// Represents all of the descendants of the current node
    async fn descendants(&self) -> Vec<ElementNode<'a>> {
        self.tree
            .descendants(&self.node)
            .map(|x| ElementNode::from((self.tree, x)))
            .collect()
    }

    /// Represents the region within the file comprising the element
    async fn region(&self) -> Region {
        Region::from(self.node.region())
    }

    /// Represents the element contained within the node
    async fn element(&self) -> Element {
        Element::from(self.node.clone().into_inner())
    }
}
