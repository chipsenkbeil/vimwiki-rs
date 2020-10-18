use crate::elements::*;
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Represents a tree structure for some `Element` and all of its decendents.
///
/// An `ElementTree` will maintaing references to generic `Element` instances,
/// borrowing where possible to maintain an easily-traversable structure that
/// can be used to search for `Element` instances by their `Region` as well
/// as provide means to move up and down levels of elements via their
/// parent and children references.
#[derive(Clone, Debug)]
struct ElementTree<'a> {
    /// Internal mapping of all tree node ids to their data, enabling us to
    /// more easily link nodes together
    nodes: Rc<RefCell<HashMap<usize, ElementTree<'a>>>>,

    /// Used to determine uniqueness of location in tree
    id: usize,

    /// Optional parent; if not present, this is the root of the tree
    parent: Option<usize>,

    /// Element contained within this node in the tree
    element: &'a Element<'a>,

    /// Region associated with this node in the tree
    region: Region,

    /// Children found below this node in the tree
    children: Vec<usize>,
}

impl<'a> ElementTree<'a> {
    /// Build an `ElementTree` from the given `Element` reference and its
    /// associated region. The newly-created `ElementTree` instance represents
    /// the root node in the tree.
    pub fn build_from(element: &'a Element<'a>, region: Region) -> Self {
        #[inline]
        fn new_id(counter: &AtomicUsize) -> usize {
            counter.fetch_add(1, Ordering::Relaxed)
        }

        fn inner<'a>(
            nodes: Rc<RefCell<HashMap<usize, ElementTree<'a>>>>,
            id: usize,
            counter: &AtomicUsize,
            element: &'a Element<'a>,
            region: Region,
            parent: Option<usize>,
        ) -> ElementTree<'a> {
            let children = Vec::new();
            for located_child in element.to_children().iter() {
                let child_id = new_id(counter);
                let region = located_child.region();
                let child_tree = inner(
                    Rc::clone(&nodes),
                    child_id,
                    counter,
                    located_child.as_inner(),
                    region,
                    Some(id),
                );
                nodes.borrow_mut().insert(child_id, child_tree);
                children.push(child_id);
            }

