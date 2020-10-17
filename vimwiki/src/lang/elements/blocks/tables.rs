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

impl Table<'_> {
    pub fn to_borrowed(&self) -> Table {
        Table {
            rows: self
                .rows
                .iter()
                .map(|x| Located::new(x.as_inner().to_borrowed(), x.region))
                .collect(),
            centered: self.centered,
        }
    }

    pub fn into_owned(self) -> Table<'static> {
        Table {
            rows: self
                .rows
                .iter()
                .map(|x| Located::new(x.as_inner().into_owned(), x.region))
                .collect(),
            centered: self.centered,
        }
    }
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

impl Row<'_> {
    pub fn to_borrowed(&self) -> Row {
        match self {
            Self::Content { cells } => Row::Content {
                cells: cells
                    .iter()
                    .map(|x| Located::new(x.as_inner().to_borrowed(), x.region))
                    .collect(),
            },
            Self::Divider => Row::Divider,
        }
    }

    pub fn into_owned(self) -> Row<'static> {
        match self {
            Self::Content { cells } => Row::Content {
                cells: cells
                    .iter()
                    .map(|x| Located::new(x.as_inner().into_owned(), x.region))
                    .collect(),
            },
            Self::Divider => Row::Divider,
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
