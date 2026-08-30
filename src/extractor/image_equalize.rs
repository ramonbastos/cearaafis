/// ImageEqualize: enhances contrast using histogram equalization.
/// Mirrors .NET ImageEqualize.cs.
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::histogram_cube::HistogramCube;
use crate::parameters::Parameters;

/// Image equalization parameters and results.
pub struct ImageEqualizer {
    /// Original image.
    image: DoubleMatrix,
    /// Equalized image.
    result: DoubleMatrix,
}

impl ImageEqualizer {
    /// Create equalized image from original using histogram equalization.
    pub fn new(image: &DoubleMatrix, _local_hist: &HistogramCube) -> Self {
        let w = image.width();
        let h = image.height();

        // Step 1: Compute histogram and check for uniform image
        let mut hist = vec![0.0; 256];
        let mut total = 0.0;
        let mut min_val = 255.0;
        let mut max_val = 0.0;
        let mut distinct_bins = 0usize;

        for y in 0..h {
            for x in 0..w {
                let val = image.get(x, y);
                let b = (val / 255.0 * 255.0) as usize;
                if b >= 256 {
                    hist[255] += 1.0;
                } else if b < 0 {
                    hist[0] += 1.0;
                } else {
                    hist[b] += 1.0;
                }
                total += 1.0;
                if val < min_val {
                    min_val = val;
                }
                if val > max_val {
                    max_val = val;
                }
            }
        }

        // Count distinct bins (histogram concentration check)
        for &count in &hist {
            if count > 0.0 {
                distinct_bins += 1;
            }
        }

        // Uniform image (single bin): don't equalize, return original
        if distinct_bins <= 1 {
            let mut result = DoubleMatrix::new(w, h);
            for y in 0..h {
                for x in 0..w {
                    result.set(x, y, image.get(x, y));
                }
            }
            return Self {
                image: image.clone(),
                result,
            };
        }

        // Step 2: Compute cumulative distribution and map
        let mut cuml = 0.0;
        let mut lut = [0.0f64; 256];
        for i in 0..256 {
            cuml += hist[i];
            lut[i] = (cuml / total) * 255.0;
        }

        // Step 3: Apply lookup table
        let mut result = DoubleMatrix::new(w, h);

        for y in 0..h {
            for x in 0..w {
                let val = image.get(x, y);
                let b = (val / 255.0 * 255.0) as usize;
                let clamped_b = b.min(255);
                let eq_val = lut[clamped_b];

                // Blend original with equalized value (matches .NET: val + diff * scaling)
                let diff = eq_val - val;
                let eq_result = val + diff * Parameters::MAX_EQUALIZATION_SCALING;
                result.set(x, y, eq_result.max(0.0).min(255.0));
            }
        }

        Self {
            image: image.clone(),
            result,
        }
    }

    /// Slight contrast stretch to enhance mid-tones (mirrors .NET approach).
    fn contrast_stretch(val: f64) -> f64 {
        let normalized = val / 255.0;

        // Apply contrast scaling centered on mid-tones
        let stretched = (normalized - 0.5).abs();
        let stretch_factor = stretched * Parameters::MAX_EQUALIZATION_SCALING;

        // Map back — if original was dark, keep dark; if light, keep light
        // The equalization already spread values, this just adds slight contrast
        let result = normalized + (normalized - 0.5) * stretch_factor * 0.3;
        result * 255.0
    }

    /// Get the equalized image.
    pub fn image(&self) -> &DoubleMatrix {
        &self.result
    }

    /// Get the original image.
    pub fn original(&self) -> &DoubleMatrix {
        &self.image
    }
}

impl Default for ImageEqualizer {
    fn default() -> Self {
        Self {
            image: DoubleMatrix::new(0, 0),
            result: DoubleMatrix::new(0, 0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equalize_basic() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, x as f64 * 10.0);
            }
        }
        let _hist = HistogramCube::new(img.width(), img.height(), 256);
        let equalizer = ImageEqualizer::new(&img, &_hist);
        assert_eq!(equalizer.image().width(), img.width());
        assert_eq!(equalizer.image().height(), img.height());
    }

    #[test]
    fn test_equalize_uniform() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, 128.0);
            }
        }
        let _hist = HistogramCube::new(img.width(), img.height(), 256);
        let equalizer = ImageEqualizer::new(&img, &_hist);
        let v = equalizer.image().get(5, 5);
        assert!(v >= 100.0 && v <= 200.0, "uniform image should stay around 128, got {}", v);
    }

    #[test]
    fn test_equalize_values_in_range() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, (x + y * 10) as f64);
            }
        }
        let _hist = HistogramCube::new(img.width(), img.height(), 256);
        let equalizer = ImageEqualizer::new(&img, &_hist);

        for y in 0..10 {
            for x in 0..10 {
                let v = equalizer.image().get(x, y);
                assert!(v >= 0.0 && v <= 255.0, "equalized pixel out of range: {}", v);
            }
        }
    }
}
