//! Unit tests for IntPoint primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/IntPointTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::IntPoint;

    #[test]
    fn test_int_point_new() {
        let pt = IntPoint::new(10, 20);
        assert_eq!(pt.x, 10);
        assert_eq!(pt.y, 20);
    }

    #[test]
    fn test_int_point_add() {
        let a = IntPoint::new(10, 20);
        let b = IntPoint::new(5, 3);
        let c = a + b;
        assert_eq!(c.x, 15);
        assert_eq!(c.y, 23);
    }

    #[test]
    fn test_int_point_sub() {
        let a = IntPoint::new(10, 20);
        let b = IntPoint::new(5, 3);
        let c = a - b;
        assert_eq!(c.x, 5);
        assert_eq!(c.y, 17);
    }

    #[test]
    fn test_int_point_neg() {
        let pt = IntPoint::new(10, -5);
        let neg = -pt;
        assert_eq!(neg.x, -10);
        assert_eq!(neg.y, 5);
    }

    #[test]
    fn test_int_point_to_short_point() {
        let pt = IntPoint::new(100, 200);
        let sp = pt.to_short();
        assert_eq!(sp.x, 100);
        assert_eq!(sp.y, 200);
    }

    #[test]
    fn test_int_point_area() {
        let pt = IntPoint::new(5, 10);
        assert_eq!(pt.area(), 50);
    }

    #[test]
    fn test_int_point_length_sq() {
        let pt = IntPoint::new(3, 4);
        assert_eq!(pt.length_sq(), 25);
    }

    #[test]
    fn test_int_point_contains() {
        let pt = IntPoint::new(10, 10);
        let inner = IntPoint::new(3, 3);
        assert!(pt.contains(&inner));
    }

    #[test]
    fn test_int_point_line_to() {
        let start = IntPoint::new(0, 0);
        let end = IntPoint::new(4, 3);
        let line = start.line_to(&end);
        assert!(line.len() >= 4);
        assert!(line[0] == start);
        assert!(line[line.len() - 1] == end);
    }

    #[test]
    fn test_int_point_iterate() {
        let rect = IntPoint::new(3, 3);
        let points = rect.iterate();
        assert_eq!(points.len(), 9); // 3x3
    }

    #[test]
    fn test_int_point_eq() {
        let a = IntPoint::new(5, 10);
        let b = IntPoint::new(5, 10);
        assert_eq!(a, b);
    }

    #[test]
    fn test_int_point_ord() {
        let a = IntPoint::new(10, 20);
        let b = IntPoint::new(10, 30);
        assert!(a < b); // y ordering
    }

    #[test]
    fn test_int_point_hash() {
        use std::collections::HashSet;
        let pt = IntPoint::new(10, 20);
        let mut set = HashSet::new();
        set.insert(pt);
        assert!(set.contains(&pt));
    }
}
