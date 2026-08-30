//! Unit tests for DoubleAngle utility functions.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/DoubleAngleTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::{DoubleAngle, DoublePoint, IntPoint};

    #[test]
    fn test_atan() {
        let dp = DoublePoint::new(1.0, 0.0);
        let result = DoubleAngle::atan_point(&dp);
        assert!(result >= 0.0);
        assert!(result < std::f64::consts::PI * 2.0);
    }

    #[test]
    fn test_atan_i32_int_point() {
        let pt = IntPoint::new(100, 0);
        let result = DoubleAngle::atan_i32_int_point(&pt);
        assert!(result >= 0.0);
        assert!(result < std::f64::consts::PI * 2.0);
    }

    #[test]
    fn test_atan_i32() {
        let center = IntPoint::new(0, 0);
        let pt = IntPoint::new(100, 100);
        let result = DoubleAngle::atan_i32(&center, &pt);
        assert!(result >= 0.0);
        assert!(result < std::f64::consts::PI * 2.0);
    }

    #[test]
    fn test_to_orientation() {
        let result = DoubleAngle::to_orientation(0.0);
        assert_eq!(result, 0.0);

        let result = DoubleAngle::to_orientation(std::f64::consts::PI / 2.0);
        assert!(
            (result - std::f64::consts::PI).abs() < 0.01
        );

        let result = DoubleAngle::to_orientation(std::f64::consts::PI);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_add() {
        let result = DoubleAngle::add(0.0, 0.0);
        assert_eq!(result, 0.0);

        let result = DoubleAngle::add(std::f64::consts::PI, std::f64::consts::PI);
        assert!(
            (result - 0.0).abs() < 0.01
        );
    }

    #[test]
    fn test_complementary() {
        let result = DoubleAngle::complementary(0.0);
        assert!(
            (result - 0.0).abs() < 0.01
        );
    }
}
