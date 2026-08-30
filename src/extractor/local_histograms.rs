/// LocalHistograms: block-level 256-bin grayscale histograms.
/// Mirrors .NET LocalHistograms.cs — ONE histogram per 15x15 primary block
/// (HistogramCube sized blocks.X × blocks.Y × 256), not per pixel. This is the
/// core of the .NET performance advantage: ~640 block histograms instead of
/// ~145k per-pixel histograms on a 388x374 image.
///
/// Create() accumulates each pixel into its block's histogram.
/// Smooth() builds a secondary-block histogram where secondary block (corner)
/// sums the 4 primary blocks touching that corner.
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::histogram_cube::HistogramCube;
use crate::primitives::int_point::IntPoint;

/// Build one 256-bin histogram per primary block. Mirrors .NET LocalHistograms.Create.
pub fn create(blocks: &BlockMap, image: &DoubleMatrix) -> HistogramCube {
    let mut histogram = HistogramCube::new(
        blocks.primary.blocks.x() as usize,
        blocks.primary.blocks.y() as usize,
        Parameters::HISTOGRAM_DEPTH,
    );

    for block in blocks.primary.blocks.iterate() {
        let bx = block.x() as usize;
        let by = block.y() as usize;
        let area = blocks.primary.block(block.x(), block.y());
        for y in area.top()..area.bottom() {
            if y < 0 || y as usize >= image.height() {
                continue;
            }
            for x in area.left()..area.right() {
                if x < 0 || x as usize >= image.width() {
                    continue;
                }
                // .NET: depth = (int)(image[x,y] * histogram.Bins); Constrain(depth)
                let depth = (image.get(x as usize, y as usize) * histogram.bins as f64) as i32;
                let bin = histogram.constrain(depth);
                histogram.increment(bx, by, bin);
            }
        }
    }

    histogram
}

/// Aggregate primary-block histograms into secondary-block (corner) histograms.
/// Mirrors .NET LocalHistograms.Smooth — secondary block (cx, cy) sums primary
/// blocks (cx-1, cy-1), (cx-1, cy), (cx, cy-1), (cx, cy) when inside bounds.
pub fn smooth(blocks: &BlockMap, input: &HistogramCube) -> HistogramCube {
    let blocks_around = [
        IntPoint::new(0, 0),
        IntPoint::new(-1, 0),
        IntPoint::new(0, -1),
        IntPoint::new(-1, -1),
    ];
    let mut output = HistogramCube::new(
        blocks.secondary.blocks.x() as usize,
        blocks.secondary.blocks.y() as usize,
        input.bins,
    );

    for corner in blocks.secondary.blocks.iterate() {
        for relative in &blocks_around {
            let block = corner + *relative;
            if blocks.primary.blocks.contains(&block) {
                let bx = block.x() as usize;
                let by = block.y() as usize;
                let cx = corner.x() as usize;
                let cy = corner.y() as usize;
                for i in 0..input.bins {
                    let v = input.get(bx, by, i);
                    if v != 0 {
                        output.add(cx, cy, i, v);
                    }
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_histogram_dimensions() {
        let blocks = BlockMap::new(45, 45, 15);
        let image = DoubleMatrix::new(45, 45);
        let hist = create(&blocks, &image);
        assert_eq!(hist.width, blocks.primary.blocks.x() as usize);
        assert_eq!(hist.height, blocks.primary.blocks.y() as usize);
        assert_eq!(hist.bins, 256);
    }

    #[test]
    fn test_create_counts_all_pixels() {
        // 45x45 image with block size 15 → 3x3 blocks, each block gets 225 pixels.
        let blocks = BlockMap::new(45, 45, 15);
        let mut image = DoubleMatrix::new(45, 45);
        for y in 0..45 {
            for x in 0..45 {
                image.set(x, y, 0.5); // depth = 0.5 * 256 = 128
            }
        }
        let hist = create(&blocks, &image);
        for by in 0..3 {
            for bx in 0..3 {
                assert_eq!(
                    hist.sum(bx, by),
                    225,
                    "block ({bx},{by}) should count all its 225 pixels"
                );
                assert_eq!(hist.get(bx, by, 128), 225, "all pixels in bin 128");
            }
        }
    }

    #[test]
    fn test_create_nonuniform_blocks() {
        let blocks = BlockMap::new(30, 15, 15); // 2x1 blocks
        let mut image = DoubleMatrix::new(30, 15);
        for y in 0..15 {
            for x in 0..15 {
                image.set(x, y, 0.0);
            }
        }
        for y in 0..15 {
            for x in 15..30 {
                image.set(x, y, 1.0);
            }
        }
        let hist = create(&blocks, &image);
        assert_eq!(hist.get(0, 0, 0), 225, "block 0: all dark pixels");
        assert_eq!(
            hist.get(1, 0, 255),
            225,
            "block 1: all bright pixels (depth 256 → constrained 255)"
        );
        assert_eq!(hist.get(1, 0, 0), 0);
    }

    #[test]
    fn test_smooth_sums_four_neighbors() {
        // 45x45 → 3x3 primary blocks → 4x4 secondary blocks.
        let blocks = BlockMap::new(45, 45, 15);
        let mut image = DoubleMatrix::new(45, 45);
        for y in 0..45 {
            for x in 0..45 {
                image.set(x, y, 0.5);
            }
        }
        let hist = create(&blocks, &image);
        let smoothed = smooth(&blocks, &hist);
        assert_eq!(smoothed.width, blocks.secondary.blocks.x() as usize);
        assert_eq!(smoothed.height, blocks.secondary.blocks.y() as usize);

        // Secondary corner (1,1) touches primary blocks (0,0),(0,1),(1,0),(1,1)
        // → 4 * 225 = 900 pixels.
        assert_eq!(smoothed.sum(1, 1), 900);
        // Secondary corner (0,0) touches only primary (0,0) → 225.
        assert_eq!(smoothed.sum(0, 0), 225);
        // Secondary corner (3,3) touches only primary (2,2) → 225.
        assert_eq!(smoothed.sum(3, 3), 225);
        // Edge corner (1,0) touches primary (0,0),(1,0) → 450.
        assert_eq!(smoothed.sum(1, 0), 450);
    }
}
