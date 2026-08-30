/// VoteFilter: majority vote smoothing with O(1) sliding window per pixel.
/// Mirrors .NET VoteFilter.cs exactly — uses a running integral-image trick
/// (counts[] stores ones+1 to disambiguate zero) so each pixel's window sum
/// is computed from its left/top/diagonal neighbors plus 4 corner corrections,
/// instead of rescanning the full (2r+1)^2 window.
use crate::parameters::Parameters;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::int_matrix::IntMatrix;
use crate::primitives::int_rect::IntRect;

/// Compute majority vote. `mask` == None means vote everywhere (only restricted
/// by border_distance). Only sets output=true where the vote passes; the output
/// starts all-false, exactly like .NET.
pub fn vote(
    input: &BooleanMatrix,
    mask: Option<&BooleanMatrix>,
    radius: i32,
    majority: f64,
    border_distance: i32,
) -> BooleanMatrix {
    let size = input.size();
    let width = size.x();
    let height = size.y();
    let rect = IntRect::new(
        border_distance,
        border_distance,
        width - 2 * border_distance,
        height - 2 * border_distance,
    );

    // thresholds[i] = ceil(majority * i) — precomputed so the inner check is
    // a table lookup like .NET.
    let table_len = ((2 * radius + 1) * (2 * radius + 1) + 1) as usize;
    let mut thresholds = vec![0i32; table_len];
    for (i, t) in thresholds.iter_mut().enumerate() {
        *t = (majority * i as f64).ceil() as i32;
    }

    let mut counts = IntMatrix::new(width as usize, height as usize);
    let mut output = BooleanMatrix::new(width as usize, height as usize);

    for y in rect.top()..rect.bottom() {
        let super_top = y - radius - 1;
        let super_bottom = y + radius;
        let y_min = (y - radius).max(0);
        let y_max = (y + radius).min(height - 1);
        let y_range = y_max - y_min + 1;

        for x in rect.left()..rect.right() {
            let masked_ok = match mask {
                None => true,
                Some(m) => m.get(x as usize, y as usize),
            };
            if !masked_ok {
                continue;
            }

            let left = if x > 0 {
                counts.get(x as usize - 1, y as usize)
            } else {
                0
            };
            let top = if y > 0 {
                counts.get(x as usize, y as usize - 1)
            } else {
                0
            };
            let diagonal = if x > 0 && y > 0 {
                counts.get(x as usize - 1, y as usize - 1)
            } else {
                0
            };
            let x_min = (x - radius).max(0);
            let x_max = (x + radius).min(width - 1);

            let mut ones: i32;
            if left > 0 && top > 0 && diagonal > 0 {
                // Sliding-window update: subtract the column leaving on the
                // left and the row leaving on the top, add the ones entering.
                ones = top + left - diagonal - 1;
                let super_left = x - radius - 1;
                let super_right = x + radius;
                if super_left >= 0
                    && super_top >= 0
                    && input.get(super_left as usize, super_top as usize)
                {
                    ones += 1;
                }
                if super_left >= 0
                    && super_bottom < height
                    && input.get(super_left as usize, super_bottom as usize)
                {
                    ones -= 1;
                }
                if super_right < width
                    && super_top >= 0
                    && input.get(super_right as usize, super_top as usize)
                {
                    ones -= 1;
                }
                if super_right < width
                    && super_bottom < height
                    && input.get(super_right as usize, super_bottom as usize)
                {
                    ones += 1;
                }
            } else {
                // Full scan fallback (first row/col of the integral image).
                let mut o = 0i32;
                for ny in y_min..=y_max {
                    for nx in x_min..=x_max {
                        if input.get(nx as usize, ny as usize) {
                            o += 1;
                        }
                    }
                }
                ones = o;
            }

            counts.set(x as usize, y as usize, ones + 1);
            let idx = (y_range * (x_max - x_min + 1)) as usize;
            if idx < table_len && ones >= thresholds[idx] {
                output.set(x as usize, y as usize, true);
            }
        }
    }

    output
}

/// Convenience wrappers matching the parameter sets used by the .NET pipeline.
pub fn block_errors_vote(input: &BooleanMatrix) -> BooleanMatrix {
    vote(
        input,
        None,
        Parameters::BLOCK_ERRORS_VOTE_RADIUS as i32,
        Parameters::BLOCK_ERRORS_VOTE_MAJORITY,
        Parameters::BLOCK_ERRORS_VOTE_BORDER_DISTANCE as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_all_true_stays_true() {
        let mut input = BooleanMatrix::new(40, 40);
        for y in 0..40 {
            for x in 0..40 {
                input.set(x, y, true);
            }
        }
        let voted = vote(&input, None, 2, 0.61, 4);
        // Interior region should remain true after voting.
        assert!(voted.get(20, 20));
    }

    #[test]
    fn test_vote_all_false_stays_false() {
        let input = BooleanMatrix::new(40, 40);
        let voted = vote(&input, None, 2, 0.61, 4);
        assert!(!voted.get(20, 20));
    }

    #[test]
    fn test_vote_removes_speckle() {
        // A single true pixel in a sea of false should be voted away.
        let mut input = BooleanMatrix::new(40, 40);
        input.set(20, 20, true);
        let voted = vote(&input, None, 2, 0.61, 4);
        assert!(
            !voted.get(20, 20),
            "isolated speckle should not survive vote"
        );
    }

    #[test]
    fn test_vote_output_dimensions() {
        let input = BooleanMatrix::new(30, 25);
        let voted = vote(&input, None, 1, 0.7, 4);
        assert_eq!(voted.width(), 30);
        assert_eq!(voted.height(), 25);
    }
}
