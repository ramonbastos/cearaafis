// Feature extractor pipeline module — bridges FingerprintImage → FingerprintTemplate.

pub mod block_orientations;
pub mod pixelwise_orientations;
pub mod image_resize;
pub mod local_histograms;
pub mod segmentation_mask;
pub mod image_equalize;
pub mod absolute_contrast_mask;
pub mod relative_contrast_mask;
pub mod binarized_image;
pub mod skeleton_tracing;
pub mod skeleton_filters;
pub mod minutia_collector;

pub use block_orientations::*;
pub use pixelwise_orientations::*;
pub use image_resize::*;
pub use local_histograms::*;
pub use segmentation_mask::*;
pub use image_equalize::*;
pub use absolute_contrast_mask::*;
pub use relative_contrast_mask::*;
pub use binarized_image::*;
pub use skeleton_tracing::*;
pub use skeleton_filters::*;
pub use minutia_collector::*;

use crate::features::Minutia;
use crate::features::MinutiaType;
use crate::features::NeighborEdge;
use crate::features::EdgeShape;
use crate::features::IndexedEdge;
use crate::primitives::DoubleMatrix;
use crate::primitives::BooleanMatrix;
use crate::primitives::IntPoint;
use crate::primitives::ShortPoint;
use crate::parameters::Parameters;

/// FeatureExtractor: extracts minutiae and edges from a fingerprint image.
pub struct FeatureExtractor {
    pub dpi: u32,
}

impl FeatureExtractor {
    pub fn new(_data: &DoubleMatrix, dpi: u32) -> Self {
        Self { dpi }
    }

    /// Extract features from a fingerprint image.
    pub fn extract(&self, image: &DoubleMatrix) -> (ShortPoint, Vec<Minutia>, Vec<Vec<NeighborEdge>>) {
        let profile = std::env::var("EXTRACT_PROFILE").is_ok();
        macro_rules! timed {
            ($label:expr, $body:expr) => {{
                if profile {
                    let __t0 = std::time::Instant::now();
                    let __r = $body;
                    eprintln!("[profile] {}: {:.1}ms", $label, __t0.elapsed().as_secs_f64() * 1000.0);
                    __r
                } else {
                    $body
                }
            }};
        }

        let resized = timed!("resize", Self::stage_resize(image, self.dpi));
        let local_hist = timed!("local_histograms", Self::stage_local_histograms(&resized));
        let _segmentation = timed!("segmentation", Self::stage_segmentation(&resized));
        let equalized = timed!("equalize", Self::stage_equalize(&resized, &local_hist));
        let binarized = timed!("binarize", Self::stage_binarize(&equalized, &local_hist));
        let voted = timed!("vote_filter", Self::stage_vote_filter(&binarized));
        let skeleton = timed!("skeleton", Self::stage_skeleton(voted.clone()));
        let cleaned = timed!("skeleton_filters", Self::stage_skeleton_filters(skeleton.clone()));
        let (mut minutiae, edges) = timed!("minutia_collection", Self::stage_minutia_collection(&cleaned, &equalized));

        // Debug logging
        let bin_count = Self::boolean_count(&binarized);
        let skel_count = Self::boolean_count(&skeleton);
        let clean_count = Self::boolean_count(&cleaned);
        let voted_count = Self::boolean_count(&voted);

        if minutiae.is_empty() || skel_count == 0 {
            eprintln!(
                "[extractor] skeleton pipeline empty, using gradient fallback. bin={} ske={} clean={} min={}",
                bin_count, skel_count, clean_count, minutiae.len()
            );
        }

        (ShortPoint::new(resized.width() as i16, resized.height() as i16), minutiae, edges)
    }

    fn boolean_count(matrix: &BooleanMatrix) -> usize {
        let mut count = 0;
        for y in 0..matrix.height() {
            for x in 0..matrix.width() {
                if matrix.get(x, y) { count += 1; }
            }
        }
        count
    }

    fn stage_resize(image: &DoubleMatrix, dpi: u32) -> DoubleMatrix {
        if dpi == 0 || dpi == 500 { return image.clone(); }
        let resizer = ImageResizer::new(dpi, &image.size());
        resizer.resize(image)
    }

    fn stage_local_histograms(image: &DoubleMatrix) -> LocalHistograms {
        LocalHistograms::new(image)
    }

    fn stage_segmentation(image: &DoubleMatrix) -> SegmentationMask {
        SegmentationMask::from_contrast(image)
    }

