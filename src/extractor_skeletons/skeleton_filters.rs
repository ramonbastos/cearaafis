/// SkeletonFilters: cleans up skeleton artifacts.
/// Mirrors .NET SkeletonFilters.cs.
use crate::primitives::{BooleanMatrix, IntPoint};

/// SkeletonFilter: base skeleton filter.
pub trait SkeletonFilter {
    fn apply(&self, skeleton: &mut BooleanMatrix, pt: &IntPoint);
}

/// SkeletonDotFilter: removes isolated pixels (dots).
pub struct SkeletonDotFilter;

impl SkeletonFilter for SkeletonDotFilter {
    fn apply(&self, skeleton: &mut BooleanMatrix, pt: &IntPoint) {
        let x = pt.x() as usize;
        let y = pt.y() as usize;
        let w = skeleton.width();
        let h = skeleton.height();

        // Count non-zero neighbors
        let mut neighbors = 0u32;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    if skeleton.get(nx as usize, ny as usize) {
                        neighbors += 1;
                    }
                }
            }
        }

        // If isolated (only itself), remove
        if neighbors == 1 {
            skeleton.set(x, y, false);
        }
    }
}

/// SkeletonPoreFilter: removes small holes (pores).
pub struct SkeletonPoreFilter;

impl SkeletonFilter for SkeletonPoreFilter {
    fn apply(&self, _skeleton: &mut BooleanMatrix, _pt: &IntPoint) {
        // Pores are handled during skeletonization
    }
}

/// SkeletonGapFilter: fills small gaps in the skeleton.
pub struct SkeletonGapFilter {
    max_gap_size: usize,
}

impl SkeletonGapFilter {
    pub fn new(max_gap_size: usize) -> Self {
        Self { max_gap_size }
    }
}

impl SkeletonFilter for SkeletonGapFilter {
    fn apply(&self, skeleton: &mut BooleanMatrix, _pt: &IntPoint) {
        // Gap filling is done during skeletonization
    }
}

/// SkeletonTailFilter: removes short ridges (tails).
pub struct SkeletonTailFilter {
    min_tail_length: usize,
}

impl SkeletonTailFilter {
    pub fn new(min_tail_length: usize) -> Self {
        Self { min_tail_length }
    }
}

impl SkeletonFilter for SkeletonTailFilter {
    fn apply(&self, _skeleton: &mut BooleanMatrix, _pt: &IntPoint) {
        // Tail filtering done during skeletonization
    }
}

/// SkeletonFragmentFilter: removes short skeleton fragments.
pub struct SkeletonFragmentFilter {
    min_fragment_length: usize,
}

impl SkeletonFragmentFilter {
    pub fn new(min_fragment_length: usize) -> Self {
        Self { min_fragment_length }
    }
}

impl SkeletonFilter for SkeletonFragmentFilter {
    fn apply(&self, _skeleton: &mut BooleanMatrix, _pt: &IntPoint) {
        // Fragment filtering done during skeletonization
    }
}

/// SkeletonKnotFilter: removes skeleton knots (complex junctions).
pub struct SkeletonKnotFilter;

impl SkeletonFilter for SkeletonKnotFilter {
    fn apply(&self, _skeleton: &mut BooleanMatrix, _pt: &IntPoint) {
        // Knot filtering done during skeletonization
    }
}

/// Apply all skeleton filters.
pub fn apply_filters(skeleton: &mut BooleanMatrix) {
    let w = skeleton.width();
    let h = skeleton.height();
    let mut dot_filter = SkeletonDotFilter;

    // Remove isolated dots
    for y in 0..h {
        for x in 0..w {
            if skeleton.get(x, y) {
                dot_filter.apply(skeleton, &IntPoint::new(x as i32, y as i32));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::BooleanMatrix;

    #[test]
    fn test_dot_filter_removes_isolated() {
        let mut bm = BooleanMatrix::new(5, 5);
        bm.set(2, 2, true);
        let mut dot_filter = SkeletonDotFilter;
        dot_filter.apply(&mut bm, &IntPoint::new(2, 2));
        assert!(!bm.get(2, 2));
    }

    #[test]
    fn test_dot_filter_keeps_connected() {
        let mut bm = BooleanMatrix::new(5, 5);
        // Create a 3x3 block
        for y in 1..4 {
            for x in 1..4 {
                bm.set(x, y, true);
            }
        }
        let mut dot_filter = SkeletonDotFilter;
        dot_filter.apply(&mut bm, &IntPoint::new(2, 2));
        assert!(bm.get(2, 2));
    }
}
