/// NoTransparency: a transparency implementation that discards all data — mirrors .NET NoTransparency.cs.

use super::{ConsistentHashEntry, ConsistentMinutiaPair, ConsistentPairingGraph, ConsistentSkeleton, FingerprintTransparency, ScoringData};

pub struct NoTransparency;

impl NoTransparency {
    pub fn instance() -> &'static NoTransparency {
        &NO_TRANS
    }
}

static NO_TRANS: NoTransparency = NoTransparency;

impl FingerprintTransparency for NoTransparency {
    fn accepts(&self, _key: &str) -> bool {
        false
    }

    fn take(&mut self, _key: &str, _mime: &str, _data: &[u8]) {}

    fn log<T: serde::Serialize>(&mut self, _key: &str, _data: &T) {}

    fn log_skeleton(&mut self, _keyword: &str, _skeleton: &ConsistentSkeleton) {}

    fn log_edge_hash(&mut self, _hash: &ConsistentHashEntry) {}

    fn log_root_pairs(&mut self, _count: usize, _roots: &[ConsistentMinutiaPair]) {}

    fn log_pairing(&mut self, _pairing: &ConsistentPairingGraph) {}

    fn log_best_pairing(&mut self, _pairing: &ConsistentPairingGraph) {}

    fn log_score(&mut self, _score: &ScoringData) {}

    fn log_best_score(&mut self, _score: &ScoringData) {}

    fn log_best_match(&mut self, _nth: usize) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_transparency_accepts_returns_false() {
        let nt = NoTransparency;
        assert!(!NoTransparency::accepts(&nt, "test_key"));
    }

    #[test]
    fn test_no_transparency_instance() {
        let instance = NoTransparency::instance();
        assert!(!instance.accepts("any_key"));
    }
}
