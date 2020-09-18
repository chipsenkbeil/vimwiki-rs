use super::{Position, Span};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// Represents a region in a string or file, comprised of a start and end
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
pub struct Region {
    pub start: Position,
    pub end: Position,
}

impl Region {
    /// Checks if a position is contained within this region
    pub fn contains(&self, pos: Position) -> bool {
        pos >= self.start && pos <= self.end
    }
}

impl From<(Position, Position)> for Region {
    fn from(coords: (Position, Position)) -> Self {
        Self {
            start: coords.0,
            end: coords.1,
        }
    }
}

impl From<((u32, usize), (u32, usize))> for Region {
    fn from(coords: ((u32, usize), (u32, usize))) -> Self {
        Self::from((Position::from(coords.0), Position::from(coords.1)))
    }
}

impl From<(u32, usize, u32, usize)> for Region {
    fn from(coords: (u32, usize, u32, usize)) -> Self {
        Self::from(((coords.0, coords.1), (coords.2, coords.3)))
    }
}

impl<'a> From<(Span, Span)> for Region {
    /// Converts a start and end span into a region that they represent,
    /// assuming that the end span is non-inclusive (one step past end)
    fn from((start, end): (Span, Span)) -> Self {
        use nom::Offset;
        let mut offset = start.offset(&end);

        // Assume that if the spans are not equal, the end span is one past
        // the actual end of the region
        if offset > 0 {
            offset -= 1;
        }

        Self::from((start, offset))
    }
}

impl<'a> From<(Span, usize)> for Region {
    fn from((span, offset): (Span, usize)) -> Self {
        use nom::Slice;
        let start = Position::from(span.clone());
        let end = Position::from(span.slice(offset..));

        Self::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::utils::Span;

    #[test]
    fn region_contains_should_yield_true_if_between_start_and_end() {
        let region = Region::from((1, 1, 2, 2));
        assert!(!region.contains(Position::new(0, 0)));
        assert!(!region.contains(Position::new(0, 1)));
        assert!(!region.contains(Position::new(0, 2)));
        assert!(!region.contains(Position::new(0, 999)));
        assert!(!region.contains(Position::new(1, 0)));
        assert!(region.contains(Position::new(1, 1)));
        assert!(region.contains(Position::new(1, 2)));
        assert!(region.contains(Position::new(1, 999)));
        assert!(region.contains(Position::new(2, 0)));
        assert!(region.contains(Position::new(2, 1)));
        assert!(region.contains(Position::new(2, 2)));
        assert!(!region.contains(Position::new(2, 3)));
    }

    #[test]
    fn region_from_span_tuple_should_use_start_location_for_end_if_spans_equal()
    {
        let input = Span::from("abc\n12345");

        fn take1(input: Span) -> nom::IResult<Span, Span> {
            nom::bytes::complete::take(1usize)(input)
        }

        // Start at line 0, column 1
        let (start, _) = take1(input).unwrap();

        // Start span should be at (1, 2), which is (0, 1) in our coord space
        assert_eq!(start.local_line(), 1);
        assert_eq!(start.local_utf8_column(), 2);

        let region = Region::from((start.clone(), start));
        assert_eq!(region, Region::from((0, 1, 0, 1)));
    }

    #[test]
    fn region_from_span_tuple_should_assume_second_span_is_right_after_region_ends(
    ) {
        let input = Span::from("abc\n12345");

        fn take1(input: Span) -> nom::IResult<Span, Span> {
            nom::bytes::complete::take(1usize)(input)
        }

        // Start at line 0, column 1
        let (start, _) = take1(input).unwrap();

        // Start span should be at (1, 2), which is (0, 1) in our coord space
        assert_eq!(start.local_line(), 1);
        assert_eq!(start.local_utf8_column(), 2);

        // End at line 1, column 3
        let (end, _) = take1(start.clone()).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();

        // Span should now be at (2, 4), which is (1, 3) in our coord space
        assert_eq!(end.local_line(), 2);
        assert_eq!(end.local_utf8_column(), 4);

        // region should start at (0, 1) and end at (1, 3-1)
        let region = Region::from((start, end));
        assert_eq!(region, Region::from((0, 1, 1, 2)));
    }
}
