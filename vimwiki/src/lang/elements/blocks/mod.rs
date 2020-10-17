use crate::lang::elements::Located;
use derive_more::From;
use paste::paste;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

mod blockquotes;
pub use blockquotes::*;
mod definitions;
pub use definitions::*;
mod dividers;
pub use dividers::*;
mod headers;
pub use headers::*;
mod inline;
pub use inline::*;
mod lists;
pub use lists::*;
mod math;
pub use math::*;
mod paragraphs;
pub use paragraphs::*;
mod placeholders;
pub use placeholders::*;
mod preformatted;
pub use preformatted::*;
mod tables;
pub use tables::*;

/// Represents elements that are standalone (metaphorically a block element in CSS)
#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum BlockElement<'a> {
    Blockquote(Blockquote<'a>),
    DefinitionList(DefinitionList<'a>),
    Divider(Divider),
    Header(Header<'a>),
    List(List<'a>),
    Math(MathBlock<'a>),
    Paragraph(Paragraph<'a>),
    Placeholder(Placeholder<'a>),
    PreformattedText(PreformattedText<'a>),
    Table(Table<'a>),
}

impl BlockElement<'_> {
    pub fn to_borrowed(&self) -> BlockElement {
        match self {
            Self::Blockquote(x) => BlockElement::from(x.to_borrowed()),
            Self::DefinitionList(x) => BlockElement::from(x.to_borrowed()),
            Self::Divider(x) => BlockElement::from(x.as_borrowed()),
            Self::Header(x) => BlockElement::from(x.to_borrowed()),
            Self::List(x) => BlockElement::from(x.to_borrowed()),
            Self::Math(x) => BlockElement::from(x.to_borrowed()),
            Self::Paragraph(x) => BlockElement::from(x.to_borrowed()),
            Self::Placeholder(x) => BlockElement::from(x.to_borrowed()),
            Self::PreformattedText(x) => BlockElement::from(x.to_borrowed()),
            Self::Table(x) => BlockElement::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> BlockElement<'static> {
        match self {
            Self::Blockquote(x) => BlockElement::from(x.into_owned()),
            Self::DefinitionList(x) => BlockElement::from(x.into_owned()),
            Self::Divider(x) => BlockElement::from(x.into_owned()),
            Self::Header(x) => BlockElement::from(x.into_owned()),
            Self::List(x) => BlockElement::from(x.into_owned()),
            Self::Math(x) => BlockElement::from(x.into_owned()),
            Self::Paragraph(x) => BlockElement::from(x.into_owned()),
            Self::Placeholder(x) => BlockElement::from(x.into_owned()),
            Self::PreformattedText(x) => BlockElement::from(x.into_owned()),
            Self::Table(x) => BlockElement::from(x.into_owned()),
        }
    }
}

macro_rules! le_mapping {
    ($type:ty) => {
        impl<'a> From<Located<$type>> for Located<BlockElement<'a>> {
            fn from(element: Located<$type>) -> Self {
                element.map(BlockElement::from)
            }
        }
    };
}

le_mapping!(Header<'a>);
le_mapping!(Paragraph<'a>);
le_mapping!(DefinitionList<'a>);
le_mapping!(List<'a>);
le_mapping!(Table<'a>);
le_mapping!(PreformattedText<'a>);
le_mapping!(MathBlock<'a>);
le_mapping!(Blockquote<'a>);
le_mapping!(Divider);
le_mapping!(Placeholder<'a>);

/// Represents a wrapper around a `BlockElement` where we already know the
/// type it will be and can therefore convert to either the `BlockElement`
/// or the inner type
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypedBlockElement<'a, T> {
    inner: BlockElement<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> TypedBlockElement<'a, T> {
    pub fn into_inner(self) -> BlockElement<'a> {
        self.inner
    }

    pub fn as_inner(&self) -> &BlockElement<'a> {
        &self.inner
    }

    pub fn as_mut_inner(&mut self) -> &mut BlockElement<'a> {
        &mut self.inner
    }
}

impl<T> TypedBlockElement<'_, T> {
    pub fn to_borrowed(&self) -> TypedBlockElement<'_, T> {
        let inner = self.inner.to_borrowed();

        TypedBlockElement {
            inner,
            phantom: PhantomData,
        }
    }

    pub fn into_owned(self) -> TypedBlockElement<'static, T> {
        let inner = self.inner.into_owned();

        TypedBlockElement {
            inner,
            phantom: PhantomData,
        }
    }
}

macro_rules! typed_block_element_impl {
    ($type:ty, $variant:ident, $name:ident) => {
        paste! {
            impl<'a> TypedBlockElement<'a, $type> {
                pub fn [<from_ $name>](x: $type) -> Self {
                    Self {
                        inner: BlockElement::from(x),
                        phantom: PhantomData,
                    }
                }

                pub fn into_typed(self) -> $type {
                    match self.inner {
                        BlockElement::$variant(x) => x,
                        _ => unreachable!(),
                    }
                }

                pub fn as_typed(&self) -> &$type {
                    match self.inner {
                        BlockElement::$variant(ref x) => x,
                        _ => unreachable!(),
                    }
                }

                pub fn as_mut_typed(&mut self) -> &mut $type {
                    match self.inner {
                        BlockElement::$variant(ref mut x) => x,
                        _ => unreachable!(),
                    }
                }
            }
        }
    };
}

typed_block_element_impl!(Header<'a>, Header, header);
typed_block_element_impl!(Paragraph<'a>, Paragraph, paragraph);
typed_block_element_impl!(DefinitionList<'a>, DefinitionList, definition_list);
typed_block_element_impl!(List<'a>, List, list);
typed_block_element_impl!(Table<'a>, Table, table);
typed_block_element_impl!(
    PreformattedText<'a>,
    PreformattedText,
    preformatted_text
);
typed_block_element_impl!(MathBlock<'a>, Math, math_block);
typed_block_element_impl!(Blockquote<'a>, Blockquote, blockquote);
typed_block_element_impl!(Divider, Divider, divider);
typed_block_element_impl!(Placeholder<'a>, Placeholder, placeholder);
