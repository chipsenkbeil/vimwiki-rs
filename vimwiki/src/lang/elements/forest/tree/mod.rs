use crate::elements::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

mod node;
pub use node::ElementNode;

/// Alias to the storage used to maintain tree nodes
type TreeNodeStore<'a> = HashMap<usize, ElementNode<'a>>;

/// Represents a tree structure for some `Element` and all of its decendents.
///
/// An `ElementTree` will maintain references to generic `Element` instances,
/// borrowing where possible to maintain an easily-traversable structure that
/// can be used to search for `Element` instances by their `Region` as well
/// as provide means to move up and down levels of elements via their
/// parent and children references.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ElementTree<'a> {
    /// Internal storage of all nodes within the tree
    nodes: TreeNodeStore<'a>,

    /// Id of the root node in the tree
    root_id: usize,
}

impl ElementTree<'_> {
    /// Produces a tree that has borrowed node values from this tree
    pub fn to_borrowed(&self) -> ElementTree {
        ElementTree {
            nodes: self
                .nodes
                .iter()
                .map(|(id, node)| (*id, node.to_borrowed()))
                .collect(),
            root_id: self.root_id,
        }
    }

    /// Produces a fully-copied tree that owns all nodes and data within
    pub fn into_owned(self) -> ElementTree<'static> {
        ElementTree {
            nodes: self
                .nodes
                .into_iter()
                .map(|(id, node)| (id, node.into_owned()))
                .collect(),
            root_id: self.root_id,
        }
    }
}

impl<'a> ElementTree<'a> {
    /// Returns a reference to the root node of the tree
    pub fn root(&'a self) -> &'a ElementNode<'a> {
        self.nodes
            .get(&self.root_id)
            .expect("Root of tree is missing")
    }

    /// Iterates over all nodes contained within the tree in arbitrary order
    pub fn nodes(&self) -> impl Iterator<Item = &ElementNode<'a>> {
        self.nodes.values()
    }

    /// Iterates over all ancestors for given node by moving up one parent at
    /// a time, stopping after the root node is reached
    pub fn ancestors(
        &'a self,
        node: &'a ElementNode<'a>,
    ) -> impl Iterator<Item = &'a ElementNode<'a>> {
        let mut curr_node = node;
        std::iter::from_fn(move || {
            if let Some(node) = self.parent(curr_node) {
                curr_node = node;
                Some(curr_node)
            } else {
                None
            }
        })
    }

    /// Gets parent for given node
    pub fn parent(
        &'a self,
        node: &'a ElementNode<'a>,
    ) -> Option<&'a ElementNode<'a>> {
        node.parent.and_then(|id| self.nodes.get(&id))
    }

    /// Iterates over all descendants for given node by moving down one level
    /// of children at a time via breadth-first traversal
    pub fn descendants(
        &'a self,
        node: &'a ElementNode<'a>,
    ) -> impl Iterator<Item = &'a ElementNode<'a>> {
        let mut queue = self.children(node);
        std::iter::from_fn(move || {
            if !queue.is_empty() {
                let node = queue.remove(0);
                queue.append(&mut self.children(node));
                Some(node)
            } else {
                None
            }
        })
    }

    /// Represents all children for given node
    pub fn children(
        &'a self,
        node: &'a ElementNode<'a>,
    ) -> Vec<&'a ElementNode<'a>> {
        node.children
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Represents all sibling tree nodes (not including self)
    pub fn siblings(
        &'a self,
        node: &'a ElementNode<'a>,
    ) -> Vec<&'a ElementNode<'a>> {
        let id = node.id;
        self.parent(node)
            .iter()
            .flat_map(|n| self.children(n))
            .filter(|n| n.id != id)
            .collect()
    }

    /// Finds the deepest node in the tree whose region contains the
    /// given offset, or returns none if no element in the tree has
    /// a region containing the given offset
    pub fn find_at_offset(
        &'a self,
        offset: usize,
    ) -> Option<&'a ElementNode<'a>> {
        self._find_at_offset(self.root(), offset, 0).map(|x| x.1)
    }

    /// Finds the deepest node that supports the given offset
    fn _find_at_offset(
        &'a self,
        node: &'a ElementNode<'a>,
        offset: usize,
        depth: usize,
    ) -> Option<(usize, &'a ElementNode<'a>)> {
        if node.contains_offset(offset) {
            if let Some((depth, child)) = self
                .children(node)
                .into_iter()
                .filter_map(|n| self._find_at_offset(n, offset, depth + 1))
                .max_by_key(|(depth, _)| *depth)
            {
                Some((depth, child))
            } else {
                Some((depth, node))
            }
        } else {
            None
        }
    }
}

