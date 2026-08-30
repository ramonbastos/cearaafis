/// SkeletonTracing: build a skeleton graph from a thinned binary image.
/// Mirrors .NET SkeletonTracing.cs:
///   1. FindMinutiae — pixels with 1 or 3+ neighbors (endings/bifurcations);
///   2. LinkNeighboringMinutiae — merge adjacent minutia pixels into clusters;
///   3. MinutiaCenters — one SkeletonMinutia per cluster at the averaged position;
///   4. TraceRidges — walk from each minutia along non-minutia pixels until
///      reaching another minutia; create ridges (deduplicated via `leads`);
///   5. FixLinkingGaps — fill short lines between minutia center and ridge start.
use crate::extractor::skeleton_graph::SkeletonGraph;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::int_point::IntPoint;
use std::collections::HashMap;

/// Pixels with 1 or >2 neighbors on the thinned image. Mirrors .NET FindMinutiae.
fn find_minutiae(thinned: &BooleanMatrix) -> Vec<IntPoint> {
    let mut result = Vec::new();
    let size = thinned.size();
    for at in size.iterate() {
        if thinned.get(at.x() as usize, at.y() as usize) {
            let mut count = 0;
            for relative in IntPoint::CORNER_NEIGHBORS {
                if thinned.get_with_fallback(at.x() + relative.x(), at.y() + relative.y(), false) {
                    count += 1;
                }
            }
            if count == 1 || count > 2 {
                result.push(at);
            }
        }
    }
    result
}

/// Union adjacent minutia pixels into clusters. Returns position → shared list
/// of cluster member positions. Mirrors .NET LinkNeighboringMinutiae.
fn link_neighboring_minutiae(minutiae: &[IntPoint]) -> HashMap<IntPoint, Vec<IntPoint>> {
    let mut linking: HashMap<IntPoint, Vec<IntPoint>> = HashMap::new();
    for &minutia_pos in minutiae {
        let mut own_links: Option<Vec<IntPoint>> = None;
        for neighbor_relative in IntPoint::CORNER_NEIGHBORS {
            let neighbor_pos = minutia_pos + *neighbor_relative;
            if let Some(neighbor_links) = linking.get(&neighbor_pos) {
                // Compare by identity of the shared list via its first element.
                let is_same = own_links
                    .as_ref()
                    .map(|o| o[0] == neighbor_links[0])
                    .unwrap_or(false);
                if !is_same {
                    if let Some(own) = own_links.take() {
                        // Merge own cluster into the neighbor's cluster.
                        let mut merged = linking.get(&neighbor_pos).unwrap().clone();
                        merged.extend(own.iter().cloned());
                        for merged_pos in &own {
                            linking.insert(*merged_pos, merged.clone());
                        }
                        linking.insert(neighbor_pos, merged.clone());
                        own_links = Some(merged);
                    } else {
                        own_links = Some(neighbor_links.clone());
                    }
                }
            }
        }
        let list = own_links.unwrap_or_default();
        // The cluster list owns its members; insert position → list.
        // Note: to keep single shared ownership simple in Rust, we rebuild the
        // map value as a fresh Vec per position group; membership identity is
        // tracked by the first element (the cluster key).
        let mut list = list;
        list.push(minutia_pos);
        let key = list[0];
        linking.insert(minutia_pos, list.clone());
        if key != minutia_pos {
            linking.insert(key, list);
        }
    }
    linking
}

/// One minutia node per cluster, positioned at the cluster's average.
/// Mirrors .NET MinutiaCenters (iteration ordered by position for determinism).
fn minutia_centers(
    graph: &mut SkeletonGraph,
    linking: &HashMap<IntPoint, Vec<IntPoint>>,
) -> HashMap<IntPoint, usize> {
    let mut centers: HashMap<IntPoint, usize> = HashMap::new();
    let mut keys: Vec<IntPoint> = linking.keys().cloned().collect();
    keys.sort_by(|a, b| (a.y(), a.x()).cmp(&(b.y(), b.x())));

    let mut cluster_centers: HashMap<IntPoint, usize> = HashMap::new();
    for current_pos in keys {
        let linked = &linking[&current_pos];
        let primary_pos = linked[0];
        let center_idx = *cluster_centers.entry(primary_pos).or_insert_with(|| {
            let mut sum = IntPoint::ZERO;
            for linked_pos in linked {
                sum = sum + *linked_pos;
            }
            let center =
                IntPoint::new(sum.x() / linked.len() as i32, sum.y() / linked.len() as i32);
            graph.add_minutia(center)
        });
        centers.insert(current_pos, center_idx);
    }
    centers
}

