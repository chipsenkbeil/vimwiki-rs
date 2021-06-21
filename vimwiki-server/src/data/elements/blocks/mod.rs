use crate::data::{FromVimwikiElement, GraphqlDatabaseError};
use entity::*;
use entity_async_graphql::*;
use vimwiki::{self as v, Located};

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

/// Represents a single document element at a block-level
#[gql_ent]
pub enum BlockElement {
    Blockquote(Blockquote),
    CodeBlock(CodeBlock),
    DefinitionList(DefinitionList),
    Divider(Divider),
    Header(Header),
    List(List),
    Math(MathBlock),
    Paragraph(Paragraph),
    #[ent(wrap)]
    #[graphql(flatten)]
    Placeholder(Placeholder),
    Table(Table),
}

impl BlockElement {
    pub fn page_id(&self) -> Id {
        match self {
            Self::Blockquote(x) => x.page_id(),
            Self::CodeBlock(x) => x.page_id(),
            Self::DefinitionList(x) => x.page_id(),
            Self::Divider(x) => x.page_id(),
            Self::Header(x) => x.page_id(),
            Self::List(x) => x.page_id(),
            Self::Math(x) => x.page_id(),
            Self::Paragraph(x) => x.page_id(),
            Self::Placeholder(x) => x.page_id(),
            Self::Table(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Blockquote(x) => x.parent_id(),
            Self::CodeBlock(x) => x.parent_id(),
            Self::DefinitionList(x) => x.parent_id(),
            Self::Divider(x) => x.parent_id(),
            Self::Header(x) => x.parent_id(),
            Self::List(x) => x.parent_id(),
            Self::Math(x) => x.parent_id(),
            Self::Paragraph(x) => x.parent_id(),
            Self::Placeholder(x) => x.parent_id(),
            Self::Table(x) => x.parent_id(),
        }
    }
}

impl<'a> FromVimwikiElement<'a> for BlockElement {
    type Element = Located<v::BlockElement<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = element.region();
        Ok(match element.into_inner() {
            v::BlockElement::Header(x) => {
                Self::from(Header::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::Paragraph(x) => {
                Self::from(Paragraph::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::DefinitionList(x) => {
                Self::from(DefinitionList::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::List(x) => Self::from(List::from_vimwiki_element(
                page_id,
                parent_id,
                Located::new(x, region),
            )?),
            v::BlockElement::Table(x) => {
                Self::from(Table::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::CodeBlock(x) => {
                Self::from(CodeBlock::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::MathBlock(x) => {
                Self::from(MathBlock::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::Blockquote(x) => {
                Self::from(Blockquote::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::Divider(x) => {
                Self::from(Divider::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
            v::BlockElement::Placeholder(x) => {
                Self::from(Placeholder::from_vimwiki_element(
                    page_id,
                    parent_id,
                    Located::new(x, region),
                )?)
            }
        })
    }
}