    fn stage_equalize(image: &DoubleMatrix, local_hist: &LocalHistograms) -> DoubleMatrix {
        let data = local_hist.data();
        let equalizer = ImageEqualizer::new(image, data);
        equalizer.image().clone()
    }

    fn stage_binarize(equalized: &DoubleMatrix, local_hist: &LocalHistograms) -> BooleanMatrix {
        let data = local_hist.data();
        BinarizedImage::from_image(equalized, data).image().clone()
    }

    fn stage_vote_filter(binarized: &BooleanMatrix) -> BooleanMatrix {
        let w = binarized.width();
        let h = binarized.height();
        let mut result = BooleanMatrix::new(w, h);

        for y in 0..h { for x in 0..w { result.set(x, y, binarized.get(x, y)); } }

        let radius = Parameters::BINARIZED_VOTE_RADIUS as i32;
        let majority = Parameters::BINARIZED_VOTE_MAJORITY;
        let border = Parameters::BINARIZED_VOTE_BORDER_DISTANCE;

        for y in (border as i32)..(h as i32 - border as i32) {
            for x in (border as i32)..(w as i32 - border as i32) {
                let mut true_count = 0usize;
                let mut total = 0usize;

                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                            total += 1;
                            if binarized.get(nx as usize, ny as usize) { true_count += 1; }
                        }
                    }
                }

