/// DoubleAngle: static methods for angle arithmetic — mirrors .NET DoubleAngle.cs.
use crate::primitives::{double_point::DoublePoint, int_point::IntPoint};

pub struct DoubleAngle;

impl DoubleAngle {
    pub const PI2: f64 = 2.0 * std::f64::consts::PI;
    pub const INV_PI2: f64 = 1.0 / Self::PI2;
    pub const HALF_PI: f64 = 0.5 * std::f64::consts::PI;

    pub fn to_vector(angle: f64) -> DoublePoint {
        DoublePoint::new(angle.cos(), angle.sin())
    }

    pub fn atan(x: f64, y: f64) -> f64 {
        let angle = y.atan2(x);
        if angle >= 0.0 {
            angle
        } else {
            angle + Self::PI2
        }
    }

    pub fn atan_point(point: &DoublePoint) -> f64 {
        Self::atan(point.x(), point.y())
    }

    pub fn atan_int_point(point: &IntPoint) -> f64 {
        Self::atan(point.x() as f64, point.y() as f64)
    }

    pub fn atan_i32_int_point(point: &IntPoint) -> f64 {
        Self::atan(point.x() as f64, point.y() as f64)
    }

    pub fn atan_i32(center: &IntPoint, point: &IntPoint) -> f64 {
        let delta = IntPoint::new(point.x() - center.x(), point.y() - center.y());
        Self::atan(delta.x() as f64, delta.y() as f64)
    }

    /// Angle between center and point — mirrors .NET Atan(IntPoint, IntPoint).
    pub fn from_to(center: &IntPoint, point: &IntPoint) -> f64 {
        Self::atan_i32(center, point)
    }

    pub fn to_orientation(angle: f64) -> f64 {
        if angle < std::f64::consts::PI {
            2.0 * angle
        } else {
            2.0 * (angle - std::f64::consts::PI)
        }
    }

    pub fn from_orientation(angle: f64) -> f64 {
        0.5 * angle
    }

    pub fn add(start: f64, delta: f64) -> f64 {
        let angle = start + delta;
        if angle < Self::PI2 {
            angle
        } else {
            angle - Self::PI2
        }
    }

    pub fn bucket_center(bucket: i32, resolution: i32) -> f64 {
        Self::PI2 * (2.0 * bucket as f64 + 1.0) / (2.0 * resolution as f64)
    }

    pub fn quantize(angle: f64, resolution: i32) -> i32 {
        let result = (angle * Self::INV_PI2 * resolution as f64) as i32;
        if result < 0 {
            0
        } else if result >= resolution {
            resolution - 1
        } else {
            result
        }
    }

    pub fn opposite(angle: f64) -> f64 {
        if angle < std::f64::consts::PI {
            angle + std::f64::consts::PI
        } else {
            angle - std::f64::consts::PI
        }
    }

    pub fn distance(first: f64, second: f64) -> f64 {
        let delta = (first - second).abs();
        if delta <= std::f64::consts::PI {
            delta
        } else {
            Self::PI2 - delta
        }
    }

    pub fn difference(first: f64, second: f64) -> f64 {
        let angle = first - second;
        if angle >= 0.0 {
            angle
        } else {
            angle + Self::PI2
        }
    }

    pub fn complementary(angle: f64) -> f64 {
        let complement = Self::PI2 - angle;
        if complement < Self::PI2 {
            complement
        } else {
            complement - Self::PI2
        }
    }

    pub fn normalized(angle: f64) -> bool {
        angle >= 0.0 && angle < Self::PI2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pi_constants() {
        assert!((DoubleAngle::PI2 - 2.0 * std::f64::consts::PI).abs() < 1e-10);
        assert!((DoubleAngle::INV_PI2 * DoubleAngle::PI2 - 1.0).abs() < 1e-10);
        assert!((DoubleAngle::HALF_PI - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
    }

    #[test]
    fn test_atan() {
        assert!((DoubleAngle::atan(1.0, 0.0) - 0.0).abs() < 1e-10);
        assert!((DoubleAngle::atan(0.0, 1.0) - std::f64::consts::FRAC_PI_2).abs() < 1e-10);
        assert!((DoubleAngle::atan(1.0, 1.0) - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
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
    fn test_atan_i32_int_point() {
        let pt = IntPoint::new(100, 0);
        let result = DoubleAngle::atan_i32_int_point(&pt);
        assert!(result >= 0.0);
        assert!(result < std::f64::consts::PI * 2.0);
    }

    #[test]
    fn test_atan_point() {
        let dp = DoublePoint::new(1.0, 0.0);
        let result = DoubleAngle::atan_point(&dp);
        assert!(result >= 0.0);
        assert!(result < std::f64::consts::PI * 2.0);
    }

    #[test]
    fn test_to_vector() {
        let v = DoubleAngle::to_vector(0.0);
        assert!((v.x() - 1.0).abs() < 1e-10);
        assert!((v.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_opposite() {
        assert!((DoubleAngle::opposite(0.0) - std::f64::consts::PI).abs() < 1e-10);
        assert!((DoubleAngle::opposite(std::f64::consts::PI) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_distance() {
        assert!(DoubleAngle::distance(0.0, 0.0).abs() < 1e-10);
        assert!(
            (DoubleAngle::distance(0.0, std::f64::consts::PI) - std::f64::consts::PI).abs() < 1e-10
        );
    }

    #[test]
    fn test_normalized() {
        assert!(DoubleAngle::normalized(0.0));
        assert!(DoubleAngle::normalized(1.0));
        assert!(!DoubleAngle::normalized(-0.1));
        assert!(!DoubleAngle::normalized(DoubleAngle::PI2));
    }

    #[test]
    fn test_quantize() {
        assert_eq!(DoubleAngle::quantize(0.0, 256), 0);
        assert_eq!(DoubleAngle::quantize(DoubleAngle::PI2 * 0.9, 256), 230);
    }

    #[test]
    fn test_to_orientation() {
        assert!((DoubleAngle::to_orientation(0.0) - 0.0).abs() < 1e-10);
        assert!(
            (DoubleAngle::to_orientation(std::f64::consts::PI / 2.0) - std::f64::consts::PI).abs()
                < 1e-10
        );
    }
}
