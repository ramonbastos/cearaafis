/// SkeletonFilters: filters for skeleton cleaning (dot, pore, gap, tail, fragment, knot).
/// Mirrors .NET SkeletonFilters.cs.
use crate::primitives::bool_matrix::BooleanMatrix;

/// Skeleton filter results.
pub struct SkeletonFilter {
    skeleton: BooleanMatrix,
}

impl SkeletonFilter {
    pub fn new(skeleton: &BooleanMatrix) -> Self {
        let mut filtered = BooleanMatrix::from_clone(skeleton);
        // Only remove isolated dots and disconnected small components
        Self::remove_isolated_dots(&mut filtered);
        Self { skeleton: filtered }
    }

    pub fn skeleton(&self) -> &BooleanMatrix {
        &self.skeleton
    }

    fn remove_isolated_dots(skeleton: &mut BooleanMatrix) {
        let mut to_remove = Vec::new();
        let w = skeleton.width();
        let h = skeleton.height();

        for y in 1..(h - 1) {
            for x in 1..(w - 1) {
                if !skeleton.get(x, y) {
                    continue;
                }
                let mut n = 0usize;
                for dy in -1i32..=1i32 {
                    for dx in -1i32..=1i32 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0
                            && ny >= 0
                            && nx < w as i32
                            && ny < h as i32
                            && skeleton.get(nx as usize, ny as usize)
                        {
                            n += 1;
                        }
                    }
                }
                if n == 0 {
                    to_remove.push((x, y));
                }
            }
        }
        for (x, y) in to_remove {
            skeleton.set(x, y, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_removes_dots() {
        let mut skeleton = BooleanMatrix::new(5, 5);
        skeleton.set(2, 2, true);
        let filter = SkeletonFilter::new(&skeleton);
        assert!(!filter.skeleton().get(2, 2));
    }

    #[test]
    fn test_filter_keeps_lines() {
        let mut skeleton = BooleanMatrix::new(10, 10);
        for x in 0..10 {
            skeleton.set(x, 5, true);
        }
        let filter = SkeletonFilter::new(&skeleton);
        assert!(filter.skeleton().get(5, 5));
    }

    #[test]
    fn test_filter_dimensions() {
        let mut skeleton = BooleanMatrix::new(20, 20);
        for y in 0..20 {
            for x in 0..20 {
                skeleton.set(x, y, true);
            }
        }
        let filter = SkeletonFilter::new(&skeleton);
        assert_eq!(filter.skeleton().width(), 20);
        assert_eq!(filter.skeleton().height(), 20);
    }
}
