/// ClippedContrast: per-block contrast after clipping histogram tails.
/// Mirrors .NET ClippedContrast.cs — operates on the BlockMap's primary blocks.
/// For each block, walks the histogram inward from both ends until each side
/// accumulates more than `volume * ClippedContrast` pixels; the remaining span
/// between bounds is the block's clipped contrast in [0, 1].
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::doubles::Doubles;
use crate::primitives::histogram_cube::HistogramCube;
use crate::primitives::int_point::IntPoint;

pub fn compute(blocks: &BlockMap, histogram: &HistogramCube) -> DoubleMatrix {
    let size = blocks.primary.blocks;
    let mut result = DoubleMatrix::new(size.x() as usize, size.y() as usize);

    for block in size.iterate() {
        let bx = block.x() as usize;
        let by = block.y() as usize;

        let volume = histogram.sum(bx, by);
        let clip_limit = Doubles::round_to_int(volume as f64 * Parameters::CLIPPED_CONTRAST) as i32;

        // Walk from the dark end until the accumulator passes the clip limit.
        let mut accumulator = 0i32;
        let mut lower_bound = histogram.bins - 1;
        for i in 0..histogram.bins {
            accumulator += histogram.get(bx, by, i);
            if accumulator > clip_limit {
                lower_bound = i;
                break;
            }
        }

        // Walk from the bright end likewise.
        let mut accumulator = 0i32;
        let mut upper_bound = 0usize;
        for i in (0..histogram.bins).rev() {
            accumulator += histogram.get(bx, by, i);
            if accumulator > clip_limit {
                upper_bound = i;
                break;
            }
        }

        let contrast =
            (upper_bound as f64 - lower_bound as f64) * (1.0 / (histogram.bins - 1) as f64);
        result.set(bx, by, contrast);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_block_zero_contrast() {
        // All pixels in the same bin → both bounds converge → contrast 0.
        let blocks = BlockMap::new(30, 30, 15);
        let mut histogram = HistogramCube::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
            256,
        );
        // Fill one block's histogram: every pixel in bin 128.
        for _ in 0..225 {
            histogram.increment(0, 0, 128);
        }
        let contrast = compute(&blocks, &histogram);
        assert!(
            contrast.get(0, 0).abs() < 1e-9,
            "uniform block should have zero contrast, got {}",
            contrast.get(0, 0)
        );
    }

    #[test]
    fn test_full_range_block_high_contrast() {
        // Spread pixels across the full bin range → wide bounds → high contrast.
        let blocks = BlockMap::new(30, 30, 15);
        let mut histogram = HistogramCube::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
            256,
        );
        // 225 pixels spread over bins 0..225 (one per bin).
        for i in 0..225usize {
            histogram.increment(0, 0, i);
        }
        let contrast = compute(&blocks, &histogram);
        // clipLimit = round(225*0.08) = 18 → lower=18, upper=206 (verified
        // against the .NET algorithm by simulation) → (206-18)/255 = 0.737.
        assert!(
            contrast.get(0, 0) > 0.7,
            "wide-spread histogram should yield high contrast, got {}",
            contrast.get(0, 0)
        );
    }

    #[test]
    fn test_empty_block_zero_contrast() {
        // Volume 0 → clipLimit 0 → lowerBound=255 (never exceeded), upper=0.
        let blocks = BlockMap::new(30, 30, 15);
        let histogram = HistogramCube::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
            256,
        );
        let contrast = compute(&blocks, &histogram);
        // empty block: accumulator never > 0 → lower=255, upper=0 → negative contrast
        // .NET yields the same (0 - 255)/255 — callers mask these blocks out anyway.
        assert!(contrast.get(1, 1) < 0.0);
    }

    #[test]
    fn test_result_dimensions_match_blocks() {
        let blocks = BlockMap::new(45, 60, 15);
        let histogram = HistogramCube::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
            256,
        );
        let contrast = compute(&blocks, &histogram);
        assert_eq!(contrast.width(), blocks.primary.blocks.x() as usize);
        assert_eq!(contrast.height(), blocks.primary.blocks.y() as usize);
    }
}
