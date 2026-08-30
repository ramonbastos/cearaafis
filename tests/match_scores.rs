/// Quick scoring tests — prints actual MatcherEngine scores for real fingerprint images.
use cearaafis::*;

fn load_png(name: &str) -> FingerprintImage {
    let path = format!("test_resources/{}", name);
    let opts = FingerprintImageOptions::default();
    FingerprintImage::from_png(&path, &opts).expect(&format!("Failed to load PNG: {}", path))
}

fn load_jpeg(name: &str) -> FingerprintImage {
    let path = format!("test_resources/{}", name);
    let opts = FingerprintImageOptions::default();
    FingerprintImage::from_jpeg(&path, &opts).expect(&format!("Failed to load JPEG: {}", path))
}

fn score_probe_vs(name_probe: &str, name_other: &str, label: &str) {
    let probe = load_png(name_probe);
    let probe_tmpl = probe.to_template();
    let other = load_png(name_other);
    let other_tmpl = other.to_template();

    let score = FingerprintMatcher::new(probe_tmpl.clone()).match_with_template(&other_tmpl);

    eprintln!(
        "{}: score={:.1}, probe_minutiae={}, other_minutiae={}",
        label,
        score,
        probe_tmpl.minutiae.len(),
        other_tmpl.minutiae.len(),
    );
}

fn score_probe_vs_jpeg(name_probe: &str, name_other: &str, label: &str) {
    let probe = load_png(name_probe);
    let probe_tmpl = probe.to_template();
    let other = load_jpeg(name_other);
    let other_tmpl = other.to_template();

    let score = FingerprintMatcher::new(probe_tmpl.clone()).match_with_template(&other_tmpl);

    eprintln!(
        "{}: score={:.1}, probe_minutiae={}, jpeg_minutiae={}",
        label,
        score,
        probe_tmpl.minutiae.len(),
        other_tmpl.minutiae.len(),
    );
}

#[test]
fn scores_probe_vs_matching() {
    score_probe_vs("probe.png", "matching.png", "probe-vs-matching");
}

#[test]
fn scores_probe_vs_nonmatching() {
    score_probe_vs("probe.png", "nonmatching.png", "probe-vs-nonmatching");
}

#[test]
fn scores_probe_vs_self() {
    score_probe_vs("probe.png", "probe.png", "probe-vs-probe");
}

#[test]
fn scores_probe_jpeg_vs_matching() {
    score_probe_vs_jpeg("probe.png", "matching.png", "probe.jpeg-vs-matching");
}
