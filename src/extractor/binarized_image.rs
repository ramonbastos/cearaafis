/// BinarizedImage: threshold the image into a binary skeleton image.
/// Mirrors .NET BinarizedImage.cs.
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::histogram_cube::HistogramCube;
use crate::parameters::Parameters;

/// Binary image result from thresholding.
pub struct BinarizedImage {
    image: BooleanMatrix,
}

impl BinarizedImage {
    pub fn from_image(image: &DoubleMatrix, histogram: &HistogramCube) -> Self {
        let w = image.width();
        let h = image.height();
        let mut binary = BooleanMatrix::new(w, h);

        let threshold_percentile = Parameters::RELATIVE_CONTRAST_PERCENTILE;

        let mut ridge_count = 0usize;
        let mut valley_count = 0usize;

        for y in 0..h {
            for x in 0..w {
                let total = histogram.sum(x, y);
                let val = image.get(x, y);
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

        #[cfg(test)]
        if ridge_count == 0 && h > 10 {
            eprintln!("[binarize] ZERO ridge pixels (w={} h={} threshold_percentile={}), sample: pixel_min=0, pixel_range=0",
                w, h, threshold_percentile);
            // Print a few sample values
            for y in [10, 50, 100] {
                for x in [10, 50, 100] {
                    if y < h && x < w {
                        let p = image.get(x, y);
                        let s = histogram.sum(x, y);
                        let b = histogram.get(x, y, 0);
                        eprintln!("[binarize]   pix({},{})= {:.1} sum={:.1} bin0={:.1}", x, y, p, s, b);
                    }
                }
            }
        }

        Self { image: binary }
    }

    pub fn image(&self) -> &BooleanMatrix {
        &self.image
    }

    pub fn is_ridge(&self, x: i32, y: i32) -> bool {
        let w = self.image.width() as i32;
        let h = self.image.height() as i32;
        if x < 0 || y < 0 || x >= w || y >= h { return false; }
        self.image.get(x as usize, y as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binarize_uniform() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 { for x in 0..10 { img.set(x, y, 128.0); } }
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
}
