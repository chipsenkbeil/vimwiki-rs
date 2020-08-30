use derive_more::Constructor;
use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Represents input for the parser
pub type Span<'a> = LocatedSpan<&'a str>;

/// Represents an encapsulation of a language component and its location
/// within some string/file
#[derive(
    Constructor, Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize,
)]
pub struct LocatedComponent<T> {
    pub component: T,
    pub region: Region,
}

/// Shorthand alias for LocatedComponent
pub type LC<T> = LocatedComponent<T>;

impl<T> LocatedComponent<T> {
    /// Maps a `LocatedComponent<T>` to `LocatedComponent<U>` by applying a
    /// function to the underlying component. Useful when upleveling the
    /// component (such as wrapping a Header1) while the region remains
    /// unchanged.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> LocatedComponent<U> {
        LocatedComponent::new(f(self.component), self.region)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `LocatedComponent<T>` (with default region).
    #[inline]
    pub fn wrap<U>(f: impl Fn(U) -> T) -> impl Fn(U) -> Self {
        Self::wrap_with_region(Default::default(), f)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `LocatedComponent<T>`.
    #[inline]
    pub fn wrap_with_region<U>(
        region: Region,
        f: impl Fn(U) -> T,
    ) -> impl Fn(U) -> Self {
        move |input| {
            let component = f(input);
            Self::new(component, region)
        }
    }
}

impl<T> From<T> for LocatedComponent<T> {
    /// Creates a new located component around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, Default::default())
    }
}

impl<'a, T> From<(T, Span<'a>)> for LocatedComponent<T> {
    /// Creates a new located component around `T`, using a default location
    fn from(x: (T, Span<'a>)) -> Self {
        Self::new(x.0, From::from(x.1))
    }
}

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
    /// Constructs a position based on the start of a span
    pub fn start_of_span(span: Span) -> Self {
        // NOTE: Span from nom_locate has base 1 for line/col
        // TODO: Compare performance of naive_get_utf8_column, which is
        //       supposedly better for shorter lines (100 or less), which
        //       I imagine is more common for vimwiki
        Self::new(span.location_line() - 1, span.get_utf8_column() - 1)
    }

    /// Constructs a position based on the end of a span
    pub fn end_of_span(span: Span) -> Self {
        let mut end = Self::start_of_span(span);
        let (total_lines, last_line) = span
            .fragment()
            .lines()
            .fold((0, ""), |acc, x| (acc.0 + 1, x));

        // Adjust end position to be that of the final line, at final column
        // NOTE: Checking to ensure that we have some result as an empty
        //       span will yield zero lines
        if total_lines > 0 && !last_line.is_empty() {
            end.line += total_lines - 1;
            end.column = last_line.len() - 1;
        }

        end
    }

    /// Whether or not this position is at the beginning of a line
    pub fn is_at_beginning_of_line(&self) -> bool {
        self.column == 0
    }
}

impl<'a> From<Span<'a>> for Position {
    fn from(span: Span<'a>) -> Self {
        Self::start_of_span(span)
    }
}

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

impl<'a> From<Span<'a>> for Region {
    fn from(span: Span<'a>) -> Self {
        let start = Position::start_of_span(span);
        let end = Position::end_of_span(span);

        Self::new(start, end)
    }
}