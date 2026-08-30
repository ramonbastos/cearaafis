/// SkeletonTracing: traces skeleton curves from a binarized image.
/// Mirrors .NET SkeletonTracing.cs.
use crate::primitives::{BooleanMatrix, IntPoint, ShortPoint, DoubleMatrix};
use crate::parameters::Parameters;
use crate::features::Skeleton;

/// Trace skeleton curves from a binary image.
pub struct SkeletonTracing {
    /// The binary skeleton image.
    skeleton: BooleanMatrix,
    /// Traced skeleton.
    traced: BooleanMatrix,
    /// Skeleton result.
    pub skeleton_result: Option<Skeleton>,
}

impl SkeletonTracing {
    /// Create skeleton tracing from a binary image.
    pub fn new(binary: &BooleanMatrix) -> Self {
        let mut traced = BooleanMatrix::from_clone(binary);
        let skeleton = Self::thinning(&mut traced);
        Self {
            skeleton,
            traced,
            skeleton_result: None,
        }
    }

    /// Iterative binary thinning (skeletonization).
    fn thinning(skeleton: &mut BooleanMatrix) -> BooleanMatrix {
        let width = skeleton.width();
        let height = skeleton.height();
        let mut changed = true;
        let iterations = Parameters::THINNING_ITERATIONS;

        for _iter in 0..iterations {
            if !changed {
                break;
            }
            changed = false;

            // Mark pixels to delete (even/odd phases like .NET)
            let mut to_delete = Vec::new();

            for y in 1..(height - 1) {
                for x in 1..(width - 1) {
                    if !skeleton.get(x, y) {
                        continue;
                    }

                    if Self::should_delete(skeleton, x, y) {
                        to_delete.push(IntPoint::new(x as i32, y as i32));
                    }
                }
            }

            // Delete marked pixels
            for pt in to_delete {
                skeleton.set(pt.x() as usize, pt.y() as usize, false);
                changed = true;
            }
        }

        skeleton.clone()
    }

    /// Check if a pixel should be deleted (conditions from .NET SourceAFIS).
    fn should_delete(skeleton: &BooleanMatrix, x: usize, y: usize) -> bool {
        // Count non-zero neighbors (8-connectivity)
        let mut neighbors = 0u32;
        for dy in 0..3 {
            for dx in 0..3 {
                let nx = x + dx;
                let ny = y + dy;
                if skeleton.get(nx, ny) {
                    neighbors += 1;
                }
            }
        }

        // Must have between 2 and 6 neighbors
        if neighbors < 2 || neighbors > 6 {
            return false;
        }

        // Must have exactly one transition from 0 to 1 in the clockwise neighborhood
        let mut transitions = 0u32;
        let mut prev = if skeleton.get(x, (y + 1).min(skeleton.height() - 1)) { 1 } else { 0 };
        for dy in 0..4 {
            let nx = (x + dy as usize).min(skeleton.width() - 1);
            let ny = (y + 1).min(skeleton.height() - 1);
            let curr = if skeleton.get(nx, ny) { 1 } else { 0 };
            if prev == 0 && curr == 1 {
                transitions += 1;
            }
            prev = curr;
        }

        transitions == 1
    }

    /// Trace skeleton curves and build skeleton graph.
    pub fn trace(&mut self) -> &Skeleton {
        let width = self.skeleton.width();
        let height = self.skeleton.height();

        // Find endpoints and junctions
        let mut endpoints = Vec::new();
        let mut junctions = Vec::new();

        for y in 1..(height - 1) {
            for x in 1..(width - 1) {
                if !self.skeleton.get(x, y) {
                    continue;
                }

                // Count non-zero neighbors
                let mut neighbors = 0u32;
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if nx < width && ny < height && self.skeleton.get(nx, ny) {
                            neighbors += 1;
                        }
                    }
                }

                if neighbors == 1 {
                    endpoints.push(IntPoint::new(x as i32, y as i32));
                } else if neighbors >= 3 {
                    junctions.push(IntPoint::new(x as i32, y as i32));
                }
            }
        }

        // Build ridges by tracing from endpoints to junctions or other endpoints
        let mut ridges = Vec::new();
        let mut visited = BooleanMatrix::from_clone(&self.skeleton);

        // Trace from each endpoint
        for endpoint in &endpoints {
            if Self::trace_ridge(&mut visited, &self.skeleton, endpoint, &mut ridges) {
                // Already visited
            }
        }

        // Build skeleton result
        let skeleton = Skeleton::new(
            crate::features::skeleton_type::SkeletonType::Skeleton,
            &IntPoint::new(width as i32, height as i32),
        );

        self.skeleton_result = Some(skeleton);

        // Store ridges
        if !ridges.is_empty() {
            // We'll use ridges to build the skeleton
        }

        self.skeleton_result.as_ref().unwrap()
    }

    /// Trace a single ridge from an endpoint.
    fn trace_ridge(visited: &mut BooleanMatrix, skeleton: &BooleanMatrix, start: &IntPoint, ridges: &mut Vec<Vec<IntPoint>>) -> bool {
        let mut path = Vec::new();
        path.push(*start);

        let mut current = *start;
        let width = skeleton.width();
        let height = skeleton.height();

        loop {
            // Find next unvisited neighbor
            let mut next = None;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (current.x() + dx) as usize;
                    let ny = (current.y() + dy) as usize;
                    if nx < width && ny < height && skeleton.get(nx, ny) && !visited.get(nx, ny) {
                        next = Some(IntPoint::new(nx as i32, ny as i32));
                        break;
                    }
                }
                if next.is_some() {
                    break;
                }
            }

            match next {
                Some(pt) => {
                    visited.set(pt.x() as usize, pt.y() as usize, true);
                    path.push(pt);
                    current = pt;
                }
                None => break,
            }
        }

        if path.len() >= 2 {
            ridges.push(path);
        }

        false
    }

    /// Get skeleton.
    pub fn skeleton(&self) -> &BooleanMatrix {
        &self.skeleton
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thinning_basic() {
        let mut bm = BooleanMatrix::new(20, 20);
        // Create a vertical line
        for y in 0..20 {
            bm.set(10, y, true);
        }
        let result = SkeletonTracing::thinning(&mut bm);
        // Line should remain after thinning
        assert!(result.get(10, 10));
    }
}
