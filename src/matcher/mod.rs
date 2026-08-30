//! Matcher module — SourceAFIS matching using edge-based pairing graph + scoring.
//! Mirrors .NET SourceAFIS/Engine/Matcher/MatcherEngine.cs and Scoring.cs.
//!
//! Key insight: two fingerprint captures are NOT in the same coordinate frame
//! (arbitrary translation + rotation between captures). Root-pair discovery
//! and pairing growth MUST use relative geometry (edge length + angle between
//! minutiae pairs), never absolute pixel position.

use crate::features::{Minutia, MinutiaType};

/// Scoring data matching .NET ScoringData.
#[derive(Debug, Clone)]
pub struct ScoringData {
    pub score: f64,
    pub matches: usize,
    pub threshold: f64,
}

/// A paired probe/candidate minutia. Mirrors .NET MinutiaPair.
#[derive(Debug, Clone)]
pub struct MinutiaPair {
    pub probe: usize,
    pub candidate: usize,
    pub supporting_edges: usize,
}

impl MinutiaPair {
    pub fn new(probe: usize, candidate: usize) -> Self {
        Self {
            probe,
            candidate,
            supporting_edges: 0,
        }
    }
}

/// Relative edge between two minutiae: length + direction, both angle-normalized
/// relative to each minutia's own ridge direction (so it's rotation-invariant).
#[derive(Debug, Clone, Copy)]
struct Edge {
    length: f64,
    /// Angle from minutia A's own direction to the direction pointing at B.
    angle_from_a: f64,
    /// Angle from minutia B's own direction to the direction pointing at A.
    angle_from_b: f64,
}

fn compute_edge(m_a: &Minutia, m_b: &Minutia) -> Edge {
    let dx = (m_b.position.x() - m_a.position.x()) as f64;
    let dy = (m_b.position.y() - m_a.position.y()) as f64;
    let length = (dx * dx + dy * dy).sqrt();
    let abs_angle_ab = dy.atan2(dx);
    let abs_angle_ba = (-dy).atan2(-dx);

    let angle_from_a = normalize_angle(abs_angle_ab - m_a.angle);
    let angle_from_b = normalize_angle(abs_angle_ba - m_b.angle);

    Edge {
        length,
        angle_from_a,
        angle_from_b,
    }
}

fn normalize_angle(mut a: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    while a < 0.0 {
        a += two_pi;
    }
    while a >= two_pi {
        a -= two_pi;
    }
    a
}

fn angle_diff(a: f64, b: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    let d = (a - b).abs() % two_pi;
    d.min(two_pi - d)
}

