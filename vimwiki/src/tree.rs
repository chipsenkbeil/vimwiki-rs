use crate::elements::*;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

//
// CHIP CHIP CHIP CHIP
//
// This should be rewritten (again) since we've refactored a lot of code
// and it's easier to pass around owned references of elements. It also may
// make sense to have a separate struct for a page tree versus an element
// tree so we don't have a boatload of page references floating around
//
// Make something like this:
//
// pub struct ElementTree<'a> {
//     parent: Option<Box<Tree<'a>>>,
//     element: Element<'a>,
// }
//
// impl<'a> ElementTree<'a> {
//     pub fn from(element: impl Into<Element<'a>>) -> Self {
//         // ...
//     }
// }
//
// impl<'a> From<Header<'a>> for ElementTree<'a> {
//     // ...
// }
//
//

/// Represents an immutable tree containing references to elements within a page
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementTree<'a> {
    page: Page<'a>,
    root_nodes: Vec<usize>,
    nodes: HashMap<usize, ElementNode<'a>>,
}

impl ElementTree<'_> {
    pub fn to_borrowed(&self) -> ElementTree {
        ElementTree {
            page: self.page.to_borrowed(),
            root_nodes: self.root_nodes.clone(),
            nodes: self
                .nodes
                .iter()
                .map(|(key, value)| (*key, value.to_borrowed()))
                .collect(),
        }
    }

    pub fn into_owned(self) -> ElementTree<'static> {
        ElementTree {
            page: self.page.into_owned(),
            root_nodes: self.root_nodes.clone(),
            nodes: self
                .nodes
                .into_iter()
                .map(|(key, value)| (key, value.into_owned()))
                .collect(),
        }
    }
}

impl<'a> ElementTree<'a> {
    /// Default id for situations where a node is required but there is no node
    const EMPTY_NODE: usize = 0;

    /// Borrowed version of the page this tree references
    pub fn page(&self) -> &Page<'a> {
        &self.page
    }

    /// Finds the node deepest in the tree that has a region containing
    /// the specified position
    pub fn find_deepest_at(
        &self,
        position: Position,
    ) -> Option<&ElementNode<'a>> {
        match self.find_root_at(position) {
            Some(root) => {
                let mut curr = root;

                // NOTE: This doesn't check for any cycles within nodes, but
                //       this shouldn't be an issue given this is a tree and
                //       not a graph
                loop {
                    match self
                        .children_for(curr)
                        .iter()
                        .find(|n| n.region().contains(position))
                    {
                        Some(next) => curr = next,
                        _ => break Some(curr),
                    }
                }
            }
            _ => None,
        }
    }

    /// Finds the root node whose region contains the specified position
    pub fn find_root_at(&self, position: Position) -> Option<&ElementNode<'a>> {
        self.root_nodes()
            .iter()
            .find(|n| n.region().contains(position))
            .copied()
    }

    /// Retrieves all of the root-level nodes within the tree
    pub fn root_nodes(&self) -> Vec<&ElementNode<'a>> {
        self.root_nodes
            .iter()
            .flat_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Retrieve the root node for the given node
    pub fn root_for(&self, node: &ElementNode<'a>) -> &ElementNode<'a> {
        self.nodes
            .get(&node.root_id)
            .expect("Tree mutated after construction")
    }

    /// Retrieve the parent node for the given node
    pub fn parent_for(
        &self,
        node: &ElementNode<'a>,
    ) -> Option<&ElementNode<'a>> {
        node.parent_id.and_then(|id| self.nodes.get(&id))
    }

    /// Retrieve the children nodes for the given node
    pub fn children_for<'b>(
        &'b self,
        node: &'b ElementNode<'a>,
    ) -> Vec<&'b ElementNode<'a>> {
        node.children_ids
            .iter()
            .flat_map(|id| self.nodes.get(id))
            .collect()
    }

    /// Retrieve the sibling nodes for the given node (does not include self)
    pub fn siblings_for<'b>(
        &'b self,
        node: &'b ElementNode<'a>,
    ) -> Vec<&'b ElementNode<'a>> {
        let node_id = node.id();

        // Check if we have a parent and, if we do, gather its children to
        // return as siblings; otherwise, we are a root node and need to
        // check against all root nodes instead
        match self.parent_for(node) {
            Some(parent) => self.children_for(parent),
            _ => self.root_nodes(),
        }
        .drain(..)
        .filter(|sibling| sibling.id() != node_id)
        .collect()
    }

    /// Constructs a tree based on the top-level elements
    /// within the provided page
    pub fn from_page(page: Page<'a>) -> ElementTree<'a> {
        let mut instance = Self {
            page,
            root_nodes: vec![],
            nodes: HashMap::new(),
        };

        let counter = AtomicUsize::new(Self::EMPTY_NODE + 1);
        for element in instance.page.elements.iter() {
            let id = add_block_element(
                &mut instance.nodes,
                &counter,
                Self::EMPTY_NODE,
                None,
                element.to_borrowed(),
                element.region,
            );
            instance.root_nodes.push(id);
        }

        instance
    }
}

