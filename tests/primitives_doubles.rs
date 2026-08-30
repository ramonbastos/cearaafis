//! Unit tests for Doubles utility functions.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/DoublesTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::Doubles;

    #[test]
    fn test_round_to_int() {
        assert_eq!(Doubles::round_to_int(0.0), 0);
        assert_eq!(Doubles::round_to_int(0.4), 0);
        assert_eq!(Doubles::round_to_int(0.5), 1);
        assert_eq!(Doubles::round_to_int(0.6), 1);
        assert_eq!(Doubles::round_to_int(1.5), 2);
        assert_eq!(Doubles::round_to_int(2.4), 2);
    }

    #[test]
    fn test_sq() {
        assert_eq!(Doubles::sq(3.0), 9.0);
        assert_eq!(Doubles::sq(0.0), 0.0);
        assert_eq!(Doubles::sq(-5.0), 25.0);
    }

    #[test]
    fn test_interpolate() {
        assert_eq!(Doubles::interpolate(0.0, 10.0, 0.0), 0.0);
        assert_eq!(Doubles::interpolate(0.0, 10.0, 1.0), 10.0);
        assert_eq!(Doubles::interpolate(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_interpolate_quad() {
        // Mirrors C#: Interpolate(3, 7, 2, 4, 0.5, 0.5) = 4
        // topleft=3, bottomleft=7, topright=2, bottomright=4
        let result = Doubles::interpolate_2d(3.0, 7.0, 2.0, 4.0, 0.5, 0.5);
        assert!(
            (result - 4.0).abs() < 0.01
        );
    }

    #[test]
    fn test_interpolate_exp() {
        let result = Doubles::interpolate_exponential(1.0, 10.0, 0.5);
        assert!(
            (result - 3.1622776601683795).abs() < 0.01
        );
        // InterpolateExponential(3, 10, 0) = 3
        assert_eq!(Doubles::interpolate_exponential(1.0, 10.0, 0.0), 1.0);
    }
}
