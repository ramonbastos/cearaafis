/// BooleanMatrix: row-major 2D boolean array — mirrors .NET BooleanMatrix.cs.
use crate::primitives::int_point::IntPoint;

#[derive(Clone)]
pub struct BooleanMatrix {
    pub width: usize,
    pub height: usize,
    cells: Vec<bool>,
}

impl BooleanMatrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![false; width * height],
        }
    }

    pub fn from_point(size: &IntPoint) -> Self {
        Self::new(size.x() as usize, size.y() as usize)
    }

    pub fn from_clone(other: &BooleanMatrix) -> Self {
        Self {
            width: other.width,
            height: other.height,
            cells: other.cells.clone(),
        }
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

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[y * self.width + x]
    }

    pub fn get_int(&self, at: &IntPoint) -> bool {
        self.get(at.x() as usize, at.y() as usize)
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.cells[y * self.width + x] = value;
    }

    pub fn set_int(&mut self, at: &IntPoint, value: bool) {
        self.set(at.x() as usize, at.y() as usize, value);
    }

    pub fn get_with_fallback(&self, x: i32, y: i32, fallback: bool) -> bool {
        let x = x as usize;
        let y = y as usize;
        if x >= self.width || y >= self.height {
            return fallback;
        }
        self.cells[y * self.width + x]
    }

    pub fn invert(&mut self) {
        for cell in &mut self.cells {
            *cell = !*cell;
        }
    }

    pub fn merge(&mut self, other: &BooleanMatrix) {
        assert_eq!(self.width, other.width);
        assert_eq!(self.height, other.height);
        for i in 0..self.cells.len() {
            self.cells[i] = self.cells[i] || other.cells[i];
        }
    }
}

impl Default for BooleanMatrix {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = BooleanMatrix::new(3, 4);
        assert_eq!(m.width(), 3);
        assert_eq!(m.height(), 4);
    }

    #[test]
    fn test_set_get() {
        let mut m = BooleanMatrix::new(3, 3);
        m.set(1, 2, true);
        assert!(m.get(1, 2));
        assert!(!m.get(0, 0));
    }

    #[test]
    fn test_invert() {
        let mut m = BooleanMatrix::new(2, 2);
        m.set(0, 0, true);
        m.set(1, 0, false);
        m.invert();
        assert!(!m.get(0, 0));
        assert!(m.get(1, 0));
    }

    #[test]
    fn test_merge() {
        let mut m1 = BooleanMatrix::new(2, 2);
        m1.set(0, 0, true);
        let mut m2 = BooleanMatrix::new(2, 2);
        m2.set(1, 0, true);
        m1.merge(&m2);
        assert!(m1.get(0, 0));
        assert!(m1.get(1, 0));
    }
}
