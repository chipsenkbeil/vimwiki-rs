use entity::*;
use serde::{Deserialize, Serialize};
use vimwiki::elements as v;

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
