use crate::extractor::skeleton_graph::SkeletonGraph;
/// Graph-based minutia collection and quality filters.
/// Mirrors .NET:
///   MinutiaCollector.Collect — minutiae from skeleton graph nodes;
///   InnerMinutiaeFilter      — drop minutiae whose mask-displaced position
///                              falls outside the inner mask;
///   MinutiaCloudFilter       — drop minutiae in dense clouds (> MaxCloudSize
///                              neighbors within MinutiaCloudRadius);
///   TopMinutiaeFilter        — keep MaxMinutiae minutiae ranked by the
///                              distance to their SortByNeighbor-th neighbor.
use crate::features::Minutia;
use crate::features::MinutiaType;
use crate::parameters::Parameters;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::integers::Integers;

/// Collect minutiae from both skeletons: endings from the ridges skeleton,
/// bifurcations from the valleys skeleton. Mirrors .NET MinutiaCollector.Collect.
pub fn collect(
    ridges: &crate::extractor::skeleton_graph::SkeletonGraph,
    valleys: &crate::extractor::skeleton_graph::SkeletonGraph,
) -> Vec<Minutia> {
    let mut minutiae = Vec::new();
    collect_skeleton(&mut minutiae, ridges, MinutiaType::Ending);
    collect_skeleton(&mut minutiae, valleys, MinutiaType::Bifurcation);
    minutiae
}

fn collect_skeleton(minutiae: &mut Vec<Minutia>, skeleton: &SkeletonGraph, typ: MinutiaType) {
    let typ: MinutiaType = typ.clone();
    let typ: MinutiaType = typ; // shadow (moved per-iteration below via ref)
    for sminutia in &skeleton.minutiae {
        let live: Vec<usize> = sminutia
            .ridges
            .iter()
            .cloned()
            .filter(|&r| skeleton.is_attached(r))
            .collect();
        if live.len() == 1 {
            // Minutia direction = direction of its single ridge (pointing away
            // from the minutia toward the ridge's far end).
            let direction = skeleton.ridges[live[0]].direction();
            minutiae.push(Minutia::new(
                crate::primitives::int_point::IntPoint::new(
                    sminutia.position.x(),
                    sminutia.position.y(),
                ),
                direction,
                typ.clone(),
            ));
        }
    }
}

/// Drop minutiae displaced outside the inner mask. Mirrors .NET InnerMinutiaeFilter.
pub fn inner_filter(minutiae: &mut Vec<Minutia>, mask: &BooleanMatrix) {
    minutiae.retain(|m| {
        let arrow = (DoubleAngle::to_vector(m.angle) * (-Parameters::MASK_DISPLACEMENT)).round();
        mask.get_with_fallback(
            m.position.x() + arrow.x(),
            m.position.y() + arrow.y(),
            false,
        )
    });
}

/// Drop minutiae in clouds: keep m only if at most MaxCloudSize-1 other
/// minutiae lie within MinutiaCloudRadius. Mirrors .NET MinutiaCloudFilter.
pub fn cloud_filter(minutiae: &mut Vec<Minutia>) {
    let radius_sq = Integers::sq(Parameters::MINUTIA_CLOUD_RADIUS as i32);
    let kept: Vec<Minutia> = minutiae
        .iter()
        .filter(|m| {
            let neighbors = minutiae
                .iter()
                .filter(|n| {
                    let dx = n.position.x() - m.position.x();
                    let dy = n.position.y() - m.position.y();
                    dx * dx + dy * dy <= radius_sq
                })
                .count();
            Parameters::MAX_CLOUD_SIZE as i32 >= neighbors as i32 - 1
        })
        .cloned()
        .collect();
    minutiae.clear();
    minutiae.extend(kept);
}

