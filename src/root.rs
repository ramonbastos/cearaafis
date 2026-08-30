//! Root public API for the cearaafis fingerprint recognition engine.
//!
//! This module provides the surface API matching the .NET SourceAFIS public types:
//! - `FingerprintImage` — load fingerprint images from bytes, PNG, JPEG, BMP
//! - `FingerprintTemplate` — extract features from a fingerprint image
//! - `FingerprintMatcher` — match a probe template against candidate templates
//! - `FingerprintImageOptions` — DPI configuration
//! - `FingerprintCompatibility` — version info
//! - `FingerprintTransparency` — logging/diagnostics trait

use crate::extractor::FeatureExtractor;
use crate::features::{Minutia, MinutiaType, NeighborEdge};
use crate::matcher::MatcherEngine;
use crate::primitives::{DoubleMatrix, IntPoint, ShortPoint};
use crate::templates::FeatureTemplate;

use std::collections::HashMap;

use image;

// ─────────────────────────────────────────────────────────────────────────────
// FingerprintImage — load fingerprint images from raw bytes (grayscale), PNG,
// JPEG, BMP.  Mirrors .NET FingerprintImage.cs.
// ─────────────────────────────────────────────────────────────────────────────

/// Options for loading fingerprint images.
#[derive(Debug, Clone, Default)]
pub struct FingerprintImageOptions {
    /// Target DPI (20 ≤ dpi ≤ 20000).  0 means "unknown".
    pub dpi: u32,
}

impl FingerprintImageOptions {
    /// Create options with a specific DPI.
    pub fn new(dpi: u32) -> Self {
        assert!(
            (20..=20000).contains(&dpi),
            "dpi must be between 20 and 20000 inclusive"
        );
        Self { dpi }
    }

    /// Set DPI value.
    pub fn with_dpi(mut self, dpi: u32) -> Self {
        self.dpi = dpi;
        self
    }
}

/// A fingerprint image loaded from bytes in any supported format.
///
/// Internally stores a grayscale `DoubleMatrix` and a DPI value.
#[derive(Debug, Clone)]
pub struct FingerprintImage {
    /// Grayscale pixel data as a double-precision matrix.
    pub data: DoubleMatrix,
    /// Dots-per-inch of the source image (0 = unknown).
    pub dpi: u32,
}

impl FingerprintImage {
    /// Load a fingerprint image from raw grayscale bytes.
    ///
    /// # Arguments
    /// * `data` — pixel values (0.0–255.0) in row-major order.
    /// * `width` — image width in pixels.
    /// * `height` — image height in pixels.
    /// * `dpi` — optional DPI (0 = unknown).
    pub fn from_raw(data: Vec<u8>, width: usize, height: usize, dpi: u32) -> Self {
        assert!(
            data.len() == width * height,
            "data length must equal width * height"
        );
        let mut matrix = DoubleMatrix::new(width, height);
        for (i, &byte) in data.iter().enumerate() {
            let x = i % width;
            let y = i / width;
            // Normalize like .NET FingerprintImage: pixels become [0,1] with
            // black=1 (ink) and white=0 — inverts grayscale and scales down.
            matrix.set(x, y, 1.0 - byte as f64 / 255.0);
        }
        Self { data: matrix, dpi }
    }

    /// Load a fingerprint image from a PNG, JPEG, or BMP byte slice.
    ///
    /// Uses the `image` crate to decode and convert to grayscale.
    pub fn from_bytes(
        bytes: &[u8],
        options: &FingerprintImageOptions,
    ) -> Result<Self, anyhow::Error> {
        let img = image::load_from_memory(bytes)?;
        let gray = img.to_luma8();
        let (width, height) = gray.dimensions();
        let w = width as usize;
        let h = height as usize;

        let mut matrix = DoubleMatrix::new(w, h);
        for y in 0..h {
            for x in 0..w {
                let pixel = gray.get_pixel(x as u32, y as u32);
                // Normalize like .NET FingerprintImage: [0,1] with black=1.
                matrix.set(x, y, 1.0 - pixel[0] as f64 / 255.0);
            }
        }

        let dpi = options.dpi;
        Ok(Self { data: matrix, dpi })
    }

