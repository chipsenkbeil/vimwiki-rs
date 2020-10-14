use derive_more::From;
use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo,
    RangeToInclusive,
};

macro_rules! new_bound {
    ($base:expr, $pre:expr) => {
        match ($base, $pre) {
            (Bound::Included(b), Bound::Included(p)) => Bound::Included(b + p),
            (Bound::Included(b), Bound::Excluded(p)) => {
                Bound::Included(b + p + 1)
            }
            (Bound::Included(b), Bound::Unbounded) => Bound::Included(*b),
            (Bound::Excluded(b), Bound::Included(p)) => Bound::Excluded(b + p),
            (Bound::Excluded(b), Bound::Excluded(p)) => {
                Bound::Excluded(b + p + 1)
            }
            (Bound::Excluded(b), Bound::Unbounded) => Bound::Excluded(*b),
            (Bound::Unbounded, Bound::Included(p)) => Bound::Included(*p),
            (Bound::Unbounded, Bound::Excluded(p)) => Bound::Excluded(*p),
            (Bound::Unbounded, Bound::Unbounded) => Bound::Unbounded,
        }
    };
}

/// Represents some type of range
#[derive(Clone, Debug, From, Eq, PartialEq)]
pub enum SomeRange<Idx> {
    Range(Range<Idx>),
    RangeFrom(RangeFrom<Idx>),
    RangeFull(RangeFull),
    RangeInclusive(RangeInclusive<Idx>),
    RangeTo(RangeTo<Idx>),
    RangeToInclusive(RangeToInclusive<Idx>),
}

/// Maps one range into another range
///
/// abcdefghijklmnopqrstuvwxyz
///      |  |      |      |
///      |  8      15 (post-mapping)
///      |  |      |      |
///      |  |      |      |
///      5----------------22 (base)
///         |      |
///         3------9 (pre-mapping)
///
/// Couple of details:
///
/// 1. Post-mapping's start is relative to the start of the base and will be
///    capped to no greater than the base's end.
/// 2. Post-mapping's end will be capped to that of the base's end
///
pub fn apply_range_to_base_range<
    B: RangeBounds<usize>,
    R: RangeBounds<usize>,
>(
    base_r: B,
    pre_r: R,
) -> SomeRange<usize> {
    let start_bound = new_bound!(base_r.start_bound(), pre_r.start_bound());
    let end_bound = new_bound!(base_r.start_bound(), pre_r.end_bound());

    match (start_bound, end_bound) {
        (Bound::Included(start), Bound::Included(end)) => {
            SomeRange::from(start..=end)
        }
        (Bound::Included(start), Bound::Excluded(end)) => {
            SomeRange::from(start..end)
        }
        (Bound::Included(start), Bound::Unbounded) => SomeRange::from(start..),
        (Bound::Excluded(start), Bound::Included(end)) => {
            SomeRange::from((start + 1)..=end)
        }
        (Bound::Excluded(start), Bound::Excluded(end)) => {
            SomeRange::from((start + 1)..end)
        }
        (Bound::Excluded(start), Bound::Unbounded) => {
            SomeRange::from((start + 1)..)
        }
        (Bound::Unbounded, Bound::Included(end)) => SomeRange::from(..=end),
        (Bound::Unbounded, Bound::Excluded(end)) => SomeRange::from(..end),
        (Bound::Unbounded, Bound::Unbounded) => SomeRange::from(..),
    }
}

/// Overwrite range of remaining items in space with provided items by
/// copying each into the new location
pub fn fill_range<T: Copy>(items: &mut [T], r: SomeRange<usize>, value: T) {
    match r {
        SomeRange::Range(x) => {
            for item in &mut items[x] {
                *item = value;
            }
        }
        SomeRange::RangeFrom(x) => {
            for item in &mut items[x] {
                *item = value;
            }
        }
        SomeRange::RangeFull(x) => {
            for item in &mut items[x] {
                *item = value;
            }
        }
        SomeRange::RangeInclusive(x) => {
            for item in &mut items[x] {
                *item = value;
            }
        }
        SomeRange::RangeTo(x) => {
            for item in &mut items[x] {
                *item = value;
            }
        }
        SomeRange::RangeToInclusive(x) => {
            for item in &mut items[x] {
                *item = value;
            }
        }
    }
}
