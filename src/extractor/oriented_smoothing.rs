/// OrientedSmoothing: smooth the image along (Parallel) or across
/// (Orthogonal) the local ridge orientation. Mirrors .NET OrientedSmoothing.cs
/// — precomputes sampling lines per orientation bucket, then averages the
/// image over each line within masked primary blocks.
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::int_point::IntPoint;
use crate::primitives::int_rect::IntRect;

/// Precompute sample lines: for each orientation bucket, a symmetric set of
/// offsets along the line direction at exponentially decreasing radii.
fn lines(resolution: i32, radius: f64, step: f64) -> Vec<Vec<IntPoint>> {
    let mut result = Vec::with_capacity(resolution as usize);
    for orientation_index in 0..resolution {
        let mut line = vec![IntPoint::ZERO];
        let direction = DoubleAngle::to_vector(DoubleAngle::from_orientation(
            DoubleAngle::bucket_center(orientation_index, resolution),
        ));
        let mut r = radius;
        while r >= 0.5 {
            let sample = (direction * r).round();
            if !line.contains(&sample) {
                line.push(sample);
                line.push(-sample);
            }
            r /= step;
        }
        result.push(line);
    }
    result
}

/// Smooth with lines pre-rotated by `angle` (0 for parallel, PI for orthogonal).
fn smooth(
    input: &DoubleMatrix,
    orientation: &DoubleMatrix,
    mask: &BooleanMatrix,
    blocks: &BlockMap,
    angle: f64,
    line_set: &[Vec<IntPoint>],
) -> DoubleMatrix {
    let mut output = DoubleMatrix::new(input.width(), input.height());
    let pixel_rect = IntRect::from_size(&blocks.pixels);

    for block in blocks.primary.blocks.iterate() {
        if mask.get(block.x() as usize, block.y() as usize) {
            let block_angle = orientation.get(block.x() as usize, block.y() as usize);
            let bucket =
                DoubleAngle::quantize(DoubleAngle::add(block_angle, angle), line_set.len() as i32);
            let line = &line_set[bucket as usize];

            for line_point in line {
                let target = blocks.primary.block(block.x(), block.y());
                // .NET: source = target.Move(linePoint).Intersect(pixelRect); target = source.Move(-linePoint)
                let source = target.move_rect(line_point).intersect(&pixel_rect);
                let shifted = source.move_rect(&-*line_point);
                for y in shifted.top()..shifted.bottom() {
                    if y < 0 || y as usize >= output.height() {
                        continue;
                    }
                    for x in shifted.left()..shifted.right() {
                        if x < 0 || x as usize >= output.width() {
                            continue;
                        }
                        let sx = (x + line_point.x()) as usize;
                        let sy = (y + line_point.y()) as usize;
                        if sx < input.width() && sy < input.height() {
                            output.add(x as usize, y as usize, input.get(sx, sy));
                        }
                    }
                }
            }

            let block_area = blocks.primary.block(block.x(), block.y());
            let divisor = 1.0 / line.len() as f64;
            for y in block_area.top()..block_area.bottom() {
                if y < 0 || y as usize >= output.height() {
                    continue;
                }
                for x in block_area.left()..block_area.right() {
                    if x < 0 || x as usize >= output.width() {
                        continue;
                    }
                    let v = output.get(x as usize, y as usize) * divisor;
                    output.set(x as usize, y as usize, v);
                }
            }
        }
    }

    output
}

/// Smoothing along the ridge orientation.
pub fn parallel(
    input: &DoubleMatrix,
    orientation: &DoubleMatrix,
    mask: &BooleanMatrix,
    blocks: &BlockMap,
) -> DoubleMatrix {
    let line_set = lines(
        Parameters::PARALLEL_SMOOTHING_RESOLUTION as i32,
        Parameters::PARALLEL_SMOOTHING_RADIUS as f64,
        Parameters::PARALLEL_SMOOTHING_STEP,
    );
    smooth(input, orientation, mask, blocks, 0.0, &line_set)
}

/// Smoothing across the ridge orientation (baseline for binarization).
pub fn orthogonal(
    input: &DoubleMatrix,
    orientation: &DoubleMatrix,
    mask: &BooleanMatrix,
    blocks: &BlockMap,
) -> DoubleMatrix {
    let line_set = lines(
        Parameters::ORTHOGONAL_SMOOTHING_RESOLUTION as i32,
        Parameters::ORTHOGONAL_SMOOTHING_RADIUS as f64,
        Parameters::ORTHOGONAL_SMOOTHING_STEP,
    );
    smooth(
        input,
        orientation,
        mask,
        blocks,
        std::f64::consts::PI,
        &line_set,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_lines_cover_center_and_symmetric() {
        let line_set = lines(8, 4.0, 1.6);
        assert_eq!(line_set.len(), 8);
        for line in &line_set {
            assert_eq!(line[0], IntPoint::ZERO, "every line starts at origin");
            // Symmetry: every non-zero offset has its negation present.
            for p in line {
                assert!(line.contains(&-*p), "line must be symmetric: {:?}", p);
            }
            // No duplicates.
            for (i, p) in line.iter().enumerate() {
                assert!(!line[i + 1..].contains(p), "no duplicate offsets");
            }
        }
    }

    #[test]
    fn test_parallel_smooth_preserves_dimensions() {
        let blocks = BlockMap::new(30, 30, 15);
        let mut input = DoubleMatrix::new(30, 30);
        for y in 0..30 {
            for x in 0..30 {
                input.set(x, y, (x + y) as f64);
            }
        }
        let orientation = DoubleMatrix::new(2, 2);
        let mask = full_mask(&blocks);
        let smoothed = parallel(&input, &orientation, &mask, &blocks);
        assert_eq!(smoothed.width(), 30);
        assert_eq!(smoothed.height(), 30);
    }

    #[test]
    fn test_orthogonal_smooth_preserves_dimensions() {
        let blocks = BlockMap::new(30, 30, 15);
        let input = DoubleMatrix::new(30, 30);
        let orientation = DoubleMatrix::new(2, 2);
        let mask = full_mask(&blocks);
        let smoothed = orthogonal(&input, &orientation, &mask, &blocks);
        assert_eq!(smoothed.width(), 30);
        assert_eq!(smoothed.height(), 30);
    }
}
