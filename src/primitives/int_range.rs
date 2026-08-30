/// IntRange: a range [start, end) — mirrors .NET IntRange.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IntRange {
    pub start: i32,
    pub end: i32,
}

impl IntRange {
    pub const ZERO: Self = Self { start: 0, end: 0 };

    pub fn new(start: i32, end: i32) -> Self {
        Self { start, end }
    }

    pub fn length(&self) -> i32 {
        self.end - self.start
    }
}

impl Default for IntRange {
    fn default() -> Self {
        Self::ZERO
    }
}

impl fmt::Display for IntRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = IntRange::new(0, 10);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 10);
    }

    #[test]
    fn test_length() {
        assert_eq!(IntRange::new(0, 10).length(), 10);
        assert_eq!(IntRange::new(5, 5).length(), 0);
        assert_eq!(IntRange::new(0, 0).length(), 0);
    }

    #[test]
    fn test_display() {
        assert_eq!(IntRange::new(0, 10).to_string(), "0..10");
    }

    #[test]
    fn test_zero() {
        assert_eq!(IntRange::ZERO.start, 0);
        assert_eq!(IntRange::ZERO.end, 0);
    }
}
