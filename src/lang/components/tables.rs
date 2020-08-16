use super::InlineComponent;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Table {
    rows: Vec<Row>,
    centered: bool,
}

impl Table {
    pub fn is_centered(&self) -> bool {
        self.centered
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Serialize, Deserialize)]
pub enum Row {
    /// Represents a row containing content
    Content { cells: Vec<Cell> },

    /// Represents a row purely acting as a divider, usually for headers
    Divider,
}

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Cell {
    content: Vec<InlineComponent>,
    span: CellSpan,
}

/// Represents cell spanning relative to another cell
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellSpan {
    None,
    Left,
    Above,
}