    /// Load a fingerprint image from a PNG file path.
    pub fn from_png(path: &str, options: &FingerprintImageOptions) -> Result<Self, anyhow::Error> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes, options)
    }

    /// Load a fingerprint image from a JPEG file path.
    pub fn from_jpeg(path: &str, options: &FingerprintImageOptions) -> Result<Self, anyhow::Error> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes, options)
    }

    /// Load a fingerprint image from a BMP file path.
    pub fn from_bmp(path: &str, options: &FingerprintImageOptions) -> Result<Self, anyhow::Error> {
        let bytes = std::fs::read(path)?;
        Self::from_bytes(&bytes, options)
    }

    /// Return the image dimensions.
    pub fn size(&self) -> IntPoint {
        self.data.size()
    }

    /// Return image width.
    pub fn width(&self) -> usize {
        self.data.width()
    }

    /// Return image height.
    pub fn height(&self) -> usize {
        self.data.height()
    }

    /// Create a FingerprintTemplate by extracting features from this image.
    pub fn to_template(&self) -> FingerprintTemplate {
        let extractor = FeatureExtractor::new(&self.data, self.dpi);
        let (size, minutiae, edges) = extractor.extract(&self.data);
        FingerprintTemplate {
            size,
            minutiae,
            edges,
            dpi: self.dpi,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FingerprintTemplate — extracted features from a fingerprint image.
// Mirrors .NET FeatureTemplate.
// ─────────────────────────────────────────────────────────────────────────────

/// A fingerprint template containing extracted features (minutiae and edges).
///
/// This is the core data structure passed between image extraction and matching.
#[derive(Debug, Clone)]
pub struct FingerprintTemplate {
    /// Image dimensions.
    pub size: ShortPoint,
    /// Extracted minutiae points.
    pub minutiae: Vec<Minutia>,
    /// Extracted neighbor edges (skeleton structure).
    pub edges: Vec<Vec<NeighborEdge>>,
    /// DPI of the source image (0 = unknown).
    pub dpi: u32,
}

impl FingerprintTemplate {
    /// Create a template from size, minutiae, and edges.
    pub fn new(size: ShortPoint, minutiae: Vec<Minutia>, edges: Vec<Vec<NeighborEdge>>) -> Self {
        Self {
            size,
            minutiae,
            edges,
            dpi: 0,
        }
    }

    /// Create a template from a DoubleMatrix image and FeatureTemplate.
    pub fn from_feature_template(
        feature_template: &FeatureTemplate,
        _double_data: &DoubleMatrix,
    ) -> Self {
        let size = ShortPoint::new(feature_template.size.x, feature_template.size.y);
        let minutiae = feature_template
            .minutiae
            .iter()
            .map(|m| {
                Minutia::new(
                    IntPoint::new(m.position.x(), m.position.y()),
                    0.0,
                    MinutiaType::Ending,
                )
            })
            .collect();
        Self {
            size,
            minutiae,
            edges: Vec::new(),
            dpi: 0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FingerprintMatcher — match a probe template against candidate templates.
// Mirrors .NET Matcher.
// ─────────────────────────────────────────────────────────────────────────────

/// Match a fingerprint probe template against candidate templates.
///
/// Uses the SourceAFIS matching algorithm to compute similarity scores.
pub struct FingerprintMatcher {
    /// The probe template to match against.
    pub template: FingerprintTemplate,
    /// Cached hash map for quick candidate lookup (template_id → template).
    pub candidates: HashMap<String, FingerprintTemplate>,
}

impl FingerprintMatcher {
    /// Create a new matcher with a probe template.
    pub fn new(probe: FingerprintTemplate) -> Self {
        Self {
            template: probe,
            candidates: HashMap::new(),
        }
    }

    /// Add a candidate template to the matcher.
    pub fn add_candidate(&mut self, id: String, template: FingerprintTemplate) {
        self.candidates.insert(id, template);
    }

    /// Match the probe template against a single candidate template.
    ///
    /// Uses the MatcherEngine with root-pair enumeration and rigid transform.
    pub fn match_with_template(&self, candidate: &FingerprintTemplate) -> f64 {
        // Use the MatcherEngine for the actual scoring
        let engine = MatcherEngine::new(self.template.minutiae.clone(), candidate.minutiae.clone());
        let data = engine.score();

        // Map engine score to the 0-100 scale
        data.score.min(100.0)
    }

    /// Match the probe template against a candidate by ID.
    ///
    /// Returns the similarity score, or 0.0 if the candidate is not found.
    pub fn match_with_id(&self, id: &str) -> f64 {
        if let Some(candidate) = self.candidates.get(id) {
            self.match_with_template(candidate)
        } else {
            0.0
        }
    }

    /// Match against all registered candidates, returning sorted results.
    ///
    /// Returns a vector of (id, score) sorted by score descending.
    pub fn match_all(&self) -> Vec<(String, f64)> {
        let mut results: Vec<(String, f64)> = self
            .candidates
            .iter()
            .map(|(id, template)| (id.clone(), self.match_with_template(template)))
            .collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FingerprintCompatibility — version info.
// Mirrors .NET FingerprintCompatibility.cs.
// ─────────────────────────────────────────────────────────────────────────────

/// Compatibility information for the engine.
pub struct FingerprintCompatibility;

impl FingerprintCompatibility {
    /// Return the version string from CARGO_PKG_VERSION.
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FingerprintTransparency — trait for logging/diagnostics.
// Mirrors .NET IFingerprintTransparency.
// Re-exports the trait from the transparency module.
// ─────────────────────────────────────────────────────────────────────────────

pub use crate::transparency::FingerprintTransparency;
pub use crate::transparency::NoTransparency;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw() {
        let data = vec![128u8; 100];
        let img = FingerprintImage::from_raw(data, 10, 10, 500);
        assert_eq!(img.data.width(), 10);
        assert_eq!(img.data.height(), 10);
        assert_eq!(img.dpi, 500);
    }

    #[test]
    fn test_options_default() {
        let opts = FingerprintImageOptions::default();
        assert_eq!(opts.dpi, 0);
    }

    #[test]
    fn test_options_with_dpi() {
        let opts = FingerprintImageOptions::new(500);
        assert_eq!(opts.dpi, 500);
    }

    #[test]
    fn test_from_png_with_real_file() {
        let opts = FingerprintImageOptions::default();
        let result = FingerprintImage::from_png("test_resources/probe.png", &opts);
        assert!(result.is_ok());
        let img = result.unwrap();
        assert!(img.data.width() > 0);
        assert!(img.data.height() > 0);
    }

    #[test]
    fn test_from_jpeg_with_real_file() {
        let opts = FingerprintImageOptions::default();
        let result = FingerprintImage::from_jpeg("test_resources/probe.jpeg", &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_bmp_with_real_file() {
        let opts = FingerprintImageOptions::default();
        let result = FingerprintImage::from_bmp("test_resources/probe.bmp", &opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_to_template() {
        let opts = FingerprintImageOptions::default();
        let img = FingerprintImage::from_png("test_resources/probe.png", &opts).unwrap();
        let template = img.to_template();
        assert!(template.size.x > 0);
        assert!(template.size.y > 0);
    }

    #[test]
    fn test_template_new() {
        let tmpl = FingerprintTemplate::new(ShortPoint::new(100, 100), vec![], vec![]);
        assert_eq!(tmpl.size.x, 100);
        assert_eq!(tmpl.size.y, 100);
    }

    #[test]
    fn test_matcher_new() {
        let tmpl = FingerprintTemplate::new(ShortPoint::new(100, 100), vec![], vec![]);
        let matcher = FingerprintMatcher::new(tmpl);
        assert_eq!(matcher.candidates.len(), 0);
    }

    #[test]
    fn test_match_with_same_template() {
        // Edge length must exceed .NET's MinRootEdgeLength (58px) or no root pairs
        // form and the score is legitimately 0. Use well-separated minutiae.
        let min1 = Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending);
        let min2 = Minutia::new(IntPoint::new(130, 130), 0.0, MinutiaType::Bifurcation);
        let tmpl = FingerprintTemplate::new(ShortPoint::new(200, 200), vec![min1, min2], vec![]);
        let matcher = FingerprintMatcher::new(tmpl.clone());

        let score = matcher.match_with_template(&tmpl);
        // Score is proportional to minutiae count (.NET SourceAFIS scoring formula);
        // a 2-minutiae template can't reach 50+, real templates have 20-80+ minutiae.
        // Identical templates should still score meaningfully above zero.
        assert!(
            score > 0.0,
            "Same template should score > 0, got {:.1}",
            score
        );
    }

    #[test]
    fn test_match_with_different_template() {
        let min1 = Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending);
        let tmpl1 = FingerprintTemplate::new(ShortPoint::new(100, 100), vec![min1], vec![]);
        let min2 = Minutia::new(IntPoint::new(200, 200), 0.0, MinutiaType::Ending);
        let tmpl2 = FingerprintTemplate::new(ShortPoint::new(100, 100), vec![min2], vec![]);

        let matcher = FingerprintMatcher::new(tmpl1);
        let score = matcher.match_with_template(&tmpl2);
        assert!(score < 10.0);
    }

    #[test]
    fn test_add_and_match_by_id() {
        // Needs 2+ minutiae with edge length > MinRootEdgeLength (58px) to form
        // a root pair at all.
        let min1 = Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending);
        let min2 = Minutia::new(IntPoint::new(130, 130), 0.0, MinutiaType::Bifurcation);
        let tmpl = FingerprintTemplate::new(ShortPoint::new(200, 200), vec![min1, min2], vec![]);
        let mut matcher = FingerprintMatcher::new(tmpl.clone());

        matcher.add_candidate("candidate1".to_string(), tmpl);
        let score = matcher.match_with_id("candidate1");
        assert!(
            score > 0.0,
            "Same template should score > 0, got {:.1}",
            score
        );
    }

    #[test]
    fn test_match_with_empty_template() {
        let tmpl = FingerprintTemplate::new(ShortPoint::new(100, 100), vec![], vec![]);
        let matcher = FingerprintMatcher::new(tmpl.clone());
        let score = matcher.match_with_template(&tmpl);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_match_all_sorted() {
        let min = Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending);
        let tmpl = FingerprintTemplate::new(ShortPoint::new(100, 100), vec![min], vec![]);
        let mut matcher = FingerprintMatcher::new(tmpl.clone());

        matcher.add_candidate("a".to_string(), tmpl.clone());
        matcher.add_candidate("b".to_string(), tmpl.clone());
        matcher.add_candidate("c".to_string(), tmpl.clone());

        let results = matcher.match_all();
        assert!(results.len() == 3);
        assert!(results[0].1 >= results[2].1);
    }
}
