/// ImageEqualization: block-level histogram equalization with bilinear
/// interpolation of per-corner mappings. Mirrors .NET ImageEqualization.cs.
///
/// For each SECONDARY block (corner), builds a 256-entry mapping that spreads
/// occupied histogram bins across [-1, 1], clamped by MaxEqualizationScaling /
/// MinEqualizationScaling band widths. Each pixel then interpolates between
/// the 4 corner mappings bilinearly (rx, ry within its block). Masked-out
/// blocks become -1.
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::doubles::Doubles;
use crate::primitives::histogram_cube::HistogramCube;
use crate::primitives::int_point::IntPoint;
use std::collections::HashMap;

const RANGE_MIN: f64 = -1.0;
const RANGE_MAX: f64 = 1.0;
const RANGE_SIZE: f64 = RANGE_MAX - RANGE_MIN;

pub fn equalize(
    blocks: &BlockMap,
    image: &DoubleMatrix,
    histogram: &HistogramCube,
    block_mask: &BooleanMatrix,
) -> DoubleMatrix {
    let bins = histogram.bins;
    let width_max = RANGE_SIZE / bins as f64 * Parameters::MAX_EQUALIZATION_SCALING;
    let width_min = RANGE_SIZE / bins as f64 * Parameters::MIN_EQUALIZATION_SCALING;

    // limitedMin[i]/limitedMax[i] clamp the equalized value of bin i so a
    // single dominant bin can't smear the whole range.
    let limited_min: Vec<f64> = (0..bins)
        .map(|i| {
            ((i as f64) * width_min + RANGE_MIN)
                .max(RANGE_MAX - ((bins - 1 - i) as f64) * width_max)
        })
        .collect();
    let limited_max: Vec<f64> = (0..bins)
        .map(|i| {
            ((i as f64) * width_max + RANGE_MIN)
                .min(RANGE_MAX - ((bins - 1 - i) as f64) * width_min)
        })
        .collect();
    let dequantized: Vec<f64> = (0..bins).map(|i| i as f64 / (bins - 1) as f64).collect();

    // One mapping per secondary block (corner), like .NET's Dictionary.
    let mut mappings: HashMap<IntPoint, Vec<f64>> = HashMap::new();
    for corner in blocks.secondary.blocks.iterate() {
        let mut mapping = vec![0.0f64; bins];
        let cx = corner.x();
        let cy = corner.y();
        // .NET checks blockMask at the corner itself or any of the 3 blocks
        // up-left of it (matching which primary blocks feed this corner).
        let touches = block_mask.get_with_fallback(cx, cy, false)
            || block_mask.get_with_fallback(cx - 1, cy, false)
            || block_mask.get_with_fallback(cx, cy - 1, false)
            || block_mask.get_with_fallback(cx - 1, cy - 1, false);
        if touches {
            let total = histogram.sum(cx as usize, cy as usize);
            if total > 0 {
                let step = RANGE_SIZE / total as f64;
                let mut top = RANGE_MIN;
                for i in 0..bins {
                    let band = histogram.get(cx as usize, cy as usize, i) as f64 * step;
                    let mut equalized = top + dequantized[i] * band;
                    top += band;
                    if equalized < limited_min[i] {
                        equalized = limited_min[i];
                    }
                    if equalized > limited_max[i] {
                        equalized = limited_max[i];
                    }
                    mapping[i] = equalized;
                }
            }
        }
        mappings.insert(corner, mapping);
    }

    // Apply per-pixel with bilinear interpolation of the 4 surrounding corner
    // mappings. .NET: Doubles.Interpolate(bottomleft, bottomright, topleft,
    // topright, rx, ry) — note its argument order: interpolate_2d's x/y params
    // and the corner ordering follow its signature.
    let mut result = DoubleMatrix::new(image.width(), image.height());
    for block in blocks.primary.blocks.iterate() {
        let area = blocks.primary.block(block.x(), block.y());
        if block_mask.get(block.x() as usize, block.y() as usize) {
            let topleft = &mappings[&IntPoint::new(block.x(), block.y())];
            let topright = &mappings[&IntPoint::new(block.x() + 1, block.y())];
            let bottomleft = &mappings[&IntPoint::new(block.x(), block.y() + 1)];
            let bottomright = &mappings[&IntPoint::new(block.x() + 1, block.y() + 1)];
            for y in area.top()..area.bottom() {
                if y < 0 || y as usize >= image.height() {
                    continue;
                }
                for x in area.left()..area.right() {
                    if x < 0 || x as usize >= image.width() {
                        continue;
                    }
                    let depth = histogram
                        .constrain((image.get(x as usize, y as usize) * bins as f64) as i32);
                    let rx = ((x - area.x) as f64 + 0.5) / area.width as f64;
                    let ry = ((y - area.y) as f64 + 0.5) / area.height as f64;
                    // .NET Interpolate(bottomleft, bottomright, topleft, topright, x, y):
                    //   left  = Interpolate(topleft, bottomleft, y)
                    //   right = Interpolate(topright, bottomright, y)
                    //   return Interpolate(left, right, x)
                    let left = Doubles::interpolate(topleft[depth], bottomleft[depth], ry);
                    let right = Doubles::interpolate(topright[depth], bottomright[depth], ry);
                    result.set(
                        x as usize,
                        y as usize,
                        Doubles::interpolate(left, right, rx),
                    );
                }
            }
        } else {
            for y in area.top()..area.bottom() {
                if y < 0 || y as usize >= image.height() {
                    continue;
                }
                for x in area.left()..area.right() {
                    if x < 0 || x as usize >= image.width() {
                        continue;
                    }
                    result.set(x as usize, y as usize, -1.0);
                }
            }
        }
    }

    result
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

    fn build_45() -> (BlockMap, DoubleMatrix, HistogramCube) {
        let blocks = BlockMap::new(45, 45, 15);
        let mut image = DoubleMatrix::new(45, 45);
        for y in 0..45 {
            for x in 0..45 {
                image.set(x, y, 0.5);
            }
        }
        // Equalize consumes the SMOOTHED (secondary-block) histogram like .NET.
        let primary = crate::extractor::local_histograms::create(&blocks, &image);
        let smoothed = crate::extractor::local_histograms::smooth(&blocks, &primary);
        (blocks, image, smoothed)
    }

    #[test]
    fn test_equalize_output_dimensions() {
        let (blocks, image, histogram) = build_45();
        let mask = full_mask(&blocks);
        let result = equalize(&blocks, &image, &histogram, &mask);
        assert_eq!(result.width(), 45);
        assert_eq!(result.height(), 45);
    }

    #[test]
    fn test_equalize_uniform_image_stays_bounded() {
        // Uniform 0.5 image: every pixel in bin 128; equalized value must land
        // within [-1, 1] regardless of the exact mapping.
        let (blocks, image, histogram) = build_45();
        let mask = full_mask(&blocks);
        let result = equalize(&blocks, &image, &histogram, &mask);
        for y in 0..45 {
            for x in 0..45 {
                let v = result.get(x, y);
                assert!(
                    v >= -1.0 && v <= 1.0,
                    "equalized value out of range at ({x},{y}): {v}"
                );
            }
        }
    }

    #[test]
    fn test_equalize_masked_out_block_becomes_minus_one() {
        let (blocks, image, histogram) = build_45();
        let mut mask = full_mask(&blocks);
        mask.set(2, 2, false); // block (2,2) excluded
        let result = equalize(&blocks, &image, &histogram, &mask);
        // Block (2,2) covers pixels x=30..44, y=30..44.
        assert!(
            result.get(35, 35) == -1.0,
            "masked-out block should be -1, got {}",
            result.get(35, 35)
        );
        assert!(
            result.get(5, 5) >= -1.0,
            "masked-in block should be equalized"
        );
    }
}