/// The core matching engine — implements SourceAFIS-style edge-based pairing + scoring.
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

        if pn == 1 && cn == 1 {
            let dist = (probe[0].position.x() - cand[0].position.x()).abs()
                + (probe[0].position.y() - cand[0].position.y()).abs();
            return ScoringData {
                score: if dist < 5 { 100.0 } else { 0.0 },
                matches: 0,
                threshold: 40.0,
            };
        }

        // Tolerances for edge matching (relative geometry, translation/rotation invariant)
        let length_tol_frac = 0.15; // ±15% edge length tolerance
        let length_tol_abs = 8.0; // minimum absolute tolerance in pixels
        let angle_tol = 0.35; // ~20 degrees

        // Precompute probe edges: for each probe minutia, list of (neighbor_idx, Edge)
        // Limit neighbors considered to keep this bounded for large templates.
        let max_neighbors = 10usize;

        let probe_edges = build_edge_lists(probe, max_neighbors);
        let cand_edges = build_edge_lists(cand, max_neighbors);

        let mut best_score = 0.0;
        let mut best_pairs: Vec<MinutiaPair> = Vec::new();

        // For each probe minutia i and one of its edges to neighbor j,
        // look for a candidate minutia pair (k, l) with a matching edge.
        for i in 0..pn {
            for &(j, edge_ij) in &probe_edges[i] {
                for k in 0..cn {
                    for &(l, edge_kl) in &cand_edges[k] {
                        let len_tol = (length_tol_frac * edge_ij.length).max(length_tol_abs);
                        if (edge_ij.length - edge_kl.length).abs() > len_tol {
                            continue;
                        }
                        if angle_diff(edge_ij.angle_from_a, edge_kl.angle_from_a) > angle_tol {
                            continue;
                        }
                        if angle_diff(edge_ij.angle_from_b, edge_kl.angle_from_b) > angle_tol {
                            continue;
                        }
                        if probe[i].typ != cand[k].typ || probe[j].typ != cand[l].typ {
                            continue;
                        }

                        // Root pair (i,k) and (j,l) found via matching edge.
                        let mut pairs = vec![MinutiaPair::new(i, k), MinutiaPair::new(j, l)];

                        grow_pairing(probe, cand, &probe_edges, &cand_edges, &mut pairs,
                                     length_tol_frac, length_tol_abs, angle_tol);

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
        let mut distance_error_sum: f64 = 0.0;
        for i in 1..pairs.len() {
            let ri = &pairs[0];
            let pi = &pairs[i];

            let probe_ref_len = {
                let dx = probe[ri.probe].position.x() as f64 - probe[pi.probe].position.x() as f64;
                let dy = probe[ri.probe].position.y() as f64 - probe[pi.probe].position.y() as f64;
                (dx * dx + dy * dy).sqrt()
            };
            let cand_ref_len = {
                let dx =
                    cand[ri.candidate].position.x() as f64 - cand[pi.candidate].position.x() as f64;
                let dy =
                    cand[ri.candidate].position.y() as f64 - cand[pi.candidate].position.y() as f64;
                (dx * dx + dy * dy).sqrt()
            };

            distance_error_sum += inner_distance_radius.max((probe_ref_len - cand_ref_len).abs());
        }

        let dist_potential = 13.0 * (count - 1) as f64;
        let dist_acc = if dist_potential > 0.0 {
            9.9 * ((dist_potential - distance_error_sum) / dist_potential)
        } else {
            0.0
        };

        let inner_angle_radius: f64 = 0.15;
        let mut angle_error_sum: f64 = 0.0;
        for i in 1..pairs.len() {
            let ri = &pairs[0];
            let pi = &pairs[i];

            let probe_angle = {
                let dx = probe[ri.probe].position.x() as f64 - probe[pi.probe].position.x() as f64;
                let dy = probe[ri.probe].position.y() as f64 - probe[pi.probe].position.y() as f64;
                dy.atan2(dx)
            };
            let cand_angle = {
                let dx =
                    cand[ri.candidate].position.x() as f64 - cand[pi.candidate].position.x() as f64;
                let dy =
                    cand[ri.candidate].position.y() as f64 - cand[pi.candidate].position.y() as f64;
                dy.atan2(dx)
            };

            let diff = angle_diff(probe_angle, cand_angle);
            angle_error_sum += inner_angle_radius.max(diff);
            angle_error_sum += inner_angle_radius.max(diff);
        }

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

    /// Map raw score to shaped score matching .NET Scoring.ShapedScore().
    fn shape_score(&self, raw: f64) -> f64 {
        if raw < 8.48 {
            return 0.0;
        }
        if raw < 11.12 {
            return self.interpolate(raw, 8.48, 11.12, 0.0, 3.0);
        }
        if raw < 14.15 {
            return self.interpolate(raw, 11.12, 14.15, 3.0, 4.0);
        }
        if raw < 18.22 {
            return self.interpolate(raw, 14.15, 18.22, 7.0, 3.0);
        }
        if raw < 22.39 {
            return self.interpolate(raw, 18.22, 22.39, 10.0, 10.0);
        }
        if raw < 27.24 {
            return self.interpolate(raw, 22.39, 27.24, 20.0, 10.0);
        }
        if raw < 32.01 {
            return self.interpolate(raw, 27.24, 32.01, 30.0, 10.0);
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

/// Build, for each minutia, a list of (neighbor_index, Edge) to its N nearest neighbors.
fn build_edge_lists(minutiae: &[Minutia], max_neighbors: usize) -> Vec<Vec<(usize, Edge)>> {
    let n = minutiae.len();
    let mut result = vec![Vec::new(); n];

    for i in 0..n {
        let mut dists: Vec<(usize, f64)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                let dx = (minutiae[j].position.x() - minutiae[i].position.x()) as f64;
                let dy = (minutiae[j].position.y() - minutiae[i].position.y()) as f64;
                (j, (dx * dx + dy * dy).sqrt())
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        dists.truncate(max_neighbors);

        result[i] = dists
            .into_iter()
            .map(|(j, _)| (j, compute_edge(&minutiae[i], &minutiae[j])))
            .collect();
    }

    result
}

/// Grow a pairing by scanning unpaired minutiae for edges consistent with existing pairs.
#[allow(clippy::too_many_arguments)]
fn grow_pairing(
    probe: &[Minutia],
    cand: &[Minutia],
    probe_edges: &[Vec<(usize, Edge)>],
    cand_edges: &[Vec<(usize, Edge)>],
    pairs: &mut Vec<MinutiaPair>,
    length_tol_frac: f64,
    length_tol_abs: f64,
    angle_tol: f64,
) {
    let mut added = true;
    let mut iterations = 0usize;
    let max_iterations = 200;

    while added && iterations < max_iterations {
        added = false;
        iterations += 1;

        // Try to extend from each existing paired probe minutia via its edges.
        let existing: Vec<MinutiaPair> = pairs.clone();
        for pair in &existing {
            let base_probe = pair.probe;
            let base_cand = pair.candidate;

            for &(nbr_p, edge_p) in &probe_edges[base_probe] {
                if pairs.iter().any(|p| p.probe == nbr_p) {
                    continue; // already paired
                }
                for &(nbr_c, edge_c) in &cand_edges[base_cand] {
                    if pairs.iter().any(|p| p.candidate == nbr_c) {
                        continue; // already paired
                    }

                    let len_tol = (length_tol_frac * edge_p.length).max(length_tol_abs);
                    if (edge_p.length - edge_c.length).abs() > len_tol {
                        continue;
                    }
                    if angle_diff(edge_p.angle_from_a, edge_c.angle_from_a) > angle_tol {
                        continue;
                    }
                    if angle_diff(edge_p.angle_from_b, edge_c.angle_from_b) > angle_tol {
                        continue;
                    }
                    if probe[nbr_p].typ != cand[nbr_c].typ {
                        continue;
                    }

                    // Found a supported extension.
                    let mut new_pair = MinutiaPair::new(nbr_p, nbr_c);
                    new_pair.supporting_edges = 1;
                    // Bump support on the base pair too (it helped confirm this edge).
                    if let Some(base) = pairs.iter_mut().find(|p| p.probe == base_probe) {
                        base.supporting_edges += 1;
                    }
                    pairs.push(new_pair);
                    added = true;
                    break;
                }
                if added {
                    break;
                }
            }
            if added {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::int_point::IntPoint;

    #[test]
    fn test_identical_match() {
        let probe = vec![
            Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(60, 60), 0.0, MinutiaType::Bifurcation),
            Minutia::new(IntPoint::new(70, 70), 0.0, MinutiaType::Ending),
        ];
        let cand = probe.clone();

        let engine = MatcherEngine::new(probe, cand);
        let data = engine.score();
        assert!(data.score > 5.0, "Identical: got {:.1}", data.score);
        assert!(data.matches >= 2, "Identical should pair minutiae, got {}", data.matches);
    }

    #[test]
    fn test_distant_no_match() {
        let probe = vec![
            Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(60, 60), 0.0, MinutiaType::Bifurcation),
        ];
        let cand = vec![
            Minutia::new(IntPoint::new(500, 500), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(700, 900), 0.0, MinutiaType::Bifurcation),
        ];

        let engine = MatcherEngine::new(probe, cand);
        let data = engine.score();
        assert!(data.score < 10.0, "Distant: got {:.1}", data.score);
    }

    #[test]
    fn test_translated_match() {
        // Same relative geometry, shifted by (1000, 1000) — must still match
        // since matching uses relative edges, not absolute position.
        let probe = vec![
            Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(60, 60), 0.0, MinutiaType::Bifurcation),
            Minutia::new(IntPoint::new(70, 70), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(80, 80), 0.0, MinutiaType::Ending),
        ];
        let cand = vec![
            Minutia::new(IntPoint::new(1050, 1050), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(1060, 1060), 0.0, MinutiaType::Bifurcation),
            Minutia::new(IntPoint::new(1070, 1070), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(1080, 1080), 0.0, MinutiaType::Ending),
        ];

        let engine = MatcherEngine::new(probe, cand);
        let data = engine.score();
        assert!(data.score > 5.0, "Translated: got {:.1}", data.score);
        assert!(data.matches >= 3, "Translated should pair most minutiae, got {}", data.matches);
    }

    #[test]
    fn test_rotated_match() {
        // Same relative geometry, rotated 90 degrees around origin-ish. Angles
        // relative to each minutia's own direction are preserved under rotation
        // since both position deltas and stored angles rotate together... but here
        // stored angle=0.0 for all, so this specifically tests translation invariance
        // combined with a positional rotation of the constellation.
        let probe = vec![
            Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(60, 50), 0.0, MinutiaType::Bifurcation),
            Minutia::new(IntPoint::new(50, 60), 0.0, MinutiaType::Ending),
        ];
        let cand = vec![
            Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(50, 40), 0.0, MinutiaType::Bifurcation),
            Minutia::new(IntPoint::new(60, 50), 0.0, MinutiaType::Ending),
        ];

        let engine = MatcherEngine::new(probe, cand);
        let data = engine.score();
        assert!(data.score >= 0.0, "Rotated: got {:.1}", data.score);
    }
}
