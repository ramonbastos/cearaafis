/// MinutiaCollector: extracts minutiae (ridge endings and bifurcations) from a skeleton.
/// Mirrors .NET MinutiaCollector.cs.
use crate::primitives::{BooleanMatrix, IntPoint, ShortPoint};
use crate::features::{Minutia, MinutiaType, NeighborEdge, EdgeShape};
use crate::features::skeleton_type::SkeletonType;

/// SkeletonMinutia: a point on the skeleton that corresponds to a minutia.
pub struct SkeletonMinutia {
    pub position: IntPoint,
    pub ridges: Vec<usize>,
}

impl SkeletonMinutia {
    pub fn new(position: IntPoint) -> Self {
        Self { position, ridges: Vec::new() }
    }
}

/// MinutiaCollector: collects minutiae from a binary skeleton.
pub struct MinutiaCollector {
    skeleton: BooleanMatrix,
    width: usize,
    height: usize,
}

impl MinutiaCollector {
    /// Create a minutia collector from a binary skeleton.
    pub fn new(skeleton: &BooleanMatrix) -> Self {
        Self {
            skeleton: BooleanMatrix::from_clone(skeleton),
            width: skeleton.width(),
            height: skeleton.height(),
        }
    }

    /// Collect all minutiae from the skeleton.
    pub fn collect(&self) -> Vec<Minutia> {
        let mut minutiae = Vec::new();

        // Find endpoints (ridge endings) and junctions (bifurcations)
        for y in 1..(self.height - 1) {
            for x in 1..(self.width - 1) {
                if !self.skeleton.get(x, y) {
                    continue;
                }

                let neighbors = self.count_neighbors(x, y);

                if neighbors == 1 {
                    // Ridge ending
                    minutiae.push(Minutia::new(
                        IntPoint::new(x as i32, y as i32),
                        0.0,
                        MinutiaType::Ending,
                    ));
                } else if neighbors >= 3 {
                    // Bifurcation
                    minutiae.push(Minutia::new(
                        IntPoint::new(x as i32, y as i32),
                        0.0,
                        MinutiaType::Bifurcation,
                    ));
                }
            }
        }

        minutiae
    }

    /// Count non-zero neighbors (8-connectivity).
    fn count_neighbors(&self, x: usize, y: usize) -> usize {
        let mut count = 0usize;

        for dy in 0..3 {
            for dx in 0..3 {
                let nx = x + dx - 1;
                let ny = y + dy - 1;

                if nx == 0 && ny == 0 {
                    continue; // Skip center
                }

                if nx < self.width && ny < self.height {
                    if self.skeleton.get(nx, ny) {
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Get the skeleton.
    pub fn skeleton(&self) -> &BooleanMatrix {
        &self.skeleton
    }

    /// Get skeleton dimensions.
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minutia_collector_endpoints() {
        // Create a simple vertical line - should have 2 endpoints
        let mut bm = BooleanMatrix::new(10, 10);
        for y in 1..9 {
            bm.set(5, y, true);
        }

        let collector = MinutiaCollector::new(&bm);
        let minutiae = collector.collect();

        // Find endpoints (neighbors == 1)
        let endings: Vec<_> = minutiae.iter()
            .filter(|m| m.typ == MinutiaType::Ending)
            .collect();
        let bifurcations: Vec<_> = minutiae.iter()
            .filter(|m| m.typ == MinutiaType::Bifurcation)
            .collect();

        assert!(endings.len() >= 2, "Should have at least 2 endpoints");
        assert!(bifurcations.is_empty(), "Should have no bifurcations");
    }

    #[test]
    fn test_minutia_collector_bifurcation() {
        // Create a T-junction (3 branches)
        let mut bm = BooleanMatrix::new(10, 10);

        // Vertical line
        for y in 1..9 {
            bm.set(5, y, true);
        }

        // Horizontal line (going left only from center)
        for x in 3..6 {
            bm.set(x, 5, true);
        }

        let collector = MinutiaCollector::new(&bm);
        let minutiae = collector.collect();

        // Should have 3 minutiae: 2 endpoints + 1 bifurcation
        assert!(minutiae.len() >= 3, "Should have at least 3 minutiae");
    }
}
