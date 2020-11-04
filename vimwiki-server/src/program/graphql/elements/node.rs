use super::{Element, Region};
use derive_more::From;
use std::sync::Arc;
use vimwiki::elements;

#[derive(Debug, From)]
pub struct ElementNode<'a> {
    forest: Arc<elements::ElementForest<'a>>,
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
    async fn root(&'a self) -> ElementNode<'a> {
        ElementNode::from((
            Arc::clone(&self.forest),
            self.forest.root_for(self.node),
        ))
    }

    /// Represents the parent of the current node
    async fn parent(&'a self) -> Option<ElementNode<'a>> {
        self.forest
            .parent(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
    }

    /// Represents the children of the current node
    async fn children(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .children(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .collect()
    }

    /// Represents the siblings of the current node
    async fn siblings(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .siblings(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .collect()
    }

    /// Represents the siblings before the current node
    async fn siblings_before(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .siblings_before(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .collect()
    }

    /// Represents the previous sibling just before the current node
    async fn prev_sibling(&'a self) -> Option<ElementNode<'a>> {
        self.forest
            .siblings_before(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .last()
    }

    /// Represents the siblings after the current node
    async fn siblings_after(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .siblings_after(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .collect()
    }

    /// Represents the next sibling just after the current node
    async fn next_sibling(&'a self) -> Option<ElementNode<'a>> {
        self.forest
            .siblings_after(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .next()
    }

    /// Represents all of the ancestors of the current node
    async fn ancestors(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .ancestors(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
            .collect()
    }

    /// Represents all of the descendants of the current node
    async fn descendants(&'a self) -> Vec<ElementNode<'a>> {
        self.forest
            .descendants(&self.node)
            .map(|x| ElementNode::from((Arc::clone(&self.forest), x)))
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
