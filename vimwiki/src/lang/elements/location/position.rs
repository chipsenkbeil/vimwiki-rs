use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Represents a position in a string or file
#[derive(
    Constructor, Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

impl From<(usize, usize)> for Position {
    fn from(coords: (usize, usize)) -> Self {
        Self {
            line: coords.0,
            column: coords.1,
        }
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        // First, check if other comes before or after in line position,
        // then - if on same line - compare the column position
        match self.line.cmp(&other.line) {
            Ordering::Equal => self.column.cmp(&other.column),
            x => x,
        }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Position {
    /// Whether or not this position is at the beginning of a line
    pub fn is_at_beginning_of_line(&self) -> bool {
        self.column == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::parsers::Span;

    #[test]
    fn position_ordering_should_have_position_with_earliest_line_first() {
        let p1 = Position::new(0, 5);
        let p2 = Position::new(1, 0);
        assert!(p1 < p2)
    }

    #[test]
    fn position_ordering_should_have_position_with_earliest_column_first_if_lines_are_equal(
    ) {
        let p1 = Position::new(1, 1);
        let p2 = Position::new(1, 2);
        assert!(p1 < p2)
    }

    #[test]
    fn position_is_at_beginning_of_line_should_return_true_if_column_is_0() {
        assert!(Position::new(1, 0).is_at_beginning_of_line());
        assert!(!Position::new(1, 1).is_at_beginning_of_line());
    }
}