impl<'a> From<&'a Located<Element<'a>>> for ElementTree<'a> {
    /// Builds a new tree using the provided located element as the root. This
    /// will involving cloning data, although the tree will maintain any
    /// borrowed elements.
    ///
    /// This will first convert the provided referenced located element into
    /// a borrowed form for use in this tree.
    fn from(located: &'a Located<Element<'a>>) -> Self {
        Self::from(located.as_ref())
    }
}

impl<'a> From<Located<&'a Element<'a>>> for ElementTree<'a> {
    /// Builds a new tree using the provided located element as the root. This
    /// will involving cloning data, although the tree will maintain any
    /// borrowed elements.
    ///
    /// This will first convert the provided element reference into a borrowed
    /// form for use in this tree.
    fn from(located: Located<&'a Element<'a>>) -> Self {
        Self::from(located.map(Element::to_borrowed))
    }
}

impl<'a> From<Located<Element<'a>>> for ElementTree<'a> {
    /// Builds a new tree using the provided located element as the root. This
    /// will involving cloning data, although the tree will maintain any
    /// borrowed elements.
    fn from(located: Located<Element<'a>>) -> Self {
        let counter = AtomicUsize::new(0);
        let mut nodes = HashMap::new();

        let root_id = make_nodes(&counter, None, &mut nodes, located);

        ElementTree { nodes, root_id }
    }
}

