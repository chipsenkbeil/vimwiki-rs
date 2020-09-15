use super::{Region, Span, StrictLocatedComponent};
use derive_more::{AsMut, AsRef, Constructor, Deref, DerefMut};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

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

    /// Takes a `LocatedComponent` and replaces its region, producing the
    /// updated region. This is takes ownership of the existing component!
    pub fn take_with_region(mut self, region: Region) -> Self {
        self.region = region;
        self
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
        let slc = StrictLocatedComponent::new(3, Region::default());
        assert!(lc != slc, "{:?} unexpectedly equaled {:?}", lc, slc);
    }
}
