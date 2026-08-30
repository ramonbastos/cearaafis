//! Unit tests for EdgeShape feature type.

#[cfg(test)]
mod tests {
    use cearaafis::primitives::{DoubleAngle, DoublePoint};

    #[test]
    fn test_double_angle_atan() {
        let dp = DoublePoint::new(1.0, 1.0);
        let angle = DoubleAngle::atan_point(&dp);
        assert!(angle > 0.0);
        assert!(angle < std::f64::consts::PI * 2.0);
    }

    #[test]
    fn test_double_angle_opposite() {
        let angle = DoubleAngle::opposite(0.0);
        assert!((angle - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_double_angle_distance() {
        assert!(DoubleAngle::distance(0.0, 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_double_angle_complementary() {
        let comp = DoubleAngle::complementary(0.0);
        assert!(comp >= 0.0);
    }
}
