/// DoublePointMatrix: row-major 2D array of (x, y) double pairs — mirrors .NET DoublePointMatrix.cs.
use crate::primitives::{double_point::DoublePoint, int_point::IntPoint};

pub struct DoublePointMatrix {
    pub width: usize,
    pub height: usize,
    vectors: Vec<f64>,
}

impl DoublePointMatrix {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            vectors: vec![0.0; 2 * width * height],
        }
    }

    pub fn from_point(size: &IntPoint) -> Self {
        Self::new(size.x() as usize, size.y() as usize)
    }

    pub fn size(&self) -> IntPoint {
        IntPoint::new(self.width as i32, self.height as i32)
    }

    pub fn get(&self, x: usize, y: usize) -> DoublePoint {
        let i = y * self.width + x;
        DoublePoint::new(self.vectors[2 * i], self.vectors[2 * i + 1])
    }

    pub fn get_int(&self, at: &IntPoint) -> DoublePoint {
        self.get(at.x() as usize, at.y() as usize)
    }

    pub fn set(&mut self, x: usize, y: usize, value: DoublePoint) {
        let i = y * self.width + x;
        self.vectors[2 * i] = value.x();
        self.vectors[2 * i + 1] = value.y();
    }

    pub fn set_int(&mut self, at: &IntPoint, value: &DoublePoint) {
        self.set(at.x() as usize, at.y() as usize, *value);
    }

    pub fn set_coords(&mut self, x: usize, y: usize, px: f64, py: f64) {
        let i = y * self.width + x;
        self.vectors[2 * i] = px;
        self.vectors[2 * i + 1] = py;
    }

    pub fn add(&mut self, x: usize, y: usize, px: f64, py: f64) {
        let i = y * self.width + x;
        self.vectors[2 * i] += px;
        self.vectors[2 * i + 1] += py;
    }

    pub fn add_point(&mut self, x: usize, y: usize, point: &DoublePoint) {
        self.add(x, y, point.x(), point.y());
    }

    pub fn add_int(&mut self, at: &IntPoint, point: &DoublePoint) {
        self.add_point(at.x() as usize, at.y() as usize, point);
    }
}

impl Default for DoublePointMatrix {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = DoublePointMatrix::new(3, 4);
        assert_eq!(m.width, 3);
        assert_eq!(m.height, 4);
    }

    #[test]
    fn test_set_get() {
        let mut m = DoublePointMatrix::new(3, 3);
        m.set(1, 2, DoublePoint::new(5.0, 10.0));
        let v = m.get(1, 2);
        assert!((v.x() - 5.0).abs() < 1e-10);
        assert!((v.y() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_set_coords() {
        let mut m = DoublePointMatrix::new(3, 3);
        m.set_coords(1, 2, 3.0, 7.0);
        let v = m.get(1, 2);
        assert!((v.x() - 3.0).abs() < 1e-10);
        assert!((v.y() - 7.0).abs() < 1e-10);
    }
}
