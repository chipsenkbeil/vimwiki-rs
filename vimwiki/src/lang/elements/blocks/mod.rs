use crate::{
    lang::elements::{Element, IntoChildren, Located},
    StrictEq,
};
use derive_more::From;
use serde::{Deserialize, Serialize};

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
            Self::Divider(x) => BlockElement::from(*x),
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
            Self::Blockquote(x) => BlockElement::Blockquote(x.into_owned()),
            Self::DefinitionList(x) => {
                BlockElement::DefinitionList(x.into_owned())
            }
            Self::Divider(x) => BlockElement::Divider(x),
            Self::Header(x) => BlockElement::Header(x.into_owned()),
            Self::List(x) => BlockElement::List(x.into_owned()),
            Self::Math(x) => BlockElement::Math(x.into_owned()),
            Self::Paragraph(x) => BlockElement::Paragraph(x.into_owned()),
            Self::Placeholder(x) => BlockElement::Placeholder(x.into_owned()),
            Self::PreformattedText(x) => {
                BlockElement::PreformattedText(x.into_owned())
            }
            Self::Table(x) => BlockElement::Table(x.into_owned()),
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
            (Self::DefinitionList(x), Self::DefinitionList(y)) => {
                x.strict_eq(y)
            }
            (Self::Divider(x), Self::Divider(y)) => x.strict_eq(y),
            (Self::Header(x), Self::Header(y)) => x.strict_eq(y),
            (Self::List(x), Self::List(y)) => x.strict_eq(y),
            (Self::Math(x), Self::Math(y)) => x.strict_eq(y),
            (Self::Paragraph(x), Self::Paragraph(y)) => x.strict_eq(y),
            (Self::Placeholder(x), Self::Placeholder(y)) => x.strict_eq(y),
            (Self::PreformattedText(x), Self::PreformattedText(y)) => {
                x.strict_eq(y)
            }
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
le_mapping!(PreformattedText<'a>);
le_mapping!(MathBlock<'a>);
le_mapping!(Blockquote<'a>);
le_mapping!(Divider);
le_mapping!(Placeholder<'a>);