            ElementTree {
                nodes: Rc::clone(&nodes),
                id,
                parent,
                element,
                region,
                children,
            }
        }

        let nodes = Rc::new(RefCell::new(HashMap::new()));
        let counter = AtomicUsize::new(0);

        inner(nodes, new_id(&counter), &counter, element, region, None)
    }

    /// Whether or not this tree node represents the root of the tree
    #[inline]
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Whether or not this tree node represents a leaf in the tree, meaning
    /// that there are no children nodes underneath this one
    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Whether or not this tree node's region contains the given offset
    #[inline]
    pub fn contains_offset(&self, offset: usize) -> bool {
        self.region.contains(offset)
    }

    /// Returns a copy of the region associated with this node's element
    #[inline]
    pub fn region(&self) -> Region {
        self.region
    }

    /// Converts to the underlying reference to the element at this point
    /// in the tree
    #[inline]
    pub fn as_element(&'a self) -> &'a Element<'a> {
        self.element
    }

    /// Converts to parent tree node, if has one
    pub fn as_parent(&'a self) -> Option<&'a ElementTree<'a>> {
        self.parent.and_then(|id| self.nodes.borrow().get(&id))
    }

    /// Converts to root tree node, which can either return some ancestor
    /// of this node if there is one or this node itself if this node is
    /// the root
    pub fn as_root(&'a self) -> &'a ElementTree<'a> {
        self.as_parent().unwrap_or(self)
    }

    /// Converts to immediate children tree nodes
    pub fn to_children(&'a self) -> Vec<&'a ElementTree<'a>> {
        self.children
            .iter()
            .flat_map(|id| self.nodes.borrow().get(id))
            .collect()
    }

    /// Converts to sibling tree nodes (not including self)
    pub fn to_siblings(&'a self) -> Vec<&'a ElementTree<'a>> {
        self.as_parent()
            .map(|p| {
                p.to_children()
                    .into_iter()
                    .filter(|c| c.id != self.id)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Finds the deepest node in the tree whose region contains the
    /// given offset, or returns none if no element in the tree has
    /// a region containing the given offset
    pub fn find_at_offset(
        &'a self,
        offset: usize,
    ) -> Option<&'a ElementTree<'a>> {
        // Each of the children to see if they contain the
        // offset, then work our way back up if none of them do
        self.to_children()
            .into_iter()
            .find_map(|t| t.find_at_offset(offset))
            .or_else(|| {
                if self.contains_offset(offset) {
                    Some(self)
                } else {
                    None
                }
            })
    }
}

impl<'a> From<&'a Located<Element<'a>>> for ElementTree<'a> {
    fn from(located: &'a Located<Element<'a>>) -> Self {
        Self::from(located.as_ref())
    }
}

impl<'a> From<Located<&'a Element<'a>>> for ElementTree<'a> {
    fn from(located: Located<&'a Element<'a>>) -> Self {
        let region = located.region();
        Self::build_from(located.into_inner(), region)
    }
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
        let tree = ElementTree::from(element.as_ref());

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
        let tree = ElementTree::from(element.as_ref());

        assert_eq!(tree.find_at_offset(999), None);
    }

    #[test]
    fn as_root_should_return_reference_to_root_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Get a child at the very bottom of paragraph -> bold -> text
        let node = tree.find_at_offset(4).expect("Failed to find node");

        // Verify root node loaded (this is the paragraph)
        let root = node.as_root();

        // Root node should have tree's root id
        assert_eq!(tree.id, root.id, "Unexpected root: {:?}", root);

        // Verify the actual element to be safe
        assert!(
            matches!(
                root.as_element()
                    .as_block_element()
                    .expect("Didn't find block element"),
                BlockElement::Paragraph(_)
            ),
            "Unexpected element: {:?}",
            root.as_element()
        );
    }

    #[test]
    fn as_root_should_return_reference_to_self_if_is_root_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Load the root node
        let root = tree.as_root();

        // Root node should have tree's root id
        assert_eq!(tree.id, root.id, "Unexpected root: {:?}", root);
    }

    #[test]
    fn as_parent_should_return_parent_tree_node_of_given_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Get a child at the very bottom of paragraph -> bold -> text
        let node = tree.find_at_offset(4).expect("Failed to find node");

        // Verify parent node loaded (this is the bold text container)
        let parent = node.as_parent().expect("Failed to get parent");

        // Check that we loaded the right element
        assert!(
            matches!(
                parent
                    .as_element()
                    .as_inline_element()
                    .expect("Didn't find inline element"),
                InlineElement::DecoratedText(_)
            ),
            "Unexpected element: {:?}",
            parent.as_element()
        );
    }

    #[test]
    fn as_parent_should_return_none_if_given_tree_node_is_root() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        assert_eq!(tree.as_parent(), None);
    }

    #[test]
    fn to_children_should_return_all_children_tree_nodes_of_given_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Load paragraph children, which should be text and bold text
        let children = tree
            .to_children()
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
    fn to_siblings_should_return_all_sibling_tree_nodes_of_given_tree_node() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Get paragraph -> text, which has a sibling of paragraph -> bold text
        let node = tree.find_at_offset(0).expect("Failed to find node");

        let siblings = node
            .to_siblings()
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

    #[test]
    fn is_root_should_return_true_if_tree_node_represents_root_of_tree() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        assert!(tree.is_root());
    }

    #[test]
    fn is_root_should_return_false_if_tree_node_does_not_represent_root_of_tree(
    ) {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        let node = tree.find_at_offset(0).expect("Failed to find node");

        assert!(!node.is_root());
    }

    #[test]
    fn is_leaf_should_return_true_if_tree_node_has_no_children() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Paragraph -> Text has no children
        let node = tree.find_at_offset(0).expect("Failed to find node");
        assert!(node.is_leaf());

        // Paragraph -> Bold -> Text has no children
        let node = tree.find_at_offset(4).expect("Failed to find node");
        assert!(node.is_leaf());
    }

    #[test]
    fn is_leaf_should_return_false_if_tree_node_has_children() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());

        // Paragraph has children
        assert!(!tree.is_leaf());

        // Paragraph -> Bold has children
        let node = tree.find_at_offset(3).expect("Failed to find node");
        assert!(!node.is_leaf());
    }

    #[test]
    fn region_should_return_region_of_underlying_element() {
        let element = test_element();
        let tree = ElementTree::from(element.as_ref());
        assert_eq!(tree.region(), Region::from(0..9));

        let node = tree.find_at_offset(0).expect("Failed to find node");
        assert_eq!(node.region(), Region::from(0..3));
    }
}
