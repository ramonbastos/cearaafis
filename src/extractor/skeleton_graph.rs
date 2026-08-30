/// Skeleton graph: arena-based graph of skeleton minutiae connected by ridges.
/// Mirrors .NET's Skeleton/SkeletonMinutia/SkeletonRidge object graph, but uses
/// index-based references (arena) instead of shared mutable object references —
/// the idiomatic Rust translation of .NET's bidirectional linking.
///
/// Ridge semantics (mirroring .NET SkeletonRidge):
/// - every ridge has a "reversed" twin sharing the same points in reverse
///   (stored in consecutive arena slots: ridge i ↔ twin i+1 when even);
/// - `SkeletonMinutia.ridges` lists ridge indices attached at their START.
use crate::parameters::Parameters;
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::int_point::IntPoint;

/// One direction of a ridge: a polyline from `start` minutia to `end` minutia.
#[derive(Debug, Clone)]
pub struct GraphRidge {
    /// Index of the reversed twin ridge.
    pub reversed: usize,
    /// Start minutia index (None once detached).
    pub start: Option<usize>,
    /// End minutia index (None once detached).
    pub end: Option<usize>,
    /// Polyline points, ordered start → end.
    pub points: Vec<IntPoint>,
}

impl GraphRidge {
    /// Ridge direction: angle between two sample points, mirroring
    /// .NET SkeletonRidge.Direction() with skip/sample parameters and
    /// window shifting for ridges shorter than the sample span.
    pub fn direction(&self) -> f64 {
        let mut first = Parameters::RIDGE_DIRECTION_SKIP as i32;
        let mut last =
            (Parameters::RIDGE_DIRECTION_SKIP + Parameters::RIDGE_DIRECTION_SAMPLE - 1) as i32;
        let count = self.points.len() as i32;
        if last >= count {
            let shift = last - count + 1;
            last -= shift;
            first -= shift;
        }
        if first < 0 {
            first = 0;
        }
        let a = self.points[first as usize];
        let b = self.points[last as usize];
        DoubleAngle::atan_i32(&a, &b)
    }
}

/// Skeleton minutia: a node with attached ridges (indices into `ridges`).
#[derive(Debug, Clone)]
pub struct GraphMinutia {
    pub position: IntPoint,
    /// Ridge indices attached at their START to this minutia.
    pub ridges: Vec<usize>,
}

/// Arena-based skeleton graph.
#[derive(Debug, Clone, Default)]
pub struct SkeletonGraph {
    pub minutiae: Vec<GraphMinutia>,
    pub ridges: Vec<GraphRidge>,
}

