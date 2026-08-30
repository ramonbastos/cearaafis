//! Unit tests for IntRect primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/IntRectTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::{IntPoint, IntRect};

    #[test]
    fn test_int_rect_new() {
        let rect = IntRect::new(10, 20, 100, 50);
        assert_eq!(rect.left(), 10);
        assert_eq!(rect.top(), 20);
        assert_eq!(rect.right(), 110);
        assert_eq!(rect.bottom(), 70);
    }

    #[test]
    fn test_int_rect_area() {
        let rect = IntRect::new(0, 0, 10, 10);
        assert_eq!(rect.area(), 100);
    }

    #[test]
    fn test_int_rect_center() {
        let rect = IntRect::new(0, 0, 10, 10);
        let center = rect.center();
        assert_eq!(center.x, 5);
        assert_eq!(center.y, 5);
    }

    #[test]
    fn test_int_rect_between() {
        let rect = IntRect::between(&IntPoint::new(0, 0), &IntPoint::new(10, 10));
        assert_eq!(rect.x, 0);
        assert_eq!(rect.y, 0);
        assert_eq!(rect.width, 10);
        assert_eq!(rect.height, 10);
    }

    #[test]
    fn test_int_rect_around() {
        let rect = IntRect::around(10, 10, 5);
        assert_eq!(rect.x, 5);
        assert_eq!(rect.y, 5);
        assert_eq!(rect.width, 11);
        assert_eq!(rect.height, 11);
    }

    #[test]
    fn test_int_rect_intersect() {
        let r1 = IntRect::new(0, 0, 20, 20);
        let r2 = IntRect::new(10, 10, 20, 20);
        let intersect = r1.intersect(&r2);
        assert_eq!(intersect.left(), 10);
        assert_eq!(intersect.top(), 10);
        assert_eq!(intersect.width(), 10);
        assert_eq!(intersect.height(), 10);
    }

    #[test]
    fn test_int_rect_move_rect() {
        let rect = IntRect::new(0, 0, 10, 10);

        let moved = rect.move_rect(&IntPoint::new(5, 5));
        assert_eq!(moved.x, 5);
        assert_eq!(moved.y, 5);
        assert_eq!(moved.width, 10);
        assert_eq!(moved.height, 10);
    }

    #[test]
    fn test_int_rect_iterate() {
        let rect = IntRect::new(0, 0, 3, 3);
        let points = rect.iterate();
        assert_eq!(points.len(), 9); // 3x3
    }

    #[test]
    fn test_int_rect_eq() {
        let r1 = IntRect::new(0, 0, 10, 10);
        let r2 = IntRect::new(0, 0, 10, 10);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_int_rect_hash() {
        use std::collections::HashSet;
        let rect = IntRect::new(0, 0, 10, 10);
        let mut set = HashSet::new();
        set.insert(rect);
        assert!(set.contains(&rect));
    }
}
