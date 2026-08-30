/// LocalHistograms: computes local histogram statistics for each pixel.
/// Mirrors .NET LocalHistograms.cs — histogram computed over a neighborhood window.
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::histogram_cube::HistogramCube;
use crate::parameters::Parameters;

/// Local histogram statistics for each pixel.
pub struct LocalHistograms {
    /// The original image.
    image: DoubleMatrix,
    /// Histogram data cube (width x height x bins).
    data: HistogramCube,
}

impl LocalHistograms {
    /// Create LocalHistograms from an image with a local window around each pixel.
    pub fn new(image: &DoubleMatrix) -> Self {
        let w = image.width();
        let h = image.height();
        let bins = Parameters::HISTOGRAM_DEPTH;
        let mut data = HistogramCube::new(w, h, bins);

        let window = Parameters::LOCAL_HISTOGRAM_WINDOW_SIZE;
        let half_w = (window / 2) as i32;

        for y in 0..h {
            for x in 0..w {
                for dy in -half_w..=half_w {
                    for dx in -half_w..=half_w {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                            let val = image.get(nx as usize, ny as usize);
                            let bin = (val / 255.0 * (bins as f64 - 1.0)) as i32;
                            let b = if bin < 0 {
                                0
                            } else if bin >= bins as i32 {
                                bins - 1
                            } else {
                                bin as usize
                            };
                            data.increment(nx as usize, ny as usize, b);
                        }
                    }
                }
            }
        }

        Self {
            image: image.clone(),
            data,
        }
    }

    pub fn data(&self) -> &HistogramCube {
        &self.data
    }

    pub fn image(&self) -> &DoubleMatrix {
        &self.image
    }

    /// Compute the mean and variance in a local neighborhood.
    pub fn variance_at(&self, x: i32, y: i32, radius: usize) -> (f64, f64) {
        let w = self.image.width() as i32;
        let h = self.image.height() as i32;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        let mut count = 0usize;

        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                let px = x + dx;
                let py = y + dy;
                if px >= 0 && px < w && py >= 0 && py < h {
                    let val = self.image.get(px as usize, py as usize);
                    sum += val;
                    sum_sq += val * val;
                    count += 1;
                }
            }
        }

        if count == 0 {
            return (0.0, 0.0);
        }

        let mean = sum / count as f64;
        let variance = (sum_sq / count as f64) - (mean * mean);
        let variance = if variance < 0.0 { 0.0 } else { variance };

        (mean, variance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_creation() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, 128.0);
            }
        }
        let hist = LocalHistograms::new(&img);
        assert_eq!(hist.data().width(), 10);
        assert_eq!(hist.data().height(), 10);
    }

    #[test]
    fn test_variance_at() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, 128.0);
            }
        }
        let hist = LocalHistograms::new(&img);
        let (mean, var) = hist.variance_at(5, 5, 2);
        assert!(mean > 120.0 && mean < 135.0);
        assert!(var.abs() < 1.0);
    }

    #[test]
    fn test_variance_non_uniform() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..5 {
            for x in 0..10 { img.set(x, y, 255.0); }
        }
        for y in 5..10 {
            for x in 0..10 { img.set(x, y, 0.0); }
        }
        let local_hist = LocalHistograms::new(&img);
        let (_, var) = local_hist.variance_at(5, 2, 5);
        assert!(var > 1000.0);
    }

    #[test]
    fn test_histogram_local_sum_nonzero() {
        let mut img = DoubleMatrix::new(50, 50);
        for y in 0..50 {
            for x in 0..50 {
                img.set(x, y, (x + y) as f64);
            }
        }
        let local_hist = LocalHistograms::new(&img);
        let sum_25_25 = local_hist.data().sum(25, 25);
        assert!(sum_25_25 > 1, "Local histogram sum should include neighborhood, got {}", sum_25_25);
    }
}
