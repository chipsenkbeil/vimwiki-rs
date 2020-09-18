use super::{LocatedComponent, Region, Span};
use derive_more::{AsMut, AsRef, Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};

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

    /// Takes a `StrictLocatedComponent` and replaces its region, producing the
    /// updated region. This is takes ownership of the existing component!
    pub fn take_with_region(mut self, region: Region) -> Self {
        self.region = region;
        self
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

impl<T> From<(T, Span, Span)> for StrictLocatedComponent<T> {
    /// Creates a new strict located component around `T`, using a default location
    fn from((component, start, end): (T, Span, Span)) -> Self {
        Self::new(component, Region::from((start, end)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
        let lc = LocatedComponent::new(3, Region::from(((1, 2), (3, 4))));
        assert!(slc != lc, "{:?} unexpectedly equaled {:?}", slc, lc);
    }
}
