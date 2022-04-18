use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    ElementLike, StrictEq,
};
use derive_more::{Constructor, Display, Error, From, IntoIterator, IsVariant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{num::ParseIntError, str::FromStr};

/// Represents the position of a cell in a table
#[derive(
    Constructor,
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Display,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{},{}", row, col)]
pub struct CellPos {
    /// Represents the row number of a cell starting from 0
    pub row: usize,

    /// Represents the coumn number of a cell starting from 0
    pub col: usize,
}

#[derive(Debug, Display, Error, PartialEq, Eq)]
pub enum ParseCellPosError {
    TooFewItems,
    TooManyItems,
    BadRow(#[error(source)] ParseIntError),
    BadCol(#[error(source)] ParseIntError),
}

impl FromStr for CellPos {
    type Err = ParseCellPosError;

    /// Parses "{row},{col}" into [`CellPos`]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(',');
        let row_str = iter.next();
        let col_str = iter.next();

        if iter.next().is_some() {
            Err(ParseCellPosError::TooManyItems)
        } else {
            match (row_str, col_str) {
                (Some(row_str), Some(col_str)) => {
                    let row: usize = row_str
                        .trim()
                        .parse()
                        .map_err(ParseCellPosError::BadRow)?;
                    let col: usize = col_str
                        .trim()
                        .parse()
                        .map_err(ParseCellPosError::BadCol)?;
                    Ok(Self { row, col })
                }
                _ => Err(ParseCellPosError::TooFewItems),
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, IntoIterator, Serialize, Deserialize)]
pub struct Table<'a> {
    /// Represents the table's data (cells) as a mapping between a cell's
    /// position and its actual content (private)
    #[into_iterator(owned, ref, ref_mut)]
    #[serde(with = "serde_with::rust::map_as_tuple_list")]
    cells: HashMap<CellPos, Located<Cell<'a>>>,

    /// Represents the total rows contained in the table (private)
    row_cnt: usize,

    /// Represents the total columns contained in the table (private)
    col_cnt: usize,

    /// Represents whether or not the table is centered
    pub centered: bool,
}

impl ElementLike for Table<'_> {}

impl Table<'_> {
    pub fn to_borrowed(&self) -> Table {
        Table {
            cells: self
                .cells
                .iter()
                .map(|(k, v)| (*k, v.as_ref().map(Cell::to_borrowed)))
                .collect(),
            row_cnt: self.row_cnt,
            col_cnt: self.col_cnt,
            centered: self.centered,
        }
    }

    pub fn into_owned(self) -> Table<'static> {
        Table {
            cells: self
                .cells
                .into_iter()
                .map(|(k, v)| (k, v.map(Cell::into_owned)))
                .collect(),
            row_cnt: self.row_cnt,
            col_cnt: self.col_cnt,
            centered: self.centered,
        }
    }
}

impl<'a> Table<'a> {
    pub fn new<I: IntoIterator<Item = (CellPos, Located<Cell<'a>>)>>(
        cells: I,
        centered: bool,
    ) -> Self {
        let cells: HashMap<CellPos, Located<Cell>> =
            cells.into_iter().collect();
        let (max_row, max_col) = cells.keys().fold((0, 0), |acc, pos| {
            (
                std::cmp::max(acc.0, pos.row + 1),
                std::cmp::max(acc.1, pos.col + 1),
            )
        });

        Self {
            cells,
            row_cnt: max_row,
            col_cnt: max_col,
            centered,
        }
    }

    /// Returns an iterator over all rows that are considered header rows,
    /// which is all rows leading up to a divider row. If there is no divider
    /// row, then there are no header rows
    pub fn header_rows(&self) -> HeaderRows<'_, 'a> {
        HeaderRows::new(self)
    }

    /// Returns an iterator over all rows that are considered body rows,
    /// which is all rows following a divider row. If there is no divider
    /// row in the table, then all rows are considered body rows
    pub fn body_rows(&self) -> BodyRows<'_, 'a> {
        BodyRows::new(self)
    }

    /// Returns true if contains header rows
    pub fn has_header_rows(&self) -> bool {
        self.header_rows().next().is_some()
    }

    /// Returns true if contains body rows
    pub fn has_body_rows(&self) -> bool {
        self.body_rows().next().is_some()
    }

    /// Returns true if the table contains a divider row
    #[inline]
    pub fn has_divider_row(&self) -> bool {
        self.get_divider_row_index().is_some()
    }

    /// Returns the row index representing the divider row of the table
    /// (separation between header and body) if it exists
    pub fn get_divider_row_index(&self) -> Option<usize> {
        self.rows().enumerate().find_map(|(idx, row)| {
            if row.is_divider_row() {
                Some(idx)
            } else {
                None
            }
        })
    }

    /// Returns the alignment of the specified column within the table
    ///
    /// NOTE: This will always return an alignment, even if the column
    ///       does not exist, by using the default column alignment
    pub fn get_column_alignment(&self, col: usize) -> ColumnAlign {
        self.column(col)
            .find_map(|cell| cell.get_align().copied())
            .unwrap_or_default()
    }

    /// Returns the total rows contained in the table
    #[inline]
    pub fn row_cnt(&self) -> usize {
        self.row_cnt
    }

    /// Returns the total columns contained in the table
    #[inline]
    pub fn col_cnt(&self) -> usize {
        self.col_cnt
    }

    /// Returns the total cells (rows * columns) contained in the table
    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns true if the total cells (rows * columns) contained in the table
    /// is zero
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Returns raw table cell data as a reference to the hashmap
    #[inline]
    pub fn as_data(&self) -> &HashMap<CellPos, Located<Cell<'a>>> {
        &self.cells
    }

    /// Returns an iterator of refs through all rows in the table
    pub fn rows(&self) -> Rows<'_, 'a> {
        Rows::new(self)
    }

    /// Returns an iterator of refs through a specific row in the table
    pub fn row(&self, idx: usize) -> Row<'_, 'a> {
        Row::new(self, idx, 0)
    }

    /// Consumes the table and returns an iterator through a specific row in the table
    pub fn into_row(self, idx: usize) -> IntoRow<'a> {
        IntoRow::new(self, idx, 0)
    }

    /// Returns an iterator of refs through all columns in the table
    pub fn columns(&self) -> Columns<'_, 'a> {
        Columns::new(self)
    }

    /// Returns an iterator of refs through a specific column in the table
    pub fn column(&self, idx: usize) -> Column<'_, 'a> {
        Column::new(self, 0, idx)
    }

    /// Consumes the table and returns an iterator through a specific column in the table
    pub fn into_column(self, idx: usize) -> IntoColumn<'a> {
        IntoColumn::new(self, 0, idx)
    }

    /// Returns an iterator of refs through all cells in the table, starting
    /// from the first row, iterating through all cells from beginning to end,
    /// and then moving on to the next row
    pub fn cells(&self) -> Cells<'_, 'a> {
        Cells::new(self)
    }

    /// Consumes the table and returns an iterator through all cells in the
    /// table, starting from the first row, iterating through all cells from
    /// beginning to end, and then moving on to the next row
    pub fn into_cells(self) -> IntoCells<'a> {
        IntoCells::new(self)
    }

    /// Returns reference to the cell found at the specified row and column
    pub fn get_cell(
        &self,
        row: usize,
        col: usize,
    ) -> Option<&Located<Cell<'a>>> {
        self.cells.get(&CellPos { row, col })
    }

    /// Returns mut reference to the cell found at the specified row and column
    pub fn get_mut_cell(
        &mut self,
        row: usize,
        col: usize,
    ) -> Option<&mut Located<Cell<'a>>> {
        self.cells.get_mut(&CellPos { row, col })
    }

    /// Returns the cell's rowspan, which is the number of rows (including
    /// itself) that the cell spans. 1 means that the cell only spans its
    /// starting row whereas >1 indicates it is 1 or more rows below its
    /// starting row
    ///
    /// Returns 0 for a non-content cell or a cell that doesn't exist
    pub fn get_cell_rowspan(&self, row: usize, col: usize) -> usize {
        let mut it = self.column(col).skip(row);

        // Verify that the cell at the specified location is content,
        // otherwise we return 0
        match it.next() {
            Some(cell) if cell.is_content() => {}
            _ => return 0,
        }

        it.take_while(|cell| {
            matches!(cell.get_span().copied(), Some(CellSpan::FromAbove))
        })
        .count()
            + 1
    }

    /// Returns the cell's colspan, which is the number of columns (including
    /// itself) that the cell spans. 1 means that the cell only spans its
    /// starting column whereas >1 indicates it is 1 or more columns after its
    /// starting column
    ///
    /// Returns 0 for a non-content cell or a cell that doesn't exist
    pub fn get_cell_colspan(&self, row: usize, col: usize) -> usize {
        let mut it = self.row(row).skip(col);

        // Verify that the cell at the specified location is content,
        // otherwise we return 0
        match it.next() {
            Some(cell) if cell.is_content() => {}
            _ => return 0,
        }

        it.take_while(|cell| {
            matches!(cell.get_span().copied(), Some(CellSpan::FromLeft))
        })
        .count()
            + 1
    }
}

