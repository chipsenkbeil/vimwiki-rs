use super::Span;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Represents a position in a string or file
#[derive(
    Constructor,
    Copy,
    Clone,
    Debug,
    Default,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub struct Position {
    pub line: u32,
    pub column: usize,
}

impl From<(u32, usize)> for Position {
    fn from(coords: (u32, usize)) -> Self {
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

impl From<Span> for Position {
    /// Constructs a position based on the start of a span
    fn from(span: Span) -> Self {
        // NOTE: Span from nom_locate has base 1 for line/col
        // TODO: Compare performance of naive_get_utf8_column, which is
        //       supposedly better for shorter lines (100 or less), which
        //       I imagine is more common for vimwiki
        let (line, column) = (span.global_line(), span.global_utf8_column());

        Self::new(line - 1, column - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;

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

    #[test]
    fn position_from_span_should_offset_line_and_column_by_1() {
        let input = Span::from("abc\n123");
        let p = Position::from(input.clone());
        assert_eq!(p.line, 0);
        assert_eq!(p.column, 0);

        fn take5(input: Span) -> nom::IResult<Span, Span> {
            nom::bytes::complete::take(5usize)(input)
        }

        let (input, _) = take5(input).unwrap();
        let p = Position::from(input);
        assert_eq!(p.line, 1);
        assert_eq!(p.column, 1);
    }
}
