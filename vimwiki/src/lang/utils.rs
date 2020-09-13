use derive_more::{AsMut, AsRef, Constructor, Deref, DerefMut};
use nom_locate::LocatedSpan;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// Represents input for the parser
pub type Span<'a> = LocatedSpan<&'a str>;

/// Represents an encapsulation of a language component and its location
/// within some string/file
#[derive(
    AsRef,
    AsMut,
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    Eq,
    Serialize,
    Deserialize,
)]
pub struct LocatedComponent<T> {
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
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

    /// Converts LocatedComponent to a strict variant
    pub fn into_strict(self) -> StrictLocatedComponent<T> {
        self.into()
    }
}

impl<T: PartialEq> PartialEq for LocatedComponent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component
    }
}

impl<T: PartialEq> PartialEq<T> for LocatedComponent<T> {
    fn eq(&self, other: &T) -> bool {
        &self.component == other
    }
}

impl<T: Hash> Hash for LocatedComponent<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.component.hash(state);
    }
}

impl<T> From<T> for LocatedComponent<T> {
    /// Creates a new located component around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, Default::default())
    }
}

impl<'a, T> From<(T, Span<'a>, Span<'a>)> for LocatedComponent<T> {
    /// Creates a new located component around `T`, using a default location
    fn from((component, start, end): (T, Span<'a>, Span<'a>)) -> Self {
        Self::new(component, Region::from((start, end)))
    }
}

/// Represents a located component that has strict equality enforcement
/// (component + region versus just component)
#[derive(
    AsRef,
    AsMut,
    Constructor,
    Clone,
    Debug,
    Deref,
    DerefMut,
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
)]
pub struct StrictLocatedComponent<T> {
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    pub component: T,
    pub region: Region,
}

impl<T> StrictLocatedComponent<T> {
    /// Maps a `StrictLocatedComponent<T>` to `StrictLocatedComponent<U>` by
    /// applying a function to the underlying component. Useful when upleveling
    /// the component (such as wrapping a Header) while the region remains
    /// unchanged.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> StrictLocatedComponent<U> {
        StrictLocatedComponent::new(f(self.component), self.region)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `StrictLocatedComponent<T>` (with default region).
    #[inline]
    pub fn wrap<U>(f: impl Fn(U) -> T) -> impl Fn(U) -> Self {
        Self::wrap_with_region(Default::default(), f)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `StrictLocatedComponent<T>`.
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

    /// Converts StrictLocatedComponent to a loose variant
    pub fn into_loose(self) -> LocatedComponent<T> {
        self.into()
    }
}

/// Shorthand alias for StrictLocatedComponent
pub type SLC<T> = StrictLocatedComponent<T>;

impl<T> From<LocatedComponent<T>> for StrictLocatedComponent<T> {
    fn from(lc: LocatedComponent<T>) -> Self {
        Self::new(lc.component, lc.region)
    }
}

impl<T> From<StrictLocatedComponent<T>> for LocatedComponent<T> {
    fn from(slc: StrictLocatedComponent<T>) -> Self {
        Self::new(slc.component, slc.region)
    }
}

impl<T: PartialEq> PartialEq<LocatedComponent<T>>
    for StrictLocatedComponent<T>
{
    fn eq(&self, other: &LocatedComponent<T>) -> bool {
        self.component == other.component && self.region == other.region
    }
}

impl<T: PartialEq> PartialEq<StrictLocatedComponent<T>>
    for LocatedComponent<T>
{
    fn eq(&self, other: &StrictLocatedComponent<T>) -> bool {
        self.component == other.component && self.region == other.region
    }
}

impl<T> From<T> for StrictLocatedComponent<T> {
    /// Creates a new strict located component around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, Default::default())
    }
}

