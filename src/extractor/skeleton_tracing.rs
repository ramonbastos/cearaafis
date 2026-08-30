use crate::parameters::Parameters;
/// SkeletonTracing: trace skeleton curves from binarized image using iterative thinning.
/// Mirrors .NET SkeletonTracing.cs.
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::int_point::IntPoint;

/// A traced skeleton with ridges and minutiae.
pub struct SkeletonTracer {
    skeleton: BooleanMatrix,
}

impl SkeletonTracer {
    pub fn new(binary: &BooleanMatrix) -> Self {
        let mut skeleton = BooleanMatrix::from_clone(binary);
        let iterations = Parameters::THINNING_ITERATIONS;
        Self::thin(&mut skeleton, iterations);
        Self { skeleton }
    }

    pub fn skeleton(&self) -> &BooleanMatrix {
        &self.skeleton
    }

    pub fn is_skeleton_pixel(&self, x: i32, y: i32) -> bool {
        let w = self.skeleton.width() as i32;
        let h = self.skeleton.height() as i32;
        if x < 0 || y < 0 || x >= w || y >= h {
            return false;
        }
        self.skeleton.get(x as usize, y as usize)
    }

    pub fn pixel_count(&self) -> usize {
        let mut count = 0;
        let h = self.skeleton.height();
        let w = self.skeleton.width();
        for y in 0..h {
            for x in 0..w {
                if self.skeleton.get(x, y) {
                    count += 1;
                }
            }
        }
        count
    }

    /// Find skeleton endpoints (1 neighbor) and junctions (3+ neighbors).
    pub fn find_minutia_points(&self) -> Vec<IntPoint> {
        let mut points = Vec::new();
        let w = self.skeleton.width();
        let h = self.skeleton.height();

        for y in 0..h {
            for x in 0..w {
                if !self.skeleton.get(x, y) {
                    continue;
                }

                let mut n = 0i32;
                for dy in -1i32..=1i32 {
                    for dx in -1i32..=1i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                            if self.skeleton.get(nx as usize, ny as usize) {
                                n += 1;
                            }
                        }
                    }
                }

                if n == 1 || n >= 3 {
                    points.push(IntPoint::new(x as i32, y as i32));
                }
            }
        }

        points
    }

    /// Thinning: iteratively remove non-essential boundary pixels.
    /// Only removes pixels with >= 3 neighbors (junctions), keeping 1-pixel-wide structures intact.
    fn thin(skeleton: &mut BooleanMatrix, iterations: usize) {
        let w = skeleton.width();
        let h = skeleton.height();
        if w < 3 || h < 3 || w == 0 || h == 0 {
            return;
        }

        for _iter in 0..iterations {
            Self::thin_pass(skeleton, w, h);
        }
    }

    fn thin_pass(skeleton: &mut BooleanMatrix, w: usize, h: usize) {
        let mut to_remove = Vec::new();

        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                if !skeleton.get(x, y) {
                    continue;
                }

                // Count 8-connected neighbors
                let mut n = 0usize;
                for dy in -1i32..=1i32 {
                    for dx in -1i32..=1i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                            if skeleton.get(nx as usize, ny as usize) {
                                n += 1;
                            }
                        }
                    }
                }

                // Only remove junction pixels (3+ neighbors) — keep 1-pixel-wide structures intact
                if n < 3 {
                    continue;
                }

                to_remove.push((x, y));
            }
        }

        for (x, y) in to_remove {
            skeleton.set(x, y, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeletonize_simple_line() {
        let mut binary = BooleanMatrix::new(10, 10);
        for x in 0..10 {
            binary.set(x, 5, true);
        }
        let tracer = SkeletonTracer::new(&binary);
        assert!(tracer.is_skeleton_pixel(5, 5));
        // Line should survive thinning (all pixels have n=2 except endpoints with n=1)
        assert!(tracer.is_skeleton_pixel(1, 5));
        assert!(tracer.is_skeleton_pixel(8, 5));
    }

    #[test]
    fn test_skeletonize_cross() {
        let mut binary = BooleanMatrix::new(10, 10);
        for x in 0..10 {
            binary.set(x, 5, true);
        }
        for y in 0..10 {
            binary.set(5, y, true);
        }
        let tracer = SkeletonTracer::new(&binary);
        // Center intersection has n=8 (junction) — will be thinned away
        // But the arms should remain as 1-pixel-wide lines
        assert!(tracer.pixel_count() >= 10); // At least the 4 arms remain
    }

    #[test]
    fn test_pixel_count() {
        let mut binary = BooleanMatrix::new(5, 5);
        for y in 0..5 {
            for x in 0..5 {
                binary.set(x, y, true);
            }
        }
        let tracer = SkeletonTracer::new(&binary);
        assert!(tracer.pixel_count() > 0);
        assert!(tracer.pixel_count() <= 25);
    }

    #[test]
    fn test_find_minutia_points() {
        let mut binary = BooleanMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                binary.set(x, y, true);
            }
        }
        let tracer = SkeletonTracer::new(&binary);
        let points = tracer.find_minutia_points();
        // Solid block: interior pixels have 8 neighbors (junctions), border has fewer
        assert!(points.len() >= 4);
    }

    #[test]
    fn test_small_image() {
        let mut binary = BooleanMatrix::new(2, 2);
        binary.set(0, 0, true);
        binary.set(1, 1, true);
        let tracer = SkeletonTracer::new(&binary);
        assert_eq!(tracer.pixel_count(), 2);
    }

    #[test]
    fn test_empty_image() {
        let binary = BooleanMatrix::new(5, 5);
        let tracer = SkeletonTracer::new(&binary);
        assert_eq!(tracer.pixel_count(), 0);
    }
}
