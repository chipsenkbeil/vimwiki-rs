use crate::{elements::*, Region};

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

#[derive(Clone, Debug)]
pub struct ElementNode<'a> {
    page: &'a Page,
    root: &'a BlockElement,
    parent: Option<&'a ElementNode<'a>>,
    element: ElementRef<'a>,
    region: Region,
    children: Vec<ElementNode<'a>>,
}

impl<'a> ElementNode<'a> {
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    pub fn root(&self) -> &'a BlockElement {
        self.root
    }

    pub fn parent(&self) -> Option<&'a Self> {
        self.parent
    }

    pub fn region(&self) -> &Region {
        &self.region
    }

    /// Constructs a series of `ElementNode` based on the top-level elements
    /// within the provided page
    pub fn from_page(page: &'a Page) -> Vec<ElementNode<'a>> {
        page.elements
            .iter()
            .map(|e| {
                Self::from_block_element(
                    page,
                    e.as_inner(),
                    None,
                    e.as_inner(),
                    e.region,
                )
            })
            .collect()
    }

    fn from_block_element(
        page: &'a Page,
        root: &'a BlockElement,
        parent: Option<&'a ElementNode<'a>>,
        element: &'a BlockElement,
        region: Region,
    ) -> Self {
        Self {
            page,
            root,
            parent,
            element: ElementRef::Block(element),
            region,
            children: match element {
                BlockElement::Blockquote(_) => vec![],
                BlockElement::DefinitionList(x) => vec![],
                BlockElement::Divider(_) => vec![],
                BlockElement::Header(x) => vec![],
                BlockElement::List(x) => vec![],
                BlockElement::Math(_) => vec![],
                BlockElement::Paragraph(x) => vec![],
                BlockElement::Placeholder(_) => vec![],
                BlockElement::PreformattedText(_) => vec![],
                BlockElement::Table(x) => vec![],
            },
        }
    }

    fn from_inline_element(
        page: &'a Page,
        root: &'a BlockElement,
        parent: Option<&'a ElementNode<'a>>,
        element: &'a InlineElement,
        region: Region,
    ) -> Self {
        Self {
            page,
            root,
            parent,
            element: ElementRef::Inline(element),
            region,
            children: match element {
                InlineElement::Code(_) => vec![],
                InlineElement::DecoratedText(x) => x
                    .as_contents()
                    .iter()
                    .map(|c| {
                        Self::from_inline_element(
                            page,
                            root,
                            // TODO: How do we get a reference to the value
                            //       returned by this function
                            None,
                            c.element.as_inline_element(),
                            c.region,
                        )
                    })
                    .collect(),
                InlineElement::Keyword(_) => vec![],
                InlineElement::Link(_) => vec![],
                InlineElement::Math(_) => vec![],
                InlineElement::Tags(_) => vec![],
                InlineElement::Text(_) => vec![],
            },
        }
    }
}
