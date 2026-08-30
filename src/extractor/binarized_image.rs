/// BinarizedImage: threshold the image into a binary skeleton image.
/// Mirrors .NET BinarizedImage.cs.
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::histogram_cube::HistogramCube;
use crate::parameters::Parameters;

/// Binary image result from thresholding.
pub struct BinarizedImage {
    image: BooleanMatrix,
    /// Mask: pixels where histogram total > 0 (valid pixels)
    mask: BooleanMatrix,
}

impl BinarizedImage {
    pub fn from_image(image: &DoubleMatrix, histogram: &HistogramCube) -> Self {
        let w = image.width();
        let h = image.height();

        // Build binary image
        let mut binary = BooleanMatrix::new(w, h);
        // Build mask: pixels where histogram total > 0
        let mut mask = BooleanMatrix::new(w, h);

        let threshold_percentile = Parameters::RELATIVE_CONTRAST_PERCENTILE;

        let mut ridge_count = 0usize;
        let mut valley_count = 0usize;

        for y in 0..h {
            for x in 0..w {
                let total = histogram.sum(x, y);
                let val = image.get(x, y);

                // Mask: pixel is valid if histogram total > 0
                if total > 0 {
                    mask.set(x, y, true);
                }

                if total == 0 {
                    binary.set(x, y, val > 128.0);
                    continue;
                }

                let threshold_count = (total as f64 * threshold_percentile) as i32;
                let mut cumulative = 0i32;
                let mut threshold_bin = 0usize;

                for b in 0..histogram.bins {
                    cumulative += histogram.get(x, y, b);
                    if cumulative >= threshold_count {
                        threshold_bin = b;
                        break;
                    }
                }

                let threshold_val = (threshold_bin as f64 / (histogram.bins as f64 - 1.0)) * 255.0;
                binary.set(x, y, val < threshold_val);

                if binary.get(x, y) {
                    ridge_count += 1;
                } else {
                    valley_count += 1;
                }
            }
        }

        // Island cleanup: remove pixels in binary that are NOT in the mask.
        // Mirrors .NET BinarizedImage.Binarize() island detection:
        // "pixels that are binary=true but mask=false are islands — remove them"
        for y in 0..h {
            for x in 0..w {
                if binary.get(x, y) && !mask.get(x, y) {
                    binary.set(x, y, false);
                }
            }
        }

        Self { image: binary, mask }
    }

    pub fn image(&self) -> &BooleanMatrix {
        &self.image
    }

    pub fn mask(&self) -> &BooleanMatrix {
        &self.mask
    }

    pub fn is_ridge(&self, x: i32, y: i32) -> bool {
        let w = self.image.width() as i32;
        let h = self.image.height() as i32;
        if x < 0 || y < 0 || x >= w || y >= h {
            return false;
        }
        self.image.get(x as usize, y as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binarize_uniform() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, 128.0);
            }
        }
        let hist = HistogramCube::new(10, 10, 256);
        let bin = BinarizedImage::from_image(&img, &hist);
        assert!(!bin.is_ridge(5, 5));
    }

    #[test]
    fn test_binarize_dimensions() {
        let img = DoubleMatrix::new(30, 30);
        let hist = HistogramCube::new(30, 30, 256);
        let bin = BinarizedImage::from_image(&img, &hist);
        assert_eq!(bin.image().width(), 30);
        assert_eq!(bin.image().height(), 30);
    }

    #[test]
    fn test_binarize_island_cleanup() {
        // Create an image with a single bright pixel surrounded by dark pixels
        let mut img = DoubleMatrix::new(5, 5);
        img.set(2, 2, 255.0); // center pixel bright
        for y in 0..5 {
            for x in 0..5 {
                if x == 2 && y == 2 { continue; }
                img.set(x, y, 0.0);
            }
        }

        // Build histogram with non-zero total for center pixel
        let mut hist = HistogramCube::new(5, 5, 256);
        // Set only the center pixel to have non-zero histogram total
        // (this makes mask=true only for (2,2))
        // But in reality the histogram comes from local contrast computation,
        // so all pixels will have some total. Let's test with realistic scenario.

        // For the island cleanup test: we need pixels where binary=true but mask=false.
        // This happens when total=0 → mask=false, and val>128 → binary=true.
        // Since our histogram cube initializes to zeros, ALL pixels have total=0.
        // So mask=false everywhere, and binary=true wherever val>128.
        // Island cleanup should remove all those.
        let bin = BinarizedImage::from_image(&img, &hist);
        for y in 0..5 {
            for x in 0..5 {
                assert!(!bin.is_ridge(x, y), "Island cleanup should remove isolated pixels");
            }
        }
    }
}
