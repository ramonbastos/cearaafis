/// AbsoluteContrastMask: marks blocks whose clipped contrast is below the
/// absolute threshold. Mirrors .NET AbsoluteContrastMask.cs — block-level
/// (size of the contrast matrix = primary blocks), NOT pixel-level.
use crate::parameters::Parameters;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;

pub fn compute(contrast: &DoubleMatrix) -> BooleanMatrix {
    let size = contrast.width().min(contrast.height());
    let _ = size;
    let mut result = BooleanMatrix::new(contrast.width(), contrast.height());
    for y in 0..contrast.height() {
        for x in 0..contrast.width() {
            if contrast.get(x, y) < Parameters::MIN_ABSOLUTE_CONTRAST {
                result.set(x, y, true);
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_contrast_blocks_marked() {
        let mut contrast = DoubleMatrix::new(3, 3);
        contrast.set(0, 0, 0.01); // below 17/255 ≈ 0.0667
        contrast.set(1, 1, 0.5); // high
        let mask = compute(&contrast);
        assert!(mask.get(0, 0));
        assert!(!mask.get(1, 1));
    }

    #[test]
    fn test_default_all_false_when_high() {
        let mut contrast = DoubleMatrix::new(2, 2);
        for y in 0..2 {
            for x in 0..2 {
                contrast.set(x, y, 0.5);
            }
        }
        let mask = compute(&contrast);
        for y in 0..2 {
            for x in 0..2 {
                assert!(!mask.get(x, y));
            }
        }
    }
}
