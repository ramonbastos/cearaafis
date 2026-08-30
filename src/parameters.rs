//! SourceAFIS algorithm parameters — mirrors .NET Parameters.cs exactly.

/// Configuration constants. All values match .NET SourceAFIS Parameters.cs.
pub struct Parameters;

impl Parameters {
    // === Extractor Pipeline Parameters ===
    pub const BLOCK_SIZE: usize = 15;
    pub const HISTOGRAM_DEPTH: usize = 256;
    pub const LOCAL_HISTOGRAM_WINDOW_SIZE: usize = 15;
    pub const CLIPPED_CONTRAST: f64 = 0.08;
    pub const MIN_ABSOLUTE_CONTRAST: f64 = 17.0 / 255.0;
    pub const MIN_RELATIVE_CONTRAST: f64 = 0.34;
    pub const RELATIVE_CONTRAST_SAMPLE: usize = 168_568;
    pub const RELATIVE_CONTRAST_PERCENTILE: f64 = 0.49;
    pub const MASK_VOTE_RADIUS: usize = 7;
    pub const MASK_VOTE_MAJORITY: f64 = 0.51;
    pub const MASK_VOTE_BORDER_DISTANCE: usize = 4;
    pub const BLOCK_ERRORS_VOTE_RADIUS: usize = 1;
    pub const BLOCK_ERRORS_VOTE_MAJORITY: f64 = 0.7;
    pub const BLOCK_ERRORS_VOTE_BORDER_DISTANCE: usize = 4;
    pub const MAX_EQUALIZATION_SCALING: f64 = 3.99;
    pub const MIN_EQUALIZATION_SCALING: f64 = 0.25;
    pub const MIN_ORIENTATION_RADIUS: f64 = 2.0;
    pub const MAX_ORIENTATION_RADIUS: f64 = 6.0;
    pub const ORIENTATION_SPLIT: usize = 50;
    pub const ORIENTATIONS_CHECKED: usize = 20;
    pub const ORIENTATION_SMOOTHING_RADIUS: usize = 1;
    pub const PARALLEL_SMOOTHING_RESOLUTION: usize = 32;
    pub const PARALLEL_SMOOTHING_RADIUS: usize = 7;
    pub const PARALLEL_SMOOTHING_STEP: f64 = 1.59;
    pub const ORTHOGONAL_SMOOTHING_RESOLUTION: usize = 11;
    pub const ORTHOGONAL_SMOOTHING_RADIUS: usize = 4;
    pub const ORTHOGONAL_SMOOTHING_STEP: f64 = 1.11;
    pub const BINARIZED_VOTE_RADIUS: usize = 2;
    pub const BINARIZED_VOTE_MAJORITY: f64 = 0.61;
    pub const BINARIZED_VOTE_BORDER_DISTANCE: usize = 17;
    pub const INNER_MASK_BORDER_DISTANCE: usize = 14;
    pub const MASK_DISPLACEMENT: f64 = 10.06;

    // === Minutiae Extraction Parameters ===
    pub const MINUTIA_CLOUD_RADIUS: usize = 20;
    pub const MAX_CLOUD_SIZE: usize = 4;
    pub const MAX_MINUTIAE: usize = 100;
    pub const SORT_BY_NEIGHBOR: usize = 5;
    pub const EDGE_TABLE_NEIGHBORS: usize = 9;

    // === Skeleton Parameters ===
    pub const THINNING_ITERATIONS: usize = 26;
    pub const MAX_PORE_ARM: usize = 41;
    pub const SHORTEST_ENDED_MINUTIA: usize = 7;
    pub const MAX_RUPTURE_SIZE: usize = 5;
    pub const MAX_GAP_SIZE: usize = 20;
    pub const GAP_ANGLE_OFFSET: usize = 22;
    pub const TOLERATED_GAP_OVERLAP: usize = 2;
    pub const MIN_TAIL_LENGTH: usize = 21;
    pub const MIN_FRAGMENT_LENGTH: usize = 22;

    // === Matcher Parameters ===
    pub const MAX_DISTANCE_ERROR: usize = 13;
    pub const MAX_ANGLE_ERROR: f32 = std::f32::consts::PI / 180.0 * 10.0;
    pub const MAX_GAP_ANGLE: f64 = std::f64::consts::PI / 180.0 * 45.0;
    pub const RIDGE_DIRECTION_SAMPLE: usize = 21;
    pub const RIDGE_DIRECTION_SKIP: usize = 1;
    pub const MAX_TRIED_ROOTS: usize = 70;
    pub const MIN_ROOT_EDGE_LENGTH: usize = 58;
    pub const MAX_ROOT_EDGE_LOOKUPS: usize = 1633;
    pub const MIN_SUPPORTING_EDGES: usize = 1;
    pub const DISTANCE_ERROR_FLATNESS: f64 = 0.69;
    pub const ANGLE_ERROR_FLATNESS: f64 = 0.27;

    // === Scoring Parameters ===
    pub const MINUTIA_SCORE: f64 = 0.032;
    pub const MINUTIA_FRACTION_SCORE: f64 = 8.98;
    pub const MINUTIA_TYPE_SCORE: f64 = 0.629;
    pub const SUPPORTED_MINUTIA_SCORE: f64 = 0.193;
    pub const EDGE_SCORE: f64 = 0.265;
    pub const DISTANCE_ACCURACY_SCORE: f64 = 9.9;
    pub const ANGLE_ACCURACY_SCORE: f64 = 2.79;

    // === Thresholds (FMR-based) ===
    pub const THRESHOLD_FMR_MAX: f64 = 8.48;
    pub const THRESHOLD_FMR_2: f64 = 11.12;
    pub const THRESHOLD_FMR_10: f64 = 14.15;
    pub const THRESHOLD_FMR_100: f64 = 18.22;
    pub const THRESHOLD_FMR_1000: f64 = 22.39;
    pub const THRESHOLD_FMR_10K: f64 = 27.24;
    pub const THRESHOLD_FMR_100K: f64 = 32.01;
}
