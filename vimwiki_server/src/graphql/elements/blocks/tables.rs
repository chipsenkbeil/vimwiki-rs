use super::{InlineElement, Region};
use vimwiki::{elements, LC};

/// Represents a single document table
#[derive(async_graphql::SimpleObject)]
pub struct Table {
    /// The segment of the document this table covers
    region: Region,

    /// The rows contained in this table
    rows: Vec<Row>,

    /// Whether or not the table is centered
    centered: bool,
}

impl From<LC<elements::Table>> for Table {
    fn from(mut lc: LC<elements::Table>) -> Self {
        let region = Region::from(lc.region);
        Self {
            region,
            rows: lc
                .element
                .rows
                .drain(..)
                .enumerate()
                .map(|(pos, row)| Row::from_at_pos(pos as i32, row))
                .collect(),
            centered: lc.element.centered,
        }
    }
}

/// Represents a single row within a table in a document
#[derive(async_graphql::Union)]
pub enum Row {
    Content(ContentRow),
    Divider(DividerRow),
}

impl Row {
    fn from_at_pos(position: i32, lc: LC<elements::Row>) -> Self {
        let region = Region::from(lc.region);

        match lc.element {
            elements::Row::Content { mut cells } => Self::from(ContentRow {
                region,
                position,
                cells: cells
                    .drain(..)
                    .enumerate()
                    .map(|(pos, cell)| {
                        Cell::from_at_pos(position, pos as i32, cell)
                    })
                    .collect(),
            }),
            elements::Row::Divider => {
                Self::from(DividerRow { region, position })
            }
        }
    }
}

/// Represents a row that acts as a divider between other rows, usually for
/// a header and later data rows
#[derive(async_graphql::SimpleObject)]
pub struct DividerRow {
    /// The segment of the document this row covers
    region: Region,

    /// The position of this row amongst all rows in the table
    position: i32,
}

/// Represents a row that contains one or more cells of data
#[derive(async_graphql::SimpleObject)]
pub struct ContentRow {
    /// The segment of the document this row covers
    region: Region,

    /// The position of this row amongst all rows in the table
    position: i32,

    /// The cells contained within this row
    cells: Vec<Cell>,
}

/// Represents a cell within a row
#[derive(async_graphql::Union)]
pub enum Cell {
    Content(ContentCell),
    SpanLeft(SpanLeftCell),
    SpanAbove(SpanAboveCell),
}

impl Cell {
    fn from_at_pos(
        row_position: i32,
        position: i32,
        lc: LC<elements::Cell>,
    ) -> Self {
        let region = Region::from(lc.region);

        match lc.element {
            elements::Cell::Content(mut x) => Self::from(ContentCell {
                region,
                row_position,
                position,
                contents: x
                    .elements
                    .drain(..)
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
#[derive(async_graphql::SimpleObject)]
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
#[derive(async_graphql::SimpleObject)]
pub struct SpanLeftCell {
    /// The segment of the document this cell covers
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,
}

/// Represents a cell with no content that spans the above row
#[derive(async_graphql::SimpleObject)]
pub struct SpanAboveCell {
    /// The segment of the document this cell covers
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,
}
