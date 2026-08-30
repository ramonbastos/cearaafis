/// IntMatrix: row-major 2D array of integers — mirrors .NET IntMatrix.cs.
use crate::primitives::int_point::IntPoint;

pub struct IntMatrix {
    pub width: usize,
    pub height: usize,
    cells: Vec<i32>,
}

impl IntMatrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![0; width * height],
        }
    }

    pub fn from_point(size: &IntPoint) -> Self {
        Self::new(size.x() as usize, size.y() as usize)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn size(&self) -> IntPoint {
        IntPoint::new(self.width as i32, self.height as i32)
    }

    pub fn get(&self, x: usize, y: usize) -> i32 {
        self.cells[y * self.width + x]
    }

    pub fn get_int(&self, at: &IntPoint) -> i32 {
        self.get(at.x() as usize, at.y() as usize)
    }

    pub fn set(&mut self, x: usize, y: usize, value: i32) {
        self.cells[y * self.width + x] = value;
    }

    pub fn set_int(&mut self, at: &IntPoint, value: i32) {
        self.set(at.x() as usize, at.y() as usize, value);
    }
}

impl Default for IntMatrix {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = IntMatrix::new(3, 4);
        assert_eq!(m.width(), 3);
        assert_eq!(m.height(), 4);
    }

    #[test]
    fn test_set_get() {
        let mut m = IntMatrix::new(3, 3);
        m.set(1, 2, 42);
        assert_eq!(m.get(1, 2), 42);
    }

    #[test]
    fn test_size() {
        let m = IntMatrix::new(5, 10);
        assert_eq!(m.size().x(), 5);
        assert_eq!(m.size().y(), 10);
    }
}
