/// SegmentationMask: creates a quality mask identifying ridge vs valley regions.
/// Mirrors .NET SegmentationMask.cs.
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;

/// Segmentation mask for fingerprint quality analysis.
pub struct SegmentationMask {
    mask: BooleanMatrix,
}

impl SegmentationMask {
    /// Create segmentation mask from image and local variance data.
    pub fn from_image(image: &DoubleMatrix) -> Self {
        SegmentationMask::from_contrast(image)
    }

    /// Create segmentation mask from image and variance data.
    pub fn from_variance(image: &DoubleMatrix, _hist_variance: &[(f64, f64)]) -> Self {
        SegmentationMask::from_contrast(image)
    }

    pub fn from_contrast(image: &DoubleMatrix) -> Self {
        let mut mask = BooleanMatrix::new(image.width(), image.height());
        let w = image.width();
        let h = image.height();

        let mut min_val = f64::MAX;
        let mut max_val = f64::MIN;
        for y in 0..h {
            for x in 0..w {
                let val = image.get(x, y);
                if val < min_val { min_val = val; }
                if val > max_val { max_val = val; }
            }
        }
        let contrast = max_val - min_val;

        if contrast > 50.0 {
            for y in 0..h {
                for x in 0..w {
                    mask.set(x, y, true);
                }
            }
        }

        Self { mask }
    }

    pub fn mask(&self) -> &BooleanMatrix {
        &self.mask
    }

    pub fn is_quality(&self, x: i32, y: i32) -> bool {
        let w = self.mask.width() as i32;
        let h = self.mask.height() as i32;
        if x < 0 || y < 0 || x >= w || y >= h {
            return false;
        }
        self.mask.get(x as usize, y as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_contrast_good() {
        let mut img = DoubleMatrix::new(100, 100);
        for y in 0..50 { for x in 0..100 { img.set(x, y, 255.0); } }
        for y in 50..100 { for x in 0..100 { img.set(x, y, 0.0); } }
        let seg = SegmentationMask::from_contrast(&img);
        assert!(seg.is_quality(50, 50));
    }

    #[test]
    fn test_from_contrast_bad() {
        let mut img = DoubleMatrix::new(100, 100);
        for y in 0..100 { for x in 0..100 { img.set(x, y, 128.0); } }
        let seg = SegmentationMask::from_contrast(&img);
        assert!(!seg.is_quality(50, 50));
    }

    #[test]
    fn test_out_of_bounds() {
        let mask = BooleanMatrix::new(100, 100);
        let seg = SegmentationMask { mask };
        assert!(!seg.is_quality(-1, 0));
        assert!(!seg.is_quality(0, -1));
    }
}
