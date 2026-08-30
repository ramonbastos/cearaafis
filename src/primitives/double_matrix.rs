/// DoubleMatrix: row-major 2D array of doubles — mirrors .NET DoubleMatrix.cs.
use crate::primitives::int_point::IntPoint;

#[derive(Debug, Clone, Default)]
pub struct DoubleMatrix {
    pub width: usize,
    pub height: usize,
    cells: Vec<f64>,
}

impl DoubleMatrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![0.0; width * height],
        }
    }

    pub fn from_point(size: &IntPoint) -> Self {
        Self::new(size.x() as usize, size.y() as usize)
    }

    pub fn size(&self) -> IntPoint {
        IntPoint::new(self.width as i32, self.height as i32)
    }

    pub fn get(&self, x: usize, y: usize) -> f64 {
        self.cells[y * self.width + x]
    }

    pub fn get_int(&self, at: &IntPoint) -> f64 {
        self.get(at.x() as usize, at.y() as usize)
    }

    pub fn set(&mut self, x: usize, y: usize, value: f64) {
        self.cells[y * self.width + x] = value;
    }

    pub fn set_int(&mut self, at: &IntPoint, value: f64) {
        self.set(at.x() as usize, at.y() as usize, value);
    }

    pub fn add(&mut self, x: usize, y: usize, value: f64) {
        self.cells[y * self.width + x] += value;
    }

    pub fn add_int(&mut self, at: &IntPoint, value: f64) {
        self.add(at.x() as usize, at.y() as usize, value);
    }

    pub fn multiply(&mut self, x: usize, y: usize, value: f64) {
        self.cells[y * self.width + x] *= value;
    }

    pub fn multiply_int(&mut self, at: &IntPoint, value: f64) {
        self.multiply(at.x() as usize, at.y() as usize, value);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Indexer: m[x][y]
    pub fn get_at(&self, x: usize, y: usize) -> f64 {
        self.get(x, y)
    }

    pub fn set_at(&mut self, x: usize, y: usize, value: f64) {
        self.set(x, y, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = DoubleMatrix::new(3, 4);
        assert_eq!(m.width, 3);
        assert_eq!(m.height, 4);
    }

    #[test]
    fn test_set_get() {
        let mut m = DoubleMatrix::new(3, 3);
        m.set(1, 2, 42.0);
        assert!((m.get(1, 2) - 42.0).abs() < 1e-10);
    }

    #[test]
    fn test_size() {
        let m = DoubleMatrix::new(5, 10);
        assert_eq!(m.size().x(), 5);
        assert_eq!(m.size().y(), 10);
    }
}