impl SkeletonGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_minutia(&mut self, position: IntPoint) -> usize {
        self.minutiae.push(GraphMinutia {
            position,
            ridges: Vec::new(),
        });
        self.minutiae.len() - 1
    }

    /// Connect two minutiae with a ridge through `points` (which must start at
    /// the start minutia's position and end at the end minutia's position).
    /// Creates both directions and registers them in each minutia's list.
    pub fn connect(&mut self, start: usize, end: usize, points: Vec<IntPoint>) -> usize {
        let ridge_idx = self.ridges.len();
        let reversed_idx = ridge_idx + 1;
        self.ridges.push(GraphRidge {
            reversed: reversed_idx,
            start: Some(start),
            end: Some(end),
            points: points.clone(),
        });
        self.ridges.push(GraphRidge {
            reversed: ridge_idx,
            start: Some(end),
            end: Some(start),
            points: points.into_iter().rev().collect(),
        });
        self.minutiae[start].ridges.push(ridge_idx);
        self.minutiae[end].ridges.push(reversed_idx);
        ridge_idx
    }

    /// Detach a ridge from both ends (mirrors .NET SkeletonRidge.Detach).
    /// The twin is detached as well — .NET links them implicitly.
    pub fn detach_ridge(&mut self, ridge_idx: usize) {
        let reversed = self.ridges[ridge_idx].reversed;
        for &idx in &[ridge_idx, reversed] {
            if let Some(start) = self.ridges[idx].start {
                self.minutiae[start].ridges.retain(|&r| r != idx);
            }
            self.ridges[idx].start = None;
            self.ridges[idx].end = None;
        }
    }

    /// Whether a ridge is still attached (mirrors .NET's implicit check
    /// `ridge.Start != null`).
    pub fn is_attached(&self, ridge_idx: usize) -> bool {
        self.ridges[ridge_idx].start.is_some() && self.ridges[ridge_idx].end.is_some()
    }

    /// Compact the graph: drop detached ridges and minutiae with no ridges,
    /// reindexing everything. Returns old→new minutia index mapping.
    pub fn compact(&mut self) -> Vec<Option<usize>> {
        // Ridges alive = attached on both ends.
        let ridge_alive: Vec<bool> = self
            .ridges
            .iter()
            .map(|r| r.start.is_some() && r.end.is_some())
            .collect();

        // Minutiae alive = still references at least one alive ridge.
        let minutia_alive: Vec<bool> = self
            .minutiae
            .iter()
            .map(|m| m.ridges.iter().any(|&r| ridge_alive[r]))
            .collect();

        // Old→new index maps.
        let mut minutia_map: Vec<Option<usize>> = vec![None; self.minutiae.len()];
        let mut ridge_map: Vec<Option<usize>> = vec![None; self.ridges.len()];

        let mut new_minutiae: Vec<GraphMinutia> = Vec::new();
        for (i, m) in self.minutiae.iter().enumerate() {
            if minutia_alive[i] {
                minutia_map[i] = Some(new_minutiae.len());
                new_minutiae.push(GraphMinutia {
                    position: m.position,
                    ridges: Vec::new(),
                });
            }
        }

        let mut new_ridges: Vec<GraphRidge> = Vec::new();
        for (i, r) in self.ridges.iter().enumerate() {
            if ridge_alive[i] {
                ridge_map[i] = Some(new_ridges.len());
                new_ridges.push(GraphRidge {
                    // Keep the OLD twin index here; remap in the pass below.
                    reversed: r.reversed,
                    start: r.start.and_then(|s| minutia_map[s]),
                    end: r.end.and_then(|e| minutia_map[e]),
                    points: r.points.clone(),
                });
            }
        }
        // Remap reversed pointers from old→new indices now that ridge_map is full.
        for (new_i, r) in new_ridges.iter_mut().enumerate() {
            let _ = new_i;
            r.reversed = ridge_map[r.reversed].unwrap();
        }

        // Rebuild minutia ridge lists with new ridge indices; a ridge belongs
        // to a minutia's list iff the ridge STARTS there (mirrors .NET).
        for (old_i, m) in self.minutiae.iter().enumerate() {
            if !minutia_alive[old_i] {
                continue;
            }
            let new_i = minutia_map[old_i].unwrap();
            for &r in &m.ridges {
                if ridge_alive[r] {
                    if let Some(start) = self.ridges[r].start {
                        if start == old_i {
                            new_minutiae[new_i].ridges.push(ridge_map[r].unwrap());
                        }
                    }
                }
            }
        }

        self.minutiae = new_minutiae;
        self.ridges = new_ridges;
        minutia_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_creates_bidirectional_ridges() {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(10, 0));
        let ridge = g.connect(a, b, vec![IntPoint::new(0, 0), IntPoint::new(10, 0)]);
        assert_eq!(g.ridges[ridge].start, Some(a));
        let rev = g.ridges[ridge].reversed;
        assert_eq!(g.ridges[rev].start, Some(b));
        assert_eq!(g.minutiae[a].ridges, vec![ridge]);
        assert_eq!(g.minutiae[b].ridges, vec![rev]);
        // Reversed points are in reverse order.
        assert_eq!(g.ridges[rev].points[0], IntPoint::new(10, 0));
        assert_eq!(g.ridges[rev].points[1], IntPoint::new(0, 0));
    }

    #[test]
    fn test_detach_ridge_removes_from_both_ends() {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(5, 5));
        let r = g.connect(a, b, vec![IntPoint::new(0, 0), IntPoint::new(5, 5)]);
        let rev = g.ridges[r].reversed;
        g.detach_ridge(r);
        assert!(g.minutiae[a].ridges.is_empty());
        assert!(g.minutiae[b].ridges.is_empty());
        assert!(g.ridges[r].start.is_none());
        assert!(g.ridges[rev].start.is_none());
        assert!(!g.is_attached(r));
    }

    #[test]
    fn test_compact_drops_detached() {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(5, 5));
        let c = g.add_minutia(IntPoint::new(10, 10));
        let _keep = g.connect(a, b, vec![IntPoint::new(0, 0), IntPoint::new(5, 5)]);
        let drop = g.connect(b, c, vec![IntPoint::new(5, 5), IntPoint::new(10, 10)]);
        g.detach_ridge(drop);
        let map = g.compact();
        // Minutia c had only the dropped ridge → gone.
        assert_eq!(g.minutiae.len(), 2);
        assert_eq!(map[c], None);
        assert_eq!(map[a], Some(0));
        assert_eq!(map[b], Some(1));
        // One live ridge pair remains.
        assert_eq!(g.ridges.len(), 2);
    }

    #[test]
    fn test_direction_long_ridge() {
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(87, 0));
        let mut pts = Vec::new();
        for i in 0..30 {
            pts.push(IntPoint::new(i * 3, 0));
        }
        let r = g.connect(a, b, pts);
        let d = g.ridges[r].direction();
        // Horizontal ridge pointing +x: DoubleAngle::atan(x>0, y=0) = 0.
        assert!(d.abs() < 1e-9, "expected ~0, got {}", d);
    }

    #[test]
    fn test_direction_short_ridge_window_shift() {
        // 3-point ridge: last = 21 → shift → window [0, 1].
        let mut g = SkeletonGraph::new();
        let a = g.add_minutia(IntPoint::new(0, 0));
        let b = g.add_minutia(IntPoint::new(4, 0));
        let r = g.connect(
            a,
            b,
            vec![
                IntPoint::new(0, 0),
                IntPoint::new(2, 0),
                IntPoint::new(4, 0),
            ],
        );
        let d = g.ridges[r].direction();
        assert!(d.abs() < 1e-9, "short ridge still direction 0, got {}", d);
    }
}
