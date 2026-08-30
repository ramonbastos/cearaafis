//! Unit tests for FloatAngle — mirrors .NET SourceAFIS.Tests/Engine/Primitives/FloatAngleTest.cs.

#[cfg(test)]
mod tests {
    use cearaafis::primitives::FloatAngle;

    #[test]
    fn test_pi_constants() {
        assert!((FloatAngle::PI - std::f32::consts::PI).abs() < 0.001);
        assert!((FloatAngle::PI2 - 2.0 * std::f32::consts::PI).abs() < 0.001);
        assert!((FloatAngle::HALF_PI - 0.5 * std::f32::consts::PI).abs() < 0.001);
    }

    #[test]
    fn test_add() {
        // add(0, PI) = PI (no wrap)
        let result = FloatAngle::add(0.0, FloatAngle::PI);
        assert!(result >= 0.0);
        assert!(result < FloatAngle::PI2);
        assert!((result - FloatAngle::PI).abs() < 0.001);

        // add(PI, PI) = 0 (wraps around since PI + PI = PI2, PI2 < PI2 is false)
        let result = FloatAngle::add(FloatAngle::PI, FloatAngle::PI);
        assert!(result.abs() < 0.001); // wraps to 0

        // add(PI2, 0) = 0 (PI2 is not < PI2, so wraps to 0)
        let result = FloatAngle::add(FloatAngle::PI2, 0.0);
        assert!(result.abs() < 0.001);
    }

    #[test]
    fn test_opposite() {
        // opposite(0) = PI, opposite(PI) = 0
        assert!((FloatAngle::opposite(0.0) - FloatAngle::PI).abs() < 0.001);
        assert!(FloatAngle::opposite(FloatAngle::PI).abs() < 0.001);
    }

    #[test]
    fn test_distance() {
        // distance is the shortest arc between two angles
        assert!(FloatAngle::distance(0.0, 0.0).abs() < 0.001);
        assert!((FloatAngle::distance(0.0, FloatAngle::PI) - FloatAngle::PI).abs() < 0.001);
        assert!(
            (FloatAngle::distance(FloatAngle::PI, FloatAngle::PI2) - FloatAngle::PI).abs() < 0.001
        );
    }

    #[test]
    fn test_difference() {
        // difference(first, second) = first - second (normalized to [0, PI2))
        assert!(FloatAngle::difference(0.0, 0.0).abs() < 0.001);
        // difference(PI2, 0) = PI2 - 0 = PI2 (non-negative, no wrap)
        assert!((FloatAngle::difference(FloatAngle::PI2, 0.0) - FloatAngle::PI2).abs() < 0.001);
        assert!((FloatAngle::difference(FloatAngle::PI, 0.0) - FloatAngle::PI).abs() < 0.001);
    }

    #[test]
    fn test_complementary() {
        // complementary(angle) = PI2 - angle (wraps if needed)
        // complementary(0) = PI2 - 0 = PI2; PI2 < PI2 is false, so wraps to 0
        assert!(FloatAngle::complementary(0.0).abs() < 0.001);
        // complementary(PI) = PI2 - PI = PI; PI < PI2 is true, no wrap
        assert!((FloatAngle::complementary(FloatAngle::PI) - FloatAngle::PI).abs() < 0.001);
    }

    #[test]
    fn test_normalized() {
        assert!(FloatAngle::normalized(0.0));
        assert!(FloatAngle::normalized(1.0));
        assert!(FloatAngle::normalized(FloatAngle::PI));
        assert!(!FloatAngle::normalized(-0.1));
        assert!(!FloatAngle::normalized(FloatAngle::PI2));
    }
}
