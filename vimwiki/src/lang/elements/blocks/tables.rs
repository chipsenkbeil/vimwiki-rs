use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Table<'a> {
    pub rows: Vec<Located<Row<'a>>>,
    pub centered: bool,
}

impl Table<'_> {
    pub fn to_borrowed(&self) -> Table {
        Table {
            rows: self
                .rows
                .iter()
                .map(|x| x.as_ref().map(Row::to_borrowed))
                .collect(),
            centered: self.centered,
        }
    }

    pub fn into_owned(self) -> Table<'static> {
        Table {
            rows: self
                .rows
                .into_iter()
                .map(|x| x.map(Row::into_owned))
                .collect(),
            centered: self.centered,
        }
    }
}

impl<'a> Table<'a> {
    /// Returns the alignment of the specified column within the table
    ///
    /// NOTE: This will always return an alignment, even if the column
    ///       does not exist, by using the default column alignment
    pub fn get_column_alignment(&self, col: usize) -> ColumnAlign {
        self.rows
            .iter()
            .find_map(|r| match r.as_inner() {
                Row::Divider { columns } => Some(columns),
                _ => None,
            })
            .and_then(|columns| columns.get(col))
            .copied()
            .unwrap_or_default()
    }

    /// Returns reference to the cell found at the specified row and column
    pub fn get_cell(
        &self,
        row: usize,
        col: usize,
    ) -> Option<&Located<Cell<'a>>> {
        self.rows.get(row).and_then(|r| match r.as_inner() {
            Row::Content { cells } => cells.get(col),
            _ => None,
        })
    }
}

impl<'a> IntoChildren for Table<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.rows
            .into_iter()
            .flat_map(|x| x.into_inner().into_children())
            .collect()
    }
}

impl<'a> StrictEq for Table<'a> {
    /// Performs strict_eq on rows and centered status
    fn strict_eq(&self, other: &Self) -> bool {
        self.centered == other.centered
            && self.rows.len() == other.rows.len()
            && self
                .rows
                .iter()
                .zip(other.rows.iter())
                .all(|(x, y)| x.strict_eq(y))
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Row<'a> {
    /// Represents a row containing content
    Content { cells: Vec<Located<Cell<'a>>> },

    /// Represents a row purely acting as a divider, usually for headers
    Divider { columns: Vec<ColumnAlign> },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ColumnAlign {
    Left,
    Center,
    Right,
}

impl Default for ColumnAlign {
    /// By default, columns align to the left
    fn default() -> Self {
        Self::Left
    }
}

impl StrictEq for ColumnAlign {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl Row<'_> {
    pub fn to_borrowed(&self) -> Row {
        match self {
            Self::Content { cells } => Row::Content {
                cells: cells
                    .iter()
                    .map(|x| x.as_ref().map(Cell::to_borrowed))
                    .collect(),
            },
            Self::Divider { columns } => Row::Divider {
                columns: columns.to_vec(),
            },
        }
    }

    pub fn into_owned(self) -> Row<'static> {
        match self {
            Self::Content { cells } => Row::Content {
                cells: cells
                    .into_iter()
                    .map(|x| x.map(Cell::into_owned))
                    .collect(),
            },
            Self::Divider { columns } => Row::Divider { columns },
        }
    }
}

impl<'a> IntoChildren for Row<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        match self {
            Self::Content { cells } => cells
                .into_iter()
                .flat_map(|x| x.into_inner().into_children())
                .collect(),
            _ => vec![],
        }
    }
}

impl<'a> StrictEq for Row<'a> {
    /// Performs strict_eq check on columns or cells depending on row type
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Content { cells: x }, Self::Content { cells: y }) => {
                x.len() == y.len()
                    && x.iter().zip(y.iter()).all(|(x, y)| x.strict_eq(y))
            }
            (Self::Divider { columns: x }, Self::Divider { columns: y }) => {
                x.len() == y.len()
                    && x.iter().zip(y.iter()).all(|(x, y)| x.strict_eq(y))
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Cell<'a> {
    Content(InlineElementContainer<'a>),
    SpanLeft,
    SpanAbove,
}

impl Cell<'_> {
    pub fn to_borrowed(&self) -> Cell {
        match self {
            Self::Content(x) => Cell::Content(x.to_borrowed()),
            Self::SpanLeft => Cell::SpanLeft,
            Self::SpanAbove => Cell::SpanAbove,
        }
    }

    pub fn into_owned(self) -> Cell<'static> {
        match self {
            Self::Content(x) => Cell::Content(x.into_owned()),
            Self::SpanLeft => Cell::SpanLeft,
            Self::SpanAbove => Cell::SpanAbove,
        }
    }
}

impl<'a> IntoChildren for Cell<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        match self {
            Self::Content(x) => x.into_children(),
            _ => vec![],
        }
    }
}

impl<'a> StrictEq for Cell<'a> {
    /// Performs strict_eq on cell content
    fn strict_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Content(x), Self::Content(y)) => x.strict_eq(y),
            (Self::SpanLeft, Self::SpanLeft) => true,
            (Self::SpanAbove, Self::SpanAbove) => true,
            _ => false,
        }
    }
}
