/// BinarizedImage: block-gated pixel thresholding against a baseline.
/// Mirrors .NET BinarizedImage.cs — a pixel is "ridge" (true) when
/// `input[x,y] - baseline[x,y] > 0`, but ONLY inside masked primary blocks.
/// input = parallel-smoothed image, baseline = orthogonal-smoothed image.
/// Cleanup removes islands/holes via voting and eliminates diagonal crosses.
use crate::extractor::vote_filter;
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;

/// Binarize: true where input exceeds baseline, restricted to masked blocks.
pub fn binarize(
    input: &DoubleMatrix,
    baseline: &DoubleMatrix,
    mask: &BooleanMatrix,
    blocks: &BlockMap,
) -> BooleanMatrix {
    let size = input.size();
    let mut binarized = BooleanMatrix::new(size.x() as usize, size.y() as usize);

    for block in blocks.primary.blocks.iterate() {
        if mask.get(block.x() as usize, block.y() as usize) {
            let rect = blocks.primary.block(block.x(), block.y());
            for y in rect.top()..rect.bottom() {
                if y < 0 || y as usize >= input.height() {
                    continue;
                }
                for x in rect.left()..rect.right() {
                    if x < 0 || x as usize >= input.width() {
                        continue;
                    }
                    if input.get(x as usize, y as usize) - baseline.get(x as usize, y as usize)
                        > 0.0
                    {
                        binarized.set(x as usize, y as usize, true);
                    }
                }
            }
        }
    }

    binarized
}

/// Remove diagonal checkerboard artifacts in-place.
/// Mirrors .NET BinarizedImage.RemoveCrosses.
fn remove_crosses(input: &mut BooleanMatrix) {
    let size = input.size();
    let mut any = true;
    while any {
        any = false;
        for y in 0..size.y() - 1 {
            for x in 0..size.x() - 1 {
                let p00 = input.get(x as usize, y as usize);
                let p01 = input.get(x as usize, (y + 1) as usize);
                let p10 = input.get((x + 1) as usize, y as usize);
                let p11 = input.get((x + 1) as usize, (y + 1) as usize);
                // .NET condition: main diagonal set and anti-diagonal clear, or vice versa
                let cross_a = p00 && p11 && !p01 && !p10;
                let cross_b = p01 && p10 && !p00 && !p11;
                if cross_a || cross_b {
                    input.set(x as usize, y as usize, false);
                    input.set(x as usize, (y + 1) as usize, false);
                    input.set((x + 1) as usize, y as usize, false);
                    input.set((x + 1) as usize, (y + 1) as usize, false);
                    any = true;
                }
            }
        }
    }
}

/// Cleanup: remove islands (inverted-image vote) and holes (direct vote),
/// then eliminate diagonal crosses. Modifies `binary` in place.
/// Mirrors .NET BinarizedImage.Cleanup.
pub fn cleanup(binary: &mut BooleanMatrix, mask: &BooleanMatrix) {
    let size = binary.size();

    let mut inverted = BooleanMatrix::from_clone(binary);
    inverted.invert();
    let islands = vote_filter::vote(
        &inverted,
        Some(mask),
        Parameters::BINARIZED_VOTE_RADIUS as i32,
        Parameters::BINARIZED_VOTE_MAJORITY,
        Parameters::BINARIZED_VOTE_BORDER_DISTANCE as i32,
    );
    let holes = vote_filter::vote(
        binary,
        Some(mask),
        Parameters::BINARIZED_VOTE_RADIUS as i32,
        Parameters::BINARIZED_VOTE_MAJORITY,
        Parameters::BINARIZED_VOTE_BORDER_DISTANCE as i32,
    );

    for y in 0..size.y() as usize {
        for x in 0..size.x() as usize {
            let v = binary.get(x, y) && !islands.get(x, y) || holes.get(x, y);
            binary.set(x, y, v);
        }
    }

    remove_crosses(binary);
}

/// Invert binary image restricted to the mask. Mirrors .NET BinarizedImage.Invert.
pub fn invert(binary: &BooleanMatrix, mask: &BooleanMatrix) -> BooleanMatrix {
    let size = binary.size();
    let mut inverted = BooleanMatrix::new(size.x() as usize, size.y() as usize);
    for y in 0..size.y() as usize {
        for x in 0..size.x() as usize {
            inverted.set(x, y, !binary.get(x, y) && mask.get(x, y));
        }
    }
    inverted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::int_point::IntPoint;

    fn full_mask(blocks: &BlockMap) -> BooleanMatrix {
        let mut mask = BooleanMatrix::new(
            blocks.primary.blocks.x() as usize,
            blocks.primary.blocks.y() as usize,
        );
        for y in 0..mask.height() {
            for x in 0..mask.width() {
                mask.set(x, y, true);
            }
        }
        mask
    }

    #[test]
    fn test_binarize_input_above_baseline() {
        let blocks = BlockMap::new(30, 30, 15);
        let mut input = DoubleMatrix::new(30, 30);
        let baseline = DoubleMatrix::new(30, 30);
        input.set(10, 10, 0.5); // brighter than baseline 0
        input.set(20, 5, -0.5); // darker than baseline 0
        let mask = full_mask(&blocks);
        let binary = binarize(&input, &baseline, &mask, &blocks);
        assert!(binary.get(10, 10), "input > baseline should be true");
        assert!(!binary.get(20, 5), "input < baseline should be false");
    }

    #[test]
    fn test_binarize_skips_unmasked_blocks() {
        let blocks = BlockMap::new(30, 30, 15);
        let mut input = DoubleMatrix::new(30, 30);
        let baseline = DoubleMatrix::new(30, 30);
        input.set(25, 25, 0.9);
        let mut mask = full_mask(&blocks);
        mask.set(1, 1, false); // block containing (25,25)
        let binary = binarize(&input, &baseline, &mask, &blocks);
        assert!(
            !binary.get(25, 25),
            "pixel in unmasked block must not binarize"
        );
    }

    #[test]
    fn test_remove_crosses_clears_checkerboard() {
        let mut binary = BooleanMatrix::new(4, 4);
        // 2x2 checkerboard at (0,0)..(1,1): p00=true, p11=true, p01=false, p10=false
        binary.set(0, 0, true);
        binary.set(1, 1, true);
        let mut b = binary;
        remove_crosses(&mut b);
        assert!(!b.get(0, 0), "cross should be removed");
        assert!(!b.get(1, 1), "cross should be removed");
    }

    #[test]
    fn test_invert_respects_mask() {
        let mut binary = BooleanMatrix::new(4, 4);
        binary.set(1, 1, true);
        let mut mask = BooleanMatrix::new(4, 4);
        mask.set(1, 1, true);
        mask.set(2, 2, true);
        let inverted = invert(&binary, &mask);
        assert!(!inverted.get(1, 1), "binary true + mask true → false");
        assert!(inverted.get(2, 2), "binary false + mask true → true");
        assert!(!inverted.get(0, 0), "mask false → false");
    }

    #[test]
    fn test_cleanup_preserves_dimensions() {
        let mut binary = BooleanMatrix::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                binary.set(x, y, (x + y) % 2 == 0);
            }
        }
        let mask = BooleanMatrix::from_point(&IntPoint::new(40, 40));
        let mut m = mask.clone();
        for y in 0..40 {
            for x in 0..40 {
                m.set(x, y, true);
            }
        }
        cleanup(&mut binary, &m);
        assert_eq!(binary.width(), 40);
        assert_eq!(binary.height(), 40);
    }
}
