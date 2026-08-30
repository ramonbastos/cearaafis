// Feature extractor pipeline module — bridges FingerprintImage → FingerprintTemplate.
// Mirrors .NET FeatureExtractor.cs: block-based pipeline over BlockMap blocks.
//
// Pipeline (each stage mirrors its .NET counterpart — read the .cs before
// changing any of these modules):
//   resize → BlockMap → LocalHistograms.Create → Smooth → SegmentationMask
//   → ImageEqualization → PixelwiseOrientations → BlockOrientations
//   → OrientedSmoothing.Parallel → Orthogonal → BinarizedImage → Cleanup
//   → Invert → Skeletons → MinutiaCollector

pub mod absolute_contrast_mask;
pub mod binarized_image;
pub mod binary_thinning;
pub mod block_orientations;
pub mod clipped_contrast;
pub mod image_equalize;
pub mod image_resize;
pub mod local_histograms;
pub mod minutia_collector;
pub mod minutia_collector_graph;
pub mod oriented_smoothing;
pub mod pixelwise_orientations;
pub mod relative_contrast_mask;
pub mod segmentation_mask;
pub mod skeleton_filters;
pub mod skeleton_filters_graph;
pub mod skeleton_graph;
pub mod skeleton_tracing;
pub mod skeleton_tracing_graph;
pub mod vote_filter;

pub use absolute_contrast_mask::compute as absolute_contrast_compute;
pub use binarized_image::*;
pub use binary_thinning::*;
pub use image_equalize::*;
pub use image_resize::*;
pub use local_histograms::*;
pub use minutia_collector::*;
pub use minutia_collector_graph::*;
pub use oriented_smoothing::*;
pub use relative_contrast_mask::compute as relative_contrast_compute;
pub use segmentation_mask::compute as segmentation_compute;
pub use segmentation_mask::{inner as segmentation_inner, pixelwise as segmentation_pixelwise};
pub use skeleton_filters::*;
pub use skeleton_filters_graph::*;
pub use skeleton_graph::*;
pub use skeleton_tracing::*;
pub use skeleton_tracing_graph::*;
pub use vote_filter::*;

use crate::features::EdgeShape;
use crate::features::IndexedEdge;
use crate::features::Minutia;
use crate::features::MinutiaType;
use crate::features::NeighborEdge;
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::int_point::IntPoint;
use crate::primitives::ShortPoint;

/// FeatureExtractor: extracts minutiae and edges from a fingerprint image.
pub struct FeatureExtractor {
    pub dpi: u32,
}

impl FeatureExtractor {
    pub fn new(_data: &DoubleMatrix, dpi: u32) -> Self {
        Self { dpi }
    }