/// Keep only MaxMinutiae minutiae ranked by distance to their
/// SortByNeighbor-th nearest neighbor (descending — isolated minutiae win).
/// Mirrors .NET TopMinutiaeFilter.
pub fn top_filter(minutiae: &mut Vec<Minutia>) {
    if minutiae.len() <= Parameters::MAX_MINUTIAE {
        return;
    }
    let sort_by = Parameters::SORT_BY_NEIGHBOR;
    let mut ranked: Vec<(i64, usize)> = minutiae
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut distances: Vec<i64> = minutiae
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, n)| {
                    let dx = (n.position.x() - m.position.x()) as i64;
                    let dy = (n.position.y() - m.position.y()) as i64;
                    dx * dx + dy * dy
                })
                .collect();
            distances.sort_unstable();
            let radius_sq = distances
                .get(Parameters::SORT_BY_NEIGHBOR)
                .copied()
                .unwrap_or(i64::MAX);
            (radius_sq, i)
        })
        .collect();
    // Descending radius (isolated minutiae first), stable like .NET LINQ.
    ranked.sort_by(|a, b| b.0.cmp(&a.0));
    let mut kept: Vec<Minutia> = ranked
        .into_iter()
        .take(Parameters::MAX_MINUTIAE)
        .map(|(_, i)| minutiae[i].clone())
        .collect();
    // .NET preserves the original relative order of the kept minutiae
    // (LINQ orderby is stable, Take preserves selection order). We sort kept
    // back by their original index for stability.
    kept.sort_by_key(|m| {
        minutiae
            .iter()
            .position(|o| o.position == m.position)
            .unwrap_or(usize::MAX)
    });
    *minutiae = kept;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor::skeleton_graph::SkeletonGraph;
    use crate::primitives::int_point::IntPoint;

    fn ridge_line_graph(len: i32) -> SkeletonGraph {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(len, 0));
        let pts: Vec<IntPoint> = (0..=len).step_by(3).map(|i| IntPoint::new(i, 0)).collect();
        let _ = g.connect(a, b, pts);
        g
    }

    #[test]
    fn test_collect_endings_only_from_single_ridge_nodes() {
        // 2-node line: both nodes have exactly 1 ridge → 2 endings.
        let g = ridge_line_graph(60);
        let mins = collect(&g, &SkeletonGraph::new());
        assert_eq!(mins.len(), 2);
        assert!(mins.iter().all(|m| m.typ == MinutiaType::Ending));
    }

    #[test]
    fn test_inner_filter_drops_outside_mask() {
        let mut mask = BooleanMatrix::new(100, 100);
        for y in 30..70 {
            for x in 30..70 {
                mask.set(x, y, true);
            }
        }
        let mut mins = vec![
            Minutia::new(IntPoint::new(50, 50), 0.0, MinutiaType::Ending),
            Minutia::new(IntPoint::new(5, 5), 0.0, MinutiaType::Ending),
        ];
        inner_filter(&mut mins, &mask);
        // Minutia at (5,5): direction 0 → arrow at (5-10, 5) = outside → dropped.
        assert_eq!(mins.len(), 1);
        assert_eq!(mins[0].position, IntPoint::new(50, 50));
    }

    #[test]
    fn test_cloud_filter_removes_dense_clusters() {
        // 6 minutiae within radius 20 of each other (MaxCloudSize=4):
        // each has 5 neighbors → 4 >= 5-1 is false → all removed.
        let mut mins: Vec<Minutia> = (0..6)
            .map(|i| Minutia::new(IntPoint::new(i * 3, 0), 0.0, MinutiaType::Ending))
            .collect();
        cloud_filter(&mut mins);
        assert!(mins.is_empty(), "dense cloud should be removed");
    }

    #[test]
    fn test_top_filter_keeps_isolated() {
        // 120 minutiae; the isolated ones (far from others) must survive.
        let mut mins: Vec<Minutia> = Vec::new();
        // 120 clustered tightly
        for i in 0..120 {
            mins.push(Minutia::new(
                IntPoint::new(i % 20, i / 20),
                0.0,
                MinutiaType::Ending,
            ));
        }
        // plus 1 far away
        mins.push(Minutia::new(
            IntPoint::new(1000, 1000),
            0.0,
            MinutiaType::Ending,
        ));
        top_filter(&mut mins);
        assert_eq!(mins.len(), Parameters::MAX_MINUTIAE);
        assert!(
            mins.iter().any(|m| m.position == IntPoint::new(1000, 1000)),
            "isolated minutia should rank highest and survive"
        );
    }
}
