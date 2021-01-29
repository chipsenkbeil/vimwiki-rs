use async_graphql::SimpleObject;
use entity::*;
use vimwiki::elements::*;

/// Represents a segment of a document marked by a byte offset and length
#[derive(Debug, SimpleObject, ValueLike)]
pub struct GqlRegion {
    /// The byte offset within a file where this element begins
    offset: usize,

    /// The byte length of this element within a file
    len: usize,

    /// Extra information about the region, specifying the file-based line
    /// and column details for the beginning and end of the region
    position: Option<GqlPosition>,
}

impl From<Region> for GqlRegion {
    fn from(region: Region) -> Self {
        Self {
            offset: region.offset(),
            len: region.len(),
            position: region.position().map(GqlPosition::from),
        }
    }
}

/// Represents a segment of a document marked by a byte offset and length
#[derive(Debug, SimpleObject, ValueLike)]
pub struct GqlPosition {
    /// The starting line & column
    start: GqlLineColumn,

    /// The ending line & column
    end: GqlLineColumn,
}

impl From<Position> for GqlPosition {
    fn from(position: Position) -> Self {
        Self {
            start: GqlLineColumn::from(position.start()),
            end: GqlLineColumn::from(position.end()),
        }
    }
}

/// Represents a segment of a document marked by a byte offset and length
#[derive(Debug, SimpleObject, ValueLike)]
pub struct GqlLineColumn {
    /// The line in the file, starting at 1
    line: usize,

    /// The column in the file, starting at 1
    column: usize,
}

impl From<LineColumn> for GqlLineColumn {
    fn from(line_column: LineColumn) -> Self {
        Self {
            line: line_column.line(),
            column: line_column.column(),
        }
    }
}
