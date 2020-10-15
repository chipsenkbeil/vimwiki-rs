use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

mod blocks;
pub use blocks::*;
mod comments;
pub use comments::*;
mod location;
pub use location::{Located, Position, Region};

/// Represents a full page containing different elements
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page<'a> {
    /// Comprised of the elements within a page
    pub elements: Vec<Located<BlockElement<'a>>>,

    /// Comprised of the comments within a page
    pub comments: Vec<Located<Comment<'a>>>,
}

/// Represents either a `BlockElement` or `InlineElement`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element<'a, 'b> {
    Block(Cow<'a, BlockElement<'b>>),
    Inline(Cow<'a, InlineElement<'b>>),
}

impl<'a, 'b> Element<'a, 'b> {
    pub fn is_block_element(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn is_inline_element(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    pub fn as_borrowed(&self) -> Element<'_, 'b> {
        match self {
            Self::Block(ref x) => Element::Block(match x {
                Cow::Borrowed(x) => Cow::Borrowed(*x),
                Cow::Owned(x) => Cow::Borrowed(&x),
            }),
            Self::Inline(ref x) => Element::Inline(match x {
                Cow::Borrowed(x) => Cow::Borrowed(*x),
                Cow::Owned(x) => Cow::Borrowed(&x),
            }),
        }
    }

    #[inline]
    pub fn as_block_element(&self) -> Option<&BlockElement<'b>> {
        match self {
            Self::Block(ref x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_inline_element(&self) -> Option<&InlineElement<'b>> {
        match self {
            Self::Inline(ref x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_mut_block_element(&mut self) -> Option<&mut BlockElement<'b>> {
        match self {
            Self::Block(ref mut x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_mut_inline_element(&mut self) -> Option<&mut InlineElement<'b>> {
        match self {
            Self::Inline(ref mut x) => Some(x),
            _ => None,
        }
    }
}

impl<'a, 'b> From<&'a BlockElement<'b>> for Element<'a, 'b> {
    fn from(element: &'a BlockElement<'b>) -> Self {
        Self::Block(Cow::Borrowed(element))
    }
}

impl<'b> From<BlockElement<'b>> for Element<'static, 'b> {
    fn from(element: BlockElement<'b>) -> Self {
        Self::Block(Cow::Owned(element))
    }
}

impl<'a, 'b> From<&'a InlineElement<'b>> for Element<'a, 'b> {
    fn from(element: &'a InlineElement<'b>) -> Self {
        Self::Inline(Cow::Borrowed(element))
    }
}

impl<'b> From<InlineElement<'b>> for Element<'static, 'b> {
    fn from(element: InlineElement<'b>) -> Self {
        Self::Inline(Cow::Owned(element))
    }
}
