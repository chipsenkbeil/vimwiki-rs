use super::{Region, Span};
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
pub type LE<T> = LocatedElement<T>;

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
    /// updated region. This takes ownership of the existing element!
    pub fn take_with_region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    /// Takes a `LocatedElement` and shifts its region such that it starts
    /// at the specified line. This takes ownership of the existing element!
    pub fn take_at_line(mut self, line: usize) -> Self {
        let diff = self.region.end.line - self.region.start.line;
        self.region.start.line = line;
        self.region.end.line = line + diff;
        self
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
    fn located_element_map_should_transform_inner_element_and_keep_region() {
        let le = LE::new(3, Region::from(((1, 2), (3, 4))));
        let mapped_le = le.map(|c| c + 1);
        assert_eq!(mapped_le.element, 4);
        assert_eq!(mapped_le.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_element_wrap_should_apply_function_and_wrap_in_default_region() {
        let le = LE::wrap(|x: usize| x.to_string())(3);
        assert_eq!(le.element, String::from("3"));
        assert_eq!(le.region, Region::default());
    }

    #[test]
    fn located_element_wrap_with_region_should_apply_function_and_wrap_in_provided_region(
    ) {
        let le =
            LE::wrap_with_region(Region::from(((1, 2), (3, 4))), |x: usize| {
                x.to_string()
            })(3);
        assert_eq!(le.element, String::from("3"));
        assert_eq!(le.region, Region::from(((1, 2), (3, 4))));
    }

    #[test]
    fn located_element_equality_with_other_located_element_should_only_use_inner_element(
    ) {
        let le1 = LE::new(3, Region::from(((1, 2), (3, 4))));
        let le2 = LE::new(3, Region::default());
        assert_eq!(le1, le2);
    }

    #[test]
    fn located_element_equality_with_inner_type_should_only_use_inner_element()
    {
        let le = LE::new(3, Region::from(((1, 2), (3, 4))));
        let inner = 3;
        assert_eq!(le, inner);
        assert!(le != inner + 1);
    }

    #[test]
    fn located_element_hashing_should_only_use_inner_element() {
        let le1 = LE::new(3, Region::from(((1, 2), (3, 4))));
        let le2 = LE::new(3, Region::default());
        let le3 = LE::new(4, Region::from(((1, 2), (3, 4))));
        let le4 = LE::new(3, Region::from(((1, 2), (3, 4))));

        let mut m = HashSet::new();
        m.insert(le1);

        let le = m.get(&le2).expect("Failed to retrieve LE with another LE");
        assert_eq!(le.element, 3);
        assert_eq!(le.region, Region::from(((1, 2), (3, 4))));

        assert_eq!(m.get(&le3), None);

        let le = m.get(&le4).expect("Failed to retrieve LE with another LE");
        assert_eq!(le.element, 3);
        assert_eq!(le.region, Region::from(((1, 2), (3, 4))));
    }
}
