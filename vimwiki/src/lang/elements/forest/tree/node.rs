use crate::elements::*;
use serde::{Deserialize, Serialize};

/// Represents a node in an `ElementTree` that points to or contains singular
/// data about some located element.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ElementNode<'a> {
    /// Id of this node
    pub(super) id: usize,

    /// Id of parent node in tree
    pub(super) parent: Option<usize>,

    /// Id of children nodes in tree
    pub(super) children: Vec<usize>,

    /// Located element contained within this node in the tree
    pub(super) data: Located<Element<'a>>,
}

impl ElementNode<'_> {
    /// Produces a node whose inner value is borrowed
    pub fn to_borrowed(&self) -> ElementNode {
        ElementNode {
            id: self.id,
            parent: self.parent,
            children: self.children.clone(),
            data: self.data.as_ref().map(Element::to_borrowed),
        }
    }

    /// Produces a node that has full ownership over its data, usually through
    /// allocating a complete copy
    pub fn into_owned(self) -> ElementNode<'static> {
        ElementNode {
            id: self.id,
            parent: self.parent,
            children: self.children,
            data: self.data.map(Element::into_owned),
        }
    }
}

impl<'a> ElementNode<'a> {
    #[inline]
    pub fn is_root(&'a self) -> bool {
        self.parent.is_none()
    }

    #[inline]
    pub fn is_leaf(&'a self) -> bool {
        self.children.is_empty()
    }

    /// Returns reference to data contained within node
    pub fn as_inner(&'a self) -> &'a Located<Element<'a>> {
        &self.data
    }

    /// Consumes node and returns the inner data
    pub fn into_inner(self) -> Located<Element<'a>> {
        self.data
    }

    /// Returns a copy of the region associated with this node
    pub fn region(&'a self) -> Region {
        self.as_inner().region()
    }

    /// Converts to the underlying reference to the element at this point
    /// in the tree
    pub fn as_element(&'a self) -> &Element<'a> {
        self.as_inner().as_inner()
    }

    /// Consumes node and returns the element contained within
    pub fn into_element(self) -> Element<'a> {
        self.into_inner().into_inner()
    }

    /// Returns whether or not this node's region contains the given offset
    pub fn contains_offset(&'a self, offset: usize) -> bool {
        self.region().contains(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_root_should_return_true_if_tree_node_represents_root_of_tree() {
        let node = ElementNode {
            id: 999,
            parent: None,
            children: vec![],
            data: Located::from(Element::from(Divider)),
        };

        assert!(node.is_root());
    }

    #[test]
    fn is_root_should_return_false_if_tree_node_does_not_represent_root_of_tree(
    ) {
        let node = ElementNode {
            id: 999,
            parent: Some(1000),
            children: vec![],
            data: Located::from(Element::from(Divider)),
        };

        assert!(!node.is_root());
    }

    #[test]
    fn is_leaf_should_return_true_if_tree_node_has_no_children() {
        let node = ElementNode {
            id: 999,
            parent: None,
            children: vec![],
            data: Located::from(Element::from(Divider)),
        };

        assert!(node.is_leaf());
    }

    #[test]
    fn is_leaf_should_return_false_if_tree_node_has_children() {
        let node = ElementNode {
            id: 999,
            parent: None,
            children: vec![1000],
            data: Located::from(Element::from(Divider)),
        };

        assert!(!node.is_leaf());
    }

    #[test]
    fn region_should_return_region_of_underlying_element() {
        let node = ElementNode {
            id: 999,
            parent: None,
            children: vec![],
            data: Located::new(Element::from(Divider), Region::from(3..9)),
        };

        assert_eq!(node.region(), Region::from(3..9));
    }
}
