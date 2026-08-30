# Score Comparison: Rust (CearaAFIS) vs .NET (SourceAFIS)

This document compares the matching scores produced by the Rust implementation
against the .NET baseline (sourceafis-net v3.14.0, FingerprintMatcherTest assertions).

## Reference: .NET FingerprintMatcherTest Assertions

Source: https://github.com/robertvazan/sourceafis-net/blob/master/SourceAFIS.Tests/FingerprintMatcherTest.cs

| Test Method | Probe | Candidate | Assertion | Description |
|-------------|-------|-----------|-----------|-------------|
| `MatchingPair()` | probe.png | matching.png | `score > 40` | Same finger |
| `NonmatchingPair()` | probe.png | nonmatching.png | `score < 20` | Different finger |
| `MatchingGray()` | gray-probe.dat | gray-matching.dat | `score > 40` | Same finger (raw) |
| `NonmatchingGray()` | gray-probe.dat | gray-nonmatching.dat | `score < 20` | Different finger (raw) |

## Scoring Thresholds (from .NET Parameters.cs)

| Threshold (raw) | Shaped Score | FMR | Description |
|-----------------|-------------|-------|-------------|
| 8.48 | 0 | N/A | Minimum score |
| 11.12 | ~3 | 50% | Barely distinguishable |
| 14.15 | ~7 | 10% | Weak match |
| 18.22 | ~10 | 1% | Possible match |
| 22.39 | ~20 | 0.1% | Recommended match |
| 27.24 | ~30 | 0.01% | Strong match |
| 32.01 | ~40 | 0.001% | Very strong match |

The recommended application threshold is **40** (FMR 0.01%), which maps to a raw
score of approximately 32.01+.

## Rust vs .NET Score Comparison

All Rust scores from `tests/match_demo.rs` using the SAME test resources as .NET.

| Test Pair | .NET Threshold | Rust Score | Rust Status | Within ±5 Tolerance? |
|-----------|----------------|------------|-------------|---------------------|
| probe.png ↔ matching.png | > 40 (assertion) | **63.4** | MATCH | ✅ YES (+16.4) |
| probe.png ↔ nonmatching.png | < 20 (assertion) | **27.7** | AMBIGUOUS | ⚠️ NO (+7.7) |
| probe.png ↔ probe.png (self) | N/A | **100.0** | MATCH | N/A |
| probe-jpeg ↔ probe.png | N/A | **62.4** | MATCH | ✅ Similar to PNG |
| probe-bmp ↔ probe.png | N/A | **100.0** | MATCH | ✅ Identical |
| gray-probe ↔ gray-matching | > 40 (assertion) | **57.9** | MATCH | ✅ YES |
| gray-probe ↔ gray-nonmatching | < 20 (assertion) | **23.7** | AMBIGUOUS | ⚠️ NO (+3.7) |
| gray-probe ↔ probe.png | N/A | **20.5** | AMBIGUOUS | Near threshold |

## Analysis

### ✅ PASSING (within or above tolerance)

1. **probe.png ↔ matching.png = 63.4** (target > 40): PASSING — strong match, +16.4 above threshold.
   - .NET assertion: `Assert.Greater(score, 40)` — Rust achieves 63.4 ✓
   
2. **probe-jpeg ↔ probe.png = 62.4**: PASSING — cross-format matching works correctly.
   - Confirms image decoder path (JPEG) produces same templates as PNG.
   
3. **probe-bmp ↔ probe.png = 100.0**: PASSING — self-match with different format.
   - Identical score to probe.png ↔ probe.png self-match (100.0).
   
4. **gray-probe ↔ gray-matching = 57.9** (target > 40): PASSING — raw grayscale matching.
   - .NET assertion: `Assert.Greater(score, 40)` — Rust achieves 57.9 ✓

### ⚠️ DEVIATING (above expected ceiling)

1. **probe.png ↔ nonmatching.png = 27.7** (target < 20): DEVIATING — +7.7 above assertion.
   - .NET assertion: `Assert.Less(score, 20)` — Rust fails this assertion.
   - **Cause**: The Rust edge-based root-pair enumeration may find too many false positive
     root pairs because the 10px tolerance (relaxed to 30px) combined with edge length
     tolerance (±15%, ±8px abs) and angle tolerance (0.35 rad ≈ 20°) allows non-matching
     minutiae to form initial pairs. In the .NET implementation, `EdgeHashes.Build(probe)`
     creates a hash-indexed edge table that filters more aggressively — only edges with
     EXACTLY matching hash entries are considered, reducing false positives significantly.
   
2. **gray-probe ↔ gray-nonmatching = 23.7** (target < 20): DEVIATING — +3.7 above assertion.
   - Similar root cause: broader edge matching tolerances in Rust allow false pairings.

## Root Cause: Non-Matching Scores Too High

The .NET implementation uses `EdgeHashes.Build()` to create a hash-indexed lookup table
from the probe minutiae. When matching candidate minutiae, it only considers candidate
minutiae whose edge hashes **exactly match** the probe hash table. This is a strict
filtering mechanism that dramatically reduces false positive root pairs.

The Rust implementation uses a simpler approach: for each probe minutia's N nearest
neighbors (max_neighbors=10), it builds an edge list and matches candidate edges by
length + angle tolerance. The tolerances are too generous:

| Tolerance | Rust Value | .NET Equivalent |
|-----------|-----------|-----------------|
| Length fraction | ±15% | Hash bucket (exact) |
| Length absolute | 8px | Hash bucket (exact) |
| Angle | ±0.35 rad (20°) | Hash bucket (exact) |

**Fix needed**: Implement `EdgeHashes` hash-based filtering before root-pair enumeration,
similar to the .NET `EdgeHashes.Build()` + `EdgeHashes.Lookup()` pattern.

## Scoring Formula Parity

| Component | .NET Parameter | Rust | Match? |
|-----------|---------------|------|--------|
| MinutiaScore | 0.032 | 0.032 | ✅ |
| MinutiaFractionScore | 8.98 | 8.98 | ✅ |
| EdgeScore | 0.265 | 0.265 | ✅ |
| SupportedMinutiaScore | 0.193 | 0.193 | ✅ |
| MinutiaTypeScore | 0.629 | 0.629 | ✅ |
| DistanceAccuracyScore | 9.9 | 9.9 | ✅ |
| AngleAccuracyScore | 2.79 | 2.79 | ✅ |
| Threshold FMR Max | 8.48 | 8.48 | ✅ |
| Threshold FMR 2 | 11.12 | 11.12 | ✅ |
| Threshold FMR 10 | 14.15 | 14.15 | ✅ |
| Threshold FMR 100 | 18.22 | 18.22 | ✅ |
| Threshold FMR 1000 | 22.39 | 22.39 | ✅ |
| Threshold FMR 10K | 27.24 | 27.24 | ✅ |
| Threshold FMR 100K | 32.01 | 32.01 | ✅ |

**All scoring parameters and thresholds are byte-identical to .NET.** The scoring
formula (7 components + Shape() function) is correctly implemented.

## Summary

| Metric | .NET | Rust | Status |
|--------|------|------|--------|
| Matching score (probe ↔ matching) | > 40 | 63.4 | ✅ PASS |
| Non-matching score (probe ↔ nonmatching) | < 20 | 27.7 | ⚠️ DEVIATING |
| Self-match (probe ↔ probe) | 100.0 | 100.0 | ✅ PASS |
| Raw grayscale matching | > 40 | 57.9 | ✅ PASS |
| Scoring parameters | 15/15 | 15/15 | ✅ IDENTICAL |

**Next step**: Implement EdgeHashes hash-based root-pair filtering to bring non-matching
scores below the 20 threshold (reduce false positive root pairs).
