/// Demo: load all test images, extract templates, match probe vs all.
use cearaafis::root::*;

fn load_png(path: &str) -> Option<FingerprintTemplate> {
    let opts = FingerprintImageOptions::default();
    FingerprintImage::from_png(path, &opts)
        .ok()
        .map(|img| img.to_template())
}

fn load_raw(path: &str, w: usize, h: usize, dpi: u32) -> Option<FingerprintTemplate> {
    let raw = std::fs::read(path).ok()?;
    let img = FingerprintImage::from_raw(raw, w, h, dpi);
    Some(img.to_template())
}

#[test]
fn match_probe_against_all() {
    let resources = "test_resources";
    let probe_path = format!("{}/probe.png", resources);

    let probe_tmpl = load_png(&probe_path).expect("probe.png must load");
    eprintln!("[PNG] probe — {} minutiae", probe_tmpl.minutiae.len());

    let mut matcher = FingerprintMatcher::new(probe_tmpl);

    let matching_path = format!("{}/matching.png", resources);
    if let Some(tmpl) = load_png(&matching_path) {
        eprintln!("[PNG] matching — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("matching".into(), tmpl);
    }

    let nonmatching_path = format!("{}/nonmatching.png", resources);
    if let Some(tmpl) = load_png(&nonmatching_path) {
        eprintln!("[PNG] nonmatching — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("nonmatching".into(), tmpl);
    }

    let jpeg_path = format!("{}/probe.jpeg", resources);
    if let Some(tmpl) = load_png(&jpeg_path) {
        eprintln!("[JPEG] probe — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("probe-jpeg".into(), tmpl);
    }

    let bmp_path = format!("{}/probe.bmp", resources);
    if let Some(tmpl) = load_png(&bmp_path) {
        eprintln!("[BMP] probe — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("probe-bmp".into(), tmpl);
    }

    let raw_probe = format!("{}/gray-probe.dat", resources);
    if let Some(tmpl) = load_raw(&raw_probe, 332, 533, 500) {
        eprintln!("[RAW] gray-probe — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("gray-probe".into(), tmpl);
    }

    let raw_matching = format!("{}/gray-matching.dat", resources);
    if let Some(tmpl) = load_raw(&raw_matching, 352, 370, 500) {
        eprintln!("[RAW] gray-matching — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("gray-matching".into(), tmpl);
    }

    let raw_nonmatching = format!("{}/gray-nonmatching.dat", resources);
    if let Some(tmpl) = load_raw(&raw_nonmatching, 435, 333, 500) {
        eprintln!("[RAW] gray-nonmatching — {} minutiae", tmpl.minutiae.len());
        matcher.add_candidate("gray-nonmatching".into(), tmpl);
    }

    let results = matcher.match_all();
    let max_width = results.iter().map(|(id, _)| id.len()).max().unwrap_or(0);
    let pad = max_width.max(10);

    eprintln!("\n=== MATCH RESULTS (probe.png vs all candidates) ===\n");
    eprintln!(
        "{:<pad$} | {:>4} | {:>8} | {:>6} | {}",
        "Candidate", "Min", "Score", "Pct%", "Status"
    );
    eprintln!(
        "{:<pad$}+{:<4}+{:<8}+{:<6}+{}",
        "---", "----", "-------", "------", "------"
    );

    for (id, score) in &results {
        let score = *score;
        let status = if score >= 40.0 {
            "MATCH"
        } else if score >= 20.0 {
            "AMBIGUOUS"
        } else {
            "NO MATCH"
        };
        eprintln!(
            "{:<pad$} | {:>4} | {:>8.1} | {:>6.1}% | {}",
            id, 0, score, score, status
        );
    }

    let probe_tmpl2 = load_png(&probe_path).expect("probe.png must load again");
    let matcher2 = FingerprintMatcher::new(probe_tmpl2);
    let self_score = matcher2.match_with_template(&matcher.template);
    eprintln!(
        "\n(Self-match: probe.png vs probe.png): {:.1} ({:.1}%)",
        self_score, self_score
    );

    let match_score = matcher.match_with_id("matching");
    eprintln!("\nmatching.png score: {:.1} (threshold >= 40)", match_score);
    let nonmatch_score = matcher.match_with_id("nonmatching");
    eprintln!(
        "nonmatching.png score: {:.1} (threshold <= 20)",
        nonmatch_score
    );

    assert!(
        match_score >= nonmatch_score,
        "matching score ({:.1}) should exceed nonmatching ({:.1})",
        match_score,
        nonmatch_score
    );
}