/// Builds out the ids for a node without creating the node itself
fn make_nodes<'a>(
    counter: &AtomicUsize,
    parent: Option<usize>,
    nodes: &mut TreeNodeStore<'a>,
    located_element: Located<Element<'a>>,
) -> usize {
    // First, generate the id used for both the node and its data and store
    // the data into our data storage
    let id = counter.fetch_add(1, Ordering::Relaxed);

    // Second, process all children of the given data and add as nodes,
    // retaining their ids for use in the node being built
    //
    // NOTE: We have to clone the located element so we can convert it into
    // its children. If the element contains borrowed data, this will maintain
    // the borrowed data; however, if the element is owned, this will copy
    // the entire element structure
    let region = located_element.region();
    let element = located_element.into_inner();
    let children = element
        .clone()
        .into_children()
        .into_iter()
        .map(|child| make_nodes(counter, Some(id), nodes, child))
        .collect();

    // Third, construct the node mapping (without data) and insert it into
    // the node storage
    let node = ElementNode {
        id,
        parent,
        children,
        data: Located::new(element, region),
    };

    nodes.insert(id, node);

    id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_element() -> Located<Element<'static>> {
        // Representing
        //
        // abc*bold*
        // | |||  ||
        // 0 |3|  7|
        //   2 4   8
        //
        Located::new(
            Element::from(Paragraph::from(vec![
                Located::new(
                    InlineElement::from(Text::from("abc")),
                    Region::from(0..3),
                ),
                Located::new(
                    InlineElement::from(DecoratedText::Bold(vec![
                        Located::new(
                            Text::from("bold").into(),
                            Region::from(4..8),
                        ),
                    ])),
                    Region::from(3..9),
                ),
            ])),
            Region::from(0..9),
        )
    }

    #[test]
    fn find_at_offset_should_return_deepest_tree_node_possible() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        // Cursor on top of bold text in paragraph
        let node = tree.find_at_offset(4).expect("Failed to find node");
        assert_eq!(
            node.as_element()
                .as_inline_element()
                .expect("Didn't find inline element")
                .to_string(),
            "bold"
        );
    }

    #[test]
    fn find_at_offset_should_return_none_if_no_tree_node_is_found() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        assert!(tree.find_at_offset(999).is_none());
    }

    #[test]
    fn root_should_return_reference_to_root_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        // Verify root node loaded (this is the paragraph)
        let root = tree.root();
        let root_element = root.as_element();

        // Verify the actual element to be safe
        assert!(
            matches!(
                root_element
                    .as_block_element()
                    .expect("Didn't find block element"),
                BlockElement::Paragraph(_)
            ),
            "Unexpected element: {:?}",
            root_element
        );
    }

    #[test]
    fn parent_should_return_parent_tree_node_of_given_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        // Get a child at the very bottom of paragraph -> bold -> text
        let node = tree.find_at_offset(4).expect("Failed to find node");

        // Verify parent node loaded (this is the bold text container)
        let parent = tree.parent(node).expect("Failed to get parent");
        let parent_element = parent.as_element();

        // Check that we loaded the right element
        assert!(
            matches!(
                parent_element
                    .as_inline_element()
                    .expect("Didn't find inline element"),
                InlineElement::DecoratedText(_)
            ),
            "Unexpected element: {:?}",
            parent_element
        );
    }

    #[test]
    fn parent_should_return_none_if_given_tree_node_is_root() {
        let element = test_element();
        let tree = ElementTree::from(&element);
        let root = tree.root();

        assert!(tree.parent(root).is_none());
    }

    #[test]
    fn ancestors_should_return_iterator_through_all_ancestor_nodes_in_order() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        // Get a child at the very bottom of paragraph -> bold -> text
        let node = tree.find_at_offset(4).expect("Failed to find node");

        // Verify parent node loaded (this is the bold text container)
        let mut it = tree.ancestors(node);

        let ancestor = it.next().expect("Missing first ancestor");
        assert!(
            matches!(
                ancestor
                    .as_element()
                    .as_inline_element()
                    .expect("Didn't find inline element"),
                InlineElement::DecoratedText(_)
            ),
            "Unexpected element: {:?}",
            ancestor.as_element()
        );

        let ancestor = it.next().expect("Missing second ancestor");
        assert!(
            matches!(
                ancestor
                    .as_element()
                    .as_block_element()
                    .expect("Didn't find block element"),
                BlockElement::Paragraph(_)
            ),
            "Unexpected element: {:?}",
            ancestor.as_element()
        );

        assert!(it.next().is_none(), "Unexpectedly got an extra ancestor");
    }

    #[test]
    fn children_should_return_all_children_tree_nodes_of_given_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        // Load paragraph children, which should be text and bold text
        let children = tree
            .children(tree.root())
            .into_iter()
            .map(|node| node.as_element().clone())
            .collect::<Vec<Element<'_>>>();

        assert_eq!(
            children,
            vec![
                Element::from(Text::from("abc")),
                Element::from(DecoratedText::Bold(vec![Located::new(
                    Text::from("bold").into(),
                    Region::from(4..8),
                )])),
            ]
        );
    }

    #[test]
    fn descendants_should_return_iterator_through_all_descendants_one_level_at_a_time(
    ) {
        let element = test_element();
        let tree = ElementTree::from(&element);
        let mut it = tree.descendants(tree.root());

        let descendant = it.next().expect("Missing first descendant");
        assert_eq!(
            descendant.as_element().clone(),
            Element::from(Text::from("abc"))
        );

        let descendant = it.next().expect("Missing second descendant");
        assert_eq!(
            descendant.as_element().clone(),
            Element::from(DecoratedText::Bold(vec![Located::new(
                Text::from("bold").into(),
                Region::from(4..8),
            )])),
        );

        let descendant = it.next().expect("Missing third descendant");
        assert_eq!(descendant.as_element().clone(), Text::from("bold").into());

        assert!(it.next().is_none(), "Unexpectedly got an extra descendant");
    }

    #[test]
    fn siblings_should_return_all_sibling_tree_nodes_of_given_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(&element);

        // Get paragraph -> text, which has a sibling of paragraph -> bold text
        let node = tree.find_at_offset(0).expect("Failed to find node");

        let siblings = tree
            .siblings(node)
            .into_iter()
            .map(|node| node.as_element().clone())
            .collect::<Vec<Element<'_>>>();

        assert_eq!(
            siblings,
            vec![Element::from(DecoratedText::Bold(vec![Located::new(
                Text::from("bold").into(),
                Region::from(4..8),
            )]))],
        );
    }
}
