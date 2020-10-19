use super::Region;
use vimwiki::{elements, Located};

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

impl<'a> From<Located<elements::BlockElement<'a>>> for BlockElement {
    fn from(le: Located<elements::BlockElement<'a>>) -> Self {
        let region = le.region();
        match le.into_inner() {
            elements::BlockElement::Header(x) => {
                Self::from(Header::from(Located::new(x, region)))
            }
            elements::BlockElement::Paragraph(x) => {
                Self::from(Paragraph::from(Located::new(x, region)))
            }
            elements::BlockElement::DefinitionList(x) => {
                Self::from(DefinitionList::from(Located::new(x, region)))
            }
            elements::BlockElement::List(x) => {
                Self::from(List::from(Located::new(x, region)))
            }
            elements::BlockElement::Table(x) => {
                Self::from(Table::from(Located::new(x, region)))
            }
            elements::BlockElement::PreformattedText(x) => {
                Self::from(PreformattedText::from(Located::new(x, region)))
            }
            elements::BlockElement::Math(x) => {
                Self::from(MathBlock::from(Located::new(x, region)))
            }
            elements::BlockElement::Blockquote(x) => {
                Self::from(Blockquote::from(Located::new(x, region)))
            }
            elements::BlockElement::Divider(x) => {
                Self::from(Divider::from(Located::new(x, region)))
            }
            elements::BlockElement::Placeholder(x) => {
                Self::from(Placeholder::from(Located::new(x, region)))
            }
        }
    }
}
