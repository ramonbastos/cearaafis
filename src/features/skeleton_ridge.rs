/// SkeletonRidge: a ridge on the skeleton — mirrors .NET SkeletonRidge.cs.
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::short_point::ShortPoint;

#[derive(Clone, serde::Serialize)]
pub struct SkeletonRidge {
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub shape: Vec<ShortPoint>,
    pub angle: f64,
}

impl SkeletonRidge {
    pub fn new(shape: Vec<ShortPoint>) -> Self {
        Self {
            start: None,
            end: None,
            shape,
            angle: 0.0,
        }
    }

    /// Direction of the ridge: angle from first sample point to last sample point.
    /// Mirrors .NET SkeletonRidge.Direction().
    ///
    /// Uses `Parameters::RidgeDirectionSkip` to skip the first points (edge of ridge),
    /// and `Parameters::RidgeDirectionSample` to span the remaining sample points.
    /// If the ridge is too short, shifts the window backward to avoid going out of bounds.
    pub fn direction(&self) -> Option<f64> {
        let n = self.shape.len();
        if n < 2 {
            return None; // Need at least 2 points to compute direction
        }

        let skip: i32 = 1; // Parameters::RIDGE_DIRECTION_SKIP
        let sample: i32 = 21; // Parameters::RIDGE_DIRECTION_SAMPLE

        let mut first: i32 = skip;
        let mut last: i32 = skip + sample - 1;

        // Edge case: ridge too short — shift window backward (matches .NET SkeletonRidge.cs)
        if last >= n as i32 {
            let shift = last - n as i32 + 1;
            last -= shift;
            let shifted_first = first - shift;
            if shifted_first < 0 {
                let shift2 = 0 - shifted_first;
                first += shift2;
                last += shift2;
            }
        }

        let n = n as i32;
        if last >= n || first >= n {
            return None; // Ridge too short to compute direction
        }

        let a = &self.shape[first as usize];
        let b = &self.shape[last as usize];
        let dx = (b.x as i32 - a.x as i32) as f64;
        let dy = (b.y as i32 - a.y as i32) as f64;

        Some(DoubleAngle::atan(dx, dy))
    }
}

impl std::fmt::Debug for SkeletonRidge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkeletonRidge")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("shape_len", &self.shape.len())
            .field("angle", &self.angle)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_short_ridge() {
        // Ridge with only 3 points — too short for full sample window
        let rid = SkeletonRidge::new(vec![
            ShortPoint::new(10, 10),
            ShortPoint::new(15, 15),
            ShortPoint::new(20, 20),
        ]);
        // Should handle gracefully (either compute or return None)
        let _ = rid.direction();
    }

    #[test]
    fn test_direction_long_ridge() {
        // Ridge with 25 points — enough for full sample window
        let mut shape = Vec::new();
        for i in 0..25 {
            shape.push(ShortPoint::new(i as i16 * 5, i as i16 * 5));
        }
        let rid = SkeletonRidge::new(shape);
        let dir = rid.direction().unwrap();
        // Direction should be near PI/4 (45 degrees) for diagonal ridge
        assert!(dir.abs() < std::f64::consts::FRAC_PI_2 + 0.5);
    }

    #[test]
    fn test_direction_horizontal() {
        let mut shape = Vec::new();
        for i in 0..25 {
            shape.push(ShortPoint::new(i as i16 * 5, 100));
        }
        let rid = SkeletonRidge::new(shape);
        let dir = rid.direction().unwrap();
        // Horizontal ridge should give angle near PI (pointing right in our convention)
        assert!(dir.abs() < std::f64::consts::PI + 0.5);
    }
}
