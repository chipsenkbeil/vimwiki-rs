use super::Region;
use vimwiki::{components, LC};

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

/// Represents a single document page
#[derive(async_graphql::Union)]
pub enum BlockComponent {
    BlankLine(BlankLine),
    Blockquote(Blockquote),
    DefinitionList(DefinitionList),
    Divider(Divider),
    Header(Header),
    List(List),
    Math(MathBlock),
    NonBlankLine(NonBlankLine),
    Paragraph(Paragraph),
    #[item(flatten)]
    Placeholder(Placeholder),
    PreformattedText(PreformattedText),
    Table(Table),
}

impl From<LC<components::BlockComponent>> for BlockComponent {
    fn from(lc: LC<components::BlockComponent>) -> Self {
        let region = lc.region;
        match lc.component {
            components::BlockComponent::Header(x) => {
                Self::from(Header::from(LC::new(x, region)))
            }
            components::BlockComponent::Paragraph(x) => {
                Self::from(Paragraph::from(LC::new(x, region)))
            }
            components::BlockComponent::DefinitionList(x) => {
                Self::from(DefinitionList::from(LC::new(x, region)))
            }
            components::BlockComponent::List(x) => {
                Self::from(List::from(LC::new(x, region)))
            }
            components::BlockComponent::Table(x) => {
                Self::from(Table::from(LC::new(x, region)))
            }
            components::BlockComponent::PreformattedText(x) => {
                Self::from(PreformattedText::from(LC::new(x, region)))
            }
            components::BlockComponent::Math(x) => {
                Self::from(MathBlock::from(LC::new(x, region)))
            }
            components::BlockComponent::Blockquote(x) => {
                Self::from(Blockquote::from(LC::new(x, region)))
            }
            components::BlockComponent::Divider(x) => {
                Self::from(Divider::from(LC::new(x, region)))
            }
            components::BlockComponent::Placeholder(x) => {
                Self::from(Placeholder::from(LC::new(x, region)))
            }
            components::BlockComponent::NonBlankLine(x) => {
                Self::from(NonBlankLine {
                    region: Region::from(region),
                    text: x,
                })
            }
            components::BlockComponent::BlankLine => Self::from(BlankLine {
                region: Region::from(region),
            }),
        }
    }
}

/// Represents a single non-blank line within a document that does not map
/// to any other specific component
#[derive(async_graphql::SimpleObject)]
pub struct NonBlankLine {
    /// The segment of the document this line covers
    region: Region,

    /// The content of the line
    text: String,
}

/// Represents a single blank line within a document
#[derive(async_graphql::SimpleObject)]
pub struct BlankLine {
    /// The segment of the document this line covers
    region: Region,
}
