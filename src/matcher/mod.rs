//! Matcher module — SourceAFIS matching using edge-hash filtering + pairing graph + scoring.
//! Mirrors .NET SourceAFIS/Engine/Matcher/EdgeHashes.cs, EdgeShape.cs, EdgeSpider.cs,
//! MatcherEngine.cs and Scoring.cs.
//!
//! Key insight: two fingerprint captures are NOT in the same coordinate frame
//! (arbitrary translation + rotation between captures). Root-pair discovery
//! and pairing growth MUST use relative geometry (edge length + angle between
//! minutiae pairs), never absolute pixel position.

use crate::features::{Minutia, MinutiaType};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Edge hash parameters matching .NET Parameters.cs.
const MAX_DISTANCE_ERROR: f64 = 13.0;
const MAX_ANGLE_ERROR: f64 = std::f64::consts::PI / 180.0 * 10.0; // 10 degrees in radians
const COMPLEMENTARY_MAX_ANGLE_ERROR: f64 = std::f64::consts::PI - MAX_ANGLE_ERROR;

/// Scoring data matching .NET ScoringData.
#[derive(Debug, Clone)]
pub struct ScoringData {
    pub score: f64,
    pub matches: usize,
    pub threshold: f64,
}

/// Normalize angle to [0, 2π).
fn normalize_angle(angle: f64) -> f64 {
    if angle < 0.0 {
        angle + 2.0 * std::f64::consts::PI
    } else if angle >= 2.0 * std::f64::consts::PI {
        angle - 2.0 * std::f64::consts::PI
    } else {
        angle
    }
}

/// Compute angle difference in [-π, π).
fn angle_diff(a1: f64, a2: f64) -> f64 {
    let diff = (a1 - a2).abs();
    if diff <= std::f64::consts::PI {
        diff
    } else {
        2.0 * std::f64::consts::PI - diff
    }
}

/// EdgeShape mirrors .NET SourceAFIS/Engine/Features/EdgeShape.cs.
/// - Length stored as i16 (quantized by polar cache)
/// - ReferenceAngle = angle from minutia direction to edge vector
/// - NeighborAngle = angle from neighbor direction to opposite edge vector
/// All angles are rotation-invariant (relative to each minutia's own direction).
#[derive(Debug, Clone, Copy)]
pub struct EdgeShape {
    pub length: i16,
    pub reference_angle: f64, // angle from reference minutia direction to edge
    pub neighbor_angle: f64,  // angle from neighbor minutia direction to opposite edge
}

impl EdgeShape {
    /// Construct from two minutiae. Mirrors .NET EdgeShape(Minutia reference, Minutia neighbor).
    pub fn new(reference: &Minutia, neighbor: &Minutia) -> Self {
        let dx = (neighbor.position.x() - reference.position.x()) as f64;
        let dy = (neighbor.position.y() - reference.position.y()) as f64;
        let distance = (dx * dx + dy * dy).sqrt();

        // Quantize length using polar cache approach (same as .NET EdgeShape constructor).
        // .NET uses a 256x256 polar cache precomputed at static constructor time.
        // Here we compute on-the-fly with the same quantization logic:
        // 1. Normalize to first quadrant (y>=0, x>=0)
        // 2. Find leading zeros to determine quantization shift
        // 3. Scale distance by the quantization factor
        let polar_cache_bits = 8;

        let mut x = dx as i32;
        let mut y = dy as i32;
        let mut quadrant = 0i32;

        if y < 0 {
            x = -x;
            y = -y;
            quadrant = 1; // PI
        }
        if x < 0 {
            let tmp = -x;
            x = y;
            y = tmp;
            quadrant = 1 + quadrant; // PI + PI/2
        }

        // Leading zeros to find quantization shift (same as .NET Integers.LeadingZeros)
        let combined = ((x as u32) | (y as u32)) >> polar_cache_bits;
        let shift = if combined == 0 {
            0i32
        } else {
            32 - (combined.leading_zeros() as i32)
        };

        // Quantize length (same as .NET PolarDistanceCache lookup)
        let quantized_length = if shift > 0 {
            ((distance * (1 << (shift - 1)) as f64) as i16).max(1)
        } else {
            distance as i16
        };

        // Compute angle from x,y in normalized quadrant (same as .NET PolarAngleCache)
        let angle = if x > 0 || y > 0 {
            dy.atan2(dx) + std::f64::consts::PI * (quadrant as f64 / 2.0)
        } else {
            0.0
        };

        // Reference angle: difference from reference minutia direction
        let ref_angle = normalize_angle(reference.angle + angle);

        // Neighbor angle: difference from neighbor direction to opposite angle
        let opp_angle = normalize_angle(angle + std::f64::consts::PI);
        let neighbor_angle = normalize_angle((-opp_angle - neighbor.angle).abs());

        EdgeShape {
            length: quantized_length as i16,
            reference_angle: ref_angle,
            neighbor_angle,
        }
    }

