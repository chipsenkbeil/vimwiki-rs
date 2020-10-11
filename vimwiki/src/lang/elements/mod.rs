use super::utils::LE;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

mod blocks;
pub use blocks::*;
mod comments;
pub use comments::*;

/// Represents a full page containing different elements
#[derive(
    Constructor, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Page {
    /// Comprised of the elements within a page
    pub elements: Vec<LE<BlockElement>>,

    /// Comprised of the comments within a page
    pub comments: Vec<LE<Comment>>,
}

/// Represents either a `BlockElement` or `InlineElement`
#[derive(Clone, Debug, From, PartialEq, Eq)]
pub enum Element {
    Block(BlockElement),
    Inline(InlineElement),
}

impl Element {
    pub fn is_block_element(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn is_inline_element(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    pub fn as_ref(&self) -> ElementRef<'_> {
        match self {
            Self::Block(ref x) => ElementRef::Block(x),
            Self::Inline(ref x) => ElementRef::Inline(x),
        }
    }

    pub fn as_mut(&mut self) -> ElementMutRef<'_> {
        match self {
            Self::Block(ref mut x) => ElementMutRef::Block(x),
            Self::Inline(ref mut x) => ElementMutRef::Inline(x),
        }
    }

    #[inline]
    pub fn as_block_element(&self) -> Option<&BlockElement> {
        self.as_ref().as_block_element()
    }

    #[inline]
    pub fn as_inline_element(&self) -> Option<&InlineElement> {
        self.as_ref().as_inline_element()
    }

    #[inline]
    pub fn as_mut_block_element(&mut self) -> Option<&mut BlockElement> {
        // Not sure why, but cannot do the following:
        // self.as_mut().as_mut_block_element()
        match self {
            Self::Block(ref mut x) => Some(x),
            _ => None,
        }
    }

    #[inline]
    pub fn as_mut_inline_element(&mut self) -> Option<&mut InlineElement> {
        // Not sure why, but cannot do the following:
        // self.as_mut().as_mut_inline_element()
        match self {
            Self::Inline(ref mut x) => Some(x),
            _ => None,
        }
    }
}

/// Represents a reference to either a `BlockElement` or `InlineElement`
#[derive(Clone, Debug, From, PartialEq, Eq)]
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

    pub fn to_owned(&self) -> Element {
        match self {
            Self::Block(x) => Element::from((*x).clone()),
            Self::Inline(x) => Element::from((*x).clone()),
        }
    }
}

/// Represents a mutable reference to either a `BlockElement` or `InlineElement`
#[derive(Debug, From, PartialEq, Eq)]
pub enum ElementMutRef<'a> {
    Block(&'a mut BlockElement),
    Inline(&'a mut InlineElement),
}

impl<'a> ElementMutRef<'a> {
    pub fn is_block_element(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    pub fn is_inline_element(&self) -> bool {
        matches!(self, Self::Inline(_))
    }

    pub fn as_mut_block_element(&'a mut self) -> Option<&'a mut BlockElement> {
        match self {
            Self::Block(ref mut x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_inline_element(
        &'a mut self,
    ) -> Option<&'a mut InlineElement> {
        match self {
            Self::Inline(ref mut x) => Some(x),
            _ => None,
        }
    }
}
