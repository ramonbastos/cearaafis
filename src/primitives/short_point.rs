/// ShortPoint: 2D point with signed short coordinates — mirrors .NET ShortPoint.cs.
use crate::primitives::int_point::IntPoint;
use crate::primitives::integers::Integers;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct ShortPoint {
    pub x: i16,
    pub y: i16,
}

impl ShortPoint {
    pub const MEMORY: usize = 2 * std::mem::size_of::<i16>();

    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn from_i32(x: i32, y: i32) -> Self {
        Self {
            x: x as i16,
            y: y as i16,
        }
    }

    pub fn length_sq(&self) -> i32 {
        Integers::sq(self.x as i32) + Integers::sq(self.y as i32)
    }

    pub fn to_int(&self) -> IntPoint {
        IntPoint::new(self.x as i32, self.y as i32)
    }
}

impl std::hash::Hash for ShortPoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = 31_i32 * self.x as i32 + self.y as i32;
        hash.hash(state);
    }
}

impl std::fmt::Display for ShortPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pt = ShortPoint::new(10, 20);
        assert_eq!(pt.x, 10);
        assert_eq!(pt.y, 20);
    }

    #[test]
    fn test_from_i32() {
        let pt = ShortPoint::from_i32(100, 200);
        assert_eq!(pt.x, 100);
        assert_eq!(pt.y, 200);
    }

    #[test]
    fn test_length_sq() {
        let pt = ShortPoint::new(3, 4);
        assert_eq!(pt.length_sq(), 25); // 9 + 16
    }

    #[test]
    fn test_to_int() {
        let sp = ShortPoint::new(100, 200);
        let ip = sp.to_int();
        assert_eq!(ip.x(), 100);
        assert_eq!(ip.y(), 200);
    }

    #[test]
    fn test_default() {
        let pt = ShortPoint::default();
        assert_eq!(pt.x, 0);
        assert_eq!(pt.y, 0);
    }

    #[test]
    fn test_order() {
        let a = ShortPoint::new(1, 2);
        let b = ShortPoint::new(1, 3);
        assert!(a < b);
        let c = ShortPoint::new(2, 2);
        assert!(a < c);
    }
}
