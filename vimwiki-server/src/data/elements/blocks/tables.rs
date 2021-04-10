use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Page, PageQuery,
    Region,
};
use entity::*;
use serde::{Deserialize, Serialize};
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

    /// Page containing this table
    #[ent(edge)]
    page: Page,

    /// Parent element to this table
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

impl<'a> FromVimwikiElement<'a> for Table {
    type Element = Located<v::Table<'a>>;

    fn from_vimwiki_element(
        page_id: Id,
        parent_id: Option<Id>,
        element: Self::Element,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(element.region());
        let centered = element.as_inner().centered;

        let mut ent = GraphqlDatabaseError::wrap(
            Self::build()
                .region(region)
                .centered(centered)
                .rows(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut rows = Vec::new();
        for (pos, row) in element.into_inner().rows.into_iter().enumerate() {
            rows.push(
                Row::from_vimwiki_element_at_pos(
                    page_id,
                    Some(ent.id()),
                    pos as i32,
                    row,
                )?
                .id(),
            );
        }

        ent.set_rows_ids(rows);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
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
    pub fn page_id(&self) -> Id {
        match self {
            Self::Content(x) => x.page_id(),
            Self::Divider(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Content(x) => x.parent_id(),
            Self::Divider(x) => x.parent_id(),
        }
    }
}

impl Row {
    fn from_vimwiki_element_at_pos(
        page_id: Id,
        parent_id: Option<Id>,
        position: i32,
        le: Located<v::Row>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(le.region());

        Ok(match le.into_inner() {
            v::Row::Content { cells } => {
                let mut ent = GraphqlDatabaseError::wrap(
                    ContentRow::build()
                        .region(region)
                        .position(position)
                        .cells(Vec::new())
                        .page(page_id)
                        .parent(parent_id)
                        .finish_and_commit(),
                )?;

                let mut cell_ids = Vec::new();
                for (pos, cell) in cells.into_iter().enumerate() {
                    cell_ids.push(
                        Cell::from_vimwiki_element_at_pos(
                            page_id,
                            Some(ent.id()),
                            position,
                            pos as i32,
                            cell,
                        )?
                        .id(),
                    );
                }

                ent.set_cells_ids(cell_ids);
                ent.commit().map_err(GraphqlDatabaseError::Database)?;
                Self::from(ent)
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
                        .page(page_id)
                        .parent(parent_id)
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

    /// Page containing this row
    #[ent(edge)]
    page: Page,

    /// Parent element to this row
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
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

    /// Page containing this row
    #[ent(edge)]
    page: Page,

    /// Parent element to this row
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
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
    pub fn page_id(&self) -> Id {
        match self {
            Self::Content(x) => x.page_id(),
            Self::SpanLeft(x) => x.page_id(),
            Self::SpanAbove(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Content(x) => x.parent_id(),
            Self::SpanLeft(x) => x.parent_id(),
            Self::SpanAbove(x) => x.parent_id(),
        }
    }
}

impl Cell {
    fn from_vimwiki_element_at_pos(
        page_id: Id,
        parent_id: Option<Id>,
        row_position: i32,
        position: i32,
        le: Located<v::Cell>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(le.region());
        Ok(match le.into_inner() {
            v::Cell::Content(x) => {
                let mut ent = GraphqlDatabaseError::wrap(
                    ContentCell::build()
                        .region(region)
                        .row_position(row_position)
                        .position(position)
                        .contents(Vec::new())
                        .page(page_id)
                        .parent(parent_id)
                        .finish_and_commit(),
                )?;

                let mut contents = Vec::new();
                for content in x.elements {
                    contents.push(
                        InlineElement::from_vimwiki_element(
                            page_id,
                            Some(ent.id()),
                            content,
                        )?
                        .id(),
                    );
                }

                ent.set_contents_ids(contents);
                ent.commit().map_err(GraphqlDatabaseError::Database)?;
                Self::from(ent)
            }
            v::Cell::SpanAbove => Self::from(GraphqlDatabaseError::wrap(
                SpanAboveCell::build()
                    .region(region)
                    .row_position(row_position)
                    .position(position)
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )?),
            v::Cell::SpanLeft => Self::from(GraphqlDatabaseError::wrap(
                SpanLeftCell::build()
                    .region(region)
                    .row_position(row_position)
                    .position(position)
                    .page(page_id)
                    .parent(parent_id)
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

    /// Page containing this cell
    #[ent(edge)]
    page: Page,

    /// Parent element to this cell
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
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

    /// Page containing this cell
    #[ent(edge)]
    page: Page,

    /// Parent element to this cell
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
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

    /// Page containing this cell
    #[ent(edge)]
    page: Page,

    /// Parent element to this cell
    #[ent(edge(policy = "shallow", wrap), ext(async_graphql(filter_untyped)))]
    parent: Option<Element>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vimwiki_macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_table! {r#"
                |value1|value2|value3|value4|
                |------|:-----|-----:|:----:|
            "#};
            let region = Region::from(element.region());
            let ent = Table::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.centered(), &false);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));

            let rows = ent.load_rows().expect("Failed to load rows");

            match &rows[0] {
                Row::Content(row) => {
                    assert_eq!(row.page_id(), 999);
                    assert_eq!(row.parent_id(), Some(ent.id()));

                    let cells = row.load_cells().expect("Failed to load cells");
                    for cell in cells {
                        assert_eq!(cell.page_id(), 999);
                        assert_eq!(cell.parent_id(), Some(row.id()));
                    }
                }
                x => panic!("Unexpectedly got {:?}", x),
            }

            match &rows[1] {
                Row::Divider(row) => {
                    assert_eq!(row.page_id(), 999);
                    assert_eq!(row.parent_id(), Some(ent.id()));
                }
                x => panic!("Unexpectedly got {:?}", x),
            }
        });
    }
}
