/// Debug: print raw match counts per candidate
use cearaafis::root::*;

fn load_png(path: &str) -> Option<FingerprintTemplate> {
    let opts = FingerprintImageOptions::default();
    FingerprintImage::from_png(path, &opts).ok().map(|img| img.to_template())
}

#[test]
fn debug_match_counts() {
    let resources = "test_resources";
    let probe_path = format!("{}/probe.png", resources);

    let probe_tmpl = load_png(&probe_path).expect("probe.png");
    eprintln!("[PNG] probe — {} minutiae", probe_tmpl.minutiae.len());

    let candidates: Vec<(&str, String)> = vec![
        ("matching", format!("{}/matching.png", resources)),
        ("nonmatching", format!("{}/nonmatching.png", resources)),
        ("probe-bmp", format!("{}/probe.bmp", resources)),
    ];

    for (id, path) in &candidates {
        if let Some(tmpl) = load_png(path.as_str()) {
            eprintln!("[PNG] {} — {} minutiae", id, tmpl.minutiae.len());

            let probe_xs = probe_tmpl.minutiae.iter().map(|m| m.position.x()).collect::<Vec<_>>();
            let probe_ys = probe_tmpl.minutiae.iter().map(|m| m.position.y()).collect::<Vec<_>>();
            let cand_xs = tmpl.minutiae.iter().map(|m| m.position.x()).collect::<Vec<_>>();
            let cand_ys = tmpl.minutiae.iter().map(|m| m.position.y()).collect::<Vec<_>>();

            eprintln!(
                "  probe: x=[{}, {}] y=[{}, {}]",
                probe_xs.iter().min().unwrap(),
                probe_xs.iter().max().unwrap(),
                probe_ys.iter().min().unwrap(),
                probe_ys.iter().max().unwrap(),
            );
            eprintln!(
                "  cand {}: x=[{}, {}] y=[{}, {}]",
                id,
                cand_xs.iter().min().unwrap(),
                cand_xs.iter().max().unwrap(),
                cand_ys.iter().min().unwrap(),
                cand_ys.iter().max().unwrap(),
            );

            let mut matcher = FingerprintMatcher::new(probe_tmpl.clone());
            matcher.add_candidate((*id).to_string(), tmpl);
            let score = matcher.match_with_id(id);
            eprintln!("  Score: {:.1}", score);
        }
    }
}
