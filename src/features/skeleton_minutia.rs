/// SkeletonMinutia: skeleton point with attached ridges — mirrors .NET SkeletonMinutia.cs.
use crate::primitives::int_point::IntPoint;

#[derive(Clone, serde::Serialize)]
pub struct SkeletonMinutia {
    pub position: IntPoint,
    pub ridges: Vec<usize>,
}

impl SkeletonMinutia {
    pub fn new(position: IntPoint) -> Self {
        Self {
            position,
            ridges: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let m = SkeletonMinutia::new(IntPoint::new(10, 20));
        assert_eq!(m.position.x(), 10);
        assert_eq!(m.position.y(), 20);
        assert!(m.ridges.is_empty());
    }
}
