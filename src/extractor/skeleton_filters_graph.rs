/// Skeleton filters over the skeleton graph — mirrors .NET SkeletonFilters.cs
/// and the individual filter classes:
///   DotFilter      — remove minutiae with zero ridges (isolated dots);
///   PoreFilter     — merge pores (3-ridge nodes with a 2-arm triangle);
///   GapFilter      — join nearby endings across gaps (priority queue);
///   KnotFilter     — merge double-ridged nodes (2 ridges = pass-through);
///   TailFilter     — remove short tails hanging off junctions;
///   FragmentFilter — remove short isolated fragments.
/// Each mutating filter runs DotFilter cleanup afterwards like .NET.
use crate::extractor::skeleton_graph::SkeletonGraph;
use crate::parameters::Parameters;
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::int_point::IntPoint;
use crate::primitives::integers::Integers;
use std::collections::HashSet;

/// Remove all minutiae with no attached ridges. Mirrors .NET SkeletonDotFilter.
pub fn dot_filter(graph: &mut SkeletonGraph) {
    let mut dots: Vec<usize> = Vec::new();
    for (i, m) in graph.minutiae.iter().enumerate() {
        let live = m.ridges.iter().any(|&r| graph.is_attached(r));
        if m.ridges.is_empty() || !live {
            dots.push(i);
        }
    }
    for i in dots {
        graph.minutiae[i].ridges.clear();
    }
    // Dots have no ridges; they get dropped by compact().
    graph.compact();
}

/// Merge pores: 3-ridge minutiae whose two non-exit arms end at the same
/// 3-ridge node within MaxPoreArm length. Mirrors .NET SkeletonPoreFilter.
pub fn pore_filter(graph: &mut SkeletonGraph) {
    for m in 0..graph.minutiae.len() {
        let live_ridges: Vec<usize> = graph.minutiae[m]
            .ridges
            .iter()
            .cloned()
            .filter(|&r| graph.is_attached(r))
            .collect();
        if live_ridges.len() != 3 {
            continue;
        }
        for exit in 0..3 {
            let exit_ridge = live_ridges[exit];
            let arm1 = live_ridges[(exit + 1) % 3];
            let arm2 = live_ridges[(exit + 2) % 3];
            let arm1_end = graph.ridges[arm1].end;
            let arm2_end = graph.ridges[arm2].end;
            let exit_end = graph.ridges[exit_ridge].end;
            if arm1_end == arm2_end
                && exit_end != arm1_end
                && arm1_end != Some(m)
                && exit_end != Some(m)
            {
                let end = arm1_end.unwrap();
                let arm1_len = graph.ridges[arm1].points.len();
                let arm2_len = graph.ridges[arm2].points.len();
                let end_ridge_count = graph.minutiae[end]
                    .ridges
                    .iter()
                    .filter(|&&r| graph.is_attached(r))
                    .count();
                if end_ridge_count == 3
                    && arm1_len <= Parameters::MAX_PORE_ARM
                    && arm2_len <= Parameters::MAX_PORE_ARM
                {
                    // Merge: detach both arms and connect minutia → end directly.
                    graph.detach_ridge(arm1);
                    graph.detach_ridge(arm2);
                    let line = graph.minutiae[m]
                        .position
                        .line_to(&graph.minutiae[end].position);
                    graph.connect(m, end, line);
                }
                break;
            }
        }
    }
    knot_filter(graph);
}

