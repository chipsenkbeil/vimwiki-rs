use vimwiki::elements;

/// Represents a segment of a document marked by a byte offset and length
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Region {
    /// The byte offset within a file where this element begins
    offset: i32,

    /// The byte length of this element within a file
    len: i32,

    /// Extra information about the region, specifying the file-based line
    /// and column details for the beginning and end of the region
    position: Option<Position>,
}

impl From<elements::Region> for Region {
    fn from(region: elements::Region) -> Self {
        Self {
            offset: region.offset() as i32,
            len: region.len() as i32,
            position: region.position().map(Position::from),
        }
    }
}

/// Represents a segment of a document marked by a byte offset and length
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Position {
    /// The starting line & column
    start: LineColumn,

    /// The ending line & column
    end: LineColumn,
}

impl From<elements::Position> for Position {
    fn from(position: elements::Position) -> Self {
        Self {
            start: LineColumn::from(position.start()),
            end: LineColumn::from(position.end()),
        }
    }
}

/// Represents a segment of a document marked by a byte offset and length
#[derive(async_graphql::SimpleObject, Debug)]
pub struct LineColumn {
    /// The line in the file, starting at 1
    line: i32,

    /// The column in the file, starting at 1
    column: i32,
}

impl From<elements::LineColumn> for LineColumn {
    fn from(line_column: elements::LineColumn) -> Self {
        Self {
            line: line_column.line() as i32,
            column: line_column.column() as i32,
        }
    }
}
