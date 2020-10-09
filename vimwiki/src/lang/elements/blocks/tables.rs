use super::{InlineElementContainer, LE};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Table {
    pub rows: Vec<LE<Row>>,
    pub centered: bool,
}

impl Table {
    pub fn get_cell(&self, row: usize, col: usize) -> Option<&LE<Cell>> {
        self.rows.get(row).and_then(|r| match &r.element {
            Row::Content { cells } => cells.get(col),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Row {
    /// Represents a row containing content
    Content { cells: Vec<LE<Cell>> },

    /// Represents a row purely acting as a divider, usually for headers
    Divider,
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Cell {
    Content(InlineElementContainer),
    SpanLeft,
    SpanAbove,
}
