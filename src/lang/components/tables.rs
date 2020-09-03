use super::{InlineComponentContainer, LC};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Table {
    pub rows: Vec<LC<Row>>,
    pub centered: bool,
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Row {
    /// Represents a row containing content
    Content { cells: Vec<LC<Cell>> },

    /// Represents a row purely acting as a divider, usually for headers
    Divider,
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Cell {
    Content(InlineComponentContainer),
    SpanLeft,
    SpanAbove,
}
