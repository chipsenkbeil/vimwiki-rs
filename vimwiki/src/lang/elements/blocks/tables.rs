use super::{InlineElement, InlineElementContainer, Located};
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

    /// Returns the alignment of the specified cell, using either an explicit
    /// alignment or the default alignment if none is provided
    ///
    /// NOTE: This will always return an alignment, even if no cell exists at
    ///       the specified row and column
    pub fn get_cell_alignment(&self, row: usize, col: usize) -> ColumnAlign {
        self.get_explicit_cell_alignment(row, col)
            .unwrap_or_default()
    }

    /// Returns the alignment of the specified cell if it has some divider
    /// above it to define the alignment, otherwise returns none
    pub fn get_explicit_cell_alignment(
        &self,
        row: usize,
        col: usize,
    ) -> Option<ColumnAlign> {
        // Find the divider that appears most recently above the cell, then
        // looks for the column alignment matching the cell's column
        self.rows
            .iter()
            .take(row)
            .rev()
            .find_map(|r| match r.as_inner() {
                Row::Divider { columns } => Some(columns),
                _ => None,
            })
            .and_then(|columns| columns.get(col))
            .copied()
    }

    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        self.rows
            .into_iter()
            .flat_map(|x| x.into_inner().into_children())
            .collect()
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

impl<'a> Row<'a> {
    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        match self {
            Self::Content { cells } => cells
                .into_iter()
                .flat_map(|x| x.into_inner().into_children())
                .collect(),
            _ => vec![],
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

impl<'a> Cell<'a> {
    pub fn into_children(self) -> Vec<Located<InlineElement<'a>>> {
        match self {
            Self::Content(x) => x.into_children(),
            _ => vec![],
        }
    }
}
