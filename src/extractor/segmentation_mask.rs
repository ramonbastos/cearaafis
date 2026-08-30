/// SegmentationMask: block-level fingerprint background mask.
/// Mirrors .NET SegmentationMask.cs — computes a BooleanMatrix over PRIMARY
/// BLOCKS (not pixels) by combining clipped-contrast masks and vote filtering:
///   1. contrast = ClippedContrast(blocks, histogram)
///   2. mask = AbsoluteContrastMask(contrast) | RelativeContrastMask(contrast)
///   3. mask |= BlockErrorsVote(mask); invert; vote twice more; final vote
///
/// The final mask uses `true` for VALID (foreground) pixels — after the
/// inversion in step 3, matching .NET semantics where mask[block] gates
/// equalization/binarization.
use crate::extractor::absolute_contrast_mask as abs_mask;
use crate::extractor::clipped_contrast;
use crate::extractor::relative_contrast_mask as rel_mask;
use crate::extractor::vote_filter;
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::histogram_cube::HistogramCube;

/// Block-level mask: true = valid fingerprint area (foreground).
pub fn compute(blocks: &BlockMap, histogram: &HistogramCube) -> BooleanMatrix {
    let contrast = clipped_contrast::compute(blocks, histogram);
    let mut mask = abs_mask::compute(&contrast);
    mask.merge(&rel_mask::compute(&contrast, blocks));

    // .NET: mask.Merge(Filter(mask)); mask.Invert();
    //       mask.Merge(Filter(mask)); mask.Merge(Filter(mask));
    //       mask.Merge(VoteFilter.Vote(mask, MaskVote*))
    // Filter uses BlockErrors* parameters; the final vote uses Mask* params.
    let block_errors_vote = |m: &BooleanMatrix| {
        vote_filter::vote(
            m,
            None,
            Parameters::BLOCK_ERRORS_VOTE_RADIUS as i32,
            Parameters::BLOCK_ERRORS_VOTE_MAJORITY,
            Parameters::BLOCK_ERRORS_VOTE_BORDER_DISTANCE as i32,
        )
    };

    mask.merge(&block_errors_vote(&mask));
    mask.invert();
    mask.merge(&block_errors_vote(&mask));
    mask.merge(&block_errors_vote(&mask));
    let mask_vote = vote_filter::vote(
        &mask,
        None,
        Parameters::MASK_VOTE_RADIUS as i32,
        Parameters::MASK_VOTE_MAJORITY,
        Parameters::MASK_VOTE_BORDER_DISTANCE as i32,
    );
    mask.merge(&mask_vote);

    mask
}

/// Expand a block-level mask to pixel resolution: each masked block sets all
/// pixels of its primary block. Mirrors .NET SegmentationMask.Pixelwise.
pub fn pixelwise(mask: &BooleanMatrix, blocks: &BlockMap) -> BooleanMatrix {
    let mut pixelized = BooleanMatrix::new(blocks.pixels.x() as usize, blocks.pixels.y() as usize);
    for block in blocks.primary.blocks.iterate() {
        if mask.get(block.x() as usize, block.y() as usize) {
            let rect = blocks.primary.block(block.x(), block.y());
            for py in rect.top()..rect.bottom() {
                for px in rect.left()..rect.right() {
                    if px >= 0
                        && py >= 0
                        && (px as usize) < pixelized.width()
                        && (py as usize) < pixelized.height()
                    {
                        pixelized.set(px as usize, py as usize, true);
                    }
                }
            }
        }
    }
    pixelized
}

/// Shrink mask by removing `amount` pixels from every border.
/// Mirrors .NET SegmentationMask.Shrink (private helper of Inner).
fn shrink(mask: &BooleanMatrix, amount: i32) -> BooleanMatrix {
    let size = mask.size();
    let mut shrunk = BooleanMatrix::new(size.x() as usize, size.y() as usize);
    let a = amount as usize;
    for y in a..size.y() as usize - a {
        for x in a..size.x() as usize - a {
            let v = mask.get(x, y - a)
                && mask.get(x, y + a)
                && mask.get(x - a, y)
                && mask.get(x + a, y);
            shrunk.set(x, y, v);
        }
    }
    shrunk
}

/// Inner mask: eroded version of the pixel mask used to filter minutiae near
/// the fingerprint border. Mirrors .NET SegmentationMask.Inner.
pub fn inner(outer: &BooleanMatrix) -> BooleanMatrix {
    let size = outer.size();
    let mut inner_mask = BooleanMatrix::new(size.x() as usize, size.y() as usize);
    for y in 1..size.y() as usize - 1 {
        for x in 1..size.x() as usize - 1 {
            inner_mask.set(x, y, outer.get(x, y));
        }
    }
    let border = Parameters::INNER_MASK_BORDER_DISTANCE as i32;
    if border >= 1 {
        inner_mask = shrink(&inner_mask, 1);
    }
    let mut total = 1i32;
    let mut step = 1i32;
    while total + step <= border {
        inner_mask = shrink(&inner_mask, step);
        total += step;
        step *= 2;
    }
    if total < border {
        inner_mask = shrink(&inner_mask, border - total);
    }
    inner_mask
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_histogram_with_contrast(
        blocks: &BlockMap,
        fg_blocks: &[(usize, usize)],
    ) -> HistogramCube {
        let mut histogram = HistogramCube::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
            256,
        );
        // Foreground blocks: histogram spread wide (high clipped contrast).
        for &(bx, by) in fg_blocks {
            for i in 0..225usize {
                histogram.increment(bx, by, i.min(255));
            }
        }
        histogram
    }

    #[test]
    fn test_pixelwise_expands_blocks() {
        let blocks = BlockMap::new(30, 30, 15);
        let mut mask = BooleanMatrix::new(2, 2);
        mask.set(0, 0, true);
        let pixelized = pixelwise(&mask, &blocks);
        assert!(pixelized.get(0, 0));
        assert!(pixelized.get(14, 14));
        // Block (1,1) not masked → its pixels stay false.
        assert!(!pixelized.get(29, 29));
    }

    #[test]
    fn test_inner_shrinks_by_border_distance() {
        // A full-true mask shrunk by INNER_MASK_BORDER_DISTANCE=14
        // should be false within 14px of every border.
        let mask = BooleanMatrix::from_point(&crate::primitives::int_point::IntPoint::new(50, 50));
        let mut m = mask.clone();
        for y in 0..50 {
            for x in 0..50 {
                m.set(x, y, true);
            }
        }
        let inner_mask = inner(&m);
        assert!(inner_mask.get(25, 25), "center should stay true");
        assert!(!inner_mask.get(0, 0), "border should be shrunk away");
        assert!(
            !inner_mask.get(5, 25),
            "5px from border < 14 should be shrunk"
        );
        assert!(
            inner_mask.get(20, 25),
            "20px from border > 14 should remain"
        );
    }

    #[test]
    fn test_compute_produces_block_mask() {
        let blocks = BlockMap::new(45, 45, 15);
        let histogram =
            build_histogram_with_contrast(&blocks, &[(0, 0), (1, 0), (0, 1), (1, 1), (2, 2)]);
        let mask = compute(&blocks, &histogram);
        assert_eq!(mask.width(), blocks.primary.blocks.x() as usize);
        assert_eq!(mask.height(), blocks.primary.blocks.y() as usize);
    }
}
