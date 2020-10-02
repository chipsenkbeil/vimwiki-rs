/// Represents a segment of a document marked by [start, end]
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Region {
    /// The inclusive start of something within a document
    start: Position,

    /// The inclusive end of something within a document
    end: Position,
}

impl From<vimwiki::Region> for Region {
    fn from(region: vimwiki::Region) -> Self {
        Self {
            start: Position::from(region.start),
            end: Position::from(region.end),
        }
    }
}

/// Represents a specific location within a document
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Position {
    /// The line within a document starting at 1
    line: i32,

    /// The column within a document starting at 1
    column: i32,
}

impl From<vimwiki::Position> for Position {
    fn from(position: vimwiki::Position) -> Self {
        Self {
            line: position.line as i32,
            column: position.column as i32,
        }
    }
}
