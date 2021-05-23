/// Represents an equality check that is considered strict. In the case of
/// a `Located<T>`, will check both the inner type AND the region.
pub trait StrictEq<Rhs: ?Sized = Self> {
    fn strict_eq(&self, other: &Rhs) -> bool;

    #[inline]
    fn strict_ne(&self, other: &Rhs) -> bool {
        !self.strict_eq(other)
    }
}

/// Blanket implementation for two vectors of similarly-typed StrictEq elements
impl<T: StrictEq> StrictEq for Vec<T> {
    /// Performs strict_eq check on inner elements
    fn strict_eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self.iter().zip(other.iter()).all(|(x, y)| x.strict_eq(y))
    }
}
