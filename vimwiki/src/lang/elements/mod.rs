use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod blocks;
pub use blocks::*;
mod location;
pub use location::{Located, Position, Region};

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

/// Represents either a `BlockElement` or `InlineElement`
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub enum Element<'a> {
    Block(BlockElement<'a>),
    Inline(InlineElement<'a>),
}

impl Element<'_> {
    pub fn to_borrowed(&self) -> Element {
        match self {
            Self::Block(x) => Element::Block(x.to_borrowed()),
            Self::Inline(x) => Element::Inline(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> Element<'static> {
        match self {
            Self::Block(x) => Element::Block(x.into_owned()),
            Self::Inline(x) => Element::Inline(x.into_owned()),
        }
    }
}

impl<'a> Element<'a> {
    /// Borrows all children below this `Element`
    pub fn to_children(&'a self) -> Vec<Located<Element<'a>>> {
        match self {
            Self::Block(x) => x.to_children(),
            Self::Inline(x) => x
                .to_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
        }
    }

    pub fn is_block_element(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn is_inline_element(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    #[inline]
    pub fn as_block_element(&self) -> Option<&BlockElement<'a>> {
        match self {
            Self::Block(ref x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_inline_element(&self) -> Option<&InlineElement<'a>> {
        match self {
            Self::Inline(ref x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_mut_block_element(&mut self) -> Option<&mut BlockElement<'a>> {
        match self {
            Self::Block(ref mut x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_mut_inline_element(&mut self) -> Option<&mut InlineElement<'a>> {
        match self {
            Self::Inline(ref mut x) => Some(x),
            _ => None,
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
