use derive_more::{Display, Error, From};
use entity::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use vimwiki::{elements as v, Located};

#[derive(Display, Error)]
pub enum ConvertToDatabaseError {
    Database(DatabaseError),
    Builder(Box<dyn std::error::Error>),
}

#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Page {
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    elements: Vec<BlockElement>,
}

#[simple_ent]
#[derive(async_graphql::Union)]
pub enum Element {
    #[graphql(flatten)]
    Block(BlockElement),

    #[graphql(flatten)]
    Inline(InlineElement),

    #[graphql(flatten)]
    InlineBlock(InlineBlockElement),
}

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

/// Represents a segment of a document marked by a byte offset and length
#[derive(
    Clone, Debug, async_graphql::SimpleObject, Serialize, Deserialize, ValueLike,
)]
pub struct Region {
    /// The byte offset within a file where this element begins
    offset: usize,

    /// The byte length of this element within a file
    len: usize,

    /// Extra information about the region, specifying the file-based line
    /// and column details for the beginning and end of the region
    position: Option<Position>,
}

impl From<v::Region> for Region {
    fn from(region: v::Region) -> Self {
        Self {
            offset: region.offset(),
            len: region.len(),
            position: region.position().map(Position::from),
        }
    }
}

/// Represents a segment of a document marked by a byte offset and length
#[derive(
    Clone, Debug, async_graphql::SimpleObject, Serialize, Deserialize, ValueLike,
)]
pub struct Position {
    /// The starting line & column
    start: LineColumn,

    /// The ending line & column
    end: LineColumn,
}

impl From<v::Position> for Position {
    fn from(position: v::Position) -> Self {
        Self {
            start: LineColumn::from(position.start()),
            end: LineColumn::from(position.end()),
        }
    }
}

/// Represents a segment of a document marked by a byte offset and length
#[derive(
    Clone, Debug, async_graphql::SimpleObject, Serialize, Deserialize, ValueLike,
)]
pub struct LineColumn {
    /// The line in the file, starting at 1
    line: usize,

    /// The column in the file, starting at 1
    column: usize,
}

impl From<v::LineColumn> for LineColumn {
    fn from(line_column: v::LineColumn) -> Self {
        Self {
            line: line_column.line(),
            column: line_column.column(),
        }
    }
}
