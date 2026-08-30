//! Direct MatcherEngine score verification — no image pipeline needed.
//! Tests root enumeration, dedup, MinRootEdgeLength, enumeration limits.
//!
//! NOTE: synthetic templates must use minutiae pairs with edge length well
//! above MinRootEdgeLength (58px) or no root pairs form and a 0.0 score is
//! the legitimate result (see AGENTS.md Key Lessons).
use cearaafis::features::{Minutia, MinutiaType};
use cearaafis::matcher::MatcherEngine;
use cearaafis::primitives::{IntPoint, ShortPoint};
use cearaafis::root::{FingerprintMatcher, FingerprintTemplate};

fn make_minutia(x: i32, y: i32, angle: f64, typ: MinutiaType) -> Minutia {
    Minutia::new(IntPoint::new(x, y), angle, typ)
}

/// Well-separated minutiae cluster (edges > 58px) matching across templates.
fn matching_cluster(offset: i32, jitter: i32) -> Vec<Minutia> {
    vec![
        make_minutia(
            100 + offset + jitter / 2,
            100 + jitter / 3,
            jitter as f64 * 0.01,
            MinutiaType::Bifurcation,
        ),
        make_minutia(
            180 + offset - jitter / 2,
            150 + jitter / 3,
            100.0 - jitter as f64 * 0.01,
            MinutiaType::Ending,
        ),
        make_minutia(
            120 + offset + jitter / 3,
            220 - jitter / 2,
            200.0 + jitter as f64 * 0.01,
            MinutiaType::Bifurcation,
        ),
        make_minutia(
            250 + offset - jitter / 3,
            180 + jitter / 2,
            300.0 - jitter as f64 * 0.01,
            MinutiaType::Ending,
        ),
    ]
}

/// Test Fix 1: identical template returns a high score.
#[test]
fn probe_vs_identical_returns_high() {
    let probe = matching_cluster(0, 0);
    let candidate = probe.clone(); // same minutiae, same positions

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    assert!(
        data.score >= 10.0,
        "probe-vs-probe (4 minutiae) should return ≥10.0, got {:.1}. \
         Note: score scales with minutiae count (see AGENTS.md lessons) — \
         full-image templates score ~100.",
        data.score
    );
}

/// Test Fix 2: root dedup prevents duplicate root pairs
#[test]
fn root_dedup_prevents_duplicates() {
    let probe = matching_cluster(0, 0);
    let candidate = matching_cluster(0, 2);

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    // Should produce a meaningful score (not 0 from dedup issues)
    assert!(
        data.score > 0.0,
        "dedup should not zero-score similar templates"
    );
}

/// Test Fix 3: MinRootEdgeLength=58 filters short edges, reducing false positives
#[test]
fn short_edge_filter_reduces_false_positives() {
    // Probe minutiae clustered in a tight area (edges < 58px)
    // Candidate at completely different location — no long edges match
    // Fix 1 does NOT short-circuit (different positions), so MinRootEdgeLength filter runs
    let probe = vec![
        make_minutia(100, 100, 0.0, MinutiaType::Bifurcation),
        make_minutia(110, 105, 100.0, MinutiaType::Ending),
        make_minutia(105, 120, 200.0, MinutiaType::Bifurcation),
    ];
    let candidate = vec![
        make_minutia(300, 300, 0.0, MinutiaType::Bifurcation),
        make_minutia(310, 305, 100.0, MinutiaType::Ending),
        make_minutia(305, 320, 200.0, MinutiaType::Bifurcation),
    ];

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    // With no long matching edges, score should stay below any match threshold.
    assert!(
        data.score < 20.0,
        "tight-cluster probe vs distant candidate should score <20, got {:.1}",
        data.score
    );
}

/// Enumeration limits: many minutiae must not hang the matcher.
#[test]
fn large_templates_complete_quickly() {
    let probe: Vec<Minutia> = (0..50)
        .map(|i| {
            make_minutia(
                50 + (i % 10) * 60,
                50 + (i / 10) * 60,
                (i * 37) as f64 % 360.0,
                MinutiaType::Bifurcation,
            )
        })
        .collect();
    let candidate: Vec<Minutia> = probe
        .iter()
        .enumerate()
        .map(|(i, m)| {
            make_minutia(
                m.position.x() + if i % 2 == 0 { 1 } else { -1 },
                m.position.y() + if i % 3 == 0 { 1 } else { -1 },
                m.angle + 1.0,
                MinutiaType::Bifurcation,
            )
        })
        .collect();

    let engine = MatcherEngine::new(probe, candidate);
    let data = engine.score();

    // Should complete quickly (no timeout from runaway enumeration).
    // Score must be a finite number.
    assert!(
        data.score.is_finite(),
        "50 minutiae should complete with finite score, got {:.1}",
        data.score
    );
}

/// Score ordering: matching should score higher than non-matching
#[test]
fn score_ordering_matching_gt_nonmatching() {
    // Similar templates (matching-like): same cluster, small jitter
    let matching_probe = matching_cluster(0, 0);
    let matching_candidate = matching_cluster(1, 3);

    // Dissimilar templates: same geometry but far apart — relative geometry
    // (edge lengths/angles) is identical, so to be truly non-matching we
    // rotate/scale the pattern differently (angles mirrored).
    let nonmatching_probe = matching_cluster(0, 0);
    let nonmatching_candidate: Vec<Minutia> = matching_cluster(0, 0)
        .into_iter()
        .map(|m| make_minutia(m.position.x(), 600 - m.position.y(), 360.0 - m.angle, m.typ))
        .collect();

    let score_matching = MatcherEngine::new(matching_probe.clone(), matching_candidate)
        .score()
        .score;
    let score_nonmatching = MatcherEngine::new(nonmatching_probe.clone(), nonmatching_candidate)
        .score()
        .score;

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
    // Templates with well-separated minutiae (edges > MinRootEdgeLength).
    let probe = matching_cluster(0, 0);
    let candidate = matching_cluster(1, 2);

    // Build FingerprintTemplate manually (no image pipeline).
    let probe_tmpl = FingerprintTemplate::new(ShortPoint::new(300, 300), probe, vec![]);
    let cand_tmpl = FingerprintTemplate::new(ShortPoint::new(300, 300), candidate, vec![]);

    let mut matcher = FingerprintMatcher::new(probe_tmpl);
    matcher.add_candidate("candidate".into(), cand_tmpl);

    let score = matcher.match_with_id("candidate");

    assert!(
        score > 0.0,
        "match_with_id should return >0.0, got {:.1}",
        score
    );
}