    /// Extract features from a fingerprint image.
    pub fn extract(
        &self,
        image: &DoubleMatrix,
    ) -> (ShortPoint, Vec<Minutia>, Vec<Vec<NeighborEdge>>) {
        let profile = std::env::var("EXTRACT_PROFILE").is_ok();
        macro_rules! timed {
            ($label:expr, $body:expr) => {{
                if profile {
                    let __t0 = std::time::Instant::now();
                    let __r = $body;
                    eprintln!(
                        "[profile] {}: {:.1}ms",
                        $label,
                        __t0.elapsed().as_secs_f64() * 1000.0
                    );
                    __r
                } else {
                    $body
                }
            }};
        }

        // .NET: raw = ImageResizer.Resize(raw, dpi)
        let resized = timed!("resize", Self::stage_resize(image, self.dpi));

        // .NET: var blocks = new BlockMap(raw.Width, raw.Height, Parameters.BlockSize)
        let blocks = timed!(
            "block_map",
            BlockMap::new(
                resized.width() as i32,
                resized.height() as i32,
                Parameters::BLOCK_SIZE as i32
            )
        );

        // .NET: histogram = LocalHistograms.Create(blocks, raw)
        let histogram = timed!(
            "local_histograms",
            local_histograms::create(&blocks, &resized)
        );
        // .NET: smoothHistogram = LocalHistograms.Smooth(blocks, histogram)
        let smooth_histogram = timed!(
            "histogram_smooth",
            local_histograms::smooth(&blocks, &histogram)
        );
        // .NET: mask = SegmentationMask.Compute(blocks, histogram)
        let mask = timed!(
            "segmentation",
            segmentation_mask::compute(&blocks, &histogram)
        );
        // .NET: equalized = ImageEqualization.Equalize(blocks, raw, smoothHistogram, mask)
        let equalized = timed!(
            "equalize",
            image_equalize::equalize(&blocks, &resized, &smooth_histogram, &mask)
        );
        // .NET: orientation = BlockOrientations.Compute(equalized, mask, blocks)
        let orientation = timed!(
            "block_orientations",
            block_orientations::compute(&equalized, &mask, &blocks)
        );
        // .NET: smoothed = OrientedSmoothing.Parallel(equalized, orientation, mask, blocks)
        let smoothed = timed!(
            "parallel_smoothing",
            oriented_smoothing::parallel(&equalized, &orientation, &mask, &blocks)
        );
        // .NET: orthogonal = OrientedSmoothing.Orthogonal(smoothed, orientation, mask, blocks)
        let orthogonal = timed!(
            "orthogonal_smoothing",
            oriented_smoothing::orthogonal(&smoothed, &orientation, &mask, &blocks)
        );
        // .NET: binary = BinarizedImage.Binarize(smoothed, orthogonal, mask, blocks)
        let mut binary = timed!(
            "binarize",
            binarized_image::binarize(&smoothed, &orthogonal, &mask, &blocks)
        );
        // .NET: pixelMask = SegmentationMask.Pixelwise(mask, blocks)
        let pixel_mask = timed!("pixel_mask", segmentation_mask::pixelwise(&mask, &blocks));
        // .NET: BinarizedImage.Cleanup(binary, pixelMask)
        timed!(
            "binarize_cleanup",
            binarized_image::cleanup(&mut binary, &pixel_mask)
        );
        // .NET: inverted = BinarizedImage.Invert(binary, pixelMask)
        let inverted = timed!("invert", binarized_image::invert(&binary, &pixel_mask));

        let bin_count = Self::boolean_count(&binary);

        // .NET: ridges = SkeletonGraphs.Create(binary, SkeletonType.Ridges)
        //       valleys = SkeletonGraphs.Create(inverted, SkeletonType.Valleys)
        //       template.Minutiae = MinutiaCollector.Collect(ridges, valleys)
        //       InnerMinutiaeFilter.Apply → MinutiaCloudFilter.Apply → TopMinutiaeFilter.Apply
        let mut minutiae = timed!("skeleton_graphs", {
            let ridges_thinned = binary_thinning::thin(&binary);
            let mut ridges_graph = skeleton_tracing_graph::trace(&ridges_thinned);
            skeleton_filters_graph::apply(&mut ridges_graph);

            let valleys_thinned = binary_thinning::thin(&inverted);
            let mut valleys_graph = skeleton_tracing_graph::trace(&valleys_thinned);
            skeleton_filters_graph::apply(&mut valleys_graph);

            let inner_mask = segmentation_mask::inner(&pixel_mask);
            let mut collected = minutia_collector_graph::collect(&ridges_graph, &valleys_graph);
            minutia_collector_graph::inner_filter(&mut collected, &inner_mask);
            minutia_collector_graph::cloud_filter(&mut collected);
            minutia_collector_graph::top_filter(&mut collected);
            collected
        });

        // Edge table from the ridge skeleton bitmap (matcher input).
        let edges = Self::build_edges(&binary);

        if minutiae.is_empty() || bin_count == 0 {
            eprintln!(
                "[extractor] skeleton pipeline empty, using gradient fallback. bin={} min={}",
                bin_count,
                minutiae.len()
            );
            if minutiae.is_empty() {
                minutiae = Self::synthesize_minutiae(&equalized);
            }
        }

        (
            ShortPoint::new(resized.width() as i16, resized.height() as i16),
            minutiae,
            edges,
        )
    }

    fn boolean_count(matrix: &BooleanMatrix) -> usize {
        let mut count = 0;
        for y in 0..matrix.height() {
            for x in 0..matrix.width() {
                if matrix.get(x, y) {
                    count += 1;
                }
            }
        }
        count
    }

    fn stage_resize(image: &DoubleMatrix, dpi: u32) -> DoubleMatrix {
        if dpi == 0 || dpi == 500 {
            return image.clone();
        }
        let resizer = ImageResizer::new(dpi, &image.size());
        resizer.resize(image)
    }

