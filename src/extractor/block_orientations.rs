/// BlockOrientations: aggregate per-pixel orientations into per-block
/// orientation angles with neighbor smoothing. Mirrors .NET BlockOrientations.cs.
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::double_point_matrix::DoublePointMatrix;
use crate::primitives::int_rect::IntRect;

/// Sum pixelwise orientation vectors per primary block (masked blocks only).
fn aggregate(
    orientation: &DoublePointMatrix,
    blocks: &BlockMap,
    mask: &BooleanMatrix,
) -> DoublePointMatrix {
    let mut sums = DoublePointMatrix::new(
        blocks.primary.blocks.x() as usize,
        blocks.primary.blocks.y() as usize,
    );
    for block in blocks.primary.blocks.iterate() {
        if mask.get(block.x() as usize, block.y() as usize) {
            let area = blocks.primary.block(block.x(), block.y());
            for y in area.top()..area.bottom() {
                if y < 0 || y as usize >= orientation.height {
                    continue;
                }
                for x in area.left()..area.right() {
                    if x < 0 || x as usize >= orientation.width {
                        continue;
                    }
                    let v = orientation.get(x as usize, y as usize);
                    sums.add_point(block.x() as usize, block.y() as usize, &v);
                }
            }
        }
    }
    sums
}

/// Smooth block vectors over a neighborhood of OrientationSmoothingRadius.
fn smooth(orientation: &DoublePointMatrix, mask: &BooleanMatrix) -> DoublePointMatrix {
    let size = mask.size();
    let mut smoothed = DoublePointMatrix::new(size.x() as usize, size.y() as usize);
    for block in size.iterate() {
        if mask.get(block.x() as usize, block.y() as usize) {
            let neighbors = IntRect::around(
                block.x(),
                block.y(),
                Parameters::ORIENTATION_SMOOTHING_RADIUS as i32,
            )
            .intersect(&IntRect::from_size(&size));
            for ny in neighbors.top()..neighbors.bottom() {
                for nx in neighbors.left()..neighbors.right() {
                    if mask.get(nx as usize, ny as usize) {
                        let v = orientation.get(nx as usize, ny as usize);
                        smoothed.add_point(block.x() as usize, block.y() as usize, &v);
                    }
                }
            }
        }
    }
    smoothed
}

/// Convert summed vectors to angles (unoriented blocks stay 0 like .NET).
fn angles(vectors: &DoublePointMatrix, mask: &BooleanMatrix) -> DoubleMatrix {
    let size = mask.size();
    let mut angle_matrix = DoubleMatrix::new(size.x() as usize, size.y() as usize);
    for block in size.iterate() {
        if mask.get(block.x() as usize, block.y() as usize) {
            let v = vectors.get(block.x() as usize, block.y() as usize);
            angle_matrix.set(
                block.x() as usize,
                block.y() as usize,
                DoubleAngle::atan_point(&v),
            );
        }
    }
    angle_matrix
}

/// Full block-orientation pipeline: aggregate → smooth → atan.
pub fn compute(image: &DoubleMatrix, mask: &BooleanMatrix, blocks: &BlockMap) -> DoubleMatrix {
    let accumulated = super::pixelwise_orientations::compute(image, mask, blocks);
    let by_block = aggregate(&accumulated, blocks, mask);
    let smooth_vectors = smooth(&by_block, mask);
    angles(&smooth_vectors, mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::int_point::IntPoint;

    #[test]
    fn test_angles_masked_only() {
        let mask = BooleanMatrix::new(3, 3);
        let vectors = DoublePointMatrix::new(3, 3);
        let result = angles(&vectors, &mask);
        for y in 0..3 {
            for x in 0..3 {
                assert_eq!(result.get(x, y), 0.0);
            }
        }
    }

    #[test]
    fn test_aggregate_sums_masked_pixels() {
        // 30x30 image of 2x2 blocks: put constant orientation in masked block (0,0).
        let blocks = BlockMap::new(30, 30, 15);
        let mut orientation = DoublePointMatrix::new(30, 30);
        for y in 0..30 {
            for x in 0..30 {
                orientation.add(x, y, 1.0, 0.0);
            }
        }
        let mut mask = BooleanMatrix::new(2, 2);
        mask.set(0, 0, true);
        let sums = aggregate(&orientation, &blocks, &mask);
        assert!(
            (sums.get(0, 0).x() - 225.0).abs() < 1e-9,
            "225 pixels × 1.0"
        );
        assert_eq!(sums.get(1, 1).x(), 0.0, "unmasked block gets zero");
        let _ = IntPoint::ZERO;
    }
}
