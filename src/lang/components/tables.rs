use super::InlineComponentContainer;
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Table {
    pub rows: Vec<Row>,
    pub centered: bool,
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Row {
    /// Represents a row containing content
    Content { cells: Vec<Cell> },

    /// Represents a row purely acting as a divider, usually for headers
    Divider,
}

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Cell {
    pub content: InlineComponentContainer,
    pub span: CellSpan,
}

/// Represents cell spanning relative to another cell
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum CellSpan {
    None,
    Left,
    Above,
}