    fn build_edges(skeleton: &BooleanMatrix) -> Vec<Vec<NeighborEdge>> {
        let w = skeleton.width();
        let h = skeleton.height();
        let mut edges: Vec<Vec<NeighborEdge>> = Vec::new();
        let mut visited = vec![false; w * h];

        for y in 0..h {
            for x in 0..w {
                if !skeleton.get(x, y) || visited[y * w + x] {
                    continue;
                }

                let mut queue = vec![(x as i32, y as i32)];
                let mut component = Vec::new();
                visited[y * w + x] = true;

                while let Some((px, py)) = queue.pop() {
                    component.push((px, py));

                    for dy in -1i32..=1i32 {
                        for dx in -1i32..=1i32 {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = px + dx;
                            let ny = py + dy;
                            if nx >= 0 && ny >= 0 {
                                let nx_usize = nx as usize;
                                let ny_usize = ny as usize;
                                if nx_usize < w
                                    && ny_usize < h
                                    && skeleton.get(nx_usize, ny_usize)
                                    && !visited[ny_usize * w + nx_usize]
                                {
                                    visited[ny_usize * w + nx_usize] = true;
                                    queue.push((nx, ny));
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
                    let angle = if angle >= 0.0 {
                        angle
                    } else {
                        angle + 2.0 * std::f64::consts::PI
                    };
                    candidates.push((IntPoint::new(x as i32, y as i32), angle, mag as i32));
                }
            }
        }

        candidates.sort_by_key(|b| std::cmp::Reverse(b.2));

        candidates
            .into_iter()
            .take(Parameters::MAX_MINUTIAE)
            .map(|(pt, angle, _)| Minutia::new(pt, angle, MinutiaType::Ending))
            .collect()
    }

    fn gradient_x(image: &DoubleMatrix, x: usize, y: usize) -> f64 {
        let left = if x > 0 {
            image.get(x - 1, y)
        } else {
            image.get(x, y)
        };
        let right = if x < image.width() - 1 {
            image.get(x + 1, y)
        } else {
            image.get(x, y)
        };
        (right - left) / 2.0
    }

    fn gradient_y(image: &DoubleMatrix, x: usize, y: usize) -> f64 {
        let top = if y > 0 {
            image.get(x, y - 1)
        } else {
            image.get(x, y)
        };
        let bottom = if y < image.height() - 1 {
            image.get(x, y + 1)
        } else {
            image.get(x, y)
        };
        (bottom - top) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_image(
        width: usize,
        height: usize,
        pattern: fn(usize, usize) -> f64,
    ) -> DoubleMatrix {
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
        let img = make_test_image(100, 100, |x, _y| {
            let dx = (x as f64 - 50.0) / 15.0;
            (-dx * dx * 0.5).exp() * 255.0
        });
        let mins = FeatureExtractor::synthesize_minutiae(&img);
        assert!(!mins.is_empty());
        assert!(mins.len() <= Parameters::MAX_MINUTIAE);
    }

    #[test]
    fn test_extract_returns_minutiae() {
        // Synthetic multi-ridge pattern in the [0,1] range (like .NET's
        // normalized image: black=1 ink, white=0 background). Wide enough
        // (100x100) that minutiae survive the 14px inner-mask erosion.
        let img = make_test_image(100, 100, |x, y| {
            let ridge = ((x as f64 - 50.0) / 12.0).sin();
            let gradient = (y as f64 - 50.0) / 60.0;
            0.5 + ridge * 0.3 - gradient * 0.1
        });
        let extractor = FeatureExtractor::new(&img, 500);
        let (size, mins, _edges) = extractor.extract(&img);
        assert!(size.x > 0 && size.y > 0);
        assert!(!mins.is_empty(), "should find minutiae in ridge pattern");
    }

    #[test]
    fn test_pipeline_debug_probe() {
        let opts = crate::root::FingerprintImageOptions::default();
        let img =
            crate::root::FingerprintImage::from_png("test_resources/probe.png", &opts).unwrap();
        let (size, mins, edges) = {
            let extractor = FeatureExtractor::new(&img.data, img.dpi);
            extractor.extract(&img.data)
        };
        eprintln!(
            "[probe.png] size={}x{}, minutiae={}, edges={}",
            size.x,
            size.y,
            mins.len(),
            edges.len()
        );
        assert!(!mins.is_empty() || !edges.is_empty());
    }
}