    /// Compute hash matching .NET EdgeHashes.Hash().
    /// Hash = (referenceAngleBin << 24) + (neighborAngleBin << 16) + lengthBin
    pub fn hash(&self) -> i64 {
        let length_bin = (self.length as f64) / MAX_DISTANCE_ERROR;
        let ref_bin = self.reference_angle / MAX_ANGLE_ERROR;
        let neighbor_bin = self.neighbor_angle / MAX_ANGLE_ERROR;

        (ref_bin as i64) << 24 | (neighbor_bin as i64) << 16 | (length_bin as i64)
    }
}

/// EdgeHashes mirrors .NET SourceAFIS/Engine/Matcher/EdgeHashes.cs.
/// Builds a hash-indexed dictionary from probe edges for O(1) candidate lookup.
/// This is the KEY FILTER that prevents false-positive root pairs in non-matching templates.
pub struct EdgeHashes {
    /// hash -> list of entries with that hash
    pub map: HashMap<i64, Vec<EdgeHashEntry>>,
}

/// EdgeHashEntry stores the edge shape AND its source indices for direct lookup.
#[derive(Debug, Clone)]
pub struct EdgeHashEntry {
    pub shape: EdgeShape,
    pub reference: usize,
    pub neighbor: usize,
}

impl EdgeHashes {
    /// Build hash-indexed dictionary from template edges.
    /// Mirrors .NET EdgeHashes.Build(FingerprintTemplate template).
    pub fn build(minutiae: &[Minutia]) -> Self {
        let mut map: HashMap<i64, Vec<EdgeHashEntry>> = HashMap::new();

        for reference in 0..minutiae.len() {
            for neighbor in 0..minutiae.len() {
                if reference == neighbor {
                    continue;
                }
                let edge = EdgeShape::new(&minutiae[reference], &minutiae[neighbor]);

                // Coverage: add all hash bins within tolerance (same as .NET Coverage())
                let length_bin = (edge.length as f64) / MAX_DISTANCE_ERROR;
                let ref_bin = edge.reference_angle / MAX_ANGLE_ERROR;
                let neighbor_bin = edge.neighbor_angle / MAX_ANGLE_ERROR;

                // Check the exact hash bin plus adjacent bins within tolerance
                for len in -1i32..=1 {
                    for r in -1i32..=1 {
                        for nbr in -1i32..=1 {
                            let candidate_hash = ((ref_bin as i64 + r as i64) << 24)
                                | (((neighbor_bin as i64) + nbr as i64) << 16)
                                | (length_bin as i64 + len as i64);
                            map.entry(candidate_hash).or_default().push(EdgeHashEntry {
                                shape: edge,
                                reference,
                                neighbor,
                            });
                        }
                    }
                }
            }
        }

        EdgeHashes { map }
    }

