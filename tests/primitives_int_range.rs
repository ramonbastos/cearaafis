//! Unit tests for IntRange primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/IntRangeTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::IntRange;

    #[test]
    fn test_int_range_new() {
        let range = IntRange::new(0, 10);
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 10);
    }

    #[test]
    fn test_int_range_length() {
        let range = IntRange::new(0, 10);
        assert_eq!(range.length(), 10);

        let range = IntRange::new(5, 15);
        assert_eq!(range.length(), 10);
    }

    #[test]
    fn test_int_range_zero() {
        let range = IntRange::ZERO;
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 0);
        assert_eq!(range.length(), 0);
    }

    #[test]
    fn test_int_range_display() {
        let range = IntRange::new(0, 10);
        let display = format!("{}", range);
        assert_eq!(display, "0..10");
    }

    #[test]
    fn test_int_range_ord() {
        let a = IntRange::new(0, 10);
        let b = IntRange::new(0, 20);
        assert!(a < b);
    }
}
