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
    BlankLine(BlankLine),
    Blockquote(Blockquote),
    DefinitionList(DefinitionList),
    Divider(Divider),
    Header(Header),
    List(List),
    Math(MathBlock),
    NonBlankLine(NonBlankLine),
    Paragraph(Paragraph),
    #[graphql(flatten)]
    Placeholder(Placeholder),
    PreformattedText(PreformattedText),
    Table(Table),
}

impl From<LE<elements::BlockElement>> for BlockElement {
    fn from(lc: LE<elements::BlockElement>) -> Self {
        let region = lc.region;
        match lc.element {
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
            elements::BlockElement::NonBlankLine(x) => {
                Self::from(NonBlankLine {
                    region: Region::from(region),
                    text: x,
                })
            }
            elements::BlockElement::BlankLine => Self::from(BlankLine {
                region: Region::from(region),
            }),
        }
    }
}

/// Represents a single non-blank line within a document that does not map
/// to any other specific element
#[derive(async_graphql::SimpleObject, Debug)]
pub struct NonBlankLine {
    /// The segment of the document this line covers
    region: Region,

    /// The content of the line
    text: String,
}

/// Represents a single blank line within a document
#[derive(async_graphql::SimpleObject, Debug)]
pub struct BlankLine {
    /// The segment of the document this line covers
    region: Region,
}