/// Join nearby endings across gaps. Mirrors .NET SkeletonGapFilter.
pub fn gap_filter(graph: &mut SkeletonGraph) {
    #[derive(Clone, PartialEq, Eq, Default)]
    struct SkeletonGap {
        distance: i32,
        end1: usize,
        end2: usize,
    }
    impl PartialOrd for SkeletonGap {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for SkeletonGap {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            // PriorityQueue pops the SMALLEST first; smaller distance first.
            other.distance.cmp(&self.distance)
        }
    }

    // Enumerate candidate gap pairs (ordered iteration for determinism).
    let mut queue = crate::primitives::priority_queue::PriorityQueue::new();
    let count = graph.minutiae.len();
    for end1 in 0..count {
        let r1: Vec<usize> = graph.minutiae[end1]
            .ridges
            .iter()
            .cloned()
            .filter(|&r| graph.is_attached(r))
            .collect();
        if r1.len() != 1 || graph.ridges[r1[0]].points.len() < Parameters::SHORTEST_ENDED_MINUTIA {
            continue;
        }
        for end2 in 0..count {
            if end2 == end1 {
                continue;
            }
            let r2: Vec<usize> = graph.minutiae[end2]
                .ridges
                .iter()
                .cloned()
                .filter(|&r| graph.is_attached(r))
                .collect();
            if r2.len() != 1
                || graph.ridges[r1[0]].end == Some(end2)
                || graph.ridges[r2[0]].points.len() < Parameters::SHORTEST_ENDED_MINUTIA
            {
                continue;
            }
            if is_within_gap_limits(graph, end1, end2) {
                let distance =
                    (graph.minutiae[end1].position.x() - graph.minutiae[end2].position.x()).pow(2)
                        + (graph.minutiae[end1].position.y() - graph.minutiae[end2].position.y())
                            .pow(2);
                queue.add(SkeletonGap {
                    distance,
                    end1,
                    end2,
                });
            }
        }
    }

    // Shadow of all live ridge points (avoid creating overlapping ridges).
    let mut shadow: HashSet<(i32, i32)> = HashSet::new();
    for r in 0..graph.ridges.len() {
        if graph.is_attached(r) {
            for p in &graph.ridges[r].points {
                shadow.insert((p.x(), p.y()));
            }
        }
    }

    let mut added: Vec<(usize, usize)> = Vec::new();
    while !queue.is_empty() {
        let gap = queue.pop();
        let live1 = graph.minutiae[gap.end1]
            .ridges
            .iter()
            .filter(|&&r| graph.is_attached(r))
            .count();
        let live2 = graph.minutiae[gap.end2]
            .ridges
            .iter()
            .filter(|&&r| graph.is_attached(r))
            .count();
        if live1 == 1 && live2 == 1 {
            let p1 = graph.minutiae[gap.end1].position;
            let p2 = graph.minutiae[gap.end2].position;
            let line = p1.line_to(&p2);
            // Check overlap in the interior of the line (clamped for short lines).
            let skip = Parameters::TOLERATED_GAP_OVERLAP.min(line.len() / 2);
            let end = line.len().saturating_sub(skip);
            let overlap = line[skip..end]
                .iter()
                .any(|p| shadow.contains(&(p.x(), p.y())));
            if !overlap {
                added.push((gap.end1, gap.end2));
                for p in &line {
                    shadow.insert((p.x(), p.y()));
                }
            }
        }
    }
    for (a, b) in added {
        if graph.minutiae[a]
            .ridges
            .iter()
            .filter(|&&r| graph.is_attached(r))
            .count()
            == 1
            && graph.minutiae[b]
                .ridges
                .iter()
                .filter(|&&r| graph.is_attached(r))
                .count()
                == 1
        {
            let p1 = graph.minutiae[a].position;
            let p2 = graph.minutiae[b].position;
            let line = p1.line_to(&p2);
            graph.connect(a, b, line);
        }
    }

    knot_filter(graph);
}

/// Angle sample point used for gap-direction checks. Mirrors .NET AngleSampleForGapRemoval.
fn angle_sample_for_gap_removal(graph: &SkeletonGraph, minutia: usize) -> IntPoint {
    let ridge = graph.minutiae[minutia]
        .ridges
        .iter()
        .cloned()
        .find(|&r| graph.is_attached(r))
        .expect("gap filter expects a minutia with exactly 1 ridge");
    if Parameters::GAP_ANGLE_OFFSET < graph.ridges[ridge].points.len() {
        graph.ridges[ridge].points[Parameters::GAP_ANGLE_OFFSET]
    } else {
        let end = graph.ridges[ridge].end.unwrap();
        graph.minutiae[end].position
    }
}

