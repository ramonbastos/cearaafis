/// NeighborEdge: an edge representing a neighbor relationship — mirrors .NET NeighborEdge.cs.
use super::indexed_edge;

#[derive(Debug, Clone, PartialEq)]
pub struct NeighborEdge {
    pub indexed: indexed_edge::IndexedEdge,
    pub neighbor: Option<usize>,
}

impl NeighborEdge {
    pub fn new(indexed: indexed_edge::IndexedEdge) -> Self {
        Self {
            indexed,
            neighbor: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::edge_shape::EdgeShape;
    use super::*;
    use crate::primitives::short_point::ShortPoint;

    #[test]
    fn test_new() {
        let edge = NeighborEdge::new(indexed_edge::IndexedEdge::new(
            0,
            EdgeShape::new(vec![ShortPoint::new(1, 2)]),
        ));
        assert!(edge.neighbor.is_none());
    }
}
