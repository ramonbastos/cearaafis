//! Unit tests for BooleanMatrix primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/BooleanMatrixTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::BooleanMatrix;

    #[test]
    fn test_boolean_matrix_new() {
        let m = BooleanMatrix::new(5, 5);
        assert_eq!(m.width(), 5);
        assert_eq!(m.height(), 5);
    }

    #[test]
    fn test_boolean_matrix_set_get() {
        let mut m = BooleanMatrix::new(5, 5);
        m.set(2, 2, true);
        assert!(m.get(2, 2));
    }

    #[test]
    fn test_boolean_matrix_default_is_false() {
        let m = BooleanMatrix::new(5, 5);
        assert!(!m.get(0, 0));
        assert!(!m.get(4, 4));
    }

    #[test]
    fn test_boolean_matrix_invert() {
        let mut m = BooleanMatrix::new(3, 3);
        m.set(1, 1, true);
        m.invert();
        assert!(!m.get(1, 1));
        assert!(m.get(0, 0));
    }

    #[test]
    fn test_boolean_matrix_merge() {
        let mut m1 = BooleanMatrix::new(3, 3);
        let m2 = BooleanMatrix::new(3, 3);
        m1.set(1, 1, true);
        m1.merge(&m2); // merge does nothing if m2 is all false
        assert!(m1.get(1, 1));
    }

    #[test]
    fn test_boolean_matrix_clone() {
        let mut m = BooleanMatrix::new(3, 3);
        m.set(1, 1, true);
        let clone = BooleanMatrix::from_clone(&m);
        assert!(clone.get(1, 1));
        assert!(clone.width() == m.width());
        assert!(clone.height() == m.height());
        // clone is independent — modifying m shouldn't affect clone
        m.set(1, 1, false);
        assert!(clone.get(1, 1));
    }

    #[test]
    fn test_boolean_matrix_index() {
        let mut m = BooleanMatrix::new(3, 3);
        m.set(1, 1, true);
        assert!(m.get(1, 1));
    }
}
