//! Unit tests for DoublePoint primitive type.

#[cfg(test)]
mod tests {
    use cearaafis::primitives::{DoublePoint, IntPoint};

    #[test]
    fn test_double_point_new() {
        let pt = DoublePoint::new(1.0, 2.0);
        assert!((pt.x() - 1.0).abs() < 1e-10);
        assert!((pt.y() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_double_point_from_int() {
        let ip = IntPoint::new(3, 4);
        let dp = DoublePoint::from_int_point(&ip);
        assert!((dp.x() - 3.0).abs() < 1e-10);
        assert!((dp.y() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_double_point_round() {
        let dp = DoublePoint::new(3.7, 4.2);
        let ip = dp.round();
        assert_eq!(ip.x(), 4);
        assert_eq!(ip.y(), 4);
    }

    #[test]
    fn test_double_point_length_sq() {
        let dp = DoublePoint::new(3.0, 4.0);
        assert!((dp.length_sq() - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_double_point_normalize() {
        let dp = DoublePoint::new(1.0, 0.0);
        let norm = dp.normalize();
        assert!((norm.x() - 1.0).abs() < 1e-10);
        assert!((norm.y() - 0.0).abs() < 1e-10);
    }
}
