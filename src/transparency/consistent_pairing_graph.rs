/// ConsistentPairingGraph: pairing graph data for transparency logging — mirrors .NET ConsistentPairingGraph.cs.

use super::consistent_edge_pair::ConsistentEdgePair;
use super::consistent_minutia_pair::ConsistentMinutiaPair;

pub struct ConsistentPairingGraph {
    pub root: ConsistentMinutiaPair,
    pub tree: Vec<ConsistentEdgePair>,
    pub support: Vec<ConsistentEdgePair>,
}

impl ConsistentPairingGraph {
    pub fn new(root: ConsistentMinutiaPair, tree: Vec<ConsistentEdgePair>, support: Vec<ConsistentEdgePair>) -> Self {
        Self { root, tree, support }
    }
}

impl serde::Serialize for ConsistentPairingGraph {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ConsistentPairingGraph", 3)?;
        state.serialize_field("Root", &self.root)?;
        state.serialize_field("Tree", &&self.tree)?;
        state.serialize_field("Support", &&self.support)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_pairing_graph_new() {
        let root = ConsistentMinutiaPair::new(0, 0);
        let tree = vec![ConsistentEdgePair::new(0, 1, 0, 1)];
        let support = vec![ConsistentEdgePair::new(0, 1, 0, 1)];
        let graph = ConsistentPairingGraph::new(root, tree, support);

        assert_eq!(graph.root.probe, 0);
        assert_eq!(graph.root.candidate, 0);
        assert_eq!(graph.tree.len(), 1);
        assert_eq!(graph.support.len(), 1);
    }

    #[test]
    fn test_consistent_pairing_graph_empty() {
        let graph = ConsistentPairingGraph::new(
            ConsistentMinutiaPair::new(0, 0),
            vec![],
            vec![],
        );

        assert!(graph.tree.is_empty());
        assert!(graph.support.is_empty());
    }
}
