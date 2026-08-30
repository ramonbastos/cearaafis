/// RelativeContrastMask: marks blocks whose contrast is below a fraction of
/// the average contrast of the best blocks. Mirrors .NET RelativeContrastMask.cs
/// — block-level: sorts block contrasts descending, averages the top
/// `sample * percentile` fraction, and flags blocks under avg * MinRelativeContrast.
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::doubles::Doubles;

pub fn compute(contrast: &DoubleMatrix, blocks: &BlockMap) -> BooleanMatrix {
    // Collect all block contrasts and sort descending (like .NET: Sort + Reverse).
    let mut sorted_contrast: Vec<f64> = Vec::with_capacity(contrast.width() * contrast.height());
    for y in 0..contrast.height() {
        for x in 0..contrast.width() {
            sorted_contrast.push(contrast.get(x, y));
        }
    }
    sorted_contrast.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let pixels_per_block = (blocks.pixels.area() / blocks.primary.blocks.area()).max(1);
    // .NET: sampleCount = min(count, RelativeContrastSample / pixelsPerBlock)
    let sample_count = (sorted_contrast.len())
        .min(Parameters::RELATIVE_CONTRAST_SAMPLE / pixels_per_block as usize);
    // .NET: consideredBlocks = max(RoundToInt(sampleCount * percentile), 1)
    let considered_blocks =
        (Doubles::round_to_int(sample_count as f64 * Parameters::RELATIVE_CONTRAST_PERCENTILE)
            as usize)
            .max(1);

    let mut average_contrast = 0.0;
    for item in sorted_contrast.iter().take(considered_blocks) {
        average_contrast += *item;
    }
    average_contrast /= considered_blocks as f64;

    let limit = average_contrast * Parameters::MIN_RELATIVE_CONTRAST;

    let mut result = BooleanMatrix::new(contrast.width(), contrast.height());
    for y in 0..contrast.height() {
        for x in 0..contrast.width() {
            if contrast.get(x, y) < limit {
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
    fn test_uniform_contrast_no_mask() {
        // All blocks same contrast → limit == value → nothing below limit.
        let blocks = BlockMap::new(45, 45, 15);
        let mut contrast = DoubleMatrix::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                contrast.set(x, y, 0.5);
            }
        }
        let mask = compute(&contrast, &blocks);
        for y in 0..3 {
            for x in 0..3 {
                assert!(!mask.get(x, y), "uniform contrast should not mask anything");
            }
        }
    }

    #[test]
    fn test_low_contrast_block_masked() {
        let blocks = BlockMap::new(45, 45, 15);
        let mut contrast = DoubleMatrix::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                contrast.set(x, y, 0.5);
            }
        }
        contrast.set(2, 2, 0.01);
        let mask = compute(&contrast, &blocks);
        assert!(
            mask.get(2, 2),
            "weak block should be masked as low relative contrast"
        );
        assert!(!mask.get(0, 0));
    }

    #[test]
    fn test_dimensions_match_blocks() {
        let blocks = BlockMap::new(45, 60, 15);
        let contrast = DoubleMatrix::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
        );
        let mask = compute(&contrast, &blocks);
        assert_eq!(mask.width(), blocks.primary.blocks.x() as usize);
        assert_eq!(mask.height(), blocks.primary.blocks.y() as usize);
    }
}
