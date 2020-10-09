use super::Region;
use vimwiki::{elements, LE};

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

/// Represents a single document element at a block-level
#[derive(async_graphql::Union, Debug)]
pub enum BlockElement {
    Blockquote(Blockquote),
    DefinitionList(DefinitionList),
    Divider(Divider),
    Header(Header),
    List(List),
    Math(MathBlock),
    Paragraph(Paragraph),
    #[graphql(flatten)]
    Placeholder(Placeholder),
    PreformattedText(PreformattedText),
    Table(Table),
}

impl From<LE<elements::BlockElement>> for BlockElement {
    fn from(le: LE<elements::BlockElement>) -> Self {
        let region = le.region;
        match le.element {
            elements::BlockElement::Header(x) => {
                Self::from(Header::from(LE::new(x, region)))
            }
            elements::BlockElement::Paragraph(x) => {
                Self::from(Paragraph::from(LE::new(x, region)))
            }
            elements::BlockElement::DefinitionList(x) => {
                Self::from(DefinitionList::from(LE::new(x, region)))
            }
            elements::BlockElement::List(x) => {
                Self::from(List::from(LE::new(x, region)))
            }
            elements::BlockElement::Table(x) => {
                Self::from(Table::from(LE::new(x, region)))
            }
            elements::BlockElement::PreformattedText(x) => {
                Self::from(PreformattedText::from(LE::new(x, region)))
            }
            elements::BlockElement::Math(x) => {
                Self::from(MathBlock::from(LE::new(x, region)))
            }
            elements::BlockElement::Blockquote(x) => {
                Self::from(Blockquote::from(LE::new(x, region)))
            }
            elements::BlockElement::Divider(x) => {
                Self::from(Divider::from(LE::new(x, region)))
            }
            elements::BlockElement::Placeholder(x) => {
                Self::from(Placeholder::from(LE::new(x, region)))
            }
        }
    }
}
