/// ConsistentMinutiaPair: a paired minutia for transparency logging — mirrors .NET ConsistentMinutiaPair.cs.

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConsistentMinutiaPair {
    pub probe: usize,
    pub candidate: usize,
}

impl ConsistentMinutiaPair {
    pub fn new(probe: usize, candidate: usize) -> Self {
        Self { probe, candidate }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_minutia_pair_new() {
        let pair = ConsistentMinutiaPair::new(5, 10);
        assert_eq!(pair.probe, 5);
        assert_eq!(pair.candidate, 10);
    }
}
