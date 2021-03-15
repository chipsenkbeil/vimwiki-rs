use derive_more::From;
use entity::*;
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

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
#[simple_ent]
#[derive(async_graphql::Union, Debug, From)]
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

impl<'a> TryFrom<Located<v::BlockElement<'a>>> for BlockElement {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::BlockElement<'a>>) -> Result<Self, Self::Error> {
        let region = le.region();
        match le.into_inner() {
            v::BlockElement::Header(x) => {
                Self::from(Header::try_from(Located::new(x, region))?)
            }
            v::BlockElement::Paragraph(x) => {
                Self::from(Paragraph::try_from(Located::new(x, region))?)
            }
            v::BlockElement::DefinitionList(x) => {
                Self::from(DefinitionList::try_from(Located::new(x, region))?)
            }
            v::BlockElement::List(x) => {
                Self::from(List::try_from(Located::new(x, region))?)
            }
            v::BlockElement::Table(x) => {
                Self::from(Table::try_from(Located::new(x, region))?)
            }
            v::BlockElement::PreformattedText(x) => {
                Self::from(PreformattedText::try_from(Located::new(x, region))?)
            }
            v::BlockElement::Math(x) => {
                Self::from(MathBlock::try_from(Located::new(x, region))?)
            }
            v::BlockElement::Blockquote(x) => {
                Self::from(Blockquote::try_from(Located::new(x, region))?)
            }
            v::BlockElement::Divider(x) => {
                Self::from(Divider::try_from(Located::new(x, region))?)
            }
            v::BlockElement::Placeholder(x) => {
                Self::from(Placeholder::try_from(Located::new(x, region))?)
            }
        }
    }
}
