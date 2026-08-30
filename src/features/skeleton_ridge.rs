/// SkeletonRidge: a ridge on the skeleton — mirrors .NET SkeletonRidge.cs.
use crate::primitives::short_point::ShortPoint;

#[derive(Clone, serde::Serialize)]
pub struct SkeletonRidge {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub shape: Vec<ShortPoint>,
    pub angle: f64,
}

impl SkeletonRidge {
    pub fn new(shape: Vec<ShortPoint>) -> Self {
        Self {
            start: None,
            end: None,
            shape,
            angle: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let rid = SkeletonRidge::new(vec![ShortPoint::new(0, 0)]);
        assert!(rid.start.is_none());
        assert!(rid.end.is_none());
    }
}
