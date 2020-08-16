use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(Constructor, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Table {
    rows: Vec<Row>,
    isCentered: bool,
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
    // TODO: This can support inline wiki syntax such as typeface, links
    content: String,
    span: CellSpan,
}

/// Represents cell spanning relative to another cell
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CellSpan {
    None,
    Left,
    Above,
}
