use crate::StrictEq;
use derive_more::{Constructor, Deref, DerefMut, Display};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

mod region;
pub use region::Region;

/// Represents a trait that provides the ability to get the children of an
/// element as a slice
pub trait AsChildrenSlice {
    /// The type of child contained within
    type Child;

    /// Returns a slice to children contained within
    fn as_children_slice(&self) -> &[Self::Child];
}

/// Represents a trait that provides the ability to get the children of an
/// element as a mut slice
pub trait AsChildrenMutSlice {
    /// The type of child contained within
    type Child;

    /// Returns a mutable slice to children contained within
    fn as_children_mut_slice(&mut self) -> &mut [Self::Child];
}

/// Represents a trait that provides the ability to get the children of an
/// element through a consuming conversion
pub trait IntoChildren {
    /// The type of child contained within
    type Child;

    /// Returns a vec of children contained within
    fn into_children(self) -> Vec<Self::Child>;
}

/// Represents an encapsulation of a language element and its location
/// within some string/file
#[derive(
    Constructor,
    Copy,
    Clone,
    Debug,
    Display,
    Deref,
    DerefMut,
    Eq,
    Serialize,
    Deserialize,
)]
#[display(fmt = "{}", inner)]
pub struct Located<T> {
    #[deref]
    #[deref_mut]
    inner: T,
    region: Region,
}

impl<T> Located<T> {
    /// Maps a `Located<T>` to `Located<U>` by applying a
    /// function to the underlying element. Useful when upleveling the
    /// element (such as wrapping a Header1) while the region remains
    /// unchanged.
    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Located<U> {
        Located::new(f(self.inner), self.region)
    }

    /// Takes a `Located` and replaces its region, producing the
    /// updated region. This takes ownership of the existing element!
    pub fn take_with_region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    /// Converts from `&Located<T>` to `Located<&T>`
    pub fn as_ref(&self) -> Located<&T> {
        Located {
            inner: &self.inner,
            region: self.region,
        }
    }

    /// Converts from `&mut Located<T>` to `Located<&mut T>`
    pub fn as_mut(&mut self) -> Located<&mut T> {
        Located {
            inner: &mut self.inner,
            region: self.region,
        }
    }

    /// Converts from `&Located<T>` to `&T`
    pub fn as_inner(&self) -> &T {
        &self.inner
    }

    /// Converts from `&mut Located<T>` to `&mut T`
    pub fn as_mut_inner(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Converts from `Located<T>` to `T`
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Returns depth of the inner value among other Located objects
    pub fn depth(&self) -> u16 {
        self.region.depth()
    }

    /// Returns a copy of the region associated with the inner value
    pub fn region(&self) -> Region {
        self.region
    }
}

impl<T> Located<Option<T>> {
    /// Transposes a `Located` of an [`Option`] into an [`Option`] of a `Located`.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use vimwiki::Located;
    /// let x: Located<Option<usize>> = Located::from(Some(5));
    /// let y: Option<Located<usize>> = Some(Located::from(5));
    /// assert_eq!(x.transpose(), y);
    /// ```
    pub fn transpose(self) -> Option<Located<T>> {
        let region = self.region();
        self.into_inner().map(|inner| Located::new(inner, region))
    }
}

impl<T: PartialEq> PartialEq for Located<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: PartialEq> PartialEq<T> for Located<T> {
    fn eq(&self, other: &T) -> bool {
        &self.inner == other
    }
}

impl<T: StrictEq> StrictEq for Located<T> {
    /// Performs strict equality check by verifying that inner value is
    /// strict equal and that the regions of both located are equal
    fn strict_eq(&self, other: &Self) -> bool {
        self.inner.strict_eq(&other.inner) && self.region == other.region
    }
}

impl<T: StrictEq> StrictEq<T> for Located<T> {
    /// Performs strict equality check by verifying that inner value is
    /// strict equal to the provided value
    fn strict_eq(&self, other: &T) -> bool {
        self.inner.strict_eq(&other)
    }
}

impl<T: Hash> Hash for Located<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<T> From<T> for Located<T> {
    /// Creates around `T`, using a default location
    fn from(t: T) -> Self {
        Self::new(t, Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn map_should_transform_inner_value_and_keep_region() {
        let le = Located::new(3, Region::new(1, 4));
        let mapped_le = le.map(|c| c + 1);
        assert_eq!(*mapped_le.as_inner(), 4);
        assert_eq!(mapped_le.region(), Region::new(1, 4));
    }

    #[test]
    fn equality_with_other_should_only_use_inner_value() {
        let le1 = Located::new(3, Region::new(1, 4));
        let le2 = Located::new(3, Region::default());
        assert_eq!(le1, le2);
    }

    #[test]
    fn equality_with_inner_type_should_only_use_inner_value() {
        let le = Located::new(3, Region::new(1, 4));
        let inner = 3;
        assert_eq!(le, inner);
        assert!(le != inner + 1);
    }

    #[test]
    fn hashing_should_only_use_inner_value() {
        let le1 = Located::new(3, Region::new(1, 4));
        let le2 = Located::new(3, Region::default());
        let le3 = Located::new(4, Region::new(1, 4));
        let le4 = Located::new(3, Region::new(1, 4));

        let mut m = HashSet::new();
        m.insert(le1);

        let le = m
            .get(&le2)
            .expect("Failed to retrieve Located with another Located");
        assert_eq!(*le.as_inner(), 3);
        assert_eq!(le.region(), Region::new(1, 4));

        assert_eq!(m.get(&le3), None);

        let le = m
            .get(&le4)
            .expect("Failed to retrieve Located with another Located");
        assert_eq!(*le.as_inner(), 3);
        assert_eq!(le.region(), Region::new(1, 4));
    }

    #[test]
    fn as_ref_should_return_new_element_with_ref_and_same_region() {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let le = Located::new(Test(5), Region::new(1, 4));
        let le_ref = le.as_ref();

        assert_eq!(le_ref.inner, &Test(5));
        assert_eq!(le_ref.region(), Region::new(1, 4));
    }

    #[test]
    fn as_mut_should_return_new_element_with_mut_and_same_region() {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let mut le = Located::new(Test(5), Region::new(1, 4));
        let le_mut = le.as_mut();

        assert_eq!(le_mut.inner, &mut Test(5));
        assert_eq!(le_mut.region(), Region::new(1, 4));
    }

    #[test]
    fn as_inner_should_return_new_element_with_ref_to_inner_and_same_region() {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let le = Located::new(Test(5), Region::new(1, 4));
        let inner = le.as_inner();

        assert_eq!(inner, &Test(5));
    }

    #[test]
    fn as_mut_inner_should_return_new_element_with_mut_ref_to_inner_and_same_region(
    ) {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let mut le = Located::new(Test(5), Region::new(1, 4));
        let inner = le.as_mut_inner();

        assert_eq!(inner, &mut Test(5));
    }

    #[test]
    fn into_inner_should_return_inner_value_as_owned() {
        #[derive(Debug, PartialEq, Eq)]
        struct Test(usize);

        let le = Located::new(Test(5), Region::new(1, 4));
        let inner = le.into_inner();

        assert_eq!(inner, Test(5));
    }
}
