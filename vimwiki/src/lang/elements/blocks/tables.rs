use crate::{
    lang::elements::{
        InlineElement, InlineElementContainer, IntoChildren, Located,
    },
    StrictEq,
};
use derive_more::{Constructor, From, IntoIterator};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the position of a cell in a table
#[derive(
    Constructor, Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize,
)]
pub struct CellPos {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, IntoIterator, Serialize, Deserialize)]
pub struct Table<'a> {
    #[into_iterator(owned, ref, ref_mut)]
    cells: HashMap<CellPos, Located<Cell<'a>>>,
    row_cnt: usize,
    col_cnt: usize,
    centered: bool,
}

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
    pub fn new(
        cells: HashMap<CellPos, Located<Cell<'a>>>,
        centered: bool,
    ) -> Self {
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
    pub fn header_rows(&self) -> iter::HeaderRows<'_, 'a> {
        iter::HeaderRows::new(self)
    }

    /// Returns an iterator over all rows that are considered body rows,
    /// which is all rows following a divider row. If there is no divider
    /// row in the table, then all rows are considered body rows
    pub fn body_rows(&self) -> iter::BodyRows<'_, 'a> {
        iter::BodyRows::new(self)
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

    /// Returns true if table is centered
    #[inline]
    pub fn is_centered(&self) -> bool {
        self.centered
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
    pub fn rows(&self) -> iter::Rows<'_, 'a> {
        iter::Rows::new(self)
    }

    /// Returns an iterator of refs through a specific row in the table
    pub fn row(&self, idx: usize) -> iter::Row<'_, 'a> {
        iter::Row::new(self, idx, 0)
    }

    /// Consumes the table and returns an iterator through a specific row in the table
    pub fn into_row(self, idx: usize) -> iter::IntoRow<'a> {
        iter::IntoRow::new(self, idx, 0)
    }

    /// Returns an iterator of refs through all columns in the table
    pub fn columns(&self) -> iter::Columns<'_, 'a> {
        iter::Columns::new(self)
    }

    /// Returns an iterator of refs through a specific column in the table
    pub fn column(&self, idx: usize) -> iter::Column<'_, 'a> {
        iter::Column::new(self, 0, idx)
    }

    /// Consumes the table and returns an iterator through a specific column in the table
    pub fn into_column(self, idx: usize) -> iter::IntoColumn<'a> {
        iter::IntoColumn::new(self, 0, idx)
    }

    /// Returns an iterator of refs through all cells in the table, starting
    /// from the first row, iterating through all cells from beginning to end,
    /// and then moving on to the next row
    pub fn cells(&self) -> iter::Cells<'_, 'a> {
        iter::Cells::new(self)
    }

    /// Consumes the table and returns an iterator through all cells in the
    /// table, starting from the first row, iterating through all cells from
    /// beginning to end, and then moving on to the next row
    pub fn into_cells(self) -> iter::IntoCells<'a> {
        iter::IntoCells::new(self)
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
#[derive(Clone, Debug, From, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Cell<'a> {
    Content(InlineElementContainer<'a>),
    Span(CellSpan),
    Align(ColumnAlign),
}

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
    /// Returns true if cell represents a content cell
    #[inline]
    pub fn is_content(&self) -> bool {
        matches!(self, Cell::Content(_))
    }

    /// Returns true if cell represents a span cell
    #[inline]
    pub fn is_span(&self) -> bool {
        matches!(self, Cell::Span(_))
    }

    /// Returns true if cell represents a column alignment cell
    #[inline]
    pub fn is_align(&self) -> bool {
        matches!(self, Cell::Align(_))
    }

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

pub mod iter {
    use super::{Cell, CellPos, Located, Table};
    use derive_more::Constructor;

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
                idx: table.get_divider_row_index().unwrap_or_default(),
            }
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

    #[derive(Constructor)]
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

        pub fn zip_with_position(
            self,
        ) -> impl Iterator<Item = (CellPos, &'a Located<Cell<'b>>)> {
            let pos = CellPos::new(self.row, self.col);
            self.map(move |cell| (pos, cell))
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

    #[derive(Constructor)]
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

        pub fn zip_with_position(
            self,
        ) -> impl Iterator<Item = (CellPos, Located<Cell<'a>>)> {
            let pos = CellPos::new(self.row, self.col);
            self.map(move |cell| (pos, cell))
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
    }

    impl<'a, 'b> Iterator for Columns<'a, 'b> {
        type Item = Column<'a, 'b>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.idx < self.table.col_cnt() {
                let col = Column::new(self.table, self.idx, 0);
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

    #[derive(Constructor)]
    pub struct Column<'a, 'b> {
        table: &'a Table<'b>,
        row: usize,
        col: usize,
    }

    impl<'a, 'b> Column<'a, 'b> {
        pub fn zip_with_position(
            self,
        ) -> impl Iterator<Item = (CellPos, &'a Located<Cell<'b>>)> {
            let pos = CellPos::new(self.row, self.col);
            self.map(move |cell| (pos, cell))
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

    #[derive(Constructor)]
    pub struct IntoColumn<'a> {
        table: Table<'a>,
        row: usize,
        col: usize,
    }

    impl<'a> IntoColumn<'a> {
        pub fn zip_with_position(
            self,
        ) -> impl Iterator<Item = (CellPos, Located<Cell<'a>>)> {
            let pos = CellPos::new(self.row, self.col);
            self.map(move |cell| (pos, cell))
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

        pub fn zip_with_position(
            self,
        ) -> impl Iterator<Item = (CellPos, &'a Located<Cell<'b>>)> {
            let pos = CellPos::new(self.row, self.col);
            self.map(move |cell| (pos, cell))
        }
    }

    impl<'a, 'b> Iterator for Cells<'a, 'b> {
        type Item = &'a Located<Cell<'b>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell = self.table.get_cell(self.row, self.col);

            // If not yet reached end of row, advance column ptr
            if self.col < self.table.col_cnt() {
                self.col += 1;

            // Else if not yet reached end of all rows, advance row ptr and
            // reset column ptr
            } else if self.row < self.table.row_cnt() {
                self.row += 1;
                self.col = 0;
            }

            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining =
                self.table.len() - (self.row * self.table.col_cnt()) - self.col;
            (remaining, Some(remaining))
        }
    }

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

        pub fn zip_with_position(
            self,
        ) -> impl Iterator<Item = (CellPos, Located<Cell<'a>>)> {
            let pos = CellPos::new(self.row, self.col);
            self.map(move |cell| (pos, cell))
        }
    }

    impl<'a> Iterator for IntoCells<'a> {
        type Item = Located<Cell<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            let cell =
                self.table.cells.remove(&CellPos::new(self.row, self.col));

            // If not yet reached end of row, advance column ptr
            if self.col < self.table.col_cnt() {
                self.col += 1;

            // Else if not yet reached end of all rows, advance row ptr and
            // reset column ptr
            } else if self.row < self.table.row_cnt() {
                self.row += 1;
                self.col = 0;
            }

            cell
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let remaining =
                self.table.len() - (self.row * self.table.col_cnt()) - self.col;
            (remaining, Some(remaining))
        }
    }
}