/// Adds a new node to the tree that is a `BlockElement` reference,
/// returning the id of the newly-added node
fn add_block_element<'a>(
    nodes: &mut HashMap<usize, ElementNode<'a>>,
    counter: &AtomicUsize,
    root_id: usize,
    parent_id: Option<usize>,
    element: BlockElement<'a>,
    region: Region,
) -> usize {
    let element_id = counter.fetch_add(1, Ordering::Relaxed);

    // If provided a root id that is nothing, this indicates that we are
    // the root and should therefore use our element's id
    let root_id = if root_id != ElementTree::EMPTY_NODE {
        root_id
    } else {
        element_id
    };

    let node = ElementNode {
        root_id,
        parent_id,
        element_id,
        element: Element::from(element),
        region,
        children_ids: match element {
            BlockElement::DefinitionList(x) => x
                .iter()
                .flat_map(|(term, defs)| {
                    let mut ids = add_inline_elements_from_container(
                        nodes,
                        counter,
                        root_id,
                        Some(element_id),
                        term.as_inner().to_borrowed(),
                    );

                    let mut def_ids = defs
                        .iter()
                        .flat_map(|d| {
                            add_inline_elements_from_container(
                                nodes,
                                counter,
                                root_id,
                                Some(element_id),
                                d.as_inner().to_borrowed(),
                            )
                        })
                        .collect();
                    ids.append(&mut def_ids);

                    ids
                })
                .collect(),
            BlockElement::Header(x) => add_inline_elements_from_container(
                nodes,
                counter,
                root_id,
                Some(element_id),
                x.content.to_borrowed(),
            ),
            BlockElement::List(x) => x
                .items
                .iter()
                .flat_map(|item| {
                    item.as_inner()
                        .contents
                        .iter()
                        .flat_map(|c| match c.as_inner() {
                            ListItemContent::InlineContent(x) => {
                                add_inline_elements_from_container(
                                    nodes,
                                    counter,
                                    root_id,
                                    Some(element_id),
                                    x.to_borrowed(),
                                )
                            }
                            ListItemContent::List(x) => {
                                vec![add_block_element(
                                    nodes,
                                    counter,
                                    root_id,
                                    Some(element_id),
                                    BlockElement::List(x.to_borrowed()),
                                    c.region,
                                )]
                            }
                        })
                        .collect::<Vec<usize>>()
                })
                .collect(),
            BlockElement::Paragraph(x) => add_inline_elements_from_container(
                nodes,
                counter,
                root_id,
                Some(element_id),
                x.content.to_borrowed(),
            ),
            BlockElement::Table(x) => x
                .rows
                .iter()
                .flat_map(|r| match r.as_inner() {
                    Row::Content { cells } => cells
                        .iter()
                        .flat_map(|c| match c.as_inner() {
                            Cell::Content(x) => {
                                add_inline_elements_from_container(
                                    nodes,
                                    counter,
                                    root_id,
                                    Some(element_id),
                                    x.to_borrowed(),
                                )
                            }
                            _ => vec![],
                        })
                        .collect(),
                    _ => vec![],
                })
                .collect(),
            _ => vec![],
        },
    };

    nodes.insert(element_id, node);
    element_id
}

/// Adds new nodes to the tree, one for each `InlineElement` reference
/// held within the provided container, returning the ids of the
/// newly-added nodes
fn add_inline_elements_from_container<'a>(
    nodes: &mut HashMap<usize, ElementNode<'a>>,
    counter: &AtomicUsize,
    root_id: usize,
    parent_id: Option<usize>,
    container: InlineElementContainer<'a>,
) -> Vec<usize> {
    let mut ids = Vec::with_capacity(container.elements.len());
    for e in container.elements.iter() {
        ids.push(add_inline_element(
            nodes,
            counter,
            root_id,
            parent_id,
            e.to_borrowed(),
            e.region,
        ));
    }
    ids
}

