use cearaafis::*;

fn load_png(name: &str) -> FingerprintImage {
    let path = format!("test_resources/{}", name);
    let opts = FingerprintImageOptions::default();
    FingerprintImage::from_png(&path, &opts).expect(&format!("Failed: {}", path))
}

fn load_jpeg(name: &str) -> FingerprintImage {
    let path = format!("test_resources/{}", name);
    let opts = FingerprintImageOptions::default();
    FingerprintImage::from_jpeg(&path, &opts).expect(&format!("Failed: {}", path))
}

fn main() {
    println!("=== Minutiae counts ===");
    for name in &["probe.png", "matching.png", "nonmatching.png"] {
        let img = load_png(name);
        let tmpl = img.to_template();
        println!("{}: {} minutiae", name, tmpl.minutiae.len());
    }
    let img = load_jpeg("probe.jpeg");
    let tmpl = img.to_template();
    println!("probe.jpeg: {} minutiae", tmpl.minutiae.len());

    println!("\n=== Scores (bounded) ===");

    // probe vs matching
    let probe = load_png("probe.png");
    let probe_tmpl = probe.to_template();
    let matching = load_png("matching.png");
    let matching_tmpl = matching.to_template();
    let score = FingerprintMatcher::new(probe_tmpl.clone()).match_with_template(&matching_tmpl);
    println!("probe vs matching: {:.1}", score);

    // probe vs self (identical)
    let score2 = FingerprintMatcher::new(probe_tmpl.clone()).match_with_template(&probe_tmpl);
    println!("probe vs probe (identical): {:.1}", score2);

    // probe.jpeg vs matching.png (cross-format)
    let jpeg = load_jpeg("probe.jpeg");
    let jpeg_tmpl = jpeg.to_template();
    let score3 = FingerprintMatcher::new(probe_tmpl).match_with_template(&matching_tmpl);
    println!("probe.jpeg vs matching.png: {:.1}", score3);
}
