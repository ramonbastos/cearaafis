/// IndexedEdge: an edge with an index — mirrors .NET IndexedEdge.cs.
use super::edge_shape::EdgeShape;

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct IndexedEdge {
    pub index: usize,
    pub shape: EdgeShape,
}

impl IndexedEdge {
    pub fn new(index: usize, shape: EdgeShape) -> Self {
        Self { index, shape }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::short_point::ShortPoint;

    #[test]
    fn test_new() {
        let edge = IndexedEdge::new(0, EdgeShape::new(vec![ShortPoint::new(1, 2)]));
        assert_eq!(edge.index, 0);
        assert_eq!(edge.shape.points.len(), 1);
    }
}
