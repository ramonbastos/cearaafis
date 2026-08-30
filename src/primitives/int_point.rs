/// IntPoint: 2D point with integer coordinates — mirrors .NET IntPoint.cs.
use crate::primitives::{doubles::Doubles, integers::Integers, short_point::ShortPoint};

#[derive(Clone, serde::Serialize)]
pub struct IntPoint {
    pub x: i32,
    pub y: i32,
}

impl IntPoint {
    pub const ZERO: Self = Self { x: 0, y: 0 };

    pub const EDGE_NEIGHBORS: &[Self] = &[
        Self { x: 0, y: -1 },
        Self { x: -1, y: 0 },
        Self { x: 1, y: 0 },
        Self { x: 0, y: 1 },
    ];

    pub const CORNER_NEIGHBORS: &[Self] = &[
        Self { x: -1, y: -1 },
        Self { x: 0, y: -1 },
        Self { x: 1, y: -1 },
        Self { x: -1, y: 0 },
        Self { x: 1, y: 0 },
        Self { x: -1, y: 1 },
        Self { x: 0, y: 1 },
        Self { x: 1, y: 1 },
    ];

    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn area(&self) -> i32 {
        self.x * self.y
    }

    pub fn length_sq(&self) -> i32 {
        Integers::sq(self.x) + Integers::sq(self.y)
    }

    pub fn to_short(&self) -> ShortPoint {
        ShortPoint::new(self.x as i16, self.y as i16)
    }

    pub fn contains(&self, other: &IntPoint) -> bool {
        other.x >= 0 && other.y >= 0 && other.x < self.x && other.y < self.y
    }

    pub fn line_to(&self, to: &IntPoint) -> Vec<IntPoint> {
        let relative = IntPoint::new(to.x - self.x, to.y - self.y);
        if relative.x.abs() >= relative.y.abs() {
            let abs_x = relative.x.abs();
            let mut result = Vec::with_capacity((abs_x + 1) as usize);
            let rel_x_f64 = relative.x as f64;
            let rel_y_f64 = relative.y as f64;
            if relative.x > 0 {
                for i in 0i32..=relative.x {
                    result.push(IntPoint::new(
                        self.x + i,
                        self.y + Doubles::round_to_int(i as f64 * (rel_y_f64 / rel_x_f64)) as i32,
                    ));
                }
            } else if relative.x < 0 {
                let neg_rel_x = -relative.x;
                for i in 0i32..=neg_rel_x {
                    result.push(IntPoint::new(
                        self.x - i,
                        self.y - Doubles::round_to_int(i as f64 * (rel_y_f64 / neg_rel_x as f64)) as i32,
                    ));
                }
            } else {
                result.push(*self);
            }
            result
        } else {
            let abs_y = relative.y.abs();
            let mut result = Vec::with_capacity((abs_y + 1) as usize);
            let rel_x_f64 = relative.x as f64;
            let rel_y_f64 = relative.y as f64;
            if relative.y > 0 {
                for i in 0i32..=relative.y {
                    result.push(IntPoint::new(
                        self.x + Doubles::round_to_int(i as f64 * (rel_x_f64 / rel_y_f64)) as i32,
                        self.y + i,
                    ));
                }
            } else if relative.y < 0 {
                let neg_rel_y = -relative.y;
                for i in 0i32..=neg_rel_y {
                    result.push(IntPoint::new(
                        self.x - Doubles::round_to_int(i as f64 * (rel_x_f64 / neg_rel_y as f64)) as i32,
                        self.y - i,
                    ));
                }
            } else {
                result.push(*self);
            }
            result
        }
    }

    pub fn iterate(&self) -> Vec<IntPoint> {
        let mut points = Vec::new();
        for y in 0..self.y {
            for x in 0..self.x {
                points.push(IntPoint::new(x, y));
            }
        }
        points
    }
}

impl Default for IntPoint {
    fn default() -> Self {
        Self::ZERO
    }
}


impl Copy for IntPoint {}

impl std::fmt::Debug for IntPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IntPoint({}, {})", self.x, self.y)
    }
}

impl std::ops::Add for IntPoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for IntPoint {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Neg for IntPoint {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y)
    }
}

impl PartialEq for IntPoint {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Eq for IntPoint {}

impl PartialOrd for IntPoint {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.y.partial_cmp(&other.y).map(|r| r.then(self.x.cmp(&other.x)))
    }
}

impl Ord for IntPoint {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl std::hash::Hash for IntPoint {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash = (31_i32 * self.x + self.y) as u64;
        hash.hash(state);
    }
}

impl std::fmt::Display for IntPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = IntPoint::new(5, 10);
        assert_eq!(p.x(), 5);
        assert_eq!(p.y(), 10);
    }

    #[test]
    fn test_zero() {
        assert_eq!(IntPoint::ZERO.x(), 0);
        assert_eq!(IntPoint::ZERO.y(), 0);
    }

    #[test]
    fn test_area() {
        assert_eq!(IntPoint::new(3, 4).area(), 12);
        assert_eq!(IntPoint::new(0, 5).area(), 0);
    }

    #[test]
    fn test_length_sq() {
        assert_eq!(IntPoint::new(3, 4).length_sq(), 25);
    }

    #[test]
    fn test_add() {
        assert_eq!(IntPoint::new(1, 2) + IntPoint::new(3, 4), IntPoint::new(4, 6));
    }

    #[test]
    fn test_sub() {
        assert_eq!(IntPoint::new(5, 3) - IntPoint::new(2, 1), IntPoint::new(3, 2));
    }

    #[test]
    fn test_neg() {
        assert_eq!(-IntPoint::new(1, -2), IntPoint::new(-1, 2));
    }

    #[test]
    fn test_eq() {
        assert_eq!(IntPoint::new(1, 2), IntPoint::new(1, 2));
        assert_ne!(IntPoint::new(1, 2), IntPoint::new(2, 1));
    }

    #[test]
    fn test_to_short() {
        let p = IntPoint::new(100, 200);
        let sp = p.to_short();
        assert_eq!(sp.x, 100);
        assert_eq!(sp.y, 200);
    }

    #[test]
    fn test_contains() {
        let p = IntPoint::new(5, 5);
        assert!(p.contains(&IntPoint::new(0, 0)));
        assert!(p.contains(&IntPoint::new(4, 4)));
        assert!(!p.contains(&IntPoint::new(5, 0)));
        assert!(!p.contains(&IntPoint::new(0, 5)));
    }

    #[test]
    fn test_line_to() {
        let from = IntPoint::new(0, 0);
        let to = IntPoint::new(4, 4);
        let line = from.line_to(&to);
        assert_eq!(line.len(), 5);
        assert_eq!(line[0], IntPoint::new(0, 0));
        assert_eq!(line[4], IntPoint::new(4, 4));
    }

    #[test]
    fn test_iterate() {
        let p = IntPoint::new(2, 2);
        let points = p.iterate();
        assert_eq!(points.len(), 4);
        assert!(points.contains(&IntPoint::new(0, 0)));
        assert!(points.contains(&IntPoint::new(1, 0)));
        assert!(points.contains(&IntPoint::new(0, 1)));
        assert!(points.contains(&IntPoint::new(1, 1)));
    }

    #[test]
    fn test_edge_neighbors() {
        assert_eq!(IntPoint::EDGE_NEIGHBORS.len(), 4);
    }

    #[test]
    fn test_corner_neighbors() {
        assert_eq!(IntPoint::CORNER_NEIGHBORS.len(), 8);
    }
}