impl<'a> IntoChildren for Table<'a> {
    type Child = Located<InlineElement<'a>>;

    fn into_children(self) -> Vec<Self::Child> {
        self.cells
            .into_iter()
            .flat_map(|(_, x)| x.into_inner().into_children())
            .collect()
    }
}

impl<'a> StrictEq for Table<'a> {
    /// Performs strict_eq on cells and centered status
    fn strict_eq(&self, other: &Self) -> bool {
        self.centered == other.centered
            && self.cells.len() == other.cells.len()
            && self.cells.iter().all(|(k, v)| {
                other.cells.get(k).map_or(false, |v2| v.strict_eq(v2))
            })
    }
}

/// Represents a cell within a table that is either content, span (indicating
/// that another cell fills this cell), or a column alignment indicator
#[derive(
    Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize, IsVariant,
)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum Cell<'a> {
    Content(InlineElementContainer<'a>),
    Span(CellSpan),
    Align(ColumnAlign),
}

impl ElementLike for Cell<'_> {}

impl Cell<'_> {
    pub fn to_borrowed(&self) -> Cell {
        match self {
            Self::Content(x) => Cell::Content(x.to_borrowed()),
            Self::Span(x) => Cell::Span(*x),
            Self::Align(x) => Cell::Align(*x),
        }
    }

    pub fn into_owned(self) -> Cell<'static> {
        match self {
            Self::Content(x) => Cell::Content(x.into_owned()),
            Self::Span(x) => Cell::Span(x),
            Self::Align(x) => Cell::Align(x),
        }
    }
}

