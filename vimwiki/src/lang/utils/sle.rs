use super::{LocatedElement, Region, Span};
use derive_more::{AsMut, AsRef, Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};

/// Represents a located element that has strict equality enforcement
/// (element + region versus just element)
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
pub struct StrictLocatedElement<T> {
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    pub element: T,
    pub region: Region,
}

impl<T> StrictLocatedElement<T> {
    /// Maps a `StrictLocatedElement<T>` to `StrictLocatedElement<U>` by
    /// applying a function to the underlying element. Useful when upleveling
    /// the element (such as wrapping a Header) while the region remains
    /// unchanged.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> StrictLocatedElement<U> {
        StrictLocatedElement::new(f(self.element), self.region)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `StrictLocatedElement<T>` (with default region).
    #[inline]
    pub fn wrap<U>(f: impl Fn(U) -> T) -> impl Fn(U) -> Self {
        Self::wrap_with_region(Default::default(), f)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `StrictLocatedElement<T>`.
    #[inline]
    pub fn wrap_with_region<U>(
        region: Region,
        f: impl Fn(U) -> T,
    ) -> impl Fn(U) -> Self {
        move |input| {
            let element = f(input);
            Self::new(element, region)
        }
    }

    /// Takes a `StrictLocatedElement` and replaces its region, producing the
    /// updated region. This is takes ownership of the existing element!
    pub fn take_with_region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    /// Converts StrictLocatedElement to a loose variant
    pub fn into_loose(self) -> LocatedElement<T> {
        self.into()
    }
}

/// Shorthand alias for StrictLocatedElement
pub type SLE<T> = StrictLocatedElement<T>;

impl<T> From<LocatedElement<T>> for StrictLocatedElement<T> {
    fn from(lc: LocatedElement<T>) -> Self {
        Self::new(lc.element, lc.region)
    }
}

impl<T> From<StrictLocatedElement<T>> for LocatedElement<T> {
    fn from(slc: StrictLocatedElement<T>) -> Self {
        Self::new(slc.element, slc.region)
    }
}

impl<T: PartialEq> PartialEq<LocatedElement<T>> for StrictLocatedElement<T> {
    fn eq(&self, other: &LocatedElement<T>) -> bool {
        self.element == other.element && self.region == other.region
    }
}

impl<T: PartialEq> PartialEq<StrictLocatedElement<T>> for LocatedElement<T> {
    fn eq(&self, other: &StrictLocatedElement<T>) -> bool {
        self.element == other.element && self.region == other.region
    }
}

impl<T> From<T> for StrictLocatedElement<T> {
    /// Creates a new strict located element around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, Default::default())
    }
}

impl<T> From<(T, Span, Span)> for StrictLocatedElement<T> {
    /// Creates a new strict located element around `T`, using a default location
    fn from((element, start, end): (T, Span, Span)) -> Self {
        Self::new(element, Region::from((start, end)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn strict_located_element_map_should_transform_inner_element_and_keep_region(
    ) {
        let sle = SLE::new(3, Region::from(((1, 2), (3, 4))));
        let mapped_sle = sle.map(|c| c + 1);
        assert_eq!(mapped_sle.element, 4);
        assert_eq!(mapped_sle.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn strict_located_element_wrap_should_apply_function_and_wrap_in_default_region(
    ) {
        let sle = SLE::wrap(|x: usize| x.to_string())(3);
        assert_eq!(sle.element, String::from("3"));
        assert_eq!(sle.region, Region::default());
    }

    #[test]
    fn strict_located_element_wrap_with_region_should_apply_function_and_wrap_in_provided_region(
    ) {
        let sle = SLE::wrap_with_region(
            Region::from(((1, 2), (3, 4))),
            |x: usize| x.to_string(),
        )(3);
        assert_eq!(sle.element, String::from("3"));
        assert_eq!(sle.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn strict_located_element_equality_with_other_located_element_should_use_inner_element_and_region(
    ) {
        let sle1 = SLE::new(3, Region::from(((1, 2), (3, 4))));
        let sle2 = SLE::new(3, Region::from(((1, 2), (3, 4))));
        assert_eq!(sle1, sle2);

        let sle1 = SLE::new(3, Region::from(((1, 2), (3, 4))));
        let sle2 = SLE::new(3, Region::default());
        assert!(sle1 != sle2, "{:?} unexpectedly equaled {:?}", sle1, sle2);
    }

    #[test]
    fn strict_located_element_hashing_should_use_inner_element_and_region() {
        let sle1 = SLE::new(3, Region::from(((1, 2), (3, 4))));
        let sle2 = SLE::new(3, Region::default());
        let sle3 = SLE::new(4, Region::from(((1, 2), (3, 4))));
        let sle4 = SLE::new(3, Region::from(((1, 2), (3, 4))));

        let mut m = HashSet::new();
        m.insert(sle1);

        assert_eq!(m.get(&sle2), None);
        assert_eq!(m.get(&sle3), None);

        let sle = m.get(&sle4).expect("Failed to get SLE with exact match");
        assert_eq!(sle.element, 3);
        assert_eq!(sle.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn strict_located_element_equality_with_located_element_should_use_inner_element_and_region(
    ) {
        let sle = SLE::new(3, Region::default());
        let le = LocatedElement::new(3, Region::from(((1, 2), (3, 4))));
        assert!(sle != le, "{:?} unexpectedly equaled {:?}", sle, le);
    }
}