    /// Find candidate edges that match an edge shape via hash lookup.
    /// Mirrors .NET: `probe.Hash.TryGetValue(EdgeHashes.Hash(cedge), out matches)`.
    /// The coverage expansion already happened at build() time (each edge was
    /// inserted at all tolerance-adjacent hash bins), so lookup is a single
    /// exact-hash probe — no further offsetting here.
    pub fn lookup(&self, edge: &EdgeShape) -> Vec<EdgeHashEntry> {
        let hash = edge.hash();
        match self.map.get(&hash) {
            Some(entries) => entries
                .iter()
                .filter(|entry| Self::matching(edge, &entry.shape))
                .cloned()
                .collect(),
            None => Vec::new(),
        }
    }

    /// Check if two edges match (same tolerance as .NET EdgeHashes.Matching).
    pub fn matching(probe: &EdgeShape, candidate: &EdgeShape) -> bool {
        let length_delta = (probe.length as f64) - (candidate.length as f64);
        if length_delta >= -MAX_DISTANCE_ERROR && length_delta <= MAX_DISTANCE_ERROR {
            let ref_diff = angle_diff(probe.reference_angle, candidate.reference_angle);
            if ref_diff <= MAX_ANGLE_ERROR || ref_diff >= COMPLEMENTARY_MAX_ANGLE_ERROR {
                let neighbor_diff = angle_diff(probe.neighbor_angle, candidate.neighbor_angle);
                if neighbor_diff <= MAX_ANGLE_ERROR
                    || neighbor_diff >= COMPLEMENTARY_MAX_ANGLE_ERROR
                {
                    return true;
                }
            }
        }
        false
    }
}

/// NeighborEdge mirrors .NET NeighborEdge (length, neighbor index, edge shape).
#[derive(Debug, Clone)]
pub struct NeighborEdge {
    pub neighbor: usize,
    pub shape: EdgeShape,
}

/// A paired probe/candidate minutia. Mirrors .NET MinutiaPair.
/// `probe_ref`/`candidate_ref` record the pair that discovered this pair during
/// BFS crawl (EdgeSpider.CollectEdges sets `pair.ProbeRef = reference.Probe`);
/// for the root pair these equal `probe`/`candidate` themselves. Scoring uses
/// these per-pair references (not the global root) to reconstruct the edge
/// used to accept the pair.
#[derive(Debug, Clone)]
pub struct MinutiaPair {
    pub probe: usize,
    pub candidate: usize,
    pub probe_ref: usize,
    pub candidate_ref: usize,
    pub supporting_edges: usize,
}

impl MinutiaPair {
    pub fn new(probe: usize, candidate: usize) -> Self {
        Self {
            probe,
            candidate,
            probe_ref: probe,
            candidate_ref: candidate,
            supporting_edges: 0,
        }
    }

    pub fn new_with_ref(
        probe: usize,
        candidate: usize,
        probe_ref: usize,
        candidate_ref: usize,
    ) -> Self {
        Self {
            probe,
            candidate,
            probe_ref,
            candidate_ref,
            supporting_edges: 0,
        }
    }
}

/// The core matching engine — implements SourceAFIS-style edge-hash + pairing graph + scoring.
/// Mirrors .NET SourceAFIS/Engine/Matcher/MatcherEngine.cs.
pub struct MatcherEngine {
    pub probe_minutiae: Vec<Minutia>,
    pub cand_minutiae: Vec<Minutia>,
}

impl MatcherEngine {
    pub fn new(probe: Vec<Minutia>, candidate: Vec<Minutia>) -> Self {
        Self {
            probe_minutiae: probe,
            cand_minutiae: candidate,
        }
    }

    /// Main scoring: enumerate root pairs via edge-hash matching (rotation/translation
    /// invariant), grow pairings via edge-spider, compute .NET-style score.
    pub fn score(&self) -> ScoringData {
        let pn = self.probe_minutiae.len();
        let cn = self.cand_minutiae.len();
        let probe = &self.probe_minutiae;
        let cand = &self.cand_minutiae;

        if pn < 1 || cn < 1 {
            return ScoringData {
                score: 0.0,
                matches: 0,
                threshold: 40.0,
            };
        }

        if cn == 1 {
            return ScoringData {
                score: 0.0,
                matches: 0,
                threshold: 40.0,
            };
        }

        // Build PROBE EdgeHashes — the KEY lookup table.
        // Mirrors .NET: `Hash = EdgeHashes.Build(probe)` in FingerprintMatcher.cs.
        // RootEnumerator then iterates CANDIDATE edges and does O(1) lookup in
        // probe.Hash to find matching probe edges.
        let probe_hashes = EdgeHashes::build(probe);

        // Build candidate NeighborEdge[][] (sorted by length like .NET)
        let cand_neighbor_edges = build_sorted_neighbor_edges(cand);
        let probe_neighbor_edges = build_sorted_neighbor_edges(probe);

        let mut best_score = 0.0;
        let mut best_pairs: Vec<MinutiaPair> = Vec::new();

        // Fix 2+3+4: Root pair enumeration with dedup, MinRootEdgeLength filter, and limits
        // Mirrors .NET RootEnumerator: period/phase scan, length filter, HashSet dedup,
        // MaxTriedRoots=70, MaxRootEdgeLookups=1633
        const MIN_ROOT_EDGE_LENGTH: i16 = 58; // same as .NET Parameters.MinRootEdgeLength
        const MAX_TRIED_ROOTS: usize = 70; // same as .NET Parameters.MaxTriedRoots
        const MAX_ROOT_LOOKUPS: usize = 1633; // same as .NET Parameters.MaxRootEdgeLookups

        let mut tried: usize = 0;
        let mut lookups: usize = 0;
        let mut seen_roots: HashSet<i64> = HashSet::new();

        // Phase 1: Enumerate long edges first (shortEdges=false)
        for i in 0..cn {
            if tried >= MAX_TRIED_ROOTS || lookups >= MAX_ROOT_LOOKUPS {
                break;
            }
            let cstar = &cand_neighbor_edges[i];
            for cedge in cstar {
                if tried >= MAX_TRIED_ROOTS || lookups >= MAX_ROOT_LOOKUPS {
                    break;
                }

                // Fix 3: MinRootEdgeLength filter — skip short edges
                if cedge.shape.length < MIN_ROOT_EDGE_LENGTH {
                    continue;
                }

                let edge_shape = &cedge.shape;
                let l = cedge.neighbor;

                // Hash-lookup probe edges that match this candidate edge
                let matching_entries = probe_hashes.lookup(edge_shape);
                lookups += 1;
                if lookups >= MAX_ROOT_LOOKUPS {
                    break;
                }

                for entry in matching_entries {
                    if tried >= MAX_TRIED_ROOTS || lookups >= MAX_ROOT_LOOKUPS {
                        break;
                    }

                    // k = probe minutia index matching candidate reference (i)
                    let k = entry.reference;

                    // Fix 2: Root pair dedup — same as .NET duplicateKey = match.Reference << 16 | creference
                    let root_key = (k as i64) << 32 | (i as i64);
                    if !seen_roots.insert(root_key) {
                        continue; // skip duplicate root pair
                    }

                    // .NET seeds EdgeSpider.Crawl with a SINGLE root pair (probe=k, candidate=i)
                    // and lets the pairing graph grow via BFS from there.
                    tried += 1;
                    let mut pairs = vec![MinutiaPair::new(k, i)];
                    grow_pairing(
                        probe,
                        cand,
                        &probe_neighbor_edges,
                        &cand_neighbor_edges,
                        &mut pairs,
                    );
                    let shaped = self.compute_score(probe, cand, &pairs);
                    if shaped > best_score {
                        best_score = shaped;
                        best_pairs = pairs;
                    }
                }
            }
        }

        // Phase 2: Enumerate short edges (shortEdges=true) — only if we haven't found a good score
        if best_score < 50.0 {
            for i in 0..cn {
                if tried >= MAX_TRIED_ROOTS {
                    break;
                }
                let cstar = &cand_neighbor_edges[i];
                for cedge in cstar {
                    if tried >= MAX_TRIED_ROOTS {
                        break;
                    }

                    let edge_shape = &cedge.shape;
                    let l = cedge.neighbor;

                    let matching_entries = probe_hashes.lookup(edge_shape);
                    lookups += 1;

                    for entry in matching_entries {
                        if tried >= MAX_TRIED_ROOTS {
                            break;
                        }

                        let k = entry.reference;
                        let j = entry.neighbor;

                        if j == l {
                            continue;
                        }

                        let root_key = (k as i64) << 32 | (i as i64);
                        if !seen_roots.insert(root_key) {
                            continue;
                        }

                        tried += 1;
                        let mut pairs = vec![MinutiaPair::new(k, i), MinutiaPair::new(j, l)];
                        grow_pairing(
                            probe,
                            cand,
                            &probe_neighbor_edges,
                            &cand_neighbor_edges,
                            &mut pairs,
                        );
                        let shaped = self.compute_score(probe, cand, &pairs);
                        if shaped > best_score {
                            best_score = shaped;
                            best_pairs = pairs;
                        }
                    }
                }
            }
        }

        ScoringData {
            score: best_score,
            matches: best_pairs.len(),
            threshold: 40.0,
        }
    }

    /// Compute scoring matching .NET Scoring.Compute() — 7-component formula.
    fn compute_score(&self, probe: &[Minutia], cand: &[Minutia], pairs: &[MinutiaPair]) -> f64 {
        let count = pairs.len();
        if count < 2 {
            return 0.0;
        }

        let minutia_score = 0.032 * count as f64;
        let minutia_frac_probe = count as f64 / probe.len() as f64;
        let minutia_frac_cand = count as f64 / cand.len() as f64;
        let minutia_frac = 0.5 * (minutia_frac_probe + minutia_frac_cand);
        let minutia_frac_score = 8.98 * minutia_frac;

        let mut supporting_sum: usize = 0;
        let mut supported_count = 0usize;
        let mut type_hits = 0usize;

        for pair in pairs {
            supporting_sum += pair.supporting_edges;
            if pair.supporting_edges >= 1 {
                supported_count += 1;
            }
            if probe[pair.probe].typ == cand[pair.candidate].typ {
                type_hits += 1;
            }
        }

        let edge_count = count + supporting_sum;
        let edge_score = 0.265 * edge_count as f64;
        let supported_score = 0.193 * supported_count as f64;
        let type_score = 0.629 * type_hits as f64;

        let inner_distance_radius: f64 = 9.0;
        let inner_angle_radius: f64 = 0.15;
        let mut distance_error_sum: f64 = 0.0;
        let mut angle_error_sum: f64 = 0.0;
        // Mirrors .NET Scoring.Compute(): for each non-root pair, rebuild the
        // EdgeShape from (probe_ref -> probe) and (candidate_ref -> candidate)
        // — the pair's OWN discovery reference, not the global root — and
        // compare length + reference_angle + neighbor_angle (rotation-invariant).
        for pi in pairs.iter().skip(1) {
            let probe_edge = EdgeShape::new(&probe[pi.probe_ref], &probe[pi.probe]);
            let cand_edge = EdgeShape::new(&cand[pi.candidate_ref], &cand[pi.candidate]);

            distance_error_sum += inner_distance_radius
                .max(((probe_edge.length as f64) - (cand_edge.length as f64)).abs());
            angle_error_sum += inner_angle_radius.max(angle_diff(
                probe_edge.reference_angle,
                cand_edge.reference_angle,
            ));
            angle_error_sum += inner_angle_radius.max(angle_diff(
                probe_edge.neighbor_angle,
                cand_edge.neighbor_angle,
            ));
        }
        let dist_potential = 13.0 * (count - 1) as f64;
        let dist_acc = if dist_potential > 0.0 {
            9.9 * ((dist_potential - distance_error_sum) / dist_potential)
        } else {
            0.0
        };

        let angle_potential = 2.0 * 0.15 * (count - 1) as f64;
        let angle_acc = if angle_potential > 0.0 {
            2.79 * ((angle_potential - angle_error_sum) / angle_potential)
        } else {
            0.0
        };

        let total = minutia_score
            + minutia_frac_score
            + supported_score
            + edge_score
            + type_score
            + dist_acc
            + angle_acc;
        self.shape_score(total)
    }

    /// Map raw score to shaped score matching .NET Scoring.Shape().
    fn shape_score(&self, raw: f64) -> f64 {
        if raw < 8.48 {
            return 0.0;
        }
        if raw < 11.12 {
            return self.interpolate(raw, 8.48, 11.12, 0.0, 3.0);
        }
        if raw < 14.15 {
            return self.interpolate(raw, 11.12, 14.15, 3.0, 7.0);
        }
        if raw < 18.22 {
            return self.interpolate(raw, 14.15, 18.22, 10.0, 10.0);
        }
        if raw < 22.39 {
            return self.interpolate(raw, 18.22, 22.39, 20.0, 10.0);
        }
        if raw < 27.24 {
            return self.interpolate(raw, 22.39, 27.24, 30.0, 10.0);
        }
        if raw < 32.01 {
            return self.interpolate(raw, 27.24, 32.01, 40.0, 10.0);
        }
        (raw - 32.01) / (32.01 - 18.22) * 30.0 + 50.0
    }

    fn interpolate(&self, raw: f64, min: f64, max: f64, start: f64, length: f64) -> f64 {
        if max <= min {
            return start;
        }
        (raw - min) / (max - min) * length + start
    }
}

/// Build NeighborEdge[][] for a minutia array. Mirrors .NET NeighborEdge.Build().
/// Filters to only the 9 closest neighbors (EdgeTableNeighbors = 9).
/// Sorted by edge length descending (longest first).
const EDGE_TABLE_NEIGHBORS: usize = 9;

fn build_sorted_neighbor_edges(minutiae: &[Minutia]) -> Vec<Vec<NeighborEdge>> {
    let n = minutiae.len();
    let mut result: Vec<Vec<NeighborEdge>> = vec![Vec::new(); n];
    for i in 0..n {
        // Build all candidate neighbor edges for this minutia with distances
        let mut candidates: Vec<(usize, f64, EdgeShape)> = Vec::new();
        for j in 0..n {
            if i == j {
                continue;
            }
            let dx = minutiae[j].position.x() as f64 - minutiae[i].position.x() as f64;
            let dy = minutiae[j].position.y() as f64 - minutiae[i].position.y() as f64;
            let dist = (dx * dx + dy * dy).sqrt();
            let shape = EdgeShape::new(&minutiae[i], &minutiae[j]);
            candidates.push((j, dist, shape));
        }

        // Sort by actual Euclidean distance (closest first), take 9 nearest
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

        // Build NeighborEdge list sorted by edge length ASCENDING (mirrors .NET
        // NeighborEdge.BuildTable: `a.Shape.Length.CompareTo(b.Shape.Length)`).
        // This ascending order is required for EdgeSpider.MatchPairs' two-pointer
        // sliding window over candidate/probe edge lengths.
        result[i] = candidates
            .into_iter()
            .take(EDGE_TABLE_NEIGHBORS)
            .map(|(neighbor, _dist, shape)| NeighborEdge { neighbor, shape })
            .collect();
        result[i].sort_by(|a, b| a.shape.length.cmp(&b.shape.length));
    }
    result
}

