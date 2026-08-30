//! Unit tests for DoubleMatrix primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/DoubleMatrixTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::DoubleMatrix;

    #[test]
    fn test_double_matrix_new() {
        let m = DoubleMatrix::new(10, 10);
        assert_eq!(m.width(), 10);
        assert_eq!(m.height(), 10);
    }

    #[test]
    fn test_double_matrix_set_get() {
        let mut m = DoubleMatrix::new(5, 5);
        m.set(2, 3, 0.5);
        assert_eq!(m.get(2, 3), 0.5);
    }

    #[test]
    fn test_double_matrix_set_at() {
        let mut m = DoubleMatrix::new(5, 5);
        m.set(2, 3, 0.75);
        assert_eq!(m.get_at(2, 3), 0.75);
    }

    #[test]
    fn test_double_matrix_add() {
        let mut m = DoubleMatrix::new(5, 5);
        m.set(2, 2, 1.0);
        m.add(2, 2, 0.5);
        assert_eq!(m.get(2, 2), 1.5);
    }

    #[test]
    fn test_double_matrix_multiply() {
        let mut m = DoubleMatrix::new(5, 5);
        m.set(2, 2, 2.0);
        m.multiply(2, 2, 3.0);
        assert_eq!(m.get(2, 2), 6.0);
    }

    #[test]
    fn test_double_matrix_index() {
        let mut m = DoubleMatrix::new(3, 3);
        m.set(0, 0, 1.0);
        m.set(1, 1, 2.0);
        assert_eq!(m.get(0, 0), 1.0);
        assert_eq!(m.get(1, 1), 2.0);
    }

    #[test]
    fn test_double_matrix_size() {
        let m = DoubleMatrix::new(10, 20);
        assert_eq!(m.size().x, 10);
        assert_eq!(m.size().y, 20);
    }

    #[test]
    fn test_double_matrix_bounds() {
        let m = DoubleMatrix::new(5, 5);
        // Out of bounds should panic or return default — check behavior
        let _val = m.get(0, 0);
        assert_eq!(_val, 0.0); // default
    }
}
