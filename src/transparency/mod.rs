mod consistent_edge_pair;
mod consistent_hash_entry;
mod consistent_minutia_pair;
mod consistent_pairing_graph;
mod consistent_skeleton;
mod consistent_skeleton_ridge;
mod no_transparency;
mod serialization_utils;

pub use consistent_edge_pair::*;
pub use consistent_hash_entry::*;
pub use consistent_minutia_pair::*;
pub use consistent_pairing_graph::*;
pub use consistent_skeleton::*;
pub use consistent_skeleton_ridge::*;
pub use no_transparency::*;
pub use serialization_utils::*;

/// FingerprintTransparency: trait for optional debug/logging during matching.
pub trait FingerprintTransparency: Send {
    fn accepts(&self, key: &str) -> bool;

    fn take(&mut self, key: &str, mime: &str, data: &[u8]);

    fn log<T: serde::Serialize>(&mut self, key: &str, data: &T);

    fn log_skeleton(&mut self, keyword: &str, skeleton: &ConsistentSkeleton);

    fn log_edge_hash(&mut self, hash: &ConsistentHashEntry);

    fn log_root_pairs(&mut self, count: usize, roots: &[ConsistentMinutiaPair]);

    fn log_pairing(&mut self, pairing: &ConsistentPairingGraph);

    fn log_best_pairing(&mut self, pairing: &ConsistentPairingGraph);

    fn log_score(&mut self, score: &ScoringData);

    fn log_best_score(&mut self, score: &ScoringData);

    fn log_best_match(&mut self, nth: usize);
}

/// ScoringData: simple score container for transparency logging.
pub struct ScoringData {
    pub score: f64,
}

impl ScoringData {
    pub fn new(score: f64) -> Self {
        Self { score }
    }
}

impl serde::Serialize for ScoringData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ScoringData", 1)?;
        state.serialize_field("Score", &self.score)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scoring_data_new() {
        let sd = ScoringData::new(42.5);
        assert_eq!(sd.score, 42.5);
    }

    #[test]
    fn test_no_transparency_accepts_false() {
        let nt = NoTransparency;
        assert!(!nt.accepts("test_key"));
    }

    #[test]
    fn test_no_transparency_instance() {
        let instance = NoTransparency::instance();
        assert!(!instance.accepts("any_key"));
    }

    #[test]
    fn test_to_cbor() {
        #[derive(serde::Serialize)]
        struct TestStruct {
            value: i32,
        }

        let obj = TestStruct { value: 42 };
        let bytes = serialization_utils::to_cbor(&obj).expect("should serialize");
        assert!(!bytes.is_empty());
    }
}
