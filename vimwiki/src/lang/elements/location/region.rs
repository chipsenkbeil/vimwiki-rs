use super::Position;
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
    pub start: Position,
    pub end: Position,
}

impl Region {
    /// Checks if a position is contained within this region
    pub fn contains(&self, pos: Position) -> bool {
        pos >= self.start && pos <= self.end
    }

    /// Shifts the start & end lines by the given delta
    ///
    /// ```
    /// let mut region = Region::new(Position::new(1, 5), Position::new(4, 2));
    /// region.shift_lines(3);
    /// assert_eq!(region.start.line, 4);
    /// assert_eq!(region.end.line, 7);
    ///
    /// region.shift_lines(-3);
    /// assert_eq!(region.start.line, 1);
    /// assert_eq!(region.end.line, 4);
    /// ```
    pub fn shift_lines(&mut self, delta: isize) {
        match delta {
            delta if delta > 0 => {
                let delta_usize = delta as usize;
                self.start.line += delta_usize;
                self.end.line += delta_usize;
            }
            delta if delta < 0 => {
                let delta_usize = -delta as usize;
                self.start.line -= delta_usize;
                self.end.line -= delta_usize;
            }
            _ => {}
        }
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

impl From<((usize, usize), (usize, usize))> for Region {
    fn from(coords: ((usize, usize), (usize, usize))) -> Self {
        Self::from((Position::from(coords.0), Position::from(coords.1)))
    }
}

impl From<(usize, usize, usize, usize)> for Region {
    fn from(coords: (usize, usize, usize, usize)) -> Self {
        Self::from(((coords.0, coords.1), (coords.2, coords.3)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
