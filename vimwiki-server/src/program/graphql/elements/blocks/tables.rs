use super::{InlineElement, Region};
use vimwiki::{elements, Located};

/// Represents a single document table
#[derive(async_graphql::SimpleObject, Debug)]
pub struct Table {
    /// The segment of the document this table covers
    region: Region,

    /// The rows contained in this table
    rows: Vec<Row>,

    /// Whether or not the table is centered
    centered: bool,
}

impl<'a> From<Located<elements::Table<'a>>> for Table {
    fn from(le: Located<elements::Table<'a>>) -> Self {
        let region = Region::from(le.region());
        let element = le.into_inner();
        Self {
            region,
            rows: element
                .rows
                .into_iter()
                .enumerate()
                .map(|(pos, row)| Row::from_at_pos(pos as i32, row))
                .collect(),
            centered: element.centered,
        }
    }
}

/// Represents a single row within a table in a document
#[derive(async_graphql::Union, Debug)]
pub enum Row {
    Content(ContentRow),
    Divider(DividerRow),
}

impl Row {
    fn from_at_pos(position: i32, le: Located<elements::Row>) -> Self {
        let region = Region::from(le.region());

        match le.into_inner() {
            elements::Row::Content { cells } => Self::from(ContentRow {
                region,
                position,
                cells: cells
                    .into_iter()
                    .enumerate()
                    .map(|(pos, cell)| {
                        Cell::from_at_pos(position, pos as i32, cell)
                    })
                    .collect(),
            }),
            elements::Row::Divider { columns } => Self::from(DividerRow {
                region,
                position,
                columns: columns.into_iter().map(ColumnAlign::from).collect(),
            }),
        }
    }
}

/// Represents a row that acts as a divider between other rows, usually for
/// a header and later data rows
#[derive(async_graphql::SimpleObject, Debug)]
pub struct DividerRow {
    /// The segment of the document this row covers
    region: Region,

    /// The position of this row amongst all rows in the table
    position: i32,

    /// The alignment of each column according to this divider
    columns: Vec<ColumnAlign>,
}

#[derive(async_graphql::Enum, Copy, Clone, Debug, Eq, PartialEq)]
#[graphql(remote = "vimwiki::elements::ColumnAlign")]
pub enum ColumnAlign {
    /// Align columns left
    Left,

    /// Align columns centered
    Center,

    /// Align columns right
    Right,
}

/// Represents a row that contains one or more cells of data
#[derive(async_graphql::SimpleObject, Debug)]
pub struct ContentRow {
    /// The segment of the document this row covers
    region: Region,

    /// The position of this row amongst all rows in the table
    position: i32,

    /// The cells contained within this row
    cells: Vec<Cell>,
}

/// Represents a cell within a row
#[derive(async_graphql::Union, Debug)]
pub enum Cell {
    Content(ContentCell),
    SpanLeft(SpanLeftCell),
    SpanAbove(SpanAboveCell),
}

impl Cell {
    fn from_at_pos(
        row_position: i32,
        position: i32,
        le: Located<elements::Cell>,
    ) -> Self {
        let region = Region::from(le.region());
        match le.into_inner() {
            elements::Cell::Content(x) => Self::from(ContentCell {
                region,
                row_position,
                position,
                contents: x
                    .elements
                    .into_iter()
                    .map(InlineElement::from)
                    .collect(),
            }),
            elements::Cell::SpanAbove => Self::from(SpanAboveCell {
                region,
                row_position,
                position,
            }),
            elements::Cell::SpanLeft => Self::from(SpanLeftCell {
                region,
                row_position,
                position,
            }),
        }
    }
}

/// Represents a cell with content
#[derive(async_graphql::SimpleObject, Debug)]
pub struct ContentCell {
    /// The segment of the document this cell covers
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,

    /// Contents within the cell
    contents: Vec<InlineElement>,
}

/// Represents a cell with no content that spans the left cell
#[derive(async_graphql::SimpleObject, Debug)]
pub struct SpanLeftCell {
    /// The segment of the document this cell covers
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,
}

/// Represents a cell with no content that spans the above row
#[derive(async_graphql::SimpleObject, Debug)]
pub struct SpanAboveCell {
    /// The segment of the document this cell covers
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,
}