/// Whether two endings may be joined across a gap. Mirrors .NET IsWithinGapLimits.
fn is_within_gap_limits(graph: &SkeletonGraph, end1: usize, end2: usize) -> bool {
    let p1 = graph.minutiae[end1].position;
    let p2 = graph.minutiae[end2].position;
    let distance_sq = (p1.x() - p2.x()).pow(2) + (p1.y() - p2.y()).pow(2);
    if distance_sq <= Integers::sq(Parameters::MAX_RUPTURE_SIZE as i32) {
        return true;
    }
    if distance_sq > Integers::sq(Parameters::MAX_GAP_SIZE as i32) {
        return false;
    }
    let gap_direction = DoubleAngle::atan_i32(&p1, &p2);
    let sample1 = angle_sample_for_gap_removal(graph, end1);
    let direction1 = DoubleAngle::atan_i32(&p1, &sample1);
    if DoubleAngle::distance(direction1, DoubleAngle::opposite(gap_direction))
        > Parameters::MAX_GAP_ANGLE
    {
        return false;
    }
    let sample2 = angle_sample_for_gap_removal(graph, end2);
    let direction2 = DoubleAngle::atan_i32(&p2, &sample2);
    if DoubleAngle::distance(direction2, gap_direction) > Parameters::MAX_GAP_ANGLE {
        return false;
    }
    true
}

/// Merge pass-through nodes (2 ridges) into one longer ridge.
/// Mirrors .NET SkeletonKnotFilter.
pub fn knot_filter(graph: &mut SkeletonGraph) {
    for m in 0..graph.minutiae.len() {
        let live: Vec<usize> = graph.minutiae[m]
            .ridges
            .iter()
            .cloned()
            .filter(|&r| graph.is_attached(r))
            .collect();
        if live.len() != 2 || graph.ridges[live[0]].reversed != live[1] {
            continue;
        }
        // extended = ridge whose reversed ends at this node... in the .NET the
        // knot node's two ridges are merged into one spanning ridge. In our
        // arena: extend ridge[live[0]] through the node with ridge[live[1]].
        let (mut extended, mut removed) = (live[0], live[1]);
        if graph.ridges[extended].points.len() < graph.ridges[removed].points.len() {
            std::mem::swap(&mut extended, &mut removed);
            extended = graph.ridges[extended].reversed;
            removed = graph.ridges[removed].reversed;
        }
        // Splice removed's points (minus the shared node point) onto extended.
        let start = extended;
        let end = graph.ridges[removed].end;
        let mut points = graph.ridges[start].points.clone();
        points.pop(); // drop the shared minutia position
        points.extend(graph.ridges[removed].points.iter().cloned());
        graph.detach_ridge(start);
        graph.detach_ridge(removed);
        if let (Some(s), Some(e)) = (graph.ridges[start].start, end) {
            graph.connect(s, e, points);
        }
    }
    dot_filter(graph);
}

/// Remove short tails: single ridge ending whose far end is a junction.
/// Mirrors .NET SkeletonTailFilter.
pub fn tail_filter(graph: &mut SkeletonGraph) {
    let mut to_detach: Vec<usize> = Vec::new();
    for m in 0..graph.minutiae.len() {
        let live: Vec<usize> = graph.minutiae[m]
            .ridges
            .iter()
            .cloned()
            .filter(|&r| graph.is_attached(r))
            .collect();
        if live.len() == 1 {
            let ridge = live[0];
            if let Some(end) = graph.ridges[ridge].end {
                let end_live = graph.minutiae[end]
                    .ridges
                    .iter()
                    .filter(|&&r| graph.is_attached(r))
                    .count();
                if end_live >= 3 && graph.ridges[ridge].points.len() < Parameters::MIN_TAIL_LENGTH {
                    to_detach.push(ridge);
                }
            }
        }
    }
    for r in to_detach {
        graph.detach_ridge(r);
    }
    dot_filter(graph);
    knot_filter(graph);
}

