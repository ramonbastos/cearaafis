/// ConsistentSkeletonRidge: ridge data for transparency logging — mirrors .NET ConsistentSkeletonRidge.cs.

use crate::primitives::int_point::IntPoint;

pub struct ConsistentSkeletonRidge {
    pub start: usize,
    pub end: usize,
    pub points: Vec<IntPoint>,
}

impl ConsistentSkeletonRidge {
    pub fn new(start: usize, end: usize, points: Vec<IntPoint>) -> Self {
        Self { start, end, points }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let rid = ConsistentSkeletonRidge::new(0, 10, vec![]);
        assert_eq!(rid.start, 0);
        assert_eq!(rid.end, 10);
        assert!(rid.points.is_empty());
    }

    #[test]
    fn test_new_with_points() {
        let rid = ConsistentSkeletonRidge::new(0, 10, vec![IntPoint::new(1, 2), IntPoint::new(3, 4)]);
        assert_eq!(rid.points.len(), 2);
        assert_eq!(rid.points[0].x(), 1);
        assert_eq!(rid.points[1].y(), 4);
    }
}
