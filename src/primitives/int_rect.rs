/// IntRect: axis-aligned rectangle — mirrors .NET IntRect.cs.
use crate::primitives::int_point::IntPoint;

pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl IntRect {
    pub const MEMORY: usize = 4 * std::mem::size_of::<i32>();

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_size(size: &IntPoint) -> Self {
        Self {
            x: 0,
            y: 0,
            width: size.x(),
            height: size.y(),
        }
    }

    pub fn left(&self) -> i32 {
        self.x
    }

    pub fn top(&self) -> i32 {
        self.y
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }

    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    pub fn center(&self) -> IntPoint {
        IntPoint::new(
            (self.left() + self.right()) / 2,
            (self.top() + self.bottom()) / 2,
        )
    }

    pub fn between(start: &IntPoint, end: &IntPoint) -> Self {
        Self::new(
            start.x(),
            start.y(),
            end.x() - start.x(),
            end.y() - start.y(),
        )
    }

    pub fn around(x: i32, y: i32, radius: i32) -> Self {
        Self::between(
            &IntPoint::new(x - radius, y - radius),
            &IntPoint::new(x + radius + 1, y + radius + 1),
        )
    }

    pub fn intersect(&self, other: &IntRect) -> Self {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        let width = right - left;
        let height = bottom - top;
        Self::new(left, top, width, height)
    }

    pub fn move_rect(&self, delta: &IntPoint) -> Self {
        Self::new(
            self.x + delta.x(),
            self.y + delta.y(),
            self.width,
            self.height,
        )
    }

    pub fn iterate(&self) -> Vec<IntPoint> {
        let mut points = Vec::new();
        for y in self.top()..self.bottom() {
            for x in self.left()..self.right() {
                points.push(IntPoint::new(x, y));
            }
        }
        points
    }
}

impl PartialEq for IntRect {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.width == other.width
            && self.height == other.height
    }
}

impl Eq for IntRect {}

impl Default for IntRect {
    fn default() -> Self {
        Self::new(0, 0, 0, 0)
    }
}

impl Clone for IntRect {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for IntRect {}

impl std::fmt::Debug for IntRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IntRect({}x{} @ [{}, {}])",
            self.width, self.height, self.x, self.y
        )
    }
}

impl std::hash::Hash for IntRect {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let h = ((self.x as i64 * 31 + self.y as i64) * 31 + self.width as i64) * 31
            + self.height as i64;
        h.hash(state);
    }
}

impl std::fmt::Display for IntRect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}x{} @ [{}, {}]",
            self.width, self.height, self.x, self.y
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = IntRect::new(10, 20, 5, 8);
        assert_eq!(r.left(), 10);
        assert_eq!(r.top(), 20);
        assert_eq!(r.right(), 15);
        assert_eq!(r.bottom(), 28);
        assert_eq!(r.area(), 40);
    }

    #[test]
    fn test_from_size() {
        let size = IntPoint::new(5, 10);
        let r = IntRect::from_size(&size);
        assert_eq!(r.x(), 0);
        assert_eq!(r.y(), 0);
        assert_eq!(r.width(), 5);
        assert_eq!(r.height(), 10);
    }

    #[test]
    fn test_center() {
        let r = IntRect::new(0, 0, 10, 10);
        assert_eq!(r.center(), IntPoint::new(5, 5));
    }

    #[test]
    fn test_between() {
        let r = IntRect::between(&IntPoint::new(0, 0), &IntPoint::new(10, 10));
        assert_eq!(r.width(), 10);
        assert_eq!(r.height(), 10);
    }

    #[test]
    fn test_around() {
        let r = IntRect::around(5, 5, 2);
        assert_eq!(r.left(), 3);
        assert_eq!(r.top(), 3);
        assert_eq!(r.right(), 8);
        assert_eq!(r.bottom(), 8);
    }

    #[test]
    fn test_intersect() {
        let r1 = IntRect::new(0, 0, 10, 10);
        let r2 = IntRect::new(5, 5, 10, 10);
        let intersected = r1.intersect(&r2);
        assert_eq!(intersected.left(), 5);
        assert_eq!(intersected.top(), 5);
        assert_eq!(intersected.width(), 5);
        assert_eq!(intersected.height(), 5);
    }

    #[test]
    fn test_move() {
        let r = IntRect::new(0, 0, 10, 10);
        let moved = r.move_rect(&IntPoint::new(5, 5));
        assert_eq!(moved.left(), 5);
        assert_eq!(moved.top(), 5);
    }

    #[test]
    fn test_iterate() {
        let r = IntRect::new(0, 0, 2, 2);
        let points = r.iterate();
        assert_eq!(points.len(), 4);
        assert!(points.contains(&IntPoint::new(0, 0)));
        assert!(points.contains(&IntPoint::new(1, 0)));
        assert!(points.contains(&IntPoint::new(0, 1)));
        assert!(points.contains(&IntPoint::new(1, 1)));
    }

    #[test]
    fn test_eq() {
        assert_eq!(IntRect::new(0, 0, 10, 10), IntRect::new(0, 0, 10, 10));
        assert_ne!(IntRect::new(0, 0, 10, 10), IntRect::new(1, 0, 10, 10));
    }
}
