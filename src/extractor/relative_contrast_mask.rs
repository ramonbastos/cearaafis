/// RelativeContrastMask: identifies pixels with low relative contrast.
/// Mirrors .NET RelativeContrastMask.cs.
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::histogram_cube::HistogramCube;
use crate::parameters::Parameters;

/// Relative contrast mask for quality filtering.
pub struct RelativeContrastMask {
    mask: BooleanMatrix,
}

impl RelativeContrastMask {
    pub fn from_image_and_histogram(image: &DoubleMatrix, histogram: &HistogramCube) -> Self {
        let w = image.width();
        let h = image.height();
        let mut mask = BooleanMatrix::new(w, h);

        let sample_count = Parameters::RELATIVE_CONTRAST_SAMPLE;
        let percentile = Parameters::RELATIVE_CONTRAST_PERCENTILE;

        let step = if h > 0 && w > 0 {
            ((h.max(w) * 10) / sample_count.max(1)).max(1)
        } else {
            10
        };

        for y in (0..h).step_by(step) {
            for x in (0..w).step_by(step) {
                let total = histogram.sum(x, y);
                if total == 0 {
                    mask.set(x, y, true);
                    continue;
                }

                let median_count = (total as f64 * percentile) as i32;
                let mut cumulative = 0i32;
                let mut median_bin = 0usize;

                for b in 0..histogram.bins {
                    cumulative += histogram.get(x, y, b);
                    if cumulative >= median_count {
                        median_bin = b;
                        break;
                    }
                }

                let median_val = (median_bin as f64 / (histogram.bins as f64 - 1.0)) * 255.0;

                let local_mean = Self::local_mean(image, x, y, 5);
                let diff = (median_val - local_mean).abs();
                let denom = local_mean.max(1.0);
                let rel_contrast = diff / denom;

                if rel_contrast < Parameters::MIN_RELATIVE_CONTRAST {
                    mask.set(x, y, true);
                }
            }
        }

        Self { mask }
    }

    fn local_mean(image: &DoubleMatrix, x: usize, y: usize, radius: usize) -> f64 {
        let w = image.width();
        let h = image.height();
        let mut sum = 0.0;
        let mut count = 0usize;

        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                    sum += image.get(nx as usize, ny as usize);
                    count += 1;
                }
            }
        }

        if count == 0 { 0.0 } else { sum / count as f64 }
    }

    pub fn mask(&self) -> &BooleanMatrix {
        &self.mask
    }

    pub fn is_low_relative_contrast(&self, x: i32, y: i32) -> bool {
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
    fn test_mask_dimensions() {
        let img = DoubleMatrix::new(50, 50);
        let hist = HistogramCube::new(50, 50, 256);
        let mask = RelativeContrastMask::from_image_and_histogram(&img, &hist);
        assert_eq!(mask.mask().width(), 50);
        assert_eq!(mask.mask().height(), 50);
    }

    #[test]
    fn test_out_of_bounds() {
        let img = DoubleMatrix::new(10, 10);
        let hist = HistogramCube::new(10, 10, 256);
        let mask = RelativeContrastMask::from_image_and_histogram(&img, &hist);
        assert!(!mask.is_low_relative_contrast(-1, 5));
        assert!(!mask.is_low_relative_contrast(5, -1));
    }
}