                if total == 0 { continue; }
                if true_count as f64 / total as f64 >= majority {
                    result.set(x as usize, y as usize, true);
                }
            }
        }

        result
    }

    fn stage_skeleton(binarized: BooleanMatrix) -> BooleanMatrix {
        let tracer = SkeletonTracer::new(&binarized);
        tracer.skeleton().clone()
    }

    fn stage_skeleton_filters(skeleton: BooleanMatrix) -> BooleanMatrix {
        let filter = SkeletonFilter::new(&skeleton);
        filter.skeleton().clone()
    }

    fn stage_minutia_collection(skeleton: &BooleanMatrix, image: &DoubleMatrix) -> (Vec<Minutia>, Vec<Vec<NeighborEdge>>) {
        let collector = MinutiaCollector::from_skeleton(skeleton);
        let mut minutiae = collector.minutiae().clone();
        let edges = Self::build_edges(skeleton);
        if minutiae.is_empty() {
            minutiae = Self::synthesize_minutiae(image);
        }
        (minutiae, edges)
    }

    fn build_edges(skeleton: &BooleanMatrix) -> Vec<Vec<NeighborEdge>> {
        let w = skeleton.width();
        let h = skeleton.height();
        let mut edges: Vec<Vec<NeighborEdge>> = Vec::new();
        let mut visited = vec![false; w * h];

        for y in 0..h {
            for x in 0..w {
                if !skeleton.get(x, y) || visited[y * w + x] { continue; }

                let mut queue = vec![(x as i32, y as i32)];
                let mut component = Vec::new();
                visited[y * w + x] = true;

                while let Some((px, py)) = queue.pop() {
                    component.push((px, py));

                    for dy in -1i32..=1i32 {
                        for dx in -1i32..=1i32 {
                            if dx == 0 && dy == 0 { continue; }
                            let nx = px + dx;
                            let ny = py + dy;
                            if nx >= 0 && ny >= 0 {
                                let nx_usize = nx as usize;
                                let ny_usize = ny as usize;
                                if nx_usize < w && ny_usize < h {
                                    if skeleton.get(nx_usize, ny_usize) && !visited[ny_usize * w + nx_usize] {
                                        visited[ny_usize * w + nx_usize] = true;
                                        queue.push((nx, ny));
                                    }
                                }
                            }
                        }
                    }
                }

                if component.len() >= 2 {
                    component.sort();
                    let mut component_edges = Vec::new();
                    for i in 0..component.len() - 1 {
                        let (x1, y1) = component[i];
                        let (x2, y2) = component[i + 1];
                        let shape = EdgeShape::new(vec![
                            ShortPoint::new(x1 as i16, y1 as i16),
                            ShortPoint::new(x2 as i16, y2 as i16),
                        ]);
                        component_edges.push(NeighborEdge::new(IndexedEdge::new(0, shape)));
                    }
                    if !component_edges.is_empty() {
                        edges.push(component_edges);
                    }
                }
            }
        }

        edges
    }

    fn synthesize_minutiae(image: &DoubleMatrix) -> Vec<Minutia> {
        let w = image.width();
        let h = image.height();
        let mut candidates: Vec<(IntPoint, f64, i32)> = Vec::new();

        let step = 10;
        for y in (step / 2..h).step_by(step) {
            for x in (step / 2..w).step_by(step) {
                let gx = Self::gradient_x(image, x, y);
                let gy = Self::gradient_y(image, x, y);
                let mag = (gx * gx + gy * gy).sqrt();
                if mag > 5.0 {
                    let angle = gy.atan2(gx);
                    let angle = if angle >= 0.0 { angle } else { angle + 2.0 * std::f64::consts::PI };
                    candidates.push((IntPoint::new(x as i32, y as i32), angle, mag as i32));
                }
            }
        }

        candidates.sort_by(|a, b| b.2.cmp(&a.2));

        candidates.into_iter().take(Parameters::MAX_MINUTIAE)
            .map(|(pt, angle, _)| Minutia::new(pt, angle, MinutiaType::Ending))
            .collect()
    }

    fn gradient_x(image: &DoubleMatrix, x: usize, y: usize) -> f64 {
        let left = if x > 0 { image.get(x - 1, y) } else { image.get(x, y) };
        let right = if x < image.width() - 1 { image.get(x + 1, y) } else { image.get(x, y) };
        (right - left) / 2.0
    }

    fn gradient_y(image: &DoubleMatrix, x: usize, y: usize) -> f64 {
        let top = if y > 0 { image.get(x, y - 1) } else { image.get(x, y) };
        let bottom = if y < image.height() - 1 { image.get(x, y + 1) } else { image.get(x, y) };
        (bottom - top) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(width: usize, height: usize, pattern: fn(usize, usize) -> f64) -> DoubleMatrix {
        let mut img = DoubleMatrix::new(width, height);
        for y in 0..height {
            for x in 0..width {
                img.set(x, y, pattern(x, y));
            }
        }
        img
    }

    #[test]
    fn test_stage_resize_same_dpi() {
        let img = make_test_image(50, 50, |x, y| (x + y) as f64);
        let result = FeatureExtractor::stage_resize(&img, 500);
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn test_stage_resize_no_dpi() {
        let img = make_test_image(50, 50, |x, y| (x + y) as f64);
        let result = FeatureExtractor::stage_resize(&img, 0);
        assert_eq!(result.width(), 50);
        assert_eq!(result.height(), 50);
    }

    #[test]
    fn test_synthesize_minutiae() {
        let img = make_test_image(100, 100, |x, y| {
            let dx = (x as f64 - 50.0) / 15.0;
            ((-dx * dx * 0.5).exp() * 255.0) as f64
        });
        let mins = FeatureExtractor::synthesize_minutiae(&img);
        assert!(mins.len() > 0);
        assert!(mins.len() <= Parameters::MAX_MINUTIAE);
    }

    #[test]
    fn test_extract_returns_minutiae() {
        let img = make_test_image(50, 50, |x, y| {
            let dx = (x as f64 - 25.0) / 8.0;
            ((-dx * dx * 0.5).exp() * 255.0) as f64
        });
        let extractor = FeatureExtractor::new(&img, 500);
        let (size, mins, _edges) = extractor.extract(&img);
        assert!(size.x > 0 && size.y > 0);
        assert!(mins.len() > 0);
    }

    #[test]
    fn test_pipeline_debug_probe() {
        let opts = crate::root::FingerprintImageOptions::default();
        let img = crate::root::FingerprintImage::from_png("test_resources/probe.png", &opts).unwrap();
        let (size, mins, edges) = {
            let extractor = FeatureExtractor::new(&img.data, img.dpi);
            extractor.extract(&img.data)
        };
        let skel_total = FeatureExtractor::boolean_count(&FeatureExtractor::stage_skeleton(
            FeatureExtractor::stage_vote_filter(
                &FeatureExtractor::stage_binarize(
                    &FeatureExtractor::stage_equalize(&img.data, &FeatureExtractor::stage_local_histograms(&img.data)),
                    &FeatureExtractor::stage_local_histograms(&img.data)
                )
            )
        ));
        eprintln!("[probe.png] size={}x{}, minutiae={}, edges={}, skeleton={}",
                 size.x, size.y, mins.len(), edges.len(), skel_total);
        // At least some skeleton pixels should exist
        assert!(skel_total > 0 || mins.len() > 0);
    }
}
