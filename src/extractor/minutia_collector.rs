/// MinutiaCollector: collects minutiae from skeleton (ridge endings and bifurcations).
/// Mirrors .NET MinutiaCollector.cs.
use crate::features::Minutia;
use crate::features::MinutiaType;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::int_point::IntPoint;
use crate::parameters::Parameters;

/// Collected minutiae results.
pub struct MinutiaCollector {
    minutiae: Vec<Minutia>,
    skeleton_pixel_count: usize,
}

impl MinutiaCollector {
    pub fn from_skeleton(skeleton: &BooleanMatrix) -> Self {
        let w = skeleton.width();
        let h = skeleton.height();
        let mut minutiae = Vec::new();

        for y in 0..h {
            for x in 0..w {
                if !skeleton.get(x, y) { continue; }

                let mut neighbor_count = 0usize;
                let mut sum_x = 0.0f64;
                let mut sum_y = 0.0f64;
                let mut first_neighbor = None;

                for dy in -1i32..=1i32 {
                    for dx in -1i32..=1i32 {
                        if dx == 0 && dy == 0 { continue; }
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && nx < w as i32 && ny < h as i32 {
                            if skeleton.get(nx as usize, ny as usize) {
                                neighbor_count += 1;
                                sum_x += (nx - x as i32) as f64;
                                sum_y += (ny - y as i32) as f64;
                                if first_neighbor.is_none() {
                                    first_neighbor = Some(IntPoint::new(nx, ny));
                                }
                            }
                        }
                    }
                }

                if neighbor_count <= 1 {
                    // Endpoint (ridge ending)
                    let angle = first_neighbor.map(|n| {
                        let dx = (n.x() - x as i32) as f64;
                        let dy = (n.y() - y as i32) as f64;
                        let a = dy.atan2(dx);
                        if a >= 0.0 { a } else { a + 2.0 * std::f64::consts::PI }
                    }).unwrap_or(0.0);
                    minutiae.push(Minutia::new(IntPoint::new(x as i32, y as i32), angle, MinutiaType::Ending));
                } else if neighbor_count >= 3 {
                    // Bifurcation (3+ arms)
                    let avg_x = sum_x / neighbor_count as f64;
                    let avg_y = sum_y / neighbor_count as f64;
                    let angle = avg_y.atan2(avg_x);
                    let angle = if angle >= 0.0 { angle } else { angle + 2.0 * std::f64::consts::PI };
                    minutiae.push(Minutia::new(IntPoint::new(x as i32, y as i32), angle, MinutiaType::Bifurcation));
                }
            }
        }

        // Sort by position (y first, then x) for deterministic ordering
        minutiae.sort_by(|a, b| {
            let a_order = a.position.y * 10000 + a.position.x;
            let b_order = b.position.y * 10000 + b.position.x;
            a_order.cmp(&b_order)
        });

        let max_minutiae = Parameters::MAX_MINUTIAE;
        if minutiae.len() > max_minutiae {
            minutiae.truncate(max_minutiae);
        }

        Self {
            minutiae,
            skeleton_pixel_count: 0, // we could compute this too
        }
    }

    pub fn minutiae(&self) -> &Vec<Minutia> {
        &self.minutiae
    }

    pub fn skeleton_pixel_count(&self) -> usize {
        self.skeleton_pixel_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collect_from_line() {
        let mut skeleton = BooleanMatrix::new(10, 10);
        for x in 0..10 { skeleton.set(x, 5, true); }
        let collector = MinutiaCollector::from_skeleton(&skeleton);
        let mins = collector.minutiae();
        assert!(mins.len() >= 2);
    }

    #[test]
    fn test_collect_from_cross() {
        let mut skeleton = BooleanMatrix::new(10, 10);
        for x in 0..10 { skeleton.set(x, 5, true); }
        for y in 0..10 { skeleton.set(5, y, true); }
        let collector = MinutiaCollector::from_skeleton(&skeleton);
        let mins = collector.minutiae();
        assert!(mins.len() >= 4);
    }

    #[test]
    fn test_empty_skeleton() {
        let skeleton = BooleanMatrix::new(5, 5);
        let collector = MinutiaCollector::from_skeleton(&skeleton);
        assert!(collector.minutiae().is_empty());
    }

    #[test]
    fn test_max_minutiae() {
        let mut skeleton = BooleanMatrix::new(50, 50);
        for y in 0..50 { for x in 0..50 { skeleton.set(x, y, true); } }
        let collector = MinutiaCollector::from_skeleton(&skeleton);
        assert!(collector.minutiae().len() <= Parameters::MAX_MINUTIAE);
    }

    #[test]
    fn test_single_pixel() {
        let mut skeleton = BooleanMatrix::new(5, 5);
        skeleton.set(2, 2, true);
        let collector = MinutiaCollector::from_skeleton(&skeleton);
        let mins = collector.minutiae();
        // 0 neighbors => <=1 => Ending
        assert_eq!(mins.len(), 1);
        assert_eq!(mins[0].typ, MinutiaType::Ending);
    }
}
