use crate::lang::elements::{Element, Page};
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;

mod tree;
pub use tree::{ElementNode, ElementTree};

/// Represents a collection of `ElementTree` instances and provides an API that
/// executes across all of them. This is useful to maintain the uniqueness of
/// each `ElementTree` as it corresponds to a root-level `Element` while also
/// providing helpful functions across all of them.
#[derive(Clone, Debug, Default, From, Serialize, Deserialize)]
pub struct ElementForest<'a> {
    trees: Vec<ElementTree<'a>>,
}

impl ElementForest<'_> {
    /// Produces a forest that has borrowed its trees (indirectly borrowing
    /// the tree nodes)
    pub fn to_borrowed(&self) -> ElementForest {
        ElementForest {
            trees: self.trees().map(ElementTree::to_borrowed).collect(),
        }
    }

    /// Produces a fully-copied forest that owns all trees, nodes, and data within
    pub fn into_owned(self) -> ElementForest<'static> {
        ElementForest {
            trees: self.into_trees().map(ElementTree::into_owned).collect(),
        }
    }
}

impl<'a> ElementForest<'a> {
    /// Returns an iterator over a slice of `ElementTree` instances
    pub fn trees(&self) -> impl Iterator<Item = &ElementTree<'a>> {
        self.trees.iter()
    }

    /// Consumes the forest and returns an iterator over the trees
    pub fn into_trees(self) -> impl Iterator<Item = ElementTree<'a>> {
        self.trees.into_iter()
    }

    /// Returns an iterator over pairs of references to trees and their
    /// respective nodes
    pub fn trees_and_nodes(
        &self,
    ) -> impl Iterator<Item = (&ElementTree<'a>, &ElementNode<'a>)> {
        self.trees()
            .flat_map(|tree| tree.nodes().map(move |node| (tree, node)))
    }

    /// Finds the first tree in the forest to match the given predicate and
    /// returns the tree. If no match is found, none is returned
    pub fn find_tree<P>(&self, predicate: P) -> Option<&ElementTree<'a>>
    where
        P: Fn(&ElementTree<'a>) -> bool,
    {
        self.trees().find(|tree| predicate(*tree))
    }

    /// Finds the first node in the forest to match the given predicate and
    /// returns both the node and its tree. If no match is found, none is
    /// returned
    pub fn find_node<P>(
        &self,
        predicate: P,
    ) -> Option<(&ElementTree<'a>, &ElementNode<'a>)>
    where
        P: Fn(&ElementTree<'a>, &ElementNode<'a>) -> bool,
    {
        self.trees_and_nodes()
            .find(|(tree, node)| predicate(tree, node))
    }

    /// Finds the first tree in the forest whose region contains the
    /// given offset, or returns none if no tree in the entire forest has
    /// a region containing the given offset
    pub fn find_tree_at_offset(
        &self,
        offset: usize,
    ) -> Option<&ElementTree<'a>> {
        self.find_tree(|tree| tree.root().contains_offset(offset))
    }

    /// Finds the deepest node in the first tree where both tree and node
    /// contain the given offset, or returns none if no element in the
    /// entire forest has a region containing the given offset
    pub fn find_node_at_offset(
        &'a self,
        offset: usize,
    ) -> Option<(&'a ElementTree<'a>, &'a ElementNode<'a>)> {
        self.find_tree_at_offset(offset).and_then(|tree| {
            match tree.find_at_offset(offset) {
                Some(node) => Some((tree, node)),
                _ => None,
            }
        })
    }
}

impl<'a> From<&'a Page<'a>> for ElementForest<'a> {
    /// Borrows the page and then constructs a forest from the trees produced
    /// for each of the page's top-level elements
    fn from(page: &'a Page<'a>) -> Self {
        Self::from(page.to_borrowed())
    }
}

impl<'a> From<Page<'a>> for ElementForest<'a> {
    /// Constructs a forest from the trees produced for each of the page's
    /// top-level elements
    fn from(page: Page<'a>) -> Self {
        page.elements
            .into_iter()
            .map(|x| ElementTree::from(x.map(Element::from)))
            .collect()
    }
}

impl<'a> FromIterator<ElementTree<'a>> for ElementForest<'a> {
    /// Converts the iterator of trees into a forest
    fn from_iter<I: IntoIterator<Item = ElementTree<'a>>>(iter: I) -> Self {
        Self {
            trees: iter.into_iter().collect(),
        }
    }
}
