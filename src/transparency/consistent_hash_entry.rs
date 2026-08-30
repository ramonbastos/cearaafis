/// ConsistentHashEntry: hash entry data for transparency logging — mirrors .NET ConsistentHashEntry.cs.

use crate::features::IndexedEdge;

pub struct ConsistentHashEntry {
    pub key: i32,
    pub edges: Vec<IndexedEdge>,
}

impl ConsistentHashEntry {
    pub fn new(key: i32, edges: Vec<IndexedEdge>) -> Self {
        Self { key, edges }
    }
}

impl serde::Serialize for ConsistentHashEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ConsistentHashEntry", 2)?;
        state.serialize_field("Key", &self.key)?;
        state.serialize_field("Edges", &&self.edges)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::short_point::ShortPoint;
    use crate::features::EdgeShape;

    #[test]
    fn test_consistent_hash_entry_new() {
        let edge = IndexedEdge::new(0, EdgeShape::new(vec![ShortPoint::new(1, 2)]));
        let entry = ConsistentHashEntry::new(42, vec![edge]);
        assert_eq!(entry.key, 42);
        assert_eq!(entry.edges.len(), 1);
    }
}
