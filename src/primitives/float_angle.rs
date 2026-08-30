/// FloatAngle: static methods for angle arithmetic in radians (f32) — mirrors .NET FloatAngle.cs.

pub struct FloatAngle;

impl FloatAngle {
    pub const PI: f32 = std::f32::consts::PI;
    pub const PI2: f32 = 2.0 * std::f32::consts::PI;
    pub const HALF_PI: f32 = 0.5 * std::f32::consts::PI;

    pub fn add(start: f32, delta: f32) -> f32 {
        let angle = start + delta;
        if angle < Self::PI2 {
            angle
        } else {
            angle - Self::PI2
        }
    }

    pub fn opposite(angle: f32) -> f32 {
        if angle < Self::PI {
            angle + Self::PI
        } else {
            angle - Self::PI
        }
    }

    pub fn distance(first: f32, second: f32) -> f32 {
        let delta = (first - second).abs();
        if delta <= Self::PI {
            delta
        } else {
            Self::PI2 - delta
        }
    }

    pub fn difference(first: f32, second: f32) -> f32 {
        let angle = first - second;
        if angle >= 0.0 {
            angle
        } else {
            angle + Self::PI2
        }
    }

    pub fn complementary(angle: f32) -> f32 {
        let complement = Self::PI2 - angle;
        if complement < Self::PI2 {
            complement
        } else {
            complement - Self::PI2
        }
    }

    pub fn normalized(angle: f32) -> bool {
        angle >= 0.0 && angle < Self::PI2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_constants() {
        assert!((FloatAngle::PI - std::f32::consts::PI).abs() < 1e-6);
        assert!((FloatAngle::PI2 - 2.0 * std::f32::consts::PI).abs() < 1e-6);
        assert!((FloatAngle::HALF_PI - 0.5 * std::f32::consts::PI).abs() < 1e-6);
    }

    #[test]
    fn test_add() {
        // add(0, PI) = PI (no wrap since PI < 2*PI)
        assert!((FloatAngle::add(0.0, FloatAngle::PI) - FloatAngle::PI).abs() < 1e-6);
        // add(PI, PI) = 2*PI = PI2; PI2 is NOT < PI2, so wraps to 0
        assert!((FloatAngle::add(FloatAngle::PI, FloatAngle::PI) - 0.0).abs() < 1e-6);
        // add(PI2, 0) = PI2; PI2 is NOT < PI2, so wraps to 0
        assert!((FloatAngle::add(FloatAngle::PI2, 0.0) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_opposite() {
        assert!((FloatAngle::opposite(0.0) - FloatAngle::PI).abs() < 1e-6);
        assert!((FloatAngle::opposite(FloatAngle::PI) - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_distance() {
        assert!(FloatAngle::distance(0.0, 0.0).abs() < 1e-6);
        assert!((FloatAngle::distance(0.0, FloatAngle::PI) - FloatAngle::PI).abs() < 1e-6);
    }

    #[test]
    fn test_difference() {
        assert!(FloatAngle::difference(0.0, 0.0).abs() < 1e-6);
        assert!((FloatAngle::difference(FloatAngle::PI2, 0.0) - FloatAngle::PI2).abs() < 1e-6);
    }

    #[test]
    fn test_normalized() {
        assert!(FloatAngle::normalized(0.0));
        assert!(FloatAngle::normalized(1.0));
        assert!(!FloatAngle::normalized(-0.1));
        assert!(!FloatAngle::normalized(FloatAngle::PI2));
    }
}