/// Walk ridges from each minutia. Mirrors .NET TraceRidges.
fn trace_ridges(
    graph: &mut SkeletonGraph,
    thinned: &BooleanMatrix,
    minutiae_points: &HashMap<IntPoint, usize>,
) {
    let mut leads: HashMap<IntPoint, ()> = HashMap::new();
    let mut starts: Vec<IntPoint> = minutiae_points.keys().cloned().collect();
    starts.sort_by(|a, b| (a.y(), a.x()).cmp(&(b.y(), b.x())));

    for minutia_point in starts {
        for start_relative in IntPoint::CORNER_NEIGHBORS {
            let start = minutia_point + *start_relative;
            let start_ok = thinned.get_with_fallback(start.x(), start.y(), false)
                && !minutiae_points.contains_key(&start)
                && !leads.contains_key(&start);
            if !start_ok {
                continue;
            }

            let mut ridge_points = vec![minutia_point, start];
            let mut previous = minutia_point;
            let mut current = start;
            loop {
                let mut next = IntPoint::ZERO;
                for next_relative in IntPoint::CORNER_NEIGHBORS {
                    next = current + *next_relative;
                    if thinned.get_with_fallback(next.x(), next.y(), false) && next != previous {
                        break;
                    }
                }
                previous = current;
                current = next;
                ridge_points.push(current);
                if minutiae_points.contains_key(&current) {
                    break;
                }
            }
            let end_point = current;
            let start_idx = minutiae_points[&minutia_point];
            let end_idx = minutiae_points[&end_point];
            graph.connect(start_idx, end_idx, ridge_points.clone());
            leads.insert(ridge_points[1], ());
            // Reversed twin's second point = second-to-last point of the line.
            let rev_second = ridge_points[ridge_points.len() - 2];
            leads.insert(rev_second, ());
        }
    }
}

/// Fill the gap between a minutia's center and its ridge's first point.
/// Mirrors .NET FixLinkingGaps.
fn fix_linking_gaps(graph: &mut SkeletonGraph) {
    for m in 0..graph.minutiae.len() {
        let ridge_idxs: Vec<usize> = graph.minutiae[m].ridges.clone();
        for r in ridge_idxs {
            let minutia_pos = graph.minutiae[m].position;
            if graph.ridges[r].points[0] != minutia_pos {
                let first = graph.ridges[r].points[0];
                let filling = first.line_to(&minutia_pos);
                // Add the filling points to the REVERSED ridge (they extend
                // from the ridge's far end back toward this minutia).
                let rev = graph.ridges[r].reversed;
                for point in filling.iter().skip(1) {
                    graph.ridges[rev].points.push(*point);
                }
            }
        }
    }
}

/// Full tracing pipeline. Mirrors .NET SkeletonTracing.Trace.
pub fn trace(thinned: &BooleanMatrix) -> SkeletonGraph {
    let mut graph = SkeletonGraph::new();
    let minutia_points = find_minutiae(thinned);
    let linking = link_neighboring_minutiae(&minutia_points);
    let minutia_map = minutia_centers(&mut graph, &linking);
    trace_ridges(&mut graph, thinned, &minutia_map);
    fix_linking_gaps(&mut graph);
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_simple_line_two_endings() {
        let mut thinned = BooleanMatrix::new(10, 3);
        for x in 0..10 {
            thinned.set(x, 1, true);
        }
        let graph = trace(&thinned);
        // 2 endings → 2 minutiae connected by 1 ridge pair.
        assert_eq!(graph.minutiae.len(), 2);
        assert_eq!(graph.ridges.len(), 2);
        assert!(graph.is_attached(0));
    }

    #[test]
    fn test_trace_cross_four_endings() {
        let mut thinned = BooleanMatrix::new(9, 9);
        for x in 0..9 {
            thinned.set(x, 4, true);
        }
        for y in 0..9 {
            thinned.set(4, y, true);
        }
        let graph = trace(&thinned);
        // 4 line endings + center junction = 5 minutiae.
        assert_eq!(graph.minutiae.len(), 5);
        // 4 arms → 4 ridge pairs.
        assert_eq!(graph.ridges.len(), 8);
    }

    #[test]
    fn test_find_minutiae_counts() {
        let mut thinned = BooleanMatrix::new(10, 3);
        for x in 0..10 {
            thinned.set(x, 1, true);
        }
        let minutiae = find_minutiae(&thinned);
        // Only the two endpoints (1 neighbor each).
        assert_eq!(minutiae.len(), 2);
    }
}
