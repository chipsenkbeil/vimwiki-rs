mod blockquotes;
pub use blockquotes::*;

mod definitions;
pub use definitions::*;

mod inline;
pub use inline::*;

use derive_more::{Display, Error, From};
use entity::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

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

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Blockquote {
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,
    lines: Vec<String>,
}

impl<'a> TryFrom<Located<v::Blockquote<'a>>> for Blockquote {
    type Error = ConvertToDatabaseError;

    fn try_from(le: Located<v::Blockquote<'a>>) -> Result<Self, Self::Error> {
        Self::build()
            .region(Region::from(le.region()))
            .lines(
                le.into_inner()
                    .lines
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            )
            .finish_and_commit()
            .map_err(ConvertToDatabaseError::Database)?
            .map_err(ConvertToDatabaseError::Builder)
    }
}