impl<'a> Cell<'a> {
    /// Returns a reference to the content of the cell if it has content
    #[inline]
    pub fn get_content(&self) -> Option<&InlineElementContainer<'a>> {
        match self {
            Self::Content(x) => Some(x),
            _ => None,
        }
    }

    /// Returns a reference to the span of the cell if it is a span
    #[inline]
    pub fn get_span(&self) -> Option<&CellSpan> {
        match self {
            Self::Span(x) => Some(x),
            _ => None,
        }
    }

    /// Returns a reference to the column alignment of the cell if it is a
    /// column alignment
    #[inline]
    pub fn get_align(&self) -> Option<&ColumnAlign> {
        match self {
            Self::Align(x) => Some(x),
            _ => None,
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
            (Self::Span(x), Self::Span(y)) => x == y,
            (Self::Align(x), Self::Align(y)) => x == y,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum CellSpan {
    FromLeft,
    FromAbove,
}

impl StrictEq for CellSpan {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum ColumnAlign {
    None,
    Left,
    Center,
    Right,
}

impl Default for ColumnAlign {
    fn default() -> Self {
        Self::None
    }
}

impl StrictEq for ColumnAlign {
    /// Same as PartialEq
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

pub use iter::*;

mod iter {
    use super::{Cell, CellPos, Located, Table};
    use derive_more::Constructor;
    use std::marker::PhantomData;

    /// Represents an iterator over some part of a table at the granularity
    /// of individual cells within the table
    pub trait CellIter<T>: std::iter::Iterator<Item = T> + Sized {
        /// Returns the row of the next item returned by the iterator
        fn row(&self) -> usize;

        /// Returns the column of the next item returned by the iterator
        fn col(&self) -> usize;

        /// Consumes next item in iterator, returning it with the cell's position
        fn next_with_pos(&mut self) -> Option<(CellPos, T)> {
            let pos = CellPos {
                row: self.row(),
                col: self.col(),
            };
            self.next().map(move |x| (pos, x))
        }

        /// Zips up a cell iterator with the cell's position
        fn zip_with_position(self) -> ZipCellPos<Self, T> {
            ZipCellPos(self, PhantomData)
        }
    }

    /// Represents an iterator over some cell and its position
    #[derive(Debug)]
    pub struct ZipCellPos<I: CellIter<T>, T>(I, PhantomData<T>);

    impl<I: CellIter<T>, T> Iterator for ZipCellPos<I, T> {
        type Item = (CellPos, T);

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next_with_pos()
        }
    }

    /// Represents an iterator over rows of a table
    #[derive(Debug)]
    pub struct Rows<'a, 'b> {
        table: &'a Table<'b>,
        idx: usize,
    }

    impl<'a, 'b> Rows<'a, 'b> {
        /// Produces an iterator that will iterator through all rows from the
        /// beginning of the table
        pub fn new(table: &'a Table<'b>) -> Self {
            Self { table, idx: 0 }
        }

        /// Produces an iterator that will return no rows
        pub fn empty(table: &'a Table<'b>) -> Self {
            Self {
                table,
                idx: table.row_cnt(),
            }
        }

        /// Returns true if the iterator has at least one row remaining that
        /// contains a content cell
        pub fn has_content(&self) -> bool {
            let mut rows = Rows {
                table: self.table,
                idx: self.idx,
            };

            rows.any(|row| row.has_content())
        }
    }

    impl<'a, 'b> Iterator for Rows<'a, 'b> {
        type Item = Row<'a, 'b>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.idx < self.table.row_cnt() {
                let row = Row::new(self.table, self.idx, 0);
                self.idx += 1;
                Some(row)
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.row_cnt() - self.idx;
            (remaining, Some(remaining))
        }
    }

    /// Represents an iterator over rows of a table that belong to its header
    #[derive(Debug)]
    pub struct HeaderRows<'a, 'b> {
        table: &'a Table<'b>,
        idx: usize,
        len: usize,
    }

    impl<'a, 'b> HeaderRows<'a, 'b> {
        /// Produces an iterator that will iterator through all header rows
        /// from the beginning of the table (no divider rows included)
        pub fn new(table: &'a Table<'b>) -> Self {
            Self {
                table,
                idx: 0,
                len: table.get_divider_row_index().unwrap_or_default(),
            }
        }

        /// Returns true if the iterator has at least one row remaining that
        /// contains a content cell
        pub fn has_content(&self) -> bool {
            let mut rows = HeaderRows {
                table: self.table,
                idx: self.idx,
                len: self.len,
            };

            rows.any(|row| row.has_content())
        }
    }

    impl<'a, 'b> Iterator for HeaderRows<'a, 'b> {
        type Item = Row<'a, 'b>;

        fn next(&mut self) -> Option<Self::Item> {
            // Continually advance our ptr while we still have potential
            // header rows AND our current row is a divider
            while self.idx < self.len {
                let row = Row::new(self.table, self.idx, 0);
                self.idx += 1;

                if !row.is_divider_row() {
                    return Some(row);
                }
            }

            None
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.len - self.idx;
            (remaining, Some(remaining))
        }
    }

    /// Represents an iterator over rows of a table that belong to its body
    #[derive(Debug)]
    pub struct BodyRows<'a, 'b> {
        table: &'a Table<'b>,
        idx: usize,
    }

    impl<'a, 'b> BodyRows<'a, 'b> {
        /// Produces an iterator that will iterator through all body rows
        /// from the beginning of the table (no divider rows included)
        pub fn new(table: &'a Table<'b>) -> Self {
            Self {
                table,

                // Find index just passed divider row, or start from beginning
                idx: table
                    .get_divider_row_index()
                    .map(|x| x + 1)
                    .unwrap_or_default(),
            }
        }

        /// Returns true if the iterator has at least one row remaining that
        /// contains a content cell
        pub fn has_content(&self) -> bool {
            let mut rows = BodyRows {
                table: self.table,
                idx: self.idx,
            };

            rows.any(|row| row.has_content())
        }
    }

    impl<'a, 'b> Iterator for BodyRows<'a, 'b> {
        type Item = Row<'a, 'b>;

        fn next(&mut self) -> Option<Self::Item> {
            // Continually advance our ptr while we still have potential
            // body rows AND our current row is a divider
            while self.idx < self.table.row_cnt() {
                let row = Row::new(self.table, self.idx, 0);
                self.idx += 1;

                if !row.is_divider_row() {
                    return Some(row);
                }
            }

            None
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.row_cnt() - self.idx;
            (remaining, Some(remaining))
        }
    }

    /// Represents an iterator over cells within a row of a table
    #[derive(Constructor, Debug)]
    pub struct Row<'a, 'b> {
        table: &'a Table<'b>,
        row: usize,
        col: usize,
    }

    impl<'a, 'b> Row<'a, 'b> {
        pub fn is_divider_row(&self) -> bool {
            // NOTE: Due to way that table is built, we only need to check
            //       the first cell in a row to determine if it's a divider
            self.table
                .get_cell(self.row, 0)
                .map_or(false, |cell| cell.is_align())
        }

        /// Returns true if the iterator has at least one content cell
        pub fn has_content(&self) -> bool {
            let mut row = Row {
                table: self.table,
                row: self.row,
                col: self.col,
            };

            row.any(|cell| cell.is_content())
        }
    }

    impl<'a, 'b> Iterator for Row<'a, 'b> {
        type Item = &'a Located<Cell<'b>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell = self.table.get_cell(self.row, self.col);
            if cell.is_some() {
                self.col += 1;
            }
            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.col_cnt() - self.col;
            (remaining, Some(remaining))
        }
    }

    impl<'a, 'b> CellIter<&'a Located<Cell<'b>>> for Row<'a, 'b> {
        fn row(&self) -> usize {
            self.row
        }

        fn col(&self) -> usize {
            self.col
        }
    }

    /// Represents an iterator over cells within a row of a table
    #[derive(Constructor, Debug)]
    pub struct IntoRow<'a> {
        table: Table<'a>,
        row: usize,
        col: usize,
    }

    impl<'a> IntoRow<'a> {
        pub fn is_divider_row(&self) -> bool {
            // NOTE: Due to way that table is built, we only need to check
            //       the first cell in a row to determine if it's a divider
            self.table
                .get_cell(self.row, 0)
                .map_or(false, |cell| cell.is_align())
        }

        /// Returns true if the iterator has at least one content cell
        pub fn has_content(&self) -> bool {
            Row::from(self).has_content()
        }
    }

    impl<'a, 'b> From<&'a IntoRow<'b>> for Row<'a, 'b> {
        fn from(it: &'a IntoRow<'b>) -> Self {
            Self {
                table: &it.table,
                row: it.row,
                col: it.col,
            }
        }
    }

    impl<'a> Iterator for IntoRow<'a> {
        type Item = Located<Cell<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell =
                self.table.cells.remove(&CellPos::new(self.row, self.col));
            if cell.is_some() {
                self.col += 1;
            }
            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.col_cnt() - self.col;
            (remaining, Some(remaining))
        }
    }

    impl<'a> CellIter<Located<Cell<'a>>> for IntoRow<'a> {
        fn row(&self) -> usize {
            self.row
        }

        fn col(&self) -> usize {
            self.col
        }
    }

    /// Represents an iterator over columns of a table
    #[derive(Debug)]
    pub struct Columns<'a, 'b> {
        table: &'a Table<'b>,
        idx: usize,
    }

    impl<'a, 'b> Columns<'a, 'b> {
        /// Produces an iterator that will iterator through all columns from the
        /// beginning of the table
        pub fn new(table: &'a Table<'b>) -> Self {
            Self { table, idx: 0 }
        }

        /// Produces an iterator that will return no columns
        pub fn empty(table: &'a Table<'b>) -> Self {
            Self {
                table,
                idx: table.col_cnt(),
            }
        }

        /// Returns true if the iterator has at least one column remaining that
        /// contains a content cell
        pub fn has_content(&self) -> bool {
            let mut columns = Columns {
                table: self.table,
                idx: self.idx,
            };

            columns.any(|column| column.has_content())
        }
    }

    impl<'a, 'b> Iterator for Columns<'a, 'b> {
        type Item = Column<'a, 'b>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.idx < self.table.col_cnt() {
                let col = Column::new(self.table, 0, self.idx);
                self.idx += 1;
                Some(col)
            } else {
                None
            }
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.col_cnt() - self.idx;
            (remaining, Some(remaining))
        }
    }

    /// Represents an iterator over cells within a column of a table
    #[derive(Constructor, Debug)]
    pub struct Column<'a, 'b> {
        table: &'a Table<'b>,
        row: usize,
        col: usize,
    }

    impl<'a, 'b> Column<'a, 'b> {
        /// Returns true if the iterator has at least one content cell
        pub fn has_content(&self) -> bool {
            let mut column = Column {
                table: self.table,
                row: self.row,
                col: self.col,
            };

            column.any(|cell| cell.is_content())
        }
    }

    impl<'a, 'b> Iterator for Column<'a, 'b> {
        type Item = &'a Located<Cell<'b>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell = self.table.get_cell(self.row, self.col);
            if cell.is_some() {
                self.row += 1;
            }
            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.row_cnt() - self.row;
            (remaining, Some(remaining))
        }
    }

    impl<'a, 'b> CellIter<&'a Located<Cell<'b>>> for Column<'a, 'b> {
        fn row(&self) -> usize {
            self.row
        }

        fn col(&self) -> usize {
            self.col
        }
    }

    /// Represents an iterator over cells within a column of a table
    #[derive(Constructor, Debug)]
    pub struct IntoColumn<'a> {
        table: Table<'a>,
        row: usize,
        col: usize,
    }

    impl<'a> IntoColumn<'a> {
        /// Returns true if the iterator has at least one content cell
        pub fn has_content(&self) -> bool {
            Column::from(self).has_content()
        }
    }

    impl<'a, 'b> From<&'a IntoColumn<'b>> for Column<'a, 'b> {
        fn from(it: &'a IntoColumn<'b>) -> Self {
            Self {
                table: &it.table,
                row: it.row,
                col: it.col,
            }
        }
    }

    impl<'a> Iterator for IntoColumn<'a> {
        type Item = Located<Cell<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell =
                self.table.cells.remove(&CellPos::new(self.row, self.col));
            if cell.is_some() {
                self.row += 1;
            }
            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining = self.table.row_cnt() - self.row;
            (remaining, Some(remaining))
        }
    }

    impl<'a> CellIter<Located<Cell<'a>>> for IntoColumn<'a> {
        fn row(&self) -> usize {
            self.row
        }

        fn col(&self) -> usize {
            self.col
        }
    }

    /// Represents an iterator over cells within a table
    #[derive(Debug)]
    pub struct Cells<'a, 'b> {
        table: &'a Table<'b>,
        row: usize,
        col: usize,
    }

    impl<'a, 'b> Cells<'a, 'b> {
        pub fn new(table: &'a Table<'b>) -> Self {
            Self {
                table,
                row: 0,
                col: 0,
            }
        }

        /// Returns true if the iterator has at least one content cell
        pub fn has_content(&self) -> bool {
            let mut cells = Cells {
                table: self.table,
                row: self.row,
                col: self.col,
            };

            cells.any(|cell| cell.is_content())
        }
    }

    impl<'a, 'b> Iterator for Cells<'a, 'b> {
        type Item = &'a Located<Cell<'b>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell = self.table.get_cell(self.row, self.col);
            let col_cnt = self.table.col_cnt();
            let row_cnt = self.table.row_cnt();

            // If not yet reached end of row, advance column ptr
            if self.col + 1 < col_cnt {
                self.col += 1;

            // Else if not yet reached end of all rows, advance row ptr and
            // reset column ptr
            } else if self.row + 1 < row_cnt {
                self.row += 1;
                self.col = 0;

            // Otherwise, we have reached the end, so ensure we are done
            } else {
                self.row = row_cnt;
                self.col = col_cnt;
            }

            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let consumed = (self.row * self.table.col_cnt()) + self.col;
            let total = self.table.len();
            let remaining = if total > consumed {
                total - consumed
            } else {
                0
            };
            (remaining, Some(remaining))
        }
    }

    impl<'a, 'b> CellIter<&'a Located<Cell<'b>>> for Cells<'a, 'b> {
        fn row(&self) -> usize {
            self.row
        }

        fn col(&self) -> usize {
            self.col
        }
    }

    /// Represents an iterator over cells within a table
    #[derive(Debug)]
    pub struct IntoCells<'a> {
        table: Table<'a>,
        row: usize,
        col: usize,
    }

    impl<'a> IntoCells<'a> {
        pub fn new(table: Table<'a>) -> Self {
            Self {
                table,
                row: 0,
                col: 0,
            }
        }

        /// Returns true if the iterator has at least one content cell
        pub fn has_content(&self) -> bool {
            Cells::from(self).has_content()
        }
    }

    impl<'a, 'b> From<&'a IntoCells<'b>> for Cells<'a, 'b> {
        fn from(it: &'a IntoCells<'b>) -> Self {
            Self {
                table: &it.table,
                row: it.row,
                col: it.col,
            }
        }
    }

    impl<'a> Iterator for IntoCells<'a> {
        type Item = Located<Cell<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell =
                self.table.cells.remove(&CellPos::new(self.row, self.col));
            let col_cnt = self.table.col_cnt();
            let row_cnt = self.table.row_cnt();

            // If not yet reached end of row, advance column ptr
            if self.col + 1 < col_cnt {
                self.col += 1;

            // Else if not yet reached end of all rows, advance row ptr and
            // reset column ptr
            } else if self.row + 1 < row_cnt {
                self.row += 1;
                self.col = 0;

            // Otherwise, we have reached the end, so ensure we are done
            } else {
                self.row = row_cnt;
                self.col = col_cnt;
            }

            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let consumed = (self.row * self.table.col_cnt()) + self.col;
            let total = self.table.len();
            let remaining = if total > consumed {
                total - consumed
            } else {
                0
            };
            (remaining, Some(remaining))
        }
    }

    impl<'a> CellIter<Located<Cell<'a>>> for IntoCells<'a> {
        fn row(&self) -> usize {
            self.row
        }

        fn col(&self) -> usize {
            self.col
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Text;

    fn make_span_cell(
        row: usize,
        col: usize,
        span: CellSpan,
    ) -> (CellPos, Located<Cell<'static>>) {
        (CellPos { row, col }, Located::from(Cell::Span(span)))
    }

    fn make_align_cell(
        row: usize,
        col: usize,
        align: ColumnAlign,
    ) -> (CellPos, Located<Cell<'static>>) {
        (CellPos { row, col }, Located::from(Cell::Align(align)))
    }

    fn make_content_cell(
        row: usize,
        col: usize,
        text: &str,
    ) -> (CellPos, Located<Cell>) {
        (
            CellPos { row, col },
            Located::from(Cell::Content(InlineElementContainer::new(vec![
                Located::from(InlineElement::Text(Text::from(text))),
            ]))),
        )
    }

    #[test]
    fn cell_pos_should_support_being_parsed_from_str() {
        assert!("".parse::<CellPos>().is_err());
        assert_eq!("1".parse::<CellPos>(), Err(ParseCellPosError::TooFewItems));
        assert!(",1".parse::<CellPos>().is_err());
        assert_eq!(
            "1,2,3".parse::<CellPos>(),
            Err(ParseCellPosError::TooManyItems)
        );
        assert!(matches!(
            "a,1".parse::<CellPos>(),
            Err(ParseCellPosError::BadRow(_))
        ));
        assert!(matches!(
            "1,a".parse::<CellPos>(),
            Err(ParseCellPosError::BadCol(_))
        ));
        assert_eq!(
            "123,456".parse::<CellPos>(),
            Ok(CellPos { row: 123, col: 456 })
        );
    }

    #[test]
    fn table_new_should_calculate_row_and_column_counts_from_max_row_and_column(
    ) {
        let table = Table::new(vec![], false);
        assert_eq!(table.row_cnt(), 0);
        assert_eq!(table.col_cnt(), 0);

        let table =
            Table::new(vec![make_align_cell(3, 2, ColumnAlign::None)], false);
        assert_eq!(table.row_cnt(), 4);
        assert_eq!(table.col_cnt(), 3);

        let table = Table::new(
            vec![
                make_align_cell(3, 0, ColumnAlign::None),
                make_align_cell(0, 5, ColumnAlign::None),
            ],
            false,
        );
        assert_eq!(table.row_cnt(), 4);
        assert_eq!(table.col_cnt(), 6);
    }

    #[test]
    fn table_has_header_rows_should_indicate_whether_or_not_header_rows_exist()
    {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "body cell 1"),
                make_content_cell(1, 0, "body cell 2"),
            ],
            false,
        );
        assert!(!table.has_header_rows());

        let table = Table::new(
            vec![
                make_content_cell(0, 0, "header cell"),
                make_align_cell(1, 0, ColumnAlign::None),
                make_content_cell(2, 0, "body cell"),
            ],
            false,
        );
        assert!(table.has_header_rows());
    }

    #[test]
    fn table_has_body_rows_should_indicate_whether_or_not_body_rows_exist() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "header cell"),
                make_align_cell(1, 0, ColumnAlign::None),
                make_content_cell(2, 0, "body cell"),
            ],
            false,
        );
        assert!(table.has_body_rows());

        let table = Table::new(
            vec![
                make_content_cell(0, 0, "header cell"),
                make_align_cell(1, 0, ColumnAlign::None),
            ],
            false,
        );
        assert!(!table.has_body_rows());
    }

    #[test]
    fn table_has_divider_row_should_indicate_whether_or_not_divider_row_exists()
    {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "header cell"),
                make_align_cell(1, 0, ColumnAlign::None),
                make_content_cell(2, 0, "body cell"),
            ],
            false,
        );
        assert!(table.has_divider_row());

        let table = Table::new(
            vec![
                make_content_cell(0, 0, "body cell 1"),
                make_content_cell(1, 0, "body cell 2"),
            ],
            false,
        );
        assert!(!table.has_divider_row());
    }

    #[test]
    fn table_get_divider_row_index_should_return_index_of_divider_row_if_it_has_one(
    ) {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "header cell"),
                make_align_cell(1, 0, ColumnAlign::None),
                make_content_cell(2, 0, "body cell"),
            ],
            false,
        );
        assert_eq!(table.get_divider_row_index(), Some(1));

        let table = Table::new(
            vec![
                make_content_cell(0, 0, "body cell 1"),
                make_content_cell(1, 0, "body cell 2"),
            ],
            false,
        );
        assert_eq!(table.get_divider_row_index(), None);
    }

    #[test]
    fn table_get_column_alignment_should_return_align_from_first_align_cell_found(
    ) {
        let table = Table::new(
            vec![
                make_align_cell(0, 0, ColumnAlign::Right),
                make_align_cell(0, 1, ColumnAlign::Left),
                make_align_cell(1, 0, ColumnAlign::Center),
                make_align_cell(1, 1, ColumnAlign::None),
            ],
            false,
        );
        assert_eq!(table.get_column_alignment(0), ColumnAlign::Right);
        assert_eq!(table.get_column_alignment(1), ColumnAlign::Left);
    }

    #[test]
    fn table_get_column_alignment_should_return_default_if_no_align_cells() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "a"),
                make_content_cell(0, 1, "b"),
                make_content_cell(1, 0, "c"),
                make_content_cell(1, 1, "d"),
            ],
            false,
        );
        assert_eq!(table.get_column_alignment(0), ColumnAlign::default());
        assert_eq!(table.get_column_alignment(1), ColumnAlign::default());
    }

    #[test]
    fn table_get_cell_should_return_ref_to_cell_at_location() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "a"),
                make_content_cell(0, 1, "b"),
                make_content_cell(1, 0, "c"),
                make_content_cell(1, 1, "d"),
            ],
            false,
        );
        assert_eq!(
            table
                .get_cell(0, 0)
                .and_then(|x| x.get_content())
                .map(ToString::to_string),
            Some(String::from("a"))
        );
        assert_eq!(
            table
                .get_cell(0, 1)
                .and_then(|x| x.get_content())
                .map(ToString::to_string),
            Some(String::from("b"))
        );
        assert_eq!(
            table
                .get_cell(1, 0)
                .and_then(|x| x.get_content())
                .map(ToString::to_string),
            Some(String::from("c"))
        );
        assert_eq!(
            table
                .get_cell(1, 1)
                .and_then(|x| x.get_content())
                .map(ToString::to_string),
            Some(String::from("d"))
        );
        assert_eq!(table.get_cell(1, 2), None);
    }

    #[test]
    fn table_get_mut_cell_should_return_mut_ref_to_cell_at_location() {
        let mut table = Table::new(
            vec![
                make_content_cell(0, 0, "a"),
                make_content_cell(0, 1, "b"),
                make_content_cell(1, 0, "c"),
                make_content_cell(1, 1, "d"),
            ],
            false,
        );
        *table.get_mut_cell(0, 0).unwrap() = make_content_cell(0, 0, "e").1;
        assert_eq!(
            table
                .get_cell(0, 0)
                .and_then(|x| x.get_content())
                .map(ToString::to_string),
            Some(String::from("e"))
        );
    }

    #[test]
    fn table_get_cell_rowspan_should_return_0_if_no_cell_at_location() {
        let table = Table::new(vec![make_content_cell(0, 0, "content")], false);
        assert_eq!(table.get_cell_rowspan(999, 0), 0);
    }

    #[test]
    fn table_get_cell_rowspan_should_return_0_if_not_a_content_cell() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_span_cell(1, 0, CellSpan::FromAbove),
            ],
            false,
        );
        assert_eq!(table.get_cell_rowspan(1, 0), 0);
    }

    #[test]
    fn table_get_cell_rowspan_should_return_1_if_regular_cell_with_no_span() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_content_cell(1, 0, "content"),
                make_content_cell(2, 0, "content"),
            ],
            false,
        );
        assert_eq!(table.get_cell_rowspan(0, 0), 1);
    }

    #[test]
    fn table_get_cell_rowspan_should_return_more_than_1_if_spans_from_above() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_span_cell(1, 0, CellSpan::FromAbove),
                make_span_cell(2, 0, CellSpan::FromAbove),
            ],
            false,
        );
        assert_eq!(table.get_cell_rowspan(0, 0), 3);

        // Does not count other types of spans
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_span_cell(1, 0, CellSpan::FromAbove),
                make_span_cell(2, 0, CellSpan::FromAbove),
                make_span_cell(3, 0, CellSpan::FromLeft),
            ],
            false,
        );
        assert_eq!(table.get_cell_rowspan(0, 0), 3);
    }

    #[test]
    fn table_get_cell_colspan_should_return_0_if_no_cell_at_location() {
        let table = Table::new(vec![make_content_cell(0, 0, "content")], false);
        assert_eq!(table.get_cell_colspan(0, 999), 0);
    }

    #[test]
    fn table_get_cell_colspan_should_return_0_if_not_a_content_cell() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_span_cell(0, 1, CellSpan::FromLeft),
            ],
            false,
        );
        assert_eq!(table.get_cell_colspan(0, 1), 0);
    }

    #[test]
    fn table_get_cell_colspan_should_return_1_if_regular_cell_with_no_span() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_content_cell(0, 1, "content"),
                make_content_cell(0, 2, "content"),
            ],
            false,
        );
        assert_eq!(table.get_cell_colspan(0, 0), 1);
    }

    #[test]
    fn table_get_cell_colspan_should_return_more_than_1_if_spans_from_left() {
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_span_cell(0, 1, CellSpan::FromLeft),
                make_span_cell(0, 2, CellSpan::FromLeft),
            ],
            false,
        );
        assert_eq!(table.get_cell_colspan(0, 0), 3);

        // Does not count other types of spans
        let table = Table::new(
            vec![
                make_content_cell(0, 0, "content"),
                make_span_cell(0, 1, CellSpan::FromLeft),
                make_span_cell(0, 2, CellSpan::FromLeft),
                make_span_cell(0, 3, CellSpan::FromAbove),
            ],
            false,
        );
        assert_eq!(table.get_cell_colspan(0, 0), 3);
    }

    #[test]
    fn cell_get_content_should_return_some_content_if_content_variant() {
        let cell = Cell::Span(CellSpan::FromLeft);
        assert!(cell.get_content().is_none());

        let cell = Cell::Content(InlineElementContainer::new(Vec::new()));
        assert!(cell.get_content().is_some());
    }

    #[test]
    fn cell_get_span_should_return_some_span_if_span_variant() {
        let cell = Cell::Content(InlineElementContainer::new(Vec::new()));
        assert!(cell.get_span().is_none());

        let cell = Cell::Span(CellSpan::FromLeft);
        assert!(cell.get_span().is_some());
    }

    #[test]
    fn cell_get_align_should_return_some_align_if_align_variant() {
        let cell = Cell::Content(InlineElementContainer::new(Vec::new()));
        assert!(cell.get_align().is_none());

        let cell = Cell::Align(ColumnAlign::None);
        assert!(cell.get_align().is_some());
    }

    mod iter {
        use super::*;

        #[test]
        fn rows_has_content_should_return_true_if_any_remaining_row_has_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.rows().has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.rows().has_content());
        }

        #[test]
        fn rows_next_should_return_next_row_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut rows = table.rows();
            assert!(rows.next().is_some());
        }

        #[test]
        fn rows_next_should_return_none_if_no_more_rows_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut rows = table.rows();
            rows.next();
            assert!(rows.next().is_none());
        }

        #[test]
        fn rows_size_hint_should_return_remaining_rows_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut rows = table.rows();
            assert_eq!(rows.size_hint(), (1, Some(1)));

            rows.next();
            assert_eq!(rows.size_hint(), (0, Some(0)));
        }

        #[test]
        fn header_rows_has_content_should_return_true_if_any_remaining_header_row_has_content(
        ) {
            // Table with no header rows
            let table =
                Table::new(vec![make_content_cell(0, 0, "body")], false);
            assert!(!table.header_rows().has_content());

            // Table with one header row w/ content
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            assert!(table.header_rows().has_content());

            // Table with one header row w/o content
            let table = Table::new(
                vec![
                    make_span_cell(0, 0, CellSpan::FromLeft),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            assert!(!table.header_rows().has_content());
        }

        #[test]
        fn header_rows_next_should_return_next_header_row_if_available() {
            // Table with no header rows
            let table =
                Table::new(vec![make_content_cell(0, 0, "body")], false);

            let mut rows = table.header_rows();
            assert!(rows.next().is_none());

            // Table with one header row
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            let mut rows = table.header_rows();
            assert!(rows.next().is_some());
        }

        #[test]
        fn header_rows_next_should_return_none_if_no_more_header_rows_available(
        ) {
            // Table with no header rows
            let table =
                Table::new(vec![make_content_cell(0, 0, "body")], false);

            let mut rows = table.header_rows();
            assert!(rows.next().is_none());

            // Table with one header row
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            let mut rows = table.header_rows();
            rows.next();
            assert!(rows.next().is_none());
        }

        #[test]
        fn header_rows_size_hint_should_return_remaining_header_rows_as_both_bounds(
        ) {
            // Table with no header rows
            let table =
                Table::new(vec![make_content_cell(0, 0, "body")], false);
            assert_eq!(table.header_rows().size_hint(), (0, Some(0)));

            // Table with one header row
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            let mut rows = table.header_rows();
            assert_eq!(rows.size_hint(), (1, Some(1)));

            rows.next();
            assert_eq!(rows.size_hint(), (0, Some(0)));
        }

        #[test]
        fn body_rows_has_content_should_return_true_if_any_remaining_body_row_has_content(
        ) {
            // Table with no body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                ],
                false,
            );
            assert!(!table.body_rows().has_content());

            // Table with mix of header and body rows w/ content
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            assert!(table.body_rows().has_content());

            // Table with mix of header and body rows w/o content
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_span_cell(2, 0, CellSpan::FromAbove),
                ],
                false,
            );
            assert!(!table.body_rows().has_content());
        }

        #[test]
        fn body_rows_next_should_return_next_body_row_if_available() {
            // Table with no body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                ],
                false,
            );

            let mut rows = table.body_rows();
            assert!(rows.next().is_none());

            // Table with mix of header and body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            let mut rows = table.body_rows();
            assert!(rows.next().is_some());
        }

        #[test]
        fn body_rows_next_should_return_none_if_no_more_body_rows_available() {
            // Table with no body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                ],
                false,
            );

            let mut rows = table.body_rows();
            assert!(rows.next().is_none());

            // Table with mix of header and body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            let mut rows = table.body_rows();
            rows.next();
            assert!(rows.next().is_none());
        }

        #[test]
        fn body_rows_size_hint_should_return_remaining_body_rows_as_both_bounds(
        ) {
            // Table with no body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                ],
                false,
            );
            assert_eq!(table.body_rows().size_hint(), (0, Some(0)));

            // Table with mix of header and body rows
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "header"),
                    make_align_cell(1, 0, ColumnAlign::default()),
                    make_content_cell(2, 0, "body"),
                ],
                false,
            );
            let mut rows = table.body_rows();
            assert_eq!(rows.size_hint(), (1, Some(1)));

            rows.next();
            assert_eq!(rows.size_hint(), (0, Some(0)));
        }

        #[test]
        fn row_has_content_should_return_true_if_any_remaining_cells_have_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.row(0).has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.row(0).has_content());
        }

        #[test]
        fn row_is_divider_row_should_return_true_if_made_up_of_align_cells() {
            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(table.row(0).is_divider_row());

            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(!table.row(0).is_divider_row());
        }

        #[test]
        fn row_zip_with_position_should_map_iter_to_include_cell_position() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_span_cell(0, 1, CellSpan::FromLeft),
                    make_align_cell(1, 0, ColumnAlign::None),
                    make_align_cell(1, 1, ColumnAlign::Right),
                    make_content_cell(2, 0, "b"),
                    make_content_cell(2, 1, "c"),
                ],
                false,
            );

            let mut rows = table.rows();

            let mut row_0 = rows.next().unwrap().zip_with_position();
            assert_eq!(row_0.next().unwrap().0, CellPos { row: 0, col: 0 });
            assert_eq!(row_0.next().unwrap().0, CellPos { row: 0, col: 1 });

            let mut row_1 = rows.next().unwrap().zip_with_position();
            assert_eq!(row_1.next().unwrap().0, CellPos { row: 1, col: 0 });
            assert_eq!(row_1.next().unwrap().0, CellPos { row: 1, col: 1 });

            let mut row_2 = rows.next().unwrap().zip_with_position();
            assert_eq!(row_2.next().unwrap().0, CellPos { row: 2, col: 0 });
            assert_eq!(row_2.next().unwrap().0, CellPos { row: 2, col: 1 });
        }

        #[test]
        fn row_should_iterator_through_appropriate_cells() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(1, 0, "c"),
                    make_content_cell(1, 1, "d"),
                    make_content_cell(2, 0, "e"),
                    make_content_cell(2, 1, "f"),
                ],
                false,
            );

            assert_eq!(
                table
                    .row(1)
                    .filter_map(|x| x.get_content())
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                vec!["c", "d"]
            );
        }

        #[test]
        fn row_next_should_return_next_cell_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut row = table.row(0);
            assert!(row.next().is_some());
        }

        #[test]
        fn row_next_should_return_none_if_no_more_cells_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut row = table.row(0);
            row.next();
            assert!(row.next().is_none());
        }

        #[test]
        fn row_size_hint_should_return_remaining_cells_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut row = table.row(0);
            assert_eq!(row.size_hint(), (1, Some(1)));

            row.next();
            assert_eq!(row.size_hint(), (0, Some(0)));
        }

        #[test]
        fn into_row_has_content_should_return_true_if_any_remaining_cells_have_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.into_row(0).has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.into_row(0).has_content());
        }

        #[test]
        fn into_row_is_divider_row_should_return_true_if_made_up_of_align_cells(
        ) {
            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(table.into_row(0).is_divider_row());

            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(!table.into_row(0).is_divider_row());
        }

        #[test]
        fn into_row_zip_with_position_should_map_iter_to_include_cell_position()
        {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_span_cell(0, 1, CellSpan::FromLeft),
                    make_align_cell(1, 0, ColumnAlign::None),
                    make_align_cell(1, 1, ColumnAlign::Right),
                    make_content_cell(2, 0, "b"),
                    make_content_cell(2, 1, "c"),
                ],
                false,
            );

            let mut row_0 = table.clone().into_row(0).zip_with_position();
            assert_eq!(row_0.next().unwrap().0, CellPos { row: 0, col: 0 });
            assert_eq!(row_0.next().unwrap().0, CellPos { row: 0, col: 1 });

            let mut row_1 = table.clone().into_row(1).zip_with_position();
            assert_eq!(row_1.next().unwrap().0, CellPos { row: 1, col: 0 });
            assert_eq!(row_1.next().unwrap().0, CellPos { row: 1, col: 1 });

            let mut row_2 = table.into_row(2).zip_with_position();
            assert_eq!(row_2.next().unwrap().0, CellPos { row: 2, col: 0 });
            assert_eq!(row_2.next().unwrap().0, CellPos { row: 2, col: 1 });
        }

        #[test]
        fn into_row_should_iterator_through_appropriate_cells() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(1, 0, "c"),
                    make_content_cell(1, 1, "d"),
                    make_content_cell(2, 0, "e"),
                    make_content_cell(2, 1, "f"),
                ],
                false,
            );

            assert_eq!(
                table
                    .into_row(1)
                    .filter_map(|x| match x.into_inner() {
                        Cell::Content(x) => Some(x.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<String>>(),
                vec!["c", "d"]
            );
        }

        #[test]
        fn into_row_next_should_return_next_cell_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut row = table.into_row(0);
            assert!(row.next().is_some());
        }

        #[test]
        fn into_row_next_should_return_none_if_no_more_cells_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut row = table.into_row(0);
            row.next();
            assert!(row.next().is_none());
        }

        #[test]
        fn into_row_size_hint_should_return_remaining_cells_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut row = table.into_row(0);
            assert_eq!(row.size_hint(), (1, Some(1)));

            row.next();
            assert_eq!(row.size_hint(), (0, Some(0)));
        }

        #[test]
        fn columns_has_content_should_return_true_if_any_remaining_column_has_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.columns().has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.columns().has_content());
        }

        #[test]
        fn columns_next_should_return_next_column_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut columns = table.columns();
            assert!(columns.next().is_some());
        }

        #[test]
        fn columns_next_should_return_none_if_no_more_columns_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut columns = table.columns();
            columns.next();
            assert!(columns.next().is_none());
        }

        #[test]
        fn columns_size_hint_should_return_remaining_columns_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut columns = table.columns();
            assert_eq!(columns.size_hint(), (1, Some(1)));

            columns.next();
            assert_eq!(columns.size_hint(), (0, Some(0)));
        }

        #[test]
        fn column_has_content_should_return_true_if_any_remaining_cells_have_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.column(0).has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.column(0).has_content());
        }

        #[test]
        fn column_zip_with_position_should_map_iter_to_include_cell_position() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_align_cell(1, 0, ColumnAlign::Right),
                    make_align_cell(1, 1, ColumnAlign::Left),
                    make_align_cell(1, 2, ColumnAlign::None),
                ],
                false,
            );

            let mut columns = table.columns();

            let mut column_0 = columns.next().unwrap().zip_with_position();
            assert_eq!(column_0.next().unwrap().0, CellPos { row: 0, col: 0 });
            assert_eq!(column_0.next().unwrap().0, CellPos { row: 1, col: 0 });

            let mut column_1 = columns.next().unwrap().zip_with_position();
            assert_eq!(column_1.next().unwrap().0, CellPos { row: 0, col: 1 });
            assert_eq!(column_1.next().unwrap().0, CellPos { row: 1, col: 1 });

            let mut column_2 = columns.next().unwrap().zip_with_position();
            assert_eq!(column_2.next().unwrap().0, CellPos { row: 0, col: 2 });
            assert_eq!(column_2.next().unwrap().0, CellPos { row: 1, col: 2 });
        }

        #[test]
        fn column_should_iterator_through_appropriate_cells() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_content_cell(1, 0, "d"),
                    make_content_cell(1, 1, "e"),
                    make_content_cell(1, 2, "f"),
                ],
                false,
            );

            assert_eq!(
                table
                    .column(1)
                    .filter_map(|x| x.get_content())
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                vec!["b", "e"]
            );
        }

        #[test]
        fn column_next_should_return_next_cell_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut column = table.column(0);
            assert!(column.next().is_some());
        }

        #[test]
        fn column_next_should_return_none_if_no_more_cells_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut column = table.column(0);
            column.next();
            assert!(column.next().is_none());
        }

        #[test]
        fn column_size_hint_should_return_remaining_cells_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut column = table.column(0);
            assert_eq!(column.size_hint(), (1, Some(1)));

            column.next();
            assert_eq!(column.size_hint(), (0, Some(0)));
        }

        #[test]
        fn into_column_has_content_should_return_true_if_any_remaining_cells_have_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.into_column(0).has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.into_column(0).has_content());
        }

        #[test]
        fn into_column_zip_with_position_should_map_iter_to_include_cell_position(
        ) {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_align_cell(1, 0, ColumnAlign::Left),
                    make_align_cell(1, 1, ColumnAlign::None),
                    make_align_cell(1, 2, ColumnAlign::Right),
                ],
                false,
            );

            let mut column_0 = table.clone().into_column(0).zip_with_position();
            assert_eq!(column_0.next().unwrap().0, CellPos { row: 0, col: 0 });
            assert_eq!(column_0.next().unwrap().0, CellPos { row: 1, col: 0 });

            let mut column_1 = table.clone().into_column(1).zip_with_position();
            assert_eq!(column_1.next().unwrap().0, CellPos { row: 0, col: 1 });
            assert_eq!(column_1.next().unwrap().0, CellPos { row: 1, col: 1 });

            let mut column_2 = table.into_column(2).zip_with_position();
            assert_eq!(column_2.next().unwrap().0, CellPos { row: 0, col: 2 });
            assert_eq!(column_2.next().unwrap().0, CellPos { row: 1, col: 2 });
        }

        #[test]
        fn into_column_should_iterator_through_appropriate_cells() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_content_cell(1, 0, "d"),
                    make_content_cell(1, 1, "e"),
                    make_content_cell(1, 2, "f"),
                ],
                false,
            );

            assert_eq!(
                table
                    .into_column(1)
                    .filter_map(|x| match x.into_inner() {
                        Cell::Content(x) => Some(x.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<String>>(),
                vec!["b", "e"]
            );
        }

        #[test]
        fn into_column_next_should_return_next_cell_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut column = table.into_column(0);
            assert!(column.next().is_some());
        }

        #[test]
        fn into_column_next_should_return_none_if_no_more_cells_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut column = table.into_column(0);
            column.next();
            assert!(column.next().is_none());
        }

        #[test]
        fn into_column_size_hint_should_return_remaining_cells_as_both_bounds()
        {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut column = table.into_column(0);
            assert_eq!(column.size_hint(), (1, Some(1)));

            column.next();
            assert_eq!(column.size_hint(), (0, Some(0)));
        }

        #[test]
        fn cells_has_content_should_return_true_if_any_remaining_cells_have_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.cells().has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.cells().has_content());
        }

        #[test]
        fn cells_zip_with_position_should_map_iter_to_include_cell_position() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_align_cell(1, 0, ColumnAlign::Right),
                    make_align_cell(1, 1, ColumnAlign::Left),
                    make_align_cell(1, 2, ColumnAlign::None),
                ],
                false,
            );

            let mut cells = table.cells().zip_with_position();
            assert_eq!(cells.next().unwrap().0, CellPos { row: 0, col: 0 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 0, col: 1 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 0, col: 2 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 1, col: 0 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 1, col: 1 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 1, col: 2 });
        }

        #[test]
        fn cells_should_iterator_through_appropriate_cells() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_content_cell(1, 0, "d"),
                    make_content_cell(1, 1, "e"),
                    make_content_cell(1, 2, "f"),
                ],
                false,
            );

            assert_eq!(
                table
                    .cells()
                    .filter_map(|x| x.get_content())
                    .map(ToString::to_string)
                    .collect::<Vec<String>>(),
                vec!["a", "b", "c", "d", "e", "f"]
            );
        }

        #[test]
        fn cells_next_should_return_next_cell_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut cells = table.cells();
            assert!(cells.next().is_some());
        }

        #[test]
        fn cells_next_should_return_none_if_no_more_cells_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut cells = table.cells();
            cells.next();
            assert!(cells.next().is_none());
        }

        #[test]
        fn cells_size_hint_should_return_remaining_cells_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut cells = table.cells();
            assert_eq!(cells.size_hint(), (1, Some(1)));

            cells.next();
            assert_eq!(cells.size_hint(), (0, Some(0)));
        }

        #[test]
        fn into_cells_has_content_should_return_true_if_any_remaining_cells_have_content(
        ) {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);
            assert!(table.into_cells().has_content());

            let table = Table::new(
                vec![make_align_cell(0, 0, ColumnAlign::default())],
                false,
            );
            assert!(!table.into_cells().has_content());
        }

        #[test]
        fn into_cells_zip_with_position_should_map_iter_to_include_cell_position(
        ) {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_align_cell(1, 0, ColumnAlign::Left),
                    make_align_cell(1, 1, ColumnAlign::None),
                    make_align_cell(1, 2, ColumnAlign::Right),
                ],
                false,
            );

            let mut cells = table.into_cells().zip_with_position();
            assert_eq!(cells.next().unwrap().0, CellPos { row: 0, col: 0 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 0, col: 1 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 0, col: 2 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 1, col: 0 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 1, col: 1 });
            assert_eq!(cells.next().unwrap().0, CellPos { row: 1, col: 2 });
        }

        #[test]
        fn into_cells_should_iterator_through_all_cells() {
            let table = Table::new(
                vec![
                    make_content_cell(0, 0, "a"),
                    make_content_cell(0, 1, "b"),
                    make_content_cell(0, 2, "c"),
                    make_content_cell(1, 0, "d"),
                    make_content_cell(1, 1, "e"),
                    make_content_cell(1, 2, "f"),
                ],
                false,
            );

            assert_eq!(
                table
                    .into_cells()
                    .filter_map(|x| match x.into_inner() {
                        Cell::Content(x) => Some(x.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<String>>(),
                vec!["a", "b", "c", "d", "e", "f"]
            );
        }

        #[test]
        fn into_cells_next_should_return_next_cell_if_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut cells = table.into_cells();
            assert!(cells.next().is_some());
        }

        #[test]
        fn into_cells_next_should_return_none_if_no_more_cells_available() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut cells = table.into_cells();
            cells.next();
            assert!(cells.next().is_none());
        }

        #[test]
        fn into_cells_size_hint_should_return_remaining_cells_as_both_bounds() {
            let table = Table::new(vec![make_content_cell(0, 0, "")], false);

            let mut cells = table.into_cells();
            assert_eq!(cells.size_hint(), (1, Some(1)));

            cells.next();
            assert_eq!(cells.size_hint(), (0, Some(0)));
        }
    }
}
