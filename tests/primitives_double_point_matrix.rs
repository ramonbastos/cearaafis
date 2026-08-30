//! Unit tests for DoublePointMatrix primitive type.

#[cfg(test)]
mod tests {
    use cearaafis::primitives::{DoublePoint, DoublePointMatrix};

    #[test]
    fn test_double_point_matrix_new() {
        let m = DoublePointMatrix::new(5, 5);
        assert_eq!(m.width, 5);
        assert_eq!(m.height, 5);
    }

    #[test]
    fn test_double_point_matrix_set_get() {
        let mut m = DoublePointMatrix::new(5, 5);
        let pt = DoublePoint::new(1.0, 2.0);
        m.set(2, 3, pt);
        let got = m.get(2, 3);
        assert!((got.x() - 1.0).abs() < 0.001);
        assert!((got.y() - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_double_point_matrix_at() {
        let mut m = DoublePointMatrix::new(5, 5);
        let pt = DoublePoint::new(3.0, 4.0);
        m.set(1, 1, pt);
        let got = m.get(1, 1);
        assert!((got.x() - 3.0).abs() < 0.001);
        assert!((got.y() - 4.0).abs() < 0.001);
    }
}