/// Remove short isolated fragments (ridge with both ends being endings).
/// Mirrors .NET SkeletonFragmentFilter.
pub fn fragment_filter(graph: &mut SkeletonGraph) {
    let mut to_detach: Vec<usize> = Vec::new();
    for m in 0..graph.minutiae.len() {
        let live: Vec<usize> = graph.minutiae[m]
            .ridges
            .iter()
            .cloned()
            .filter(|&r| graph.is_attached(r))
            .collect();
        if live.len() == 1 {
            let ridge = live[0];
            if let Some(end) = graph.ridges[ridge].end {
                let end_live = graph.minutiae[end]
                    .ridges
                    .iter()
                    .filter(|&&r| graph.is_attached(r))
                    .count();
                if end_live == 1
                    && graph.ridges[ridge].points.len() < Parameters::MIN_FRAGMENT_LENGTH
                {
                    to_detach.push(ridge);
                }
            }
        }
    }
    for r in to_detach {
        graph.detach_ridge(r);
    }
    dot_filter(graph);
}

/// Full filter pipeline. Mirrors .NET SkeletonFilters.Apply.
pub fn apply(graph: &mut SkeletonGraph) {
    dot_filter(graph);
    pore_filter(graph);
    gap_filter(graph);
    tail_filter(graph);
    fragment_filter(graph);
    graph.compact();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_two_nodes(len: usize) -> (SkeletonGraph, usize, usize, usize) {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(len as i32, 0));
        let mut pts = Vec::new();
        for i in 0..len {
            pts.push(IntPoint::new(i as i32, 0));
        }
        let r = g.connect(a, b, pts);
        (g, a, b, r)
    }

    #[test]
    fn test_dot_filter_removes_isolated() {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(50, 0));
        let _ = g.connect(a, b, vec![IntPoint::new(0, 0), IntPoint::new(50, 0)]);
        let lone = g.add_minutia(IntPoint::new(200, 200));
        assert_eq!(g.minutiae.len(), 3);
        dot_filter(&mut g);
        assert_eq!(g.minutiae.len(), 2);
        let _ = lone;
    }

    #[test]
    fn test_fragment_filter_removes_short_fragments() {
        // Short ridge (5 points < MIN_FRAGMENT_LENGTH=22) with both ends
        // having 1 ridge → detached, both nodes become dots and are removed.
        let (mut g, _a, _b, _r) = build_two_nodes(5);
        fragment_filter(&mut g);
        assert_eq!(g.minutiae.len(), 0);
    }

    #[test]
    fn test_tail_filter_removes_short_tail_to_junction() {
        // Build a junction node c with 3 arms, one short (a-b of length 5).
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0)); // tail end
        let c = g.add_minutia(IntPoint::new(100, 100)); // junction (will get 3 ridges)
        let d1 = g.add_minutia(IntPoint::new(200, 100));
        let d2 = g.add_minutia(IntPoint::new(100, 200));
        let long1: Vec<IntPoint> = (0..100).map(|i| IntPoint::new(i * 2, 100)).collect();
        let long2: Vec<IntPoint> = (0..100).map(|i| IntPoint::new(100, i * 2)).collect();
        let _tail = g.connect(a, c, vec![IntPoint::new(0, 0), IntPoint::new(100, 100)]);
        let _r1 = g.connect(c, d1, long1);
        let _r2 = g.connect(c, d2, long2);
        // Tail length = 2 points < MIN_TAIL_LENGTH=21 → detached → a becomes dot.
        tail_filter(&mut g);
        // a was removed as a dot after tail detach.
        assert!(
            g.minutiae.iter().all(|m| m.position != IntPoint::new(0, 0)),
            "tail end should be removed"
        );
    }

    fn long2_pts() -> Vec<IntPoint> {
        (0..100).map(|i| IntPoint::new(100, i * 2)).collect()
    }
}