/// Adds a new node to the tree that is an `InlineElement` reference,
/// returning the id of the newly-added node
fn add_inline_element<'a>(
    nodes: &mut HashMap<usize, ElementNode<'a>>,
    counter: &AtomicUsize,
    root_id: usize,
    parent_id: Option<usize>,
    element: InlineElement<'a>,
    region: Region,
) -> usize {
    let element_id = counter.fetch_add(1, Ordering::Relaxed);
    let children_ids = match element.to_borrowed() {
        InlineElement::DecoratedText(x) => add_decorated_text(
            nodes,
            counter,
            root_id,
            Some(element_id),
            x,
            region,
        ),
        _ => vec![],
    };

    let node = ElementNode {
        root_id,
        parent_id,
        element_id,
        element: Element::from(element),
        region,
        children_ids,
    };

    nodes.insert(element_id, node);
    element_id
}

fn add_decorated_text<'a>(
    nodes: &mut HashMap<usize, ElementNode<'a>>,
    counter: &AtomicUsize,
    root_id: usize,
    parent_id: Option<usize>,
    element: DecoratedText<'a>,
    region: Region,
) -> Vec<usize> {
    let mut children = Vec::new();
    for c in element.as_contents().iter() {
        children.push(add_inline_element(
            nodes,
            counter,
            root_id,
            parent_id,
            c.element.to_inline_element(),
            c.region,
        ));
    }
    children
}

/// A node within an `ElementTree` that points to either a `BlockElement` or
/// an `InlineElement`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementNode<'a> {
    root_id: usize,
    parent_id: Option<usize>,
    element_id: usize,
    element: Element<'a>,
    region: Region,
    children_ids: Vec<usize>,
}

impl ElementNode<'_> {
    pub fn to_borrowed(&self) -> ElementNode {
        ElementNode {
            root_id: self.root_id,
            parent_id: self.parent_id,
            element_id: self.element_id,
            element: self.element.to_borrowed(),
            region: self.region,
            children_ids: self.children_ids.clone(),
        }
    }

    pub fn into_owned(self) -> ElementNode<'static> {
        ElementNode {
            root_id: self.root_id,
            parent_id: self.parent_id,
            element_id: self.element_id,
            element: self.element.into_owned(),
            region: self.region,
            children_ids: self.children_ids.clone(),
        }
    }
}

impl<'a> ElementNode<'a> {
    /// Id of node, which maps to the element it references
    pub fn id(&self) -> usize {
        self.element_id
    }

    /// Whether or not the node represents a root-level element
    pub fn is_root(&self) -> bool {
        self.root_id == self.element_id
    }

    /// The region of the element referenced by the node
    pub fn region(&self) -> &Region {
        &self.region
    }

