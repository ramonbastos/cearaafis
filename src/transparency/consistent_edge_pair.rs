/// ConsistentEdgePair: an edge pair for transparency logging — mirrors .NET ConsistentEdgePair.cs.

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsistentEdgePair {
    pub probe_from: usize,
    pub probe_to: usize,
    pub candidate_from: usize,
    pub candidate_to: usize,
}

impl ConsistentEdgePair {
    pub fn new(
        probe_from: usize,
        probe_to: usize,
        candidate_from: usize,
        candidate_to: usize,
    ) -> Self {
        Self {
            probe_from,
            probe_to,
            candidate_from,
            candidate_to,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_edge_pair_new() {
        let pair = ConsistentEdgePair::new(1, 2, 3, 4);
        assert_eq!(pair.probe_from, 1);
        assert_eq!(pair.probe_to, 2);
        assert_eq!(pair.candidate_from, 3);
        assert_eq!(pair.candidate_to, 4);
    }
}
