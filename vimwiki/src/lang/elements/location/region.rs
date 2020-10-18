use derive_more::Constructor;
use serde::{Deserialize, Serialize};

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
    /// Position within some byte array this region begins
    offset: usize,

    /// Length of this region from the offset
    len: usize,
}

impl Region {
    /// Checks if a position is contained within this region
    #[inline]
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.offset && offset < (self.offset + self.len)
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl From<(usize, usize)> for Region {
    fn from(coords: (usize, usize)) -> Self {
        Self::new(coords.0, coords.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_should_successfully_return_whether_or_not_offset_within_region()
    {
        let region = Region::new(3, 2);
        assert!(!region.contains(0));
        assert!(!region.contains(1));
        assert!(!region.contains(2));
        assert!(region.contains(3));
        assert!(region.contains(4));
        assert!(!region.contains(5));
        assert!(!region.contains(6));
        assert!(!region.contains(7));
    }
}
