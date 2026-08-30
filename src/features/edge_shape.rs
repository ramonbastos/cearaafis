/// EdgeShape: shape representation for edges — mirrors .NET EdgeShape.cs.
use crate::primitives::short_point::ShortPoint;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct EdgeShape {
    pub points: Vec<ShortPoint>,
}

impl EdgeShape {
    pub fn new(points: Vec<ShortPoint>) -> Self {
        Self { points }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let shape = EdgeShape::new(vec![]);
        assert!(shape.points.is_empty());
    }
}
