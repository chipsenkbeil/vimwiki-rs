use super::{InlineElementContainer, Located};
use derive_more::{Constructor, From};
use serde::{Deserialize, Serialize};

#[derive(
    Constructor, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct Table<'a> {
    pub rows: Vec<Located<Row<'a>>>,
    pub centered: bool,
}

impl<'a> Table<'a> {
    pub fn get_cell(
        &self,
        row: usize,
        col: usize,
    ) -> Option<&Located<Cell<'a>>> {
        self.rows.get(row).and_then(|r| match &r.element {
            Row::Content { cells } => cells.get(col),
            _ => None,
        })
    }
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Row<'a> {
    /// Represents a row containing content
    Content { cells: Vec<Located<Cell<'a>>> },

    /// Represents a row purely acting as a divider, usually for headers
    Divider,
}

#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Cell<'a> {
    Content(InlineElementContainer<'a>),
    SpanLeft,
    SpanAbove,
}