    /// Converts to ref of inner `Element`
    pub fn as_inner(&self) -> &Element<'a> {
        &self.element
    }

    /// Converts to inner `Element`
    pub fn into_inner(self) -> Element<'a> {
        self.element
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elements::Located;

    fn test_page() -> Page<'static> {
        Page::new(
            vec![
                Located::new(
                    BlockElement::from(Divider),
                    Region::from((1, 1, 1, 3)),
                ),
                Located::new(
                    BlockElement::from(Paragraph::from(vec![
                        Located::new(
                            InlineElement::from(Text::from("abc")),
                            Region::from((2, 1, 2, 3)),
                        ),
                        Located::new(
                            InlineElement::from(DecoratedText::Bold(vec![
                                Located::new(
                                    Text::from("bold").into(),
                                    Region::from((2, 4, 2, 7)),
                                ),
                            ])),
                            Region::from((2, 4, 2, 7)),
                        ),
                    ])),
                    Region::from((2, 1, 2, 7)),
                ),
            ],
            vec![],
        )
    }

    #[test]
    fn find_deepest_at_should_return_deepest_node_at_position() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        // Cursor on top of bold text in paragraph
        let node = tree.find_deepest_at(Position::from((2, 4))).unwrap();
        assert_eq!(
            node.to_owned().into_inner(),
            Element::from(match page.elements[1].as_inner() {
                BlockElement::Paragraph(ref x) => match x.content[1].as_inner()
                {
                    InlineElement::DecoratedText(ref x) =>
                        x.as_contents()[0].to_inline_element().to_borrowed(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            })
        );
    }

    #[test]
    fn find_deepest_at_should_return_none_if_no_node_at_position() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        assert_eq!(tree.find_deepest_at(Position::from((999, 999))), None);
    }

    #[test]
    fn find_root_at_should_return_root_node_at_position() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        // Cursor on top of paragraph
        let node = tree.find_root_at(Position::from((2, 4))).unwrap();
        assert_eq!(
            node.to_owned().into_inner(),
            Element::from(page.elements[1].as_inner().to_borrowed())
        );
    }

    #[test]
    fn find_root_at_should_return_none_if_no_root_node_at_position() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        assert_eq!(tree.find_root_at(Position::from((999, 999))), None);
    }

    #[test]
    fn root_nodes_should_return_all_root_level_nodes() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        assert_eq!(
            tree.root_nodes()
                .drain(..)
                .map(|node| node.as_inner().clone())
                .collect::<Vec<Element<'_>>>(),
            vec![
                Element::from(page.elements[0].as_inner().to_borrowed()),
                Element::from(page.elements[1].as_inner().to_borrowed()),
            ],
        );
    }

    #[test]
    fn root_for_should_return_root_of_given_node() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        // Get a child at the very bottom of paragraph -> bold -> text
        let node = tree.find_deepest_at(Position::from((2, 4))).unwrap();

        // Verify root node loaded (this is the paragraph)
        let root = tree.root_for(node);

        assert_eq!(root, tree.root_nodes()[1]);
    }

    #[test]
    fn parent_for_should_return_parent_of_given_node() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        // Get a child at the very bottom of paragraph -> bold -> text
        let node = tree.find_deepest_at(Position::from((2, 4))).unwrap();

        // Verify parent node loaded (this is the bold text container)
        let parent = tree.parent_for(node).expect("Missing parent");

        assert_eq!(parent, tree.children_for(tree.root_nodes()[1])[1]);
    }

    #[test]
    fn parent_for_should_return_none_if_given_node_is_root() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        assert_eq!(tree.parent_for(tree.root_nodes()[1]), None);
    }

    #[test]
    fn children_for_should_return_all_children_of_given_node() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        // Load paragraph children, which should be text and bold text
        let children = tree
            .children_for(tree.root_nodes()[1])
            .drain(..)
            .map(|node| node.as_inner().clone())
            .collect::<Vec<Element<'_>>>();

        assert_eq!(
            children,
            match page.elements[1].as_inner() {
                BlockElement::Paragraph(ref x) => vec![
                    Element::from(x.content[0].as_inner().to_borrowed()),
                    Element::from(x.content[1].as_inner().to_borrowed()),
                ],
                _ => unreachable!(),
            },
        );
    }

    #[test]
    fn siblings_for_should_return_all_siblings_of_given_node() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        // Get paragraph -> text, which has a sibling of paragraph -> bold text
        let node = tree.find_deepest_at(Position::from((2, 2))).unwrap();

        let siblings = tree
            .siblings_for(node)
            .drain(..)
            .map(|node| node.as_inner().clone())
            .collect::<Vec<Element<'_>>>();

        assert_eq!(
            siblings,
            match page.elements[1].as_inner() {
                BlockElement::Paragraph(ref x) =>
                    vec![Element::from(x.content[1].as_inner().to_borrowed())],
                _ => unreachable!(),
            },
        );
    }

    #[test]
    fn siblings_for_should_return_all_root_sibling_nodes_of_given_root_node() {
        let page = test_page();
        let tree = ElementTree::from_page(page);

        let siblings = tree
            .siblings_for(tree.root_nodes()[1])
            .drain(..)
            .map(|node| node.as_inner().clone())
            .collect::<Vec<Element<'_>>>();

        assert_eq!(
            siblings,
            vec![Element::from(page.elements[0].as_inner().to_borrowed())]
        );
    }

    mod node {
        use super::*;

        #[test]
        fn id_should_return_element_id_for_node() {
            let node = ElementNode {
                root_id: 0,
                parent_id: None,
                element_id: 999,
                element: Element::from(BlockElement::Divider(Divider)),
                region: Region::default(),
                children_ids: vec![],
            };

            assert_eq!(node.id(), 999);
        }

        #[test]
        fn is_root_should_return_true_if_node_represents_root_element() {
            let node = ElementNode {
                root_id: 999,
                parent_id: None,
                element_id: 999,
                element: Element::from(BlockElement::Divider(Divider)),
                region: Region::default(),
                children_ids: vec![],
            };

            assert!(node.is_root());
        }

        #[test]
        fn is_root_should_return_false_if_node_does_not_represent_root_element()
        {
            let node = ElementNode {
                root_id: 0,
                parent_id: None,
                element_id: 999,
                element: Element::from(BlockElement::Divider(Divider)),
                region: Region::default(),
                children_ids: vec![],
            };

            assert!(!node.is_root());
        }

        #[test]
        fn region_should_return_region_of_underlying_element() {
            let node = ElementNode {
                root_id: 0,
                parent_id: None,
                element_id: 0,
                element: Element::from(BlockElement::Divider(Divider)),
                region: Region::from((1, 2, 3, 4)),
                children_ids: vec![],
            };

            assert_eq!(*node.region(), Region::from((1, 2, 3, 4)));
        }
    }
}
