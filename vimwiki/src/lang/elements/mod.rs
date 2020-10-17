use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

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

impl Page<'_> {
    pub fn to_borrowed(&self) -> Page {
        let elements = self
            .elements
            .iter()
            .map(|x| Located::new(x.as_inner().to_borrowed(), x.region))
            .collect();
        let comments = self
            .comments
            .iter()
            .map(|x| Located::new(x.as_inner().as_borrowed(), x.region))
            .collect();

        Page { elements, comments }
    }

    pub fn into_owned(self) -> Page<'static> {
        let elements = self
            .elements
            .iter()
            .map(|x| Located::new(x.as_inner().into_owned(), x.region))
            .collect();
        let comments = self
            .comments
            .iter()
            .map(|x| Located::new(x.as_inner().into_owned(), x.region))
            .collect();

        Page { elements, comments }
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
