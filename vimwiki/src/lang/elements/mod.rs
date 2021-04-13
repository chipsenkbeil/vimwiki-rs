#![allow(clippy::large_enum_variant)]

use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod blocks;
pub use blocks::*;
mod location;
pub use location::{Located, Region};

/// Represents a full page containing different elements
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page<'a> {
    /// Comprised of the elements within a page
    pub elements: Vec<Located<BlockElement<'a>>>,
}

impl Page<'_> {
    pub fn to_borrowed(&self) -> Page {
        let elements = self
            .elements
            .iter()
            .map(|x| x.as_ref().map(BlockElement::to_borrowed))
            .collect();

        Page { elements }
    }

    pub fn into_owned(self) -> Page<'static> {
        let elements = self
            .elements
            .into_iter()
            .map(|x| x.map(BlockElement::into_owned))
            .collect();

        Page { elements }
    }
}

/// Represents a `BlockElement`, an `InlineElement`, or one of a handful of
/// special inbetween types like `ListItem`
#[derive(Clone, Debug, From, PartialEq, Eq, Serialize, Deserialize)]
pub enum Element<'a> {
    Block(BlockElement<'a>),
    Inline(InlineElement<'a>),
    InlineBlock(InlineBlockElement<'a>),
}

impl Element<'_> {
    pub fn to_borrowed(&self) -> Element {
        match self {
            Self::Block(x) => Element::Block(x.to_borrowed()),
            Self::Inline(x) => Element::Inline(x.to_borrowed()),
            Self::InlineBlock(x) => Element::InlineBlock(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> Element<'static> {
        match self {
            Self::Block(x) => Element::Block(x.into_owned()),
            Self::Inline(x) => Element::Inline(x.into_owned()),
            Self::InlineBlock(x) => Element::InlineBlock(x.into_owned()),
        }
    }
}

impl<'a> Element<'a> {
    /// Consumes element and returns all children
    pub fn into_children(self) -> Vec<Located<Element<'a>>> {
        match self {
            Self::Block(x) => x.into_children(),
            Self::Inline(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            Self::InlineBlock(x) => x.into_children(),
        }
    }

    pub fn as_block_element(&self) -> Option<&BlockElement<'a>> {
        match self {
            Self::Block(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn into_block_element(self) -> Option<BlockElement<'a>> {
        match self {
            Self::Block(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_inline_element(&self) -> Option<&InlineElement<'a>> {
        match self {
            Self::Inline(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn into_inline_element(self) -> Option<InlineElement<'a>> {
        match self {
            Self::Inline(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_inline_block_element(&self) -> Option<&InlineBlockElement<'a>> {
        match self {
            Self::InlineBlock(ref x) => Some(x),
            _ => None,
        }
    }

    pub fn into_inline_block_element(self) -> Option<InlineBlockElement<'a>> {
        match self {
            Self::InlineBlock(x) => Some(x),
            _ => None,
        }
    }
}

/// Represents a some element that is a descendant of a `BlockElement`, but
/// is not an `InlineElement` such as `ListItem`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineBlockElement<'a> {
    ListItem(ListItem<'a>),
    Term(Term<'a>),
    Definition(Definition<'a>),
}

impl<'a> From<ListItem<'a>> for InlineBlockElement<'a> {
    fn from(list_item: ListItem<'a>) -> Self {
        Self::ListItem(list_item)
    }
}

impl InlineBlockElement<'_> {
    pub fn to_borrowed(&self) -> InlineBlockElement {
        match self {
            Self::ListItem(x) => InlineBlockElement::ListItem(x.to_borrowed()),
            Self::Term(x) => InlineBlockElement::Term(x.to_borrowed()),
            Self::Definition(x) => {
                InlineBlockElement::Definition(x.to_borrowed())
            }
        }
    }

    pub fn into_owned(self) -> InlineBlockElement<'static> {
        match self {
            Self::ListItem(x) => InlineBlockElement::ListItem(x.into_owned()),
            Self::Term(x) => InlineBlockElement::Term(x.into_owned()),
            Self::Definition(x) => {
                InlineBlockElement::Definition(x.into_owned())
            }
        }
    }
}

impl<'a> InlineBlockElement<'a> {
    pub fn into_children(self) -> Vec<Located<Element<'a>>> {
        match self {
            Self::ListItem(x) => x.into_children(),
            Self::Term(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            Self::Definition(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
        }
    }
}

macro_rules! element_impl_from {
    ($type:ty, $class:ident) => {
        impl<'a> From<$type> for Element<'a> {
            fn from(value: $type) -> Self {
                Self::from($class::from(value))
            }
        }
    };
}

element_impl_from!(Blockquote<'a>, BlockElement);
element_impl_from!(DefinitionList<'a>, BlockElement);
element_impl_from!(Divider, BlockElement);
element_impl_from!(Header<'a>, BlockElement);
element_impl_from!(List<'a>, BlockElement);
element_impl_from!(MathBlock<'a>, BlockElement);
element_impl_from!(Paragraph<'a>, BlockElement);
element_impl_from!(Placeholder<'a>, BlockElement);
element_impl_from!(PreformattedText<'a>, BlockElement);
element_impl_from!(Table<'a>, BlockElement);

element_impl_from!(Text<'a>, InlineElement);
element_impl_from!(DecoratedText<'a>, InlineElement);
element_impl_from!(Keyword, InlineElement);
element_impl_from!(Link<'a>, InlineElement);
element_impl_from!(Tags<'a>, InlineElement);
element_impl_from!(CodeInline<'a>, InlineElement);
element_impl_from!(MathInline<'a>, InlineElement);

element_impl_from!(ListItem<'a>, InlineBlockElement);