impl<'a, T> From<(T, Span<'a>, Span<'a>)> for StrictLocatedComponent<T> {
    /// Creates a new strict located component around `T`, using a default location
    fn from((component, start, end): (T, Span<'a>, Span<'a>)) -> Self {
        Self::new(component, Region::from((start, end)))
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
    /// Whether or not this position is at the beginning of a line
    pub fn is_at_beginning_of_line(&self) -> bool {
        self.column == 0
    }
}

impl<'a> From<Span<'a>> for Position {
    /// Constructs a position based on the start of a span
    fn from(span: Span<'a>) -> Self {
        // NOTE: Span from nom_locate has base 1 for line/col
        // TODO: Compare performance of naive_get_utf8_column, which is
        //       supposedly better for shorter lines (100 or less), which
        //       I imagine is more common for vimwiki
        Self::new(span.location_line() - 1, span.get_utf8_column() - 1)
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

impl<'a> From<(Span<'a>, Span<'a>)> for Region {
    /// Converts a start and end span into a region that they represent,
    /// assuming that the end span is non-inclusive (one step past end)
    fn from((start, end): (Span<'a>, Span<'a>)) -> Self {
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

impl<'a> From<(Span<'a>, usize)> for Region {
    fn from((span, offset): (Span<'a>, usize)) -> Self {
        use nom::Slice;
        let start = Position::from(span);
        let end = Position::from(span.slice(offset..));

        Self::new(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn located_component_map_should_transform_inner_component_and_keep_region()
    {
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        let mapped_lc = lc.map(|c| c + 1);
        assert_eq!(mapped_lc.component, 4);
        assert_eq!(mapped_lc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_component_wrap_should_apply_function_and_wrap_in_default_region()
    {
        let lc = LC::wrap(|x: usize| x.to_string())(3);
        assert_eq!(lc.component, String::from("3"));
        assert_eq!(lc.region, Region::default());
    }

    #[test]
    fn located_component_wrap_with_region_should_apply_function_and_wrap_in_provided_region(
    ) {
        let lc =
            LC::wrap_with_region(Region::from(((1, 2), (3, 4))), |x: usize| {
                x.to_string()
            })(3);
        assert_eq!(lc.component, String::from("3"));
        assert_eq!(lc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_component_equality_with_other_located_component_should_only_use_inner_component(
    ) {
        let lc1 = LC::new(3, Region::from(((1, 2), (3, 4))));
        let lc2 = LC::new(3, Region::default());
        assert_eq!(lc1, lc2);
    }

    #[test]
    fn located_component_equality_with_inner_type_should_only_use_inner_component(
    ) {
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        let inner = 3;
        assert_eq!(lc, inner);
        assert!(lc != inner + 1);
    }

    #[test]
    fn located_component_hashing_should_only_use_inner_component() {
        let lc1 = LC::new(3, Region::from(((1, 2), (3, 4))));
        let lc2 = LC::new(3, Region::default());
        let lc3 = LC::new(4, Region::from(((1, 2), (3, 4))));
        let lc4 = LC::new(3, Region::from(((1, 2), (3, 4))));

        let mut m = HashSet::new();
        m.insert(lc1);

        let lc = m.get(&lc2).expect("Failed to retrieve LC with another LC");
        assert_eq!(lc.component, 3);
        assert_eq!(lc.region, Region::from(((1, 2), (3, 4))));

        assert_eq!(m.get(&lc3), None);

        let lc = m.get(&lc4).expect("Failed to retrieve LC with another LC");
        assert_eq!(lc.component, 3);
        assert_eq!(lc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_component_equality_with_strict_located_component_should_use_inner_component_and_region(
    ) {
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        let slc = SLC::new(3, Region::default());
        assert!(lc != slc, "{:?} unexpectedly equaled {:?}", lc, slc);
    }

    #[test]
    fn strict_located_component_map_should_transform_inner_component_and_keep_region(
    ) {
        let slc = SLC::new(3, Region::from(((1, 2), (3, 4))));
        let mapped_slc = slc.map(|c| c + 1);
        assert_eq!(mapped_slc.component, 4);
        assert_eq!(mapped_slc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn strict_located_component_wrap_should_apply_function_and_wrap_in_default_region(
    ) {
        let slc = SLC::wrap(|x: usize| x.to_string())(3);
        assert_eq!(slc.component, String::from("3"));
        assert_eq!(slc.region, Region::default());
    }

    #[test]
    fn strict_located_component_wrap_with_region_should_apply_function_and_wrap_in_provided_region(
    ) {
        let slc = SLC::wrap_with_region(
            Region::from(((1, 2), (3, 4))),
            |x: usize| x.to_string(),
        )(3);
        assert_eq!(slc.component, String::from("3"));
        assert_eq!(slc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn strict_located_component_equality_with_other_located_component_should_use_inner_component_and_region(
    ) {
        let slc1 = SLC::new(3, Region::from(((1, 2), (3, 4))));
        let slc2 = SLC::new(3, Region::from(((1, 2), (3, 4))));
        assert_eq!(slc1, slc2);

        let slc1 = SLC::new(3, Region::from(((1, 2), (3, 4))));
        let slc2 = SLC::new(3, Region::default());
        assert!(slc1 != slc2, "{:?} unexpectedly equaled {:?}", slc1, slc2);
    }

    #[test]
    fn strict_located_component_hashing_should_use_inner_component_and_region()
    {
        let slc1 = SLC::new(3, Region::from(((1, 2), (3, 4))));
        let slc2 = SLC::new(3, Region::default());
        let slc3 = SLC::new(4, Region::from(((1, 2), (3, 4))));
        let slc4 = SLC::new(3, Region::from(((1, 2), (3, 4))));

        let mut m = HashSet::new();
        m.insert(slc1);

        assert_eq!(m.get(&slc2), None);
        assert_eq!(m.get(&slc3), None);

        let slc = m.get(&slc4).expect("Failed to get SLC with exact match");
        assert_eq!(slc.component, 3);
        assert_eq!(slc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn strict_located_component_equality_with_located_component_should_use_inner_component_and_region(
    ) {
        let slc = SLC::new(3, Region::default());
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        assert!(slc != lc, "{:?} unexpectedly equaled {:?}", slc, lc);
    }

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
        let input = Span::new("abc\n123");
        let p = Position::from(input);
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
        let input = Span::new("abc\n12345");

        fn take1(input: Span) -> nom::IResult<Span, Span> {
            nom::bytes::complete::take(1usize)(input)
        }

        // Start at line 0, column 1
        let (start, _) = take1(input).unwrap();

        // Start span should be at (1, 2), which is (0, 1) in our coord space
        assert_eq!(start.location_line(), 1);
        assert_eq!(start.get_utf8_column(), 2);

        let region = Region::from((start, start));
        assert_eq!(region, Region::from((0, 1, 0, 1)));
    }

    #[test]
    fn region_from_span_tuple_should_assume_second_span_is_right_after_region_ends(
    ) {
        let input = Span::new("abc\n12345");

        fn take1(input: Span) -> nom::IResult<Span, Span> {
            nom::bytes::complete::take(1usize)(input)
        }

        // Start at line 0, column 1
        let (start, _) = take1(input).unwrap();

        // Start span should be at (1, 2), which is (0, 1) in our coord space
        assert_eq!(start.location_line(), 1);
        assert_eq!(start.get_utf8_column(), 2);

        // End at line 1, column 3
        let (end, _) = take1(start).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();
        let (end, _) = take1(end).unwrap();

        // Span should now be at (2, 4), which is (1, 3) in our coord space
        assert_eq!(end.location_line(), 2);
        assert_eq!(end.get_utf8_column(), 4);

        // region should start at (0, 1) and end at (1, 3-1)
        let region = Region::from((start, end));
        assert_eq!(region, Region::from((0, 1, 1, 2)));
    }
}
