use crate::data::{
    Element, ElementQuery, FromVimwikiElement, GqlPageFilter,
    GraphqlDatabaseError, InlineElement, InlineElementQuery, Page, PageQuery,
    Region,
};
use entity::*;
use entity_async_graphql::*;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, fmt};
use strum::{Display, EnumString};
use vimwiki::{self as v, CellIter, Located};

/// Represents a single document table
#[gql_ent]
pub struct Table {
    /// The segment of the document this table covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The cells contained in this table
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    cells: Vec<Cell>,

    /// Whether or not the table is centered
    centered: bool,

    /// Page containing this table
    #[ent(edge)]
    page: Page,

    /// Parent element to this table
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
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
                .cells(Vec::new())
                .page(page_id)
                .parent(parent_id)
                .finish_and_commit(),
        )?;

        let mut cells = Vec::new();
        for (pos, cell) in element.into_inner().into_cells().zip_with_position()
        {
            cells.push(
                Cell::from_vimwiki_element_at_pos(
                    page_id,
                    Some(ent.id()),
                    pos,
                    cell,
                )?
                .id(),
            );
        }

        ent.set_cells_ids(cells);
        ent.commit().map_err(GraphqlDatabaseError::Database)?;

        Ok(ent)
    }
}

/// Represents a cell within a table
#[gql_ent]
#[derive(Debug)]
pub enum Cell {
    Content(ContentCell),
    Span(SpanCell),
    Align(AlignCell),
}

impl Cell {
    pub fn region(&self) -> Region {
        match self {
            Self::Content(x) => *x.region(),
            Self::Span(x) => *x.region(),
            Self::Align(x) => *x.region(),
        }
    }

    pub fn position(&self) -> CellPos {
        match self {
            Self::Content(x) => *x.position(),
            Self::Span(x) => *x.position(),
            Self::Align(x) => *x.position(),
        }
    }

    pub fn page_id(&self) -> Id {
        match self {
            Self::Content(x) => x.page_id(),
            Self::Span(x) => x.page_id(),
            Self::Align(x) => x.page_id(),
        }
    }

    pub fn parent_id(&self) -> Option<Id> {
        match self {
            Self::Content(x) => x.parent_id(),
            Self::Span(x) => x.parent_id(),
            Self::Align(x) => x.parent_id(),
        }
    }
}

impl Cell {
    fn from_vimwiki_element_at_pos(
        page_id: Id,
        parent_id: Option<Id>,
        pos: v::CellPos,
        le: Located<v::Cell>,
    ) -> Result<Self, GraphqlDatabaseError> {
        let region = Region::from(le.region());
        Ok(match le.into_inner() {
            v::Cell::Content(x) => {
                let mut ent = GraphqlDatabaseError::wrap(
                    ContentCell::build()
                        .region(region)
                        .position(CellPos::from(pos))
                        .contents(Vec::new())
                        .page(page_id)
                        .parent(parent_id)
                        .finish_and_commit(),
                )?;

                let mut contents = Vec::new();
                for content in x {
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
            v::Cell::Span(x) => Self::from(GraphqlDatabaseError::wrap(
                SpanCell::build()
                    .region(region)
                    .position(CellPos::from(pos))
                    .span(CellSpan::from(x))
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )?),
            v::Cell::Align(x) => Self::from(GraphqlDatabaseError::wrap(
                AlignCell::build()
                    .region(region)
                    .position(CellPos::from(pos))
                    .alignment(ColumnAlign::from(x))
                    .page(page_id)
                    .parent(parent_id)
                    .finish_and_commit(),
            )?),
        })
    }
}

/// Represents a cell with content
#[gql_ent]
pub struct ContentCell {
    /// The segment of the document this cell covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The position of this cell in a table
    #[ent(field(graphql(filter_untyped)))]
    position: CellPos,

    /// Contents within the cell
    #[ent(edge(policy = "deep", wrap, graphql(filter_untyped)))]
    contents: Vec<InlineElement>,

    /// The content within the cell as it would be read by humans
    /// without frills
    #[ent(field(computed = "self.to_string()"))]
    text: String,

    /// Page containing this cell
    #[ent(edge)]
    page: Page,

    /// Parent element to this cell
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
}

impl fmt::Display for ContentCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.load_contents() {
            Ok(contents) => {
                for content in contents {
                    write!(f, "{}", content.to_string())?;
                }
                Ok(())
            }
            Err(x) => {
                write!(f, "{}", x)?;
                Ok(())
            }
        }
    }
}

/// Represents a cell with no content that spans from another cell
#[gql_ent]
pub struct SpanCell {
    /// The segment of the document this cell covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The position of this cell in a table
    #[ent(field(graphql(filter_untyped)))]
    position: CellPos,

    /// The span direction
    #[ent(field(graphql(filter_untyped)))]
    span: CellSpan,

    /// Page containing this cell
    #[ent(edge)]
    page: Page,

    /// Parent element to this cell
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
}

/// Represents a cell with no content that describes future column alignment
#[gql_ent]
pub struct AlignCell {
    /// The segment of the document this cell covers
    #[ent(field(graphql(filter_untyped)))]
    region: Region,

    /// The position of this cell in a table
    #[ent(field(graphql(filter_untyped)))]
    position: CellPos,

    /// The alignment direction
    #[ent(field(graphql(filter_untyped)))]
    alignment: ColumnAlign,

    /// Page containing this cell
    #[ent(edge)]
    page: Page,

    /// Parent element to this cell
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    parent: Option<Element>,

    /// Previous sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    prev_sibling: Option<Element>,

    /// Next sibling element to this element
    #[ent(edge(policy = "shallow", wrap, graphql(filter_untyped)))]
    next_sibling: Option<Element>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    derive_more::Display,
    Serialize,
    Deserialize,
    ValueLike,
)]
#[display(fmt = "({},{})", row, col)]
pub struct CellPos {
    row: usize,
    col: usize,
}

impl PartialOrd for CellPos {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellPos {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.row.cmp(&other.row), self.col.cmp(&other.col)) {
            (Ordering::Equal, x) => x,
            (x, _) => x,
        }
    }
}

impl From<v::CellPos> for CellPos {
    fn from(pos: v::CellPos) -> Self {
        Self {
            row: pos.row,
            col: pos.col,
        }
    }
}

async_graphql::scalar!(CellPos);

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
#[graphql(remote = "vimwiki::ColumnAlign")]
#[strum(serialize_all = "snake_case")]
pub enum ColumnAlign {
    /// No alignment (defaults to left)
    None,

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
#[graphql(remote = "vimwiki::CellSpan")]
#[strum(serialize_all = "snake_case")]
pub enum CellSpan {
    /// Spanning from left cell
    FromLeft,

    /// Spanning from above cell
    FromAbove,
}

impl ValueLike for CellSpan {
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

#[cfg(test)]
mod tests {
    use super::*;
    use entity_inmemory::InmemoryDatabase;
    use vimwiki::macros::*;

    #[test]
    fn should_fully_populate_from_vimwiki_element() {
        global::with_db(InmemoryDatabase::default(), || {
            let element = vimwiki_table! {r#"
                |value1|value2|value3|value4|
                |------|:-----|-----:|:----:|
                |abc   |>     |\/    |def   |
            "#};
            let region = Region::from(element.region());
            let ent = Table::from_vimwiki_element(999, Some(123), element)
                .expect("Failed to convert from element");

            assert_eq!(ent.region(), &region);
            assert_eq!(ent.centered(), &false);
            assert_eq!(ent.page_id(), 999);
            assert_eq!(ent.parent_id(), Some(123));

            // NOTE: We sort our cells to make it easier to test
            let mut cells = ent.load_cells().expect("Failed to load cells");
            cells.sort_unstable_by_key(|cell| cell.position());

            for (i, cell) in cells.into_iter().enumerate() {
                assert_eq!(cell.page_id(), 999);
                assert_eq!(cell.parent_id(), Some(ent.id()));

                match (i, cell) {
                    (0, Cell::Content(cell)) => {
                        assert_eq!(cell.to_string(), "value1")
                    }
                    (1, Cell::Content(cell)) => {
                        assert_eq!(cell.to_string(), "value2")
                    }
                    (2, Cell::Content(cell)) => {
                        assert_eq!(cell.to_string(), "value3")
                    }
                    (3, Cell::Content(cell)) => {
                        assert_eq!(cell.to_string(), "value4")
                    }
                    (4, Cell::Align(cell)) => {
                        assert_eq!(cell.alignment, ColumnAlign::None);
                    }
                    (5, Cell::Align(cell)) => {
                        assert_eq!(cell.alignment, ColumnAlign::Left);
                    }
                    (6, Cell::Align(cell)) => {
                        assert_eq!(cell.alignment, ColumnAlign::Right);
                    }
                    (7, Cell::Align(cell)) => {
                        assert_eq!(cell.alignment, ColumnAlign::Center);
                    }
                    (8, Cell::Content(cell)) => {
                        assert_eq!(cell.to_string(), "abc   ");
                    }
                    (9, Cell::Span(cell)) => {
                        assert_eq!(cell.span, CellSpan::FromLeft);
                    }
                    (10, Cell::Span(cell)) => {
                        assert_eq!(cell.span, CellSpan::FromAbove);
                    }
                    (11, Cell::Content(cell)) => {
                        assert_eq!(cell.to_string(), "def   ");
                    }
                    (idx, cell) => {
                        panic!("Unexpected cell at {}: {:?}", idx, cell);
                    }
                }
            }
        });
    }
}
