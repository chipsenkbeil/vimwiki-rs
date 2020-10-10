use crate::{elements::*, Position, Region};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Debug)]
pub enum ElementRef<'a> {
    Block(&'a BlockElement),
    Inline(&'a InlineElement),
}

impl<'a> ElementRef<'a> {
    pub fn is_block_element(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn is_inline_element(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    pub fn as_block_element(&self) -> Option<&'a BlockElement> {
        match self {
            Self::Block(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn as_inline_element(&self) -> Option<&'a InlineElement> {
        match self {
            Self::Inline(ref x) => Some(x),
            _ => None,
        }
    }
}

/// Represents an immutable tree containing references to elements within a page
#[derive(Clone, Debug)]
pub struct ElementTree<'a> {
    page: &'a Page,
    root_nodes: Vec<usize>,
    nodes: HashMap<usize, ElementNode<'a>>,
}

impl<'a> ElementTree<'a> {
    /// Default id for situations where a node is required but there is no node
    const EMPTY_NODE: usize = 0;

    /// Reference to the page whose elements this tree points to
    pub fn page(&self) -> &Page {
        self.page
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

    /// Constructs a tree based on the top-level elements
    /// within the provided page
    pub fn from_page(page: &'a Page) -> ElementTree<'a> {
        let mut instance = Self {
            page,
            root_nodes: vec![],
            nodes: HashMap::new(),
        };

        let counter = AtomicUsize::new(Self::EMPTY_NODE + 1);
        for element in page.elements.iter() {
            let id = instance.add_block_element(
                &counter,
                Self::EMPTY_NODE,
                None,
                element.as_inner(),
                element.region,
            );
            instance.root_nodes.push(id);
        }

        instance
    }

    /// Adds a new node to the tree that is a `BlockElement` reference,
    /// returning the id of the newly-added node
    fn add_block_element(
        &mut self,
        counter: &AtomicUsize,
        root_id: usize,
        parent_id: Option<usize>,
        element: &'a BlockElement,
        region: Region,
    ) -> usize {
        let element_id = counter.fetch_add(1, Ordering::Relaxed);

        // If provided a root id that is nothing, this indicates that we are
        // the root and should therefore use our element's id
        let root_id = if root_id != Self::EMPTY_NODE {
            root_id
        } else {
            element_id
        };

        let node = ElementNode {
            root_id,
            parent_id,
            element_id,
            element: ElementRef::Block(element),
            region,
            children_ids: match element {
                BlockElement::DefinitionList(x) => x
                    .iter()
                    .flat_map(|td| {
                        let mut ids = self.add_inline_elements_from_container(
                            counter,
                            root_id,
                            Some(element_id),
                            &td.term,
                        );

                        let mut def_ids = td
                            .definitions
                            .iter()
                            .flat_map(|d| {
                                self.add_inline_elements_from_container(
                                    counter,
                                    root_id,
                                    Some(element_id),
                                    &d,
                                )
                            })
                            .collect();
                        ids.append(&mut def_ids);

                        ids
                    })
                    .collect(),
                BlockElement::Header(x) => self
                    .add_inline_elements_from_container(
                        counter,
                        root_id,
                        Some(element_id),
                        &x.content,
                    ),
                BlockElement::List(x) => x
                    .items
                    .iter()
                    .flat_map(|item| {
                        item.as_inner()
                            .contents
                            .iter()
                            .flat_map(|c| match c.as_inner() {
                                ListItemContent::InlineContent(x) => self
                                    .add_inline_elements_from_container(
                                        counter,
                                        root_id,
                                        Some(element_id),
                                        &x,
                                    ),
                                ListItemContent::List(x) => vec![self
                                    .add_block_element(
                                        counter,
                                        root_id,
                                        Some(element_id),
                                        x.as_inner(),
                                        c.region,
                                    )],
                            })
                            .collect::<Vec<usize>>()
                    })
                    .collect(),
                BlockElement::Paragraph(x) => self
                    .add_inline_elements_from_container(
                        counter,
                        root_id,
                        Some(element_id),
                        &x.content,
                    ),
                BlockElement::Table(x) => x
                    .rows
                    .iter()
                    .flat_map(|r| match r.as_inner() {
                        Row::Content { cells } => cells
                            .iter()
                            .flat_map(|c| match c.as_inner() {
                                Cell::Content(x) => self
                                    .add_inline_elements_from_container(
                                        counter,
                                        root_id,
                                        Some(element_id),
                                        &x,
                                    ),
                                _ => vec![],
                            })
                            .collect(),
                        _ => vec![],
                    })
                    .collect(),
                _ => vec![],
            },
        };

        self.nodes.insert(element_id, node);
        element_id
    }

    /// Adds new nodes to the tree, one for each `InlineElement` reference
    /// held within the provided container, returning the ids of the
    /// newly-added nodes
    fn add_inline_elements_from_container(
        &mut self,
        counter: &AtomicUsize,
        root_id: usize,
        parent_id: Option<usize>,
        container: &'a InlineElementContainer,
    ) -> Vec<usize> {
        let mut ids = Vec::with_capacity(container.elements.len());
        for e in container.elements.iter() {
            ids.push(self.add_inline_element(
                counter,
                root_id,
                parent_id,
                e.as_inner(),
                e.region,
            ));
        }
        ids
    }

    /// Adds a new node to the tree that is an `InlineElement` reference,
    /// returning the id of the newly-added node
    fn add_inline_element(
        &mut self,
        counter: &AtomicUsize,
        root_id: usize,
        parent_id: Option<usize>,
        element: &'a InlineElement,
        region: Region,
    ) -> usize {
        let element_id = counter.fetch_add(1, Ordering::Relaxed);

        let node = ElementNode {
            root_id,
            parent_id,
            element_id,
            element: ElementRef::Inline(element),
            region,
            children_ids: match element {
                InlineElement::DecoratedText(x) => x
                    .as_contents()
                    .iter()
                    .map(|c| {
                        self.add_inline_element(
                            counter,
                            root_id,
                            Some(element_id),
                            c.element.as_inline_element(),
                            c.region,
                        )
                    })
                    .collect(),
                _ => vec![],
            },
        };

        self.nodes.insert(element_id, node);
        element_id
    }
}

#[derive(Clone, Debug)]
pub struct ElementNode<'a> {
    root_id: usize,
    parent_id: Option<usize>,
    element_id: usize,
    element: ElementRef<'a>,
    region: Region,
    children_ids: Vec<usize>,
}

impl<'a> ElementNode<'a> {
    /// Id of node, which maps to the element it references
    pub fn id(&self) -> usize {
        self.element_id
    }

    pub fn is_root(&self) -> bool {
        self.root_id == self.element_id
    }

    pub fn region(&self) -> &Region {
        &self.region
    }
}
