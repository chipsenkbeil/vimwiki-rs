use crate::{
    lang::elements::{Element, IntoChildren, Located},
    StrictEq,
};
use derive_more::{From, IsVariant};
use serde::{Deserialize, Serialize};

mod blockquotes;
pub use blockquotes::*;
mod code;
pub use code::*;
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
mod tables;
pub use tables::*;

/// Represents elements that are standalone (metaphorically a block element in CSS)
#[derive(
    Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize, IsVariant,
)]
pub enum BlockElement<'a> {
    Blockquote(Blockquote<'a>),
    CodeBlock(CodeBlock<'a>),
    DefinitionList(DefinitionList<'a>),
    Divider(Divider),
    Header(Header<'a>),
    List(List<'a>),
    MathBlock(MathBlock<'a>),
    Paragraph(Paragraph<'a>),
    Placeholder(Placeholder<'a>),
    Table(Table<'a>),
}

impl BlockElement<'_> {
    pub fn to_borrowed(&self) -> BlockElement {
        match self {
            Self::Blockquote(x) => BlockElement::from(x.to_borrowed()),
            Self::CodeBlock(x) => BlockElement::from(x.to_borrowed()),
            Self::DefinitionList(x) => BlockElement::from(x.to_borrowed()),
            Self::Divider(x) => BlockElement::from(*x),
            Self::Header(x) => BlockElement::from(x.to_borrowed()),
            Self::List(x) => BlockElement::from(x.to_borrowed()),
            Self::MathBlock(x) => BlockElement::from(x.to_borrowed()),
            Self::Paragraph(x) => BlockElement::from(x.to_borrowed()),
            Self::Placeholder(x) => BlockElement::from(x.to_borrowed()),
            Self::Table(x) => BlockElement::from(x.to_borrowed()),
        }
    }

    pub fn into_owned(self) -> BlockElement<'static> {
        match self {
            Self::Blockquote(x) => BlockElement::Blockquote(x.into_owned()),
            Self::CodeBlock(x) => BlockElement::CodeBlock(x.into_owned()),
            Self::DefinitionList(x) => {
                BlockElement::DefinitionList(x.into_owned())
            }
            Self::Divider(x) => BlockElement::Divider(x),
            Self::Header(x) => BlockElement::Header(x.into_owned()),
            Self::List(x) => BlockElement::List(x.into_owned()),
            Self::MathBlock(x) => BlockElement::MathBlock(x.into_owned()),
            Self::Paragraph(x) => BlockElement::Paragraph(x.into_owned()),
            Self::Placeholder(x) => BlockElement::Placeholder(x.into_owned()),
            Self::Table(x) => BlockElement::Table(x.into_owned()),
        }
    }
}

impl<'a> BlockElement<'a> {
    pub fn as_blockquote(&self) -> Option<&Blockquote<'a>> {
        match self {
            Self::Blockquote(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_blockquote(&mut self) -> Option<&mut Blockquote<'a>> {
        match self {
            Self::Blockquote(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_blockquote(self) -> Option<Blockquote<'a>> {
        match self {
            Self::Blockquote(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_code_block(&self) -> Option<&CodeBlock<'a>> {
        match self {
            Self::CodeBlock(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_code_block(&mut self) -> Option<&mut CodeBlock<'a>> {
        match self {
            Self::CodeBlock(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_code_block(self) -> Option<CodeBlock<'a>> {
        match self {
            Self::CodeBlock(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_definition_list(&self) -> Option<&DefinitionList<'a>> {
        match self {
            Self::DefinitionList(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_definition_list(
        &mut self,
    ) -> Option<&mut DefinitionList<'a>> {
        match self {
            Self::DefinitionList(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_definition_list(self) -> Option<DefinitionList<'a>> {
        match self {
            Self::DefinitionList(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_divider(&self) -> Option<&Divider> {
        match self {
            Self::Divider(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_divider(&mut self) -> Option<&mut Divider> {
        match self {
            Self::Divider(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_divider(self) -> Option<Divider> {
        match self {
            Self::Divider(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_header(&self) -> Option<&Header<'a>> {
        match self {
            Self::Header(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_header(&mut self) -> Option<&mut Header<'a>> {
        match self {
            Self::Header(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_header(self) -> Option<Header<'a>> {
        match self {
            Self::Header(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&List<'a>> {
        match self {
            Self::List(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_list(&mut self) -> Option<&mut List<'a>> {
        match self {
            Self::List(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<List<'a>> {
        match self {
            Self::List(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_math_block(&self) -> Option<&MathBlock<'a>> {
        match self {
            Self::MathBlock(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_math_block(&mut self) -> Option<&mut MathBlock<'a>> {
        match self {
            Self::MathBlock(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_math_block(self) -> Option<MathBlock<'a>> {
        match self {
            Self::MathBlock(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_paragraph(&self) -> Option<&Paragraph<'a>> {
        match self {
            Self::Paragraph(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_paragraph(&mut self) -> Option<&mut Paragraph<'a>> {
        match self {
            Self::Paragraph(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_paragraph(self) -> Option<Paragraph<'a>> {
        match self {
            Self::Paragraph(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_placeholder(&self) -> Option<&Placeholder<'a>> {
        match self {
            Self::Placeholder(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_placeholder(&mut self) -> Option<&mut Placeholder<'a>> {
        match self {
            Self::Placeholder(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_placeholder(self) -> Option<Placeholder<'a>> {
        match self {
            Self::Placeholder(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&Table<'a>> {
        match self {
            Self::Table(x) => Some(x),
            _ => None,
        }
    }

    pub fn as_mut_table(&mut self) -> Option<&mut Table<'a>> {
        match self {
            Self::Table(x) => Some(x),
            _ => None,
        }
    }

    pub fn into_table(self) -> Option<Table<'a>> {
        match self {
            Self::Table(x) => Some(x),
            _ => None,
        }
    }
}

impl<'a> IntoChildren for BlockElement<'a> {
    type Child = Located<Element<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        match self {
            Self::DefinitionList(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            Self::Header(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            Self::List(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            Self::Paragraph(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            Self::Table(x) => x
                .into_children()
                .into_iter()
                .map(|x| x.map(Element::from))
                .collect(),
            _ => vec![],
        }
    }
}

impl<'a> StrictEq for BlockElement<'a> {
    /// Performs strict_eq check on matching inner variants
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Blockquote(x), Self::Blockquote(y)) => x.strict_eq(y),
            (Self::CodeBlock(x), Self::CodeBlock(y)) => x.strict_eq(y),
            (Self::DefinitionList(x), Self::DefinitionList(y)) => {
                x.strict_eq(y)
            }
            (Self::Divider(x), Self::Divider(y)) => x.strict_eq(y),
            (Self::Header(x), Self::Header(y)) => x.strict_eq(y),
            (Self::List(x), Self::List(y)) => x.strict_eq(y),
            (Self::MathBlock(x), Self::MathBlock(y)) => x.strict_eq(y),
            (Self::Paragraph(x), Self::Paragraph(y)) => x.strict_eq(y),
            (Self::Placeholder(x), Self::Placeholder(y)) => x.strict_eq(y),
            (Self::Table(x), Self::Table(y)) => x.strict_eq(y),
            _ => false,
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
le_mapping!(CodeBlock<'a>);
le_mapping!(MathBlock<'a>);
le_mapping!(Blockquote<'a>);
le_mapping!(Divider);
le_mapping!(Placeholder<'a>);
