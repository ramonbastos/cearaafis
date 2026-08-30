/// DoublePoint: 2D point with double coordinates — mirrors .NET DoublePoint.cs.
use crate::primitives::{
    doubles::Doubles,
    int_point::IntPoint,
};

pub struct DoublePoint {
    pub x: f64,
    pub y: f64,
}

impl DoublePoint {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn from_int_point(p: &IntPoint) -> Self {
        Self::new(p.x() as f64, p.y() as f64)
    }

    pub fn round(&self) -> IntPoint {
        IntPoint::new(
            Doubles::round_to_int(self.x) as i32,
            Doubles::round_to_int(self.y) as i32,
        )
    }

    pub fn to_vector(&self) -> DoublePoint {
        let len = self.length_sq();
        if len == 0.0 {
            return DoublePoint::ZERO;
        }
        let sqrt = len.sqrt();
        DoublePoint::new(self.x / sqrt, self.y / sqrt)
    }

    pub fn normalize(&self) -> DoublePoint {
        self.to_vector()
    }

    pub fn length_sq(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

impl Default for DoublePoint {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Clone for DoublePoint {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for DoublePoint {}

impl std::fmt::Debug for DoublePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DoublePoint({}, {})", self.x, self.y)
    }
}

impl std::ops::Add for DoublePoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for DoublePoint {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Neg for DoublePoint {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl std::ops::Mul<f64> for DoublePoint {
    type Output = Self;
    fn mul(self, factor: f64) -> Self {
        Self::new(factor * self.x, factor * self.y)
    }
}

impl PartialEq for DoublePoint {
    fn eq(&self, other: &Self) -> bool {
        (self.x - other.x).abs() < 1e-10 && (self.y - other.y).abs() < 1e-10
    }
}

impl Eq for DoublePoint {}

impl std::fmt::Display for DoublePoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = DoublePoint::new(1.5, 2.5);
        assert!((p.x() - 1.5).abs() < 1e-10);
        assert!((p.y() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_zero() {
        assert!((DoublePoint::ZERO.x() - 0.0).abs() < 1e-10);
        assert!((DoublePoint::ZERO.y() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_add() {
        assert_eq!(DoublePoint::new(1.0, 2.0) + DoublePoint::new(3.0, 4.0), DoublePoint::new(4.0, 6.0));
    }

    #[test]
    fn test_sub() {
        assert_eq!(DoublePoint::new(5.0, 3.0) - DoublePoint::new(2.0, 1.0), DoublePoint::new(3.0, 2.0));
    }

    #[test]
    fn test_neg() {
        assert_eq!(-DoublePoint::new(1.0, -2.0), DoublePoint::new(-1.0, 2.0));
    }

    #[test]
    fn test_mul() {
        assert_eq!(DoublePoint::new(1.0, 2.0) * 3.0, DoublePoint::new(3.0, 6.0));
    }

    #[test]
    fn test_round() {
        let p = DoublePoint::new(1.6, 2.4);
        let rounded = p.round();
        assert_eq!(rounded.x(), 2);
        assert_eq!(rounded.y(), 2);
    }

    #[test]
    fn test_from_int_point() {
        let ip = IntPoint::new(3, 5);
        let dp = DoublePoint::from_int_point(&ip);
        assert!((dp.x() - 3.0).abs() < 1e-10);
        assert!((dp.y() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_eq() {
        assert_eq!(DoublePoint::new(1.0, 2.0), DoublePoint::new(1.0, 2.0));
        assert_ne!(DoublePoint::new(1.0, 2.0), DoublePoint::new(2.0, 1.0));
    }
}
