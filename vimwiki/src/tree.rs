use crate::elements::*;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Debug)]
pub struct ElementTree<'a> {
    root: Rc<RefCell<ElementTreeNode<'a>>>,
}

#[derive(Clone, Debug)]
struct ElementTreeNode<'a> {
    /// Used to determine uniqueness of location in tree
    id: usize,

    /// Optional parent; if not present, this is the root of the tree
    parent: Option<Rc<RefCell<ElementTreeNode<'a>>>>,

    /// Element contained within this node in the tree
    element: Rc<Element<'a>>,

    /// Region associated with this node in the tree
    region: Region,

    /// Children found below this node in the tree
    children: Vec<Rc<RefCell<ElementTreeNode<'a>>>>,
}

impl<'a> ElementTreeNode<'a> {
    pub fn from(
        element: impl Into<Element<'a>>,
        region: Region,
    ) -> Rc<RefCell<Self>> {
        fn inner<'a>(
            counter: &AtomicUsize,
            element: Rc<Element<'a>>,
            region: Region,
            parent: Option<Rc<RefCell<ElementTreeNode<'a>>>>,
        ) -> Rc<RefCell<ElementTreeNode<'a>>> {
            let id = counter.fetch_add(1, Ordering::Relaxed);

            let node = Rc::new(RefCell::new(ElementTreeNode {
                id,
                parent,
                element: Rc::clone(&element),
                region,
                children: vec![],
            }));

            for located_child in
                node.borrow().as_element().to_children().into_iter()
            {
                let region = located_child.region;
                let child_node = inner(
                    counter,
                    Rc::new(located_child.into_inner()),
                    region,
                    Some(Rc::clone(&node)),
                );
                node.borrow_mut().children.push(child_node);
            }

            Rc::clone(&node)
        }

        inner(&AtomicUsize::new(0), Rc::new(element.into()), region, None)
    }

    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn as_element(&self) -> &Element<'a> {
        &self.element
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
