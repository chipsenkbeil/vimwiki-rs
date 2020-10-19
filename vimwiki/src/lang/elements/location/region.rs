use crate::lang::parsers::Span;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::ops::{Range, RangeInclusive, RangeTo, RangeToInclusive};

/// Represents a region in a string or file, comprised of a start and end
#[derive(
    Copy, Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Region {
    /// Position within some byte array this region begins
    offset: usize,

    /// Length of this region from the offset
    len: usize,

    /// Optional extra information about the region in the form of line/column
    position: Option<Position>,
}

/// Represents the position of a region in the form of line/column
#[derive(
    Constructor, Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct Position {
    start: LineColumn,
    end: LineColumn,
}

/// Represents some position in the form of line/column
#[derive(
    Constructor, Copy, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct LineColumn {
    line: usize,
    column: usize,
}

impl Region {
    /// Constructs a new region with the given offset and length, containing
    /// no extra information
    pub fn new(offset: usize, len: usize) -> Self {
        Self {
            offset,
            len,
            position: None,
        }
    }

    /// Constructs a new region with the given offset, length, and extra
    /// information about the line & column position
    pub fn new_with_position(
        offset: usize,
        len: usize,
        position: Position,
    ) -> Self {
        Self {
            offset,
            len,
            position: Some(position),
        }
    }

    /// Consumes the given region and returns a new one with its position
    /// set to the provided position
    pub fn with_position(self, position: Position) -> Self {
        Self::new_with_position(self.offset, self.len, position)
    }

    /// Constructs a new region based on the offset and length of the given
    /// span. Additionally, computes the line & column position of the region.
    pub fn from_span_with_position(span: Span) -> Self {
        let start = LineColumn::new(span.line(), span.column());
        let end = LineColumn::new(span.end_line(), span.end_column());
        let position = Position::new(start, end);
        Self::new_with_position(
            span.start_offset(),
            span.remaining_len(),
            position,
        )
    }

    /// Checks if a position is contained within this region
    #[inline]
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.offset && offset < (self.offset + self.len)
    }

    /// The offset of the region relative to some span of input
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The length of the region
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the length of the region is zero
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the position of the region relative to some span of input
    ///
    /// Can be none if the region was not constructed with a position
    pub fn position(&self) -> Option<Position> {
        self.position
    }
}

impl<'a> From<Span<'a>> for Region {
    /// Converts a `Span` to a region, but does not calculate the line &
    /// column information
    fn from(span: Span<'a>) -> Self {
        Self::new(span.start_offset(), span.remaining_len())
    }
}

impl From<Range<usize>> for Region {
    /// Converts from `start..end` to `Region { offset: start, len: end - start }`
    /// where `end < start` will result in a length of zero
    fn from(range: Range<usize>) -> Self {
        let len = if range.end >= range.start {
            range.end - range.start
        } else {
            0
        };
        Self::new(range.start, len)
    }
}

impl From<RangeInclusive<usize>> for Region {
    /// Converts from `start..=end` to `Region { offset: start, len: end - start + 1 }`
    /// where `end + 1 < start` will result in a length of zero
    fn from(range: RangeInclusive<usize>) -> Self {
        let (start, end) = range.into_inner();
        let len = if (end + 1) >= start {
            (end + 1) - start
        } else {
            0
        };
        Self::new(start, len)
    }
}

impl From<RangeTo<usize>> for Region {
    /// Converts from `..end` to `Region { offset: 0, len: end }`
    fn from(range: RangeTo<usize>) -> Self {
        Self::new(0, range.end)
    }
}

impl From<RangeToInclusive<usize>> for Region {
    /// Converts from `..=end` to `Region { offset: 0, len: end + 1 }`
    fn from(range: RangeToInclusive<usize>) -> Self {
        Self::new(0, range.end + 1)
    }
}

impl From<(usize, usize)> for Region {
    fn from(coords: (usize, usize)) -> Self {
        Self::new(coords.0, coords.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_should_successfully_return_whether_or_not_offset_within_region()
    {
        let region = Region::new(3, 2);
        assert!(!region.contains(0));
        assert!(!region.contains(1));
        assert!(!region.contains(2));
        assert!(region.contains(3));
        assert!(region.contains(4));
        assert!(!region.contains(5));
        assert!(!region.contains(6));
        assert!(!region.contains(7));
    }

    #[test]
    fn from_should_properly_convert_range_to_region() {
        let region = Region::from(0..3);
        assert_eq!(region, Region::new(0, 3));

        let region = Region::from(3..3);
        assert_eq!(region, Region::new(3, 0));

        let region = Region::from(4..3);
        assert_eq!(region, Region::new(4, 0));
    }

    #[test]
    fn from_should_properly_convert_range_inclusive_to_region() {
        let region = Region::from(0..=3);
        assert_eq!(region, Region::new(0, 4));

        let region = Region::from(3..=3);
        assert_eq!(region, Region::new(3, 1));

        let region = Region::from(4..=3);
        assert_eq!(region, Region::new(4, 0));
    }

    #[test]
    fn from_should_properly_convert_rangeto_to_region() {
        let region = Region::from(..3);
        assert_eq!(region, Region::new(0, 3));

        let region = Region::from(..0);
        assert_eq!(region, Region::new(0, 0));

        let region = Region::from(..1);
        assert_eq!(region, Region::new(0, 1));
    }

    #[test]
    fn from_should_properly_convert_rangeto_inclusive_to_region() {
        let region = Region::from(..=3);
        assert_eq!(region, Region::new(0, 4));

        let region = Region::from(..=0);
        assert_eq!(region, Region::new(0, 1));

        let region = Region::from(..=1);
        assert_eq!(region, Region::new(0, 2));
    }
}
