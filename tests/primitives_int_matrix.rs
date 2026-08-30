//! Unit tests for IntMatrix primitive type.

#[cfg(test)]
mod primitives_int_matrix {
    use cearaafis::primitives::IntMatrix;
    use cearaafis::primitives::IntPoint;

    #[test]
    fn test_int_matrix_new() {
        let m = IntMatrix::new(5, 5);
        assert_eq!(m.width(), 5);
        assert_eq!(m.height(), 5);
    }

    #[test]
    fn test_int_matrix_set_get() {
        let mut m = IntMatrix::new(5, 5);
        m.set(2, 2, 42);
        assert_eq!(m.get(2, 2), 42);
    }

    #[test]
    fn test_int_matrix_from_point() {
        let size = IntPoint::new(10, 20);
        let m = IntMatrix::from_point(&size);
        assert_eq!(m.width(), 10);
        assert_eq!(m.height(), 20);
    }
}
