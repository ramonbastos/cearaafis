/// Debug: inspect skeleton minutiae and edges per image.
use cearaafis::extractor::FeatureExtractor;
use cearaafis::primitives::DoubleMatrix;
use image::GenericImageView;

fn build_gray(path: &str) -> DoubleMatrix {
    let img = image::io::Reader::open(path).unwrap().decode().unwrap();
    let gray = img.to_luma8();
    let (w, h) = img.dimensions();
    let ww = w as usize;
    let hh = h as usize;
    let mut matrix = DoubleMatrix::new(ww, hh);
    for y in 0..hh {
        for x in 0..ww {
            let px = gray.get_pixel(x as u32, y as u32);
            matrix.set(x, y, px[0] as f64);
        }
    }
    matrix
}

fn build_raw(path: &str, w: usize, h: usize) -> DoubleMatrix {
    let raw = std::fs::read(path).expect("read raw");
    assert_eq!(raw.len(), w * h);
    let mut matrix = DoubleMatrix::new(w, h);
    for (i, &b) in raw.iter().enumerate() {
        let x = i % w;
        let y = i / w;
        matrix.set(x, y, b as f64);
    }
    matrix
}

fn run_debug(name: &str, matrix: &DoubleMatrix, dpi: u32) {
    let extractor = FeatureExtractor::new(matrix, dpi);
    let (size, mins, edges) = extractor.extract(matrix);

    println!("=== {} ===", name);
    println!("  template size: {} x {}", size.x, size.y);
    println!("  skeleton minutiae: {}", mins.len());
    println!("  edge components: {}", edges.len());
    for (i, el) in edges.iter().enumerate().take(10) {
        println!("  component[{}]: {} edges", i, el.len());
    }
}

#[test]
fn debug_probe_png() {
    let matrix = build_gray("test_resources/probe.png");
    run_debug("probe.png", &matrix, 500);
}

#[test]
fn debug_matching_png() {
    let matrix = build_gray("test_resources/matching.png");
    run_debug("matching.png", &matrix, 500);
}

#[test]
fn debug_nonmatching_png() {
    let matrix = build_gray("test_resources/nonmatching.png");
    run_debug("nonmatching.png", &matrix, 500);
}

#[test]
fn debug_raw_probe() {
    let matrix = build_raw("test_resources/gray-probe.dat", 332, 533);
    run_debug("gray-probe.dat", &matrix, 500);
}

#[test]
fn debug_raw_matching() {
    let matrix = build_raw("test_resources/gray-matching.dat", 352, 370);
    run_debug("gray-matching.dat", &matrix, 500);
}

#[test]
fn debug_raw_nonmatching() {
    let matrix = build_raw("test_resources/gray-nonmatching.dat", 333, 435);
    run_debug("gray-nonmatching.dat", &matrix, 500);
}
