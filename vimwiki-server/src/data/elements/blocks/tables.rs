use crate::data::{
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Region,
};
use entity::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::{Display, EnumString};
use vimwiki::{elements as v, Located};

/// Represents a single document table
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct Table {
    /// The segment of the document this table covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The rows contained in this table
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    rows: Vec<Row>,

    /// Whether or not the table is centered
    centered: bool,
}

impl<'a> TryFrom<Located<v::Table<'a>>> for Table {
    type Error = GraphqlDatabaseError;

    fn try_from(le: Located<v::Table<'a>>) -> Result<Self, Self::Error> {
        let region = Region::from(le.region());
        let element = le.into_inner();
        let centered = le.as_inner().centered;

        let mut rows = Vec::new();
        for (pos, row) in le.into_inner().rows.into_iter().enumerate() {
            rows.push(Row::try_from_at_pos(pos as i32, row)?.id());
        }

        GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .centered(centered)
                .rows(rows)
                .finish_and_commit(),
        )
    }
}

/// Represents a single row within a table in a document
#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum Row {
    Content(ContentRow),
    Divider(DividerRow),
}

impl Row {
    fn try_from_at_pos(
        position: i32,
        le: Located<v::Row>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(le.region());

        Ok(match le.into_inner() {
            v::Row::Content { cells } => {
                let mut cell_ids = Vec::new();
                for (pos, cell) in cells.into_iter().enumerate() {
                    cell_ids.push(
                        Cell::try_from_at_pos(position, pos as i32, cell)?.id(),
                    );
                }

                Self::from(GraphqlDatabaseError::wrap(
                    ContentRow::build()
                        .region(region)
                        .position(position)
                        .cells(cell_ids)
                        .finish_and_commit(),
                )?)
            }
            v::Row::Divider { columns } => {
                Self::from(GraphqlDatabaseError::wrap(
                    DividerRow::build()
                        .region(region)
                        .position(position)
                        .columns(
                            columns
                                .into_iter()
                                .map(ColumnAlign::from)
                                .collect(),
                        )
                        .finish_and_commit(),
                )?)
            }
        })
    }
}

/// Represents a row that acts as a divider between other rows, usually for
/// a header and later data rows
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct DividerRow {
    /// The segment of the document this row covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The position of this row amongst all rows in the table
    position: i32,

    /// The alignment of each column according to this divider
    #[ent(field, ext(async_graphql(filter_untyped)))]
    columns: Vec<ColumnAlign>,
}

#[derive(
    async_graphql::Enum,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Display,
    EnumString,
    Serialize,
    Deserialize,
)]
#[graphql(remote = "vimwiki::elements::ColumnAlign")]
#[strum(serialize_all = "snake_case")]
pub enum ColumnAlign {
    /// Align columns left
    Left,

    /// Align columns centered
    Center,

    /// Align columns right
    Right,
}

impl ValueLike for ColumnAlign {
    fn into_value(self) -> Value {
        Value::from(self.to_string())
    }

    fn try_from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Text(x) => x.as_str().parse().map_err(|_| Value::Text(x)),
            x => Err(x),
        }
    }
}

/// Represents a row that contains one or more cells of data
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ContentRow {
    /// The segment of the document this row covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The position of this row amongst all rows in the table
    position: i32,

    /// The cells contained within this row
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    cells: Vec<Cell>,
}

/// Represents a cell within a row
#[simple_ent]
#[derive(async_graphql::Union, Debug)]
pub enum Cell {
    Content(ContentCell),
    SpanLeft(SpanLeftCell),
    SpanAbove(SpanAboveCell),
}

impl Cell {
    fn try_from_at_pos(
        row_position: i32,
        position: i32,
        le: Located<v::Cell>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(le.region());
        Ok(match le.into_inner() {
            v::Cell::Content(x) => {
                let mut contents = Vec::new();
                for content in x.elements {
                    contents.push(InlineElement::try_from(content)?.id());
                }

                Self::from(GraphqlDatabaseError::wrap(
                    ContentCell::build()
                        .region(region)
                        .row_position(row_position)
                        .contents(contents)
                        .finish_and_commit(),
                )?)
            }
            v::Cell::SpanAbove => Self::from(GraphqlDatabaseError::wrap(
                SpanAboveCell::build()
                    .region(region)
                    .row_position(row_position)
                    .position(position)
                    .finish_and_commit(),
            )?),
            v::Cell::SpanLeft => Self::from(GraphqlDatabaseError::wrap(
                SpanLeftCell::build()
                    .region(region)
                    .row_position(row_position)
                    .position(position)
                    .finish_and_commit(),
            )?),
        })
    }
}

/// Represents a cell with content
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct ContentCell {
    /// The segment of the document this cell covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,

    /// Contents within the cell
    #[ent(edge(policy = "deep", wrap), ext(async_graphql(filter_untyped)))]
    contents: Vec<InlineElement>,
}

/// Represents a cell with no content that spans the left cell
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct SpanLeftCell {
    /// The segment of the document this cell covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,
}

/// Represents a cell with no content that spans the above row
#[simple_ent]
#[derive(AsyncGraphqlEnt, AsyncGraphqlEntFilter)]
pub struct SpanAboveCell {
    /// The segment of the document this cell covers
    #[ent(field, ext(async_graphql(filter_untyped)))]
    region: Region,

    /// The position of this cell amongst all cells in the row
    position: i32,

    /// The position of this cell's row amongst all rows in the table
    row_position: i32,
}