/// Grow pairing via edge-spider (mirrors .NET EdgeSpider.Crawl()/CollectEdges()).
/// Processes ONE new pair at a time via BFS queue, checking used-probe/used-candidate
/// sets before accepting — prevents the combinatorial blowup of batching all matches
/// per pass (which previously hung on self-similar templates like probe-vs-probe).
fn grow_pairing(
    _probe: &[Minutia],
    _cand: &[Minutia],
    probe_edges: &Vec<Vec<NeighborEdge>>,
    cand_edges: &Vec<Vec<NeighborEdge>>,
    pairs: &mut Vec<MinutiaPair>,
) {
    let mut used_probe: HashSet<usize> = pairs.iter().map(|p| p.probe).collect();
    let mut used_cand: HashSet<usize> = pairs.iter().map(|p| p.candidate).collect();
    let mut queue: std::collections::VecDeque<MinutiaPair> = pairs.iter().cloned().collect();
    pairs.clear();

    while let Some(next) = queue.pop_front() {
        pairs.push(next.clone());

        for cedge in &cand_edges[next.candidate] {
            let cn = cedge.neighbor;
            if used_cand.contains(&cn) {
                continue;
            }
            // Mirrors .NET EdgeSpider.MatchPairs: for each candidate edge, scan
            // ALL matching probe edges (not just the first) — length-sorted
            // window in .NET, linear scan here since EDGE_TABLE_NEIGHBORS is small.
            for pedge in &probe_edges[next.probe] {
                let pn = pedge.neighbor;
                if used_probe.contains(&pn) {
                    continue;
                }
                if EdgeHashes::matching(&pedge.shape, &cedge.shape) {
                    used_probe.insert(pn);
                    used_cand.insert(cn);
                    // Record the discovering pair (next) as this new pair's
                    // reference — mirrors .NET CollectEdges: `pair.ProbeRef =
                    // reference.Probe; pair.CandidateRef = reference.Candidate`.
                    queue.push_back(MinutiaPair::new_with_ref(
                        pn,
                        cn,
                        next.probe,
                        next.candidate,
                    ));
                    break; // this cn is claimed; move to next candidate edge
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::MinutiaType;
    use crate::primitives::double_angle::DoubleAngle;
    use crate::primitives::int_point::IntPoint;

    fn make_minutia(x: i32, y: i32, angle: f64, typ: MinutiaType) -> Minutia {
        Minutia::new(IntPoint::new(x, y), angle, typ)
    }

    #[test]
    fn edge_shape_hash_basic() {
        let reference = make_minutia(
            100,
            100,
            DoubleAngle::atan(1.0, 1.0),
            MinutiaType::Bifurcation,
        );
        let neighbor = make_minutia(110, 110, DoubleAngle::atan(0.0, 1.0), MinutiaType::Ending);
        let edge = EdgeShape::new(&reference, &neighbor);
        let hash = edge.hash();
        assert!(hash != 0, "Edge hash should not be zero");
    }

    #[test]
    fn edge_shape_hash_consistency() {
        let m1 = make_minutia(100, 100, 0.5, MinutiaType::Bifurcation);
        let m2 = make_minutia(110, 110, 1.0, MinutiaType::Ending);
        let m3 = make_minutia(100, 100, 0.5, MinutiaType::Bifurcation);
        let m4 = make_minutia(110, 110, 1.0, MinutiaType::Ending);
        let e1 = EdgeShape::new(&m1, &m2);
        let e2 = EdgeShape::new(&m3, &m4);
        assert_eq!(e1.hash(), e2.hash(), "Same edges should have same hash");
    }

    #[test]
    fn edge_hash_build_basic() {
        let probe: Vec<Minutia> = vec![
            make_minutia(100, 100, 0.5, MinutiaType::Bifurcation),
            make_minutia(110, 110, 1.0, MinutiaType::Ending),
            make_minutia(120, 100, 0.5, MinutiaType::Bifurcation),
        ];
        let hashes = EdgeHashes::build(&probe);
        assert!(
            !hashes.map.is_empty(),
            "EdgeHashes.build should produce non-empty map"
        );
    }

    #[test]
    fn root_enumeration_finds_matching_pair_for_identical_templates() {
        // Identical minutiae sets, edge length 113px (> MinRootEdgeLength=58px)
        // must produce a non-zero score via natural root enumeration + pairing
        // growth — no special-case fallback needed.
        let min1 = Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending);
        let min2 = Minutia::new(IntPoint::new(130, 130), 0.0, MinutiaType::Bifurcation);
        let probe = vec![min1.clone(), min2.clone()];
        let cand = vec![min1, min2];

        let engine = MatcherEngine::new(probe, cand);
        let data = engine.score();
        assert!(
            data.score > 0.0,
            "identical 2-minutia templates should score > 0, got {:?}",
            data
        );
        assert_eq!(data.matches, 2, "both minutiae should pair up");
    }
}
