use super::consistent_skeleton_ridge::ConsistentSkeletonRidge;
use crate::features::SkeletonMinutia;
/// ConsistentSkeleton: skeleton data for transparency logging — mirrors .NET ConsistentSkeleton.cs.
use crate::primitives::int_point::IntPoint;

pub struct ConsistentSkeleton {
    pub width: i32,
    pub height: i32,
    pub minutiae: Vec<IntPoint>,
    pub ridges: Vec<ConsistentSkeletonRidge>,
}

impl ConsistentSkeleton {
    /// Build a ConsistentSkeleton from skeleton minutiae and ridges.
    pub fn of(minutiae: &[SkeletonMinutia], ridges: &[usize], size: &IntPoint) -> Self {
        let width = size.x();
        let height = size.y();

        // Collect minutiae positions
        let minutiae_pts: Vec<IntPoint> = minutiae.iter().map(|m| m.position).collect();

        // Build ridges (just use simple indices for now)
        let ridges_out: Vec<ConsistentSkeletonRidge> = ridges
            .iter()
            .map(|&ridx| ConsistentSkeletonRidge::new(ridx, ridx, vec![]))
            .collect();

        Self {
            width,
            height,
            minutiae: minutiae_pts,
            ridges: ridges_out,
        }
    }

    pub fn new(
        width: i32,
        height: i32,
        minutiae: Vec<IntPoint>,
        ridges: Vec<ConsistentSkeletonRidge>,
    ) -> Self {
        Self {
            width,
            height,
            minutiae,
            ridges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_skeleton_new() {
        let sk = ConsistentSkeleton::new(100, 100, vec![], vec![]);
        assert_eq!(sk.width, 100);
        assert_eq!(sk.height, 100);
        assert!(sk.minutiae.is_empty());
        assert!(sk.ridges.is_empty());
    }
}
