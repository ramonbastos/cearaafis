# CearáAFIS

Rust re-implementation of [SourceAFIS for .NET](https://github.com/robertvazan/sourceafis-net) — a fingerprint recognition engine that extracts minutiae features from fingerprint images and computes similarity scores.

**Reference implementation:** sourceafis-net (Apache-2.0) · **This port:** Apache-2.0

---

## What It Does

```
fingerprint image (PNG/JPEG/BMP/raw)
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  EXTRACTION PIPELINE (mirrors .NET FeatureExtractor.cs)         │
│                                                                 │
│  resize → BlockMap (15×15 blocks)                               │
│    → LocalHistograms.Create/Smooth   (block-level, 256 bins)    │
│    → SegmentationMask                (clipped contrast + votes) │
│    → ImageEqualization               (per-corner bilinear maps) │
│    → PixelwiseOrientations → BlockOrientations                  │
│    → OrientedSmoothing.Parallel/Orthogonal                      │
│    → BinarizedImage.Binarize/Cleanup/Invert                     │
│    → BinaryThinning (256-entry LUT)                             │
│    → SkeletonTracing → skeleton GRAPH (minutiae + ridges)       │
│    → SkeletonFilters (dot/pore/gap/knot/tail/fragment)          │
│    → MinutiaCollector (graph-based)                             │
│    → InnerMinutiaeFilter → MinutiaCloudFilter → TopMinutiaeFilter│
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
  FingerprintTemplate { minutiae, edges }
        │
        ▼
┌─────────────────────────────────────────────────────────────────┐
│  MATCHING (edge-based relative geometry)                        │
│  root pairs → BFS pairing growth → 7-component scoring          │
└─────────────────────────────────────────────────────────────────┘
        │
        ▼
     score (0–100 shaped, FMR-threshold semantics)
```

## Status: Block-Based Architecture Complete

This branch (`feature/block-based-extraction-and-skeleton-minutiae`) replaces the original
per-pixel extraction pipeline with the .NET **block-based architecture**, plus a
**skeleton-graph minutia collector**. Every ported module was written after reading and
analyzing its .NET counterpart (rule recorded in AGENTS.md).

### Extraction Performance (probe.png 388×374, warm)

| Stage / metric              | Before (per-pixel) | After (block-based) | .NET reference |
|-----------------------------|-------------------:|--------------------:|---------------:|
| `local_histograms`          |        ~331 ms     |         **~11 ms**  |         ~15 ms |
| full pipeline               |        ~406 ms     |        **~43.5 ms** |        ~72 ms  |
| **vs .NET**                 |        5.6× slower |     **~1.7× faster**|        1.0×    |

The speedup comes from .NET's key technique: ONE histogram per 15×15 block
(~640 blocks) instead of one per pixel (~145k), plus an integral-image
VoteFilter (O(1) per pixel) and precomputed orientation sampling lines.

### Minutiae Count Parity (probe / matching / nonmatching)

| Image          | Before | After | .NET |
|----------------|-------:|------:|-----:|
| probe.png      |     49 | **47**|   46 |
| matching.png   |     55 | **35**|   37 |
| nonmatching.png|     89 | **38**|   28 |

Before: raw pixel-neighbor counting over-detected spurious minutiae (89 vs 28 on
nonmatching). After: minutiae come from the skeleton graph (`Skeleton.Minutiae`
nodes with exactly 1 ridge), matching the .NET `MinutiaCollector.Collect`.

### Match Scores (test_resources images)

| Pair                    | Score | Threshold | Result |
|-------------------------|------:|-----------|--------|
| probe vs matching       |  47.8 | ≥40 match | ✅ MATCH |
| probe.jpeg vs matching  |  47.8 | ≥40 match | ✅ MATCH |
| probe vs nonmatching    |   0.0 | ≤20 non-match | ✅ NON-MATCH |
| probe vs probe          | 100.0 | highest   | ✅ |

### Test Suite

`cargo test` — **45 suites, all passing** (254 unit + integration).

## Architecture Notes

### Block-based core (ported from .NET)

| Rust module (`src/extractor/`) | .NET counterpart | Technique |
|--------------------------------|------------------|-----------|
| `local_histograms.rs`  | LocalHistograms.cs    | per-block 256-bin `HistogramCube`; Smooth aggregates the 4 blocks touching each corner |
| `clipped_contrast.rs`  | ClippedContrast.cs    | per-block contrast after clipping histogram tails (8%) |
| `vote_filter.rs`       | VoteFilter.cs         | integral-image majority vote, O(1) per pixel |
| `segmentation_mask.rs` | SegmentationMask.cs   | absolute+relative contrast masks → 3× block-error vote → invert → mask vote; Pixelwise/Inner expansion |
| `image_equalize.rs`    | ImageEqualization.cs  | per-corner mappings over smoothed histogram, clamped by Max/MinEqualizationScaling, bilinear per pixel |
| `block_orientations.rs` + `pixelwise_orientations.rs` | BlockOrientations.cs / PixelwiseOrientations.cs | deterministic LCG-sampled orientation plan (50 splits × 20 samples) |
| `oriented_smoothing.rs`| OrientedSmoothing.cs  | precomputed sampling lines per orientation bucket (parallel r=7, orthogonal r=4) |
| `binarized_image.rs`   | BinarizedImage.cs     | ridge = smoothed > orthogonal, inside masked blocks; island/hole cleanup + cross removal |
| `binary_thinning.rs`   | BinaryThinning.cs     | 256-entry neighborhood LUT, 4 interleaved subgrid passes, false-ending check |
| `skeleton_graph.rs` + `skeleton_tracing_graph.rs` | Skeleton/SkeletonMinutia/SkeletonRidge/SkeletonTracing.cs | arena-based graph (index references instead of shared mutable objects): FindMinutiae → LinkNeighboringMinutiae → MinutiaCenters → TraceRidges → FixLinkingGaps |
| `skeleton_filters_graph.rs` | Skeleton*Filter.cs | dot/pore/gap (priority queue + shadow)/knot/tail/fragment |
| `minutia_collector_graph.rs` | MinutiaCollector.cs + Inner/Cloud/Top filters | endings from ridge graph, bifurcations from valley graph; mask-displacement, cloud-radius, top-100 filters |

### Image normalization (matches .NET FingerprintImage.cs)

Pixels are normalized to **[0,1] with black=1** (`1.0 - byte/255.0`) — the .NET
pipeline's assumed range. The old port loaded raw 0–255 values, which broke
histogram depth and equalization scaling.

### Skeleton graph as arena

.NET uses bidirectional object references (`ridge.Start ↔ minutia.Ridges`,
`ridge.Reversed`). Rust translates this into an index arena:
`SkeletonGraph { minutiae: Vec<GraphMinutia>, ridges: Vec<GraphRidge> }` where each
ridge stores its reversed twin's index and each minutia lists ridges attached at
their start. `compact()` drops detached ridges/dots and reindexes.

### Scoring semantics

.NET scores are uncapped (probe-vs-probe ≈ 548 on full images). The Rust
`shape_score` maps raw scores through .NET's FMR threshold table
(8.48 / 11.12 / 14.15 / 18.22 / 22.39 / 27.24 / 32.01) into a bounded 0–100
range. Only relative ordering and threshold behavior are comparable with .NET,
not absolute magnitude.

## Usage

```rust
use cearaafis::root::{FingerprintImage, FingerprintImageOptions, FingerprintMatcher};

// Load probe (PNG/JPEG/BMP/raw grayscale; dpi=0 means unknown)
let probe = FingerprintImage::from_png("probe.png", &FingerprintImageOptions::default())?;
let matching = FingerprintImage::from_png("matching.png", &FingerprintImageOptions::default())?;

// Extract minutiae + edges
let probe_tmpl = probe.to_template();
let matching_tmpl = matching.to_template();

// Match
let mut matcher = FingerprintMatcher::new(probe_tmpl);
matcher.add_candidate("matching".to_string(), matching_tmpl);
let score = matcher.match_with_id("matching");
println!("score: {:.1}", score); // ≥40 → match, ≤20 → non-match
```

Raw grayscale:

```rust
let raw = std::fs::read("gray-probe.dat")?;
let image = FingerprintImage::from_raw(raw, 332, 533, 500); // data, width, height, dpi
```

## Development

```bash
cargo test                # full suite (unit + integration)
cargo test --lib          # unit tests only
cargo test --release --test match_scores -- --nocapture   # real-image scores
EXTRACT_PROFILE=1 cargo test --lib test_pipeline_debug_probe -- --nocapture  # per-stage timing
```

### Project rule (from AGENTS.md)

**Always read and analyze the corresponding C# in `/tmp/sourceafis-net/SourceAFIS/`
BEFORE changing any Rust module** — the .NET source is the source of truth for
algorithm, data structures, and performance techniques. Never guess a port.

## Test Resources

| File | Content |
|------|---------|
| `probe.png` / `matching.png` / `nonmatching.png` | same-finger pair + different finger (256×256) |
| `probe.jpeg`, `probe.bmp` | JPEG/BMP decoder coverage |
| `gray-*.dat` | raw grayscale (row-major, single channel) |

Matching expectations: probe↔matching ≥ 40 · probe↔nonmatching ≤ 20 ·
probe↔probe highest. Scores within ±5 of the .NET baseline behavior.
