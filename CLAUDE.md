# CearáAFIS - Rust Fingerprint Recognition Engine

## Project

CearáAFIS is a Rust re-implementation of SourceAFIS for .NET (Apache 2.0). Reference: https://github.com/robertvazan/sourceafis-net
**C# Source:** Always consult the .NET source code at `rust/cearaafis/docs/dotnet/` when translating to Rust — it is the original reference. If unsure about implementation, read the .NET equivalent first.

## Context Files

Before implementing, reading, or editing ANY file, always load these documents:

- `AGENTS.md` — Architecture map, public API, implementation order, pitfalls, test plan
- `full-analysis.md` — Complete C# → Rust file mapping, gap analysis, implementation order
- `implementation-status.md` — Phase-by-phase status tracker (what's done vs what's missing)
- `structure-analysis.md` — Current Rust code state analysis, problems, target structure
- `implementation-plan.md` — Sprint-by-sprint plan with dependencies and timeline

## Implementation Priority

1. **Cargo.toml** — Add missing deps: `serde`, `serde_cbor`, `image`, `anyhow`
2. **lib.rs** — Module declarations + pub re-exports
3. **DoubleAngle** — Standalone module under `engine/primitives/` (critical for orientation)
4. **IntRect** — Complete: left(), top(), right(), bottom(), move_point()
5. **PriorityQueue** — Add comparator trait for SkeletonGap (reverse order)
6. **Consolidate duplicates** — Minutia, ShortPoint, EdgeShape, MinutiaPair, ScoringData, SkeletonGap
7. **RelativeContrastMask** — Move from utils.rs stub to `engine/extractor/`
8. **SkeletonKnotFilter** — Missing, implement merge/extend ridges
9. **SkeletonRidge::direction()** — Needs DoubleAngle integration
10. **FingerprintCompatibility::version()** — Use env!(CARGO_PKG_VERSION)

## Code Organization

Files must follow the .NET module hierarchy under `engine/`:
```
src/lib.rs
src/FingerprintImage.rs
src/FingerprintTemplate.rs
src/FingerprintMatcher.rs
src/FingerprintTransparency.rs
src/FingerprintCompatibility.rs
src/engine/
  ├── mod.rs
  ├── configuration/Parameters.rs
  ├── primitives/{DoubleAngle, IntRect, PriorityQueue, ...}.rs
  ├── features/{EdgeShape, Minutia, Skeleton, ...}.rs
  ├── templates/{FeatureTemplate, PersistentTemplate}.rs
  ├── extractor/{FeatureExtractor, RelativeContrastMask, ...}.rs
  ├── skeletons/{BinaryThinning, SkeletonGapFilter, ...}.rs
  ├── minutiae/{MinutiaCollector, TopMinutiaeFilter, ...}.rs
  ├── matcher/{EdgeHashes, EdgeSpider, Scoring, ...}.rs
  └── transparency/{ConsistentSkeleton, NoTransparency, ...}.rs
```

## Critical Rules

- Match .NET parameter values EXACTLY (77+ constants)
- Pipeline order must be identical to FeatureExtractor.Extract()
- PriorityQueue must be MAX-heap for SkeletonGap (largest distance = highest priority)
- EdgeShape uses `#[repr(C, packed(2))]` — 12 bytes, NO padding
- Polar cache (256x256) must use `std::sync::LazyLock`, NOT `static mut`
- CBOR serialization must produce byte-identical output to .NET (use `rename_all = "camelCase"`)
- Thread-local: `#[thread_local] static mut` requires unsafe (MatcherThread)
- No mixing of `crate::`, `crate::engine::`, `crate::sourceafis::engine::` — pick ONE convention

## Known Duplicate Definitions (Consolidate)

- `ShortPoint` in primitives.rs AND features.rs
- `Minutia` in primitives.rs AND features.rs  
- `EdgeShape` in features.rs AND matcher.rs
- `MinutiaPair` defined twice in matcher.rs
- `ScoringData` defined twice in matcher.rs
- `SkeletonGap` defined twice (one with Ord, one without)
- `SkeletonTypes` duplicated

## Testing Strategy

- Unit tests: one file per component (`tests/{primitives,features,templates,extractor,skeletons,minutiae,matcher}/`)
- Integration tests: `tests/integration/` (full pipeline matching, serialization roundtrip)
- Test resources: `test_resources/` (probe.png, matching.png, nonmatching.png, gray-*.dat)
- Recommended score threshold: score >= 40 for FMR 0.01%
