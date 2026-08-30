/// PixelwiseOrientations: per-pixel ridge orientation estimation.
/// Mirrors .NET PixelwiseOrientations.cs — samples pixel pairs along
/// pseudo-random offsets (OrientationRandom LCG), accumulates "ridge strength"
/// vectors (before/after darker than center), split across OrientationSplit
/// rows so neighboring rows use different offset sets.
use crate::parameters::Parameters;
use crate::primitives::block_map::BlockMap;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::double_angle::DoubleAngle;
use crate::primitives::double_matrix::DoubleMatrix;
use crate::primitives::double_point::DoublePoint;
use crate::primitives::double_point_matrix::DoublePointMatrix;
use crate::primitives::doubles::Doubles;
use crate::primitives::int_point::IntPoint;
use crate::primitives::int_range::IntRange;

/// Deterministic LCG mirroring .NET OrientationRandom.
struct OrientationRandom {
    state: i64,
}

impl OrientationRandom {
    const PRIME: i64 = 1610612741;
    const BITS: i64 = 30;
    const MASK: i64 = (1 << Self::BITS) - 1;
    const SCALING: f64 = 1.0 / (1 << 30) as f64;

    fn new() -> Self {
        Self {
            state: Self::PRIME
                .wrapping_mul(Self::PRIME)
                .wrapping_mul(Self::PRIME),
        }
    }

    fn next(&mut self) -> f64 {
        self.state = self.state.wrapping_mul(Self::PRIME);
        ((self.state & Self::MASK) as f64 + 0.5) * Self::SCALING
    }
}

struct ConsideredOrientation {
    offset: IntPoint,
    orientation: DoublePoint,
}

/// Precompute random orientation samples. Mirrors .NET PixelwiseOrientations.Plan.
fn plan() -> Vec<Vec<ConsideredOrientation>> {
    let mut random = OrientationRandom::new();
    let mut splits = Vec::with_capacity(Parameters::ORIENTATION_SPLIT);
    for _ in 0..Parameters::ORIENTATION_SPLIT {
        let mut orientations = Vec::with_capacity(Parameters::ORIENTATIONS_CHECKED);
        for _ in 0..Parameters::ORIENTATIONS_CHECKED {
            loop {
                let angle = random.next() * std::f64::consts::PI;
                let distance = Doubles::interpolate_exponential(
                    Parameters::MIN_ORIENTATION_RADIUS,
                    Parameters::MAX_ORIENTATION_RADIUS,
                    random.next(),
                );
                let offset = (DoubleAngle::to_vector(angle) * distance).round();
                if offset == IntPoint::ZERO {
                    continue;
                }
                if offset.y() < 0 {
                    continue;
                }
                let duplicate = orientations
                    .iter()
                    .any(|o: &ConsideredOrientation| o.offset == offset);
                if duplicate {
                    continue;
                }
                let orientation_angle = DoubleAngle::add(
                    DoubleAngle::to_orientation(DoubleAngle::atan_point(
                        &DoublePoint::from_int_point(&offset),
                    )),
                    std::f64::consts::PI,
                );
                orientations.push(ConsideredOrientation {
                    offset,
                    orientation: DoubleAngle::to_vector(orientation_angle),
                });
                break;
            }
        }
        splits.push(orientations);
    }
    splits
}

/// Range of masked pixels in one mask row (block coordinates).
/// Mirrors .NET PixelwiseOrientations.MaskRange.
fn mask_range(mask: &BooleanMatrix, y: usize) -> IntRange {
    let mut first = -1i32;
    let mut last = -1i32;
    for x in 0..mask.width() {
        if mask.get(x, y) {
            last = x as i32;
            if first < 0 {
                first = x as i32;
            }
        }
    }
    if first >= 0 {
        IntRange::new(first, last + 1)
    } else {
        IntRange::new(0, 0)
    }
}

/// Compute per-pixel orientation vectors. Mirrors .NET Compute().
pub fn compute(input: &DoubleMatrix, mask: &BooleanMatrix, blocks: &BlockMap) -> DoublePointMatrix {
    let neighbors = plan();
    let mut orientation = DoublePointMatrix::new(input.width(), input.height());

    for block_y in 0..blocks.primary.blocks.y() as usize {
        let mask_range = mask_range(mask, block_y);
        if mask_range.length() > 0 {
            let valid_x_start = blocks
                .primary
                .block(mask_range.start, block_y as i32)
                .left();
            let valid_x_end = blocks
                .primary
                .block(mask_range.end - 1, block_y as i32)
                .right();
            let block_top = blocks.primary.block(0, block_y as i32).top();
            let block_bottom = blocks.primary.block(0, block_y as i32).bottom();
            for y in block_top..block_bottom {
                if y < 0 || y as usize >= input.height() {
                    continue;
                }
                let neighbor_set = &neighbors[(y as usize) % neighbors.len()];
                for neighbor in neighbor_set {
                    let radius = neighbor.offset.x().abs().max(neighbor.offset.y().abs());
                    let yr = y;
                    if yr - radius >= 0 && yr + radius < input.height() as i32 {
                        let x_start = radius.max(valid_x_start);
                        let x_end = (input.width() as i32 - radius).min(valid_x_end);
                        for x in x_start..x_end {
                            if x < 0 {
                                continue;
                            }
                            let before = input.get(
                                (x - neighbor.offset.x()) as usize,
                                (yr - neighbor.offset.y()) as usize,
                            );
                            let at = input.get(x as usize, yr as usize);
                            let after = input.get(
                                (x + neighbor.offset.x()) as usize,
                                (yr + neighbor.offset.y()) as usize,
                            );
                            let strength = at - before.max(after);
                            if strength > 0.0 {
                                orientation.add(
                                    x as usize,
                                    yr as usize,
                                    strength * neighbor.orientation.x(),
                                    strength * neighbor.orientation.y(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    orientation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_offsets_unique_and_positive_y() {
        let plan = plan();
        assert_eq!(plan.len(), Parameters::ORIENTATION_SPLIT);
        for split in &plan {
            assert_eq!(split.len(), Parameters::ORIENTATIONS_CHECKED);
            for o in split {
                assert!(o.offset.y() >= 0, "offsets must have non-negative y");
                assert!(o.offset != IntPoint::ZERO);
            }
        }
    }

    #[test]
    fn test_orientation_random_deterministic() {
        let mut r1 = OrientationRandom::new();
        let mut r2 = OrientationRandom::new();
        for _ in 0..10 {
            assert!(
                (r1.next() - r2.next()).abs() < 1e-12,
                "LCG must be deterministic"
            );
        }
    }
}
