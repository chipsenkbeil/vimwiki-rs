use super::{Region, Span, StrictLocatedElement};
use derive_more::{AsMut, AsRef, Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Represents an encapsulation of a language element and its location
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
pub struct LocatedElement<T> {
    #[as_ref]
    #[as_mut]
    #[deref]
    #[deref_mut]
    pub element: T,
    pub region: Region,
}

/// Shorthand alias for LocatedElement
pub type LC<T> = LocatedElement<T>;

impl<T> LocatedElement<T> {
    /// Maps a `LocatedElement<T>` to `LocatedElement<U>` by applying a
    /// function to the underlying element. Useful when upleveling the
    /// element (such as wrapping a Header1) while the region remains
    /// unchanged.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> LocatedElement<U> {
        LocatedElement::new(f(self.element), self.region)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `LocatedElement<T>` (with default region).
    #[inline]
    pub fn wrap<U>(f: impl Fn(U) -> T) -> impl Fn(U) -> Self {
        Self::wrap_with_region(Default::default(), f)
    }

    /// Wraps a function that would transform some input into a type `T` such
    /// that the higher-order function will transform some input into a
    /// `LocatedElement<T>`.
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

    /// Takes a `LocatedElement` and replaces its region, producing the
    /// updated region. This is takes ownership of the existing element!
    pub fn take_with_region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    /// Converts LocatedElement to a strict variant
    pub fn into_strict(self) -> StrictLocatedElement<T> {
        self.into()
    }
}

impl<T: PartialEq> PartialEq for LocatedElement<T> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}

impl<T: PartialEq> PartialEq<T> for LocatedElement<T> {
    fn eq(&self, other: &T) -> bool {
        &self.element == other
    }
}

impl<T: Hash> Hash for LocatedElement<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.element.hash(state);
    }
}

impl<T> From<T> for LocatedElement<T> {
    /// Creates a new located element around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, Default::default())
    }
}

impl<T> From<(T, Span, Span)> for LocatedElement<T> {
    /// Creates a new located element around `T`, using a default location
    fn from((element, start, end): (T, Span, Span)) -> Self {
        Self::new(element, Region::from((start, end)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn located_element_map_should_transform_inner_element_and_keep_region()
    {
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        let mapped_lc = lc.map(|c| c + 1);
        assert_eq!(mapped_lc.element, 4);
        assert_eq!(mapped_lc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_element_wrap_should_apply_function_and_wrap_in_default_region()
    {
        let lc = LC::wrap(|x: usize| x.to_string())(3);
        assert_eq!(lc.element, String::from("3"));
        assert_eq!(lc.region, Region::default());
    }

    #[test]
    fn located_element_wrap_with_region_should_apply_function_and_wrap_in_provided_region(
    ) {
        let lc =
            LC::wrap_with_region(Region::from(((1, 2), (3, 4))), |x: usize| {
                x.to_string()
            })(3);
        assert_eq!(lc.element, String::from("3"));
        assert_eq!(lc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_element_equality_with_other_located_element_should_only_use_inner_element(
    ) {
        let lc1 = LC::new(3, Region::from(((1, 2), (3, 4))));
        let lc2 = LC::new(3, Region::default());
        assert_eq!(lc1, lc2);
    }

    #[test]
    fn located_element_equality_with_inner_type_should_only_use_inner_element(
    ) {
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        let inner = 3;
        assert_eq!(lc, inner);
        assert!(lc != inner + 1);
    }

    #[test]
    fn located_element_hashing_should_only_use_inner_element() {
        let lc1 = LC::new(3, Region::from(((1, 2), (3, 4))));
        let lc2 = LC::new(3, Region::default());
        let lc3 = LC::new(4, Region::from(((1, 2), (3, 4))));
        let lc4 = LC::new(3, Region::from(((1, 2), (3, 4))));

        let mut m = HashSet::new();
        m.insert(lc1);

        let lc = m.get(&lc2).expect("Failed to retrieve LC with another LC");
        assert_eq!(lc.element, 3);
        assert_eq!(lc.region, Region::from(((1, 2), (3, 4))));

        assert_eq!(m.get(&lc3), None);

        let lc = m.get(&lc4).expect("Failed to retrieve LC with another LC");
        assert_eq!(lc.element, 3);
        assert_eq!(lc.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_element_equality_with_strict_located_element_should_use_inner_element_and_region(
    ) {
        let lc = LC::new(3, Region::from(((1, 2), (3, 4))));
        let slc = StrictLocatedElement::new(3, Region::default());
        assert!(lc != slc, "{:?} unexpectedly equaled {:?}", lc, slc);
    }
}
