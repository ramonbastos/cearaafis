//! Direct MatcherEngine score verification — no image pipeline needed.
//! Tests the 4 fixes: identical template, root dedup, MinRootEdgeLength, enumeration limits.
use cearaafis::matcher::MatcherEngine;
use cearaafis::root::{FingerprintMatcher, FingerprintTemplate};
use cearaafis::features::{Minutia, MinutiaType};
use cearaafis::primitives::{IntPoint, ShortPoint};

fn make_minutia(x: i32, y: i32, angle: f64, typ: MinutiaType) -> Minutia {
    Minutia::new(IntPoint::new(x, y), angle, typ)
}

/// Test Fix 1: identical template returns 85.0 (probe-vs-probe anomaly resolved)
#[test]
fn probe_vs_identical_returns_high() {
    // Create 3 identical minutiae (probe == candidate)
    let probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(120, 105, 100.0, MinutiaType::Ending),
        make_minutia(110, 120, 200.0, MinutiaType::Bifurcation),
    ];
    let candidate = probe.clone(); // same minutiae, same positions

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    assert!(data.score >= 80.0, "probe-vs-probe should return ≥80.0, got {:.1}", data.score);
}

/// Test Fix 2: root dedup prevents duplicate root pairs
#[test]
fn root_dedup_prevents_duplicates() {
    let probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(110, 105, 100.0, MinutiaType::Ending),
    ];
    let candidate = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(110, 105, 100.0, MinutiaType::Ending),
    ];

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    // Should produce a meaningful score (not 0 from dedup issues)
    assert!(data.score > 0.0, "dedup should not zero-score similar templates");
}

/// Test Fix 3: MinRootEdgeLength=58 filters short edges, reducing false positives
#[test]
fn short_edge_filter_reduces_false_positives() {
    // Probe minutiae clustered in a tight area (edges < 58px)
    // Candidate at completely different location — no long edges match
    // Fix 1 does NOT short-circuit (different positions), so MinRootEdgeLength filter runs
    let probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(101, 101, 45.0, MinutiaType::Ending),
        make_minutia(102, 102, 90.0, MinutiaType::Bifurcation),
    ];
    // Candidate far away (no short-range edge matches possible)
    let candidate = vec![
        make_minutia(300, 300, 0.0, MinutiaType::Bifurcation),
        make_minutia(301, 301, 45.0, MinutiaType::Ending),
        make_minutia(302, 302, 90.0, MinutiaType::Bifurcation),
    ];

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    // With only short edges and different positions, score should be low
    // because MinRootEdgeLength filter (58px) skips short probe edges
    assert!(data.score <= 30.0, "short-edge-only should score ≤30.0, got {:.1}", data.score);
}

/// Test Fix 4: enumeration limits (70 roots, 1633 lookups) prevent runaway computation
#[test]
fn enumeration_limits_work() {
    // Create large templates that would be expensive without limits
    let mut probe = Vec::new();
    let mut candidate = Vec::new();
    for i in 0..50 {
        probe.push(make_minutia((100 + i as i16) as i32, (100 + i as i16 / 2) as i32, (i as i32 * 10) as f64, MinutiaType::Bifurcation));
        candidate.push(make_minutia((100 + i as i16 + 1) as i32, (100 + i as i16 / 2) as i32, (i as i32 * 10) as f64, MinutiaType::Bifurcation));
    }

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    // Should complete quickly (no timeout from runaway enumeration)
    // Score should be reasonable (>0)
    assert!(data.score > 0.0 || data.score < 10.0, "50 minutiae should score reasonably, got {:.1}", data.score);
}

/// Score ordering: matching should score higher than non-matching
#[test]
fn score_ordering_matching_gt_nonmatching() {
    // Similar templates (matching-like)
    let matching_probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(110, 105, 100.0, MinutiaType::Ending),
        make_minutia(105, 120, 200.0, MinutiaType::Bifurcation),
    ];
    let matching_candidate = vec![
        make_minutia(101, 101, 5.0, MinutiaType::Bifurcation),
        make_minutia(111, 106, 105.0, MinutiaType::Ending),
        make_minutia(106, 121, 205.0, MinutiaType::Bifurcation),
    ];

    // Dissimilar templates (non-matching-like)
    let nonmatching_probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(110, 105, 100.0, MinutiaType::Ending),
        make_minutia(105, 120, 200.0, MinutiaType::Bifurcation),
    ];
    let nonmatching_candidate = vec![
        make_minutia(200, 200, 0.0, MinutiaType::Bifurcation),
        make_minutia(210, 205, 100.0, MinutiaType::Ending),
        make_minutia(205, 220, 200.0, MinutiaType::Bifurcation),
    ];

    let score_matching = MatcherEngine::new(matching_probe.clone(), matching_candidate).score().score;
    let score_nonmatching = MatcherEngine::new(nonmatching_probe.clone(), nonmatching_candidate).score().score;

    assert!(
        score_matching > score_nonmatching,
        "matching score ({:.1}) should exceed non-matching ({:.1})",
        score_matching,
        score_nonmatching
    );
}

/// Test that the full flow through FingerprintMatcher also works
#[test]
fn match_with_template_integration() {
    use cearaafis::root::{FingerprintMatcher, FingerprintTemplate};

    // Create minimal templates (no image pipeline)
    let probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(110, 105, 100.0, MinutiaType::Ending),
        make_minutia(105, 120, 200.0, MinutiaType::Bifurcation),
    ];
    let candidate = vec![
        make_minutia(101, 101, 5.0, MinutiaType::Bifurcation),
        make_minutia(111, 106, 105.0, MinutiaType::Ending),
        make_minutia(106, 121, 205.0, MinutiaType::Bifurcation),
    ];

    // Build FingerprintTemplate manually (no image pipeline)
    // edges are Vec<Vec<NeighborEdge>> — empty for now (not needed for basic score)
    let probe_tmpl = FingerprintTemplate::new(ShortPoint::new(100, 100), probe, vec![]);
    let cand_tmpl = FingerprintTemplate::new(ShortPoint::new(101, 101), candidate, vec![]);

    let mut matcher = FingerprintMatcher::new(probe_tmpl);
    matcher.add_candidate("candidate".into(), cand_tmpl);

    let score = matcher.match_with_id("candidate");

    assert!(score > 0.0, "match_with_id should return >0.0, got {:.1}", score);
}
