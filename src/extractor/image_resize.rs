/// ImageResize: resizes a grayscale image to a target size based on DPI.
/// Mirrors .NET ImageResizer.cs.
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::int_point::IntPoint;

/// Parameters for image resizing.
pub struct ImageResizer {
    /// Target width in pixels.
    width: usize,
    /// Target height in pixels.
    height: usize,
}

impl ImageResizer {
    /// Create a new ImageResizer.
    /// The target size is computed from DPI: target = dpi * scale_factor / 500.
    /// When dpi is 0 (unknown), we default to a target of 500x500.
    pub fn new(dpi: u32, original: &IntPoint) -> Self {
        let target = if dpi > 0 { dpi as usize } else { 500 };

        let orig_x = original.x() as usize;
        let orig_y = original.y() as usize;

        let width = if orig_y > 0 && orig_x > 0 {
            (target as f64 * orig_x as f64 / orig_y as f64) as usize
        } else {
            target
        };
        let height = target;

        // Clamp dimensions
        let width = width.clamp(50, 2000);
        let height = height.clamp(50, 2000);

        Self { width, height }
    }

    /// Resize the image to the target size using bilinear interpolation.
    pub fn resize(&self, image: &DoubleMatrix) -> DoubleMatrix {
        let mut result = DoubleMatrix::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let src_x = x as f64 * image.width() as f64 / self.width as f64;
                let src_y = y as f64 * image.height() as f64 / self.height as f64;

                let val = Self::bilinear_interpolate(image, src_x, src_y);
                result.set(x, y, val);
            }
        }

        result
    }

    fn bilinear_interpolate(image: &DoubleMatrix, sx: f64, sy: f64) -> f64 {
        let ix = sx as usize;
        let iy = sy as usize;
        let fx = sx - ix as f64;
        let fy = sy - iy as f64;

        let ix = ix.min(image.width() - 1);
        let iy = iy.min(image.height() - 1);

        let get = |x: usize, y: usize| -> f64 {
            let x = x.min(image.width() - 1);
            let y = y.min(image.height() - 1);
            if x < image.width() && y < image.height() {
                image.get(x, y)
            } else {
                0.0
            }
        };

        let v00 = get(ix, iy);
        let v10 = get(ix + 1, iy);
        let v01 = get(ix, iy + 1);
        let v11 = get(ix + 1, iy + 1);

        let left = (1.0 - fy) * v00 + fy * v01;
        let right = (1.0 - fy) * v10 + fy * v11;
        (1.0 - fx) * left + fx * right
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_with_dpi() {
        let resizer = ImageResizer::new(500, &IntPoint::new(200, 200));
        assert_eq!(resizer.width, 500);
        assert_eq!(resizer.height, 500);
    }

    #[test]
    fn test_resize_no_dpi() {
        let resizer = ImageResizer::new(0, &IntPoint::new(100, 100));
        assert_eq!(resizer.width, 500);
        assert_eq!(resizer.height, 500);
    }

    #[test]
    fn test_resize_smaller() {
        let resizer = ImageResizer::new(500, &IntPoint::new(50, 100));
        assert!(resizer.width > 0);
        assert!(resizer.height > 0);
    }

    #[test]
    fn test_resize_actual() {
        let mut img = DoubleMatrix::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                img.set(x, y, (x + y) as f64);
            }
        }
        let resizer = ImageResizer::new(500, &IntPoint::new(10, 10));
        let resized = resizer.resize(&img);
        assert_eq!(resized.width(), resizer.width);
        assert_eq!(resized.height(), resizer.height);
        assert!(resized.get(0, 0) >= 0.0);
    }
}
