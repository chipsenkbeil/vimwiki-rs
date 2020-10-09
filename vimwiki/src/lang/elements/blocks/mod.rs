use super::LE;
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
pub enum BlockElement {
    Blockquote(Blockquote),
    DefinitionList(DefinitionList),
    Divider(Divider),
    Header(Header),
    List(List),
    Math(MathBlock),
    Paragraph(Paragraph),
    Placeholder(Placeholder),
    PreformattedText(PreformattedText),
    Table(Table),
}

macro_rules! le_mapping {
    ($type:ty) => {
        impl From<LE<$type>> for LE<BlockElement> {
            fn from(element: LE<$type>) -> Self {
                element.map(BlockElement::from)
            }
        }
    };
}

le_mapping!(Header);
le_mapping!(Paragraph);
le_mapping!(DefinitionList);
le_mapping!(List);
le_mapping!(Table);
le_mapping!(PreformattedText);
le_mapping!(MathBlock);
le_mapping!(Blockquote);
le_mapping!(Divider);
le_mapping!(Placeholder);

/// Represents a wrapper around a `BlockElement` where we already know the
/// type it will be and can therefore convert to either the `BlockElement`
/// or the inner type
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypedBlockElement<T> {
    inner: BlockElement,
    phantom: PhantomData<T>,
}

impl<T> TypedBlockElement<T> {
    pub fn into_inner(self) -> BlockElement {
        self.inner
    }

    pub fn as_inner(&self) -> &BlockElement {
        &self.inner
    }

    pub fn as_mut_inner(&mut self) -> &mut BlockElement {
        &mut self.inner
    }
}

macro_rules! typed_block_element_impl {
    ($type:ty, $variant:ident, $name:ident) => {
        paste! {
            impl TypedBlockElement<$type> {
                pub fn [<from_ $name>](x: $type) -> Self {
                    Self {
                        inner: BlockElement::from(x),
                        phantom: PhantomData,
                    }
                }

                pub fn [<into_ $name>](self) -> $type {
                    match self.inner {
                        BlockElement::$variant(x) => x,
                        _ => unreachable!(),
                    }
                }

                pub fn [<as_ $name>](&self) -> &$type {
                    match self.inner {
                        BlockElement::$variant(ref x) => x,
                        _ => unreachable!(),
                    }
                }

                pub fn [<as_mut_ $name>](&mut self) -> &mut $type {
                    match self.inner {
                        BlockElement::$variant(ref mut x) => x,
                        _ => unreachable!(),
                    }
                }
            }
        }
    };
}

typed_block_element_impl!(Header, Header, header);
typed_block_element_impl!(Paragraph, Paragraph, paragraph);
typed_block_element_impl!(DefinitionList, DefinitionList, definition_list);
typed_block_element_impl!(List, List, list);
typed_block_element_impl!(Table, Table, table);
typed_block_element_impl!(
    PreformattedText,
    PreformattedText,
    preformatted_text
);
typed_block_element_impl!(MathBlock, Math, math_block);
typed_block_element_impl!(Blockquote, Blockquote, blockquote);
typed_block_element_impl!(Divider, Divider, divider);
typed_block_element_impl!(Placeholder, Placeholder, placeholder);
