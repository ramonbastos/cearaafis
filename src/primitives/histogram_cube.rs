/// HistogramCube: 3D histogram — mirrors .NET HistogramCube.cs.
use crate::primitives::int_point::IntPoint;

pub struct HistogramCube {
    pub width: usize,
    pub height: usize,
    pub bins: usize,
    counts: Vec<i32>,
}

impl HistogramCube {
    pub fn new(width: usize, height: usize, bins: usize) -> Self {
        Self {
            width,
            height,
            bins,
            counts: vec![0; width * height * bins],
        }
    }

    pub fn from_point(size: &IntPoint, bins: usize) -> Self {
        Self::new(size.x() as usize, size.y() as usize, bins)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> i32 {
        self.counts[(y * self.width + x) * self.bins + z]
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, value: i32) {
        self.counts[(y * self.width + x) * self.bins + z] = value;
    }

    /// Constrain z into [0, bins-1].
    pub fn constrain_z(&self, z: i32) -> usize {
        let mut z = z;
        if z < 0 {
            z = 0;
        }
        if z >= self.bins as i32 {
            z = self.bins as i32 - 1;
        }
        z as usize
    }

    /// Alias for constrain_z
    pub fn constrain(&self, z: i32) -> usize {
        self.constrain_z(z)
    }

    pub fn sum(&self, x: usize, y: usize) -> i32 {
        let mut s = 0;
        for i in 0..self.bins {
            s += self.get(x, y, i);
        }
        s
    }

    pub fn add(&mut self, x: usize, y: usize, z: usize, value: i32) {
        self.counts[(y * self.width + x) * self.bins + z] += value;
    }

    pub fn increment(&mut self, x: usize, y: usize, z: usize) {
        self.add(x, y, z, 1);
    }
}

impl Default for HistogramCube {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let c = HistogramCube::new(3, 4, 10);
        assert_eq!(c.width, 3);
        assert_eq!(c.height, 4);
        assert_eq!(c.bins, 10);
    }

    #[test]
    fn test_set_get() {
        let mut c = HistogramCube::new(3, 4, 10);
        c.set(1, 2, 5, 42);
        assert_eq!(c.get(1, 2, 5), 42);
    }

    #[test]
    fn test_add() {
        let mut c = HistogramCube::new(3, 4, 10);
        c.add(1, 2, 5, 10);
        c.add(1, 2, 5, 5);
        assert_eq!(c.get(1, 2, 5), 15);
    }

    #[test]
    fn test_increment() {
        let mut c = HistogramCube::new(3, 4, 10);
        c.increment(1, 2, 5);
        assert_eq!(c.get(1, 2, 5), 1);
    }

    #[test]
    fn test_constrain_z() {
        let c = HistogramCube::new(3, 4, 10);
        assert_eq!(c.constrain_z(-5), 0);
        assert_eq!(c.constrain_z(0), 0);
        assert_eq!(c.constrain_z(5), 5);
        assert_eq!(c.constrain_z(100), 9);
    }

    #[test]
    fn test_constrain_alias() {
        let c = HistogramCube::new(3, 4, 10);
        assert_eq!(c.constrain(-5), 0);
        assert_eq!(c.constrain(100), 9);
    }
}
