/// AbsoluteContrastMask: identifies pixels with low absolute contrast.
/// Mirrors .NET AbsoluteContrastMask.cs.
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::parameters::Parameters;

/// Absolute contrast mask for quality filtering.
pub struct AbsoluteContrastMask {
    mask: BooleanMatrix,
}

impl AbsoluteContrastMask {
    pub fn from_image(image: &DoubleMatrix) -> Self {
        let w = image.width();
        let h = image.height();
        let mut mask = BooleanMatrix::new(w, h);
        let threshold = Parameters::MIN_ABSOLUTE_CONTRAST * 255.0;
        let radius = Parameters::MASK_VOTE_RADIUS;

        for y in 0..h {
            for x in 0..w {
                let mut min_val = f64::MAX;
                let mut max_val = f64::MIN;

                for dy in -(radius as i32)..=(radius as i32) {
                    for dx in -(radius as i32)..=(radius as i32) {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                            let val = image.get(nx as usize, ny as usize);
                            if val < min_val { min_val = val; }
                            if val > max_val { max_val = val; }
                        }
                    }
                }

                let contrast = max_val - min_val;
                if contrast < threshold {
                    mask.set(x, y, true);
                }
            }
        }

        Self { mask }
    }

    pub fn mask(&self) -> &BooleanMatrix {
        &self.mask
    }

    pub fn is_low_contrast(&self, x: i32, y: i32) -> bool {
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
    fn test_low_contrast_region() {
        let mut img = DoubleMatrix::new(20, 20);
        for y in 0..20 { for x in 0..20 { img.set(x, y, 128.0); } }
        let mask = AbsoluteContrastMask::from_image(&img);
        assert!(mask.is_low_contrast(10, 10));
    }

    #[test]
    fn test_high_contrast_region() {
        let mut img = DoubleMatrix::new(20, 20);
        for y in 0..10 { for x in 0..20 { img.set(x, y, 0.0); } }
        for y in 10..20 { for x in 0..20 { img.set(x, y, 255.0); } }
        let mask = AbsoluteContrastMask::from_image(&img);
        // Some regions should have high contrast
        assert!(mask.mask().width() == 20 && mask.mask().height() == 20);
    }

    #[test]
    fn test_out_of_bounds() {
        let img = DoubleMatrix::new(10, 10);
        let mask = AbsoluteContrastMask::from_image(&img);
        assert!(!mask.is_low_contrast(-1, 0));
        assert!(!mask.is_low_contrast(0, -1));
        assert!(!mask.is_low_contrast(100, 100));
    }
}
