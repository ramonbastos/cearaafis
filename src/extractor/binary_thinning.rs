/// BinaryThinning: iterative Zhang-Suen-style thinning with a 256-entry
/// neighborhood lookup table. Mirrors .NET BinaryThinning.cs exactly:
/// - 4 interleaved passes (evenY/evenX subgrids) per iteration;
/// - Removable/Ending classification from the precomputed table;
/// - "false ending" check prevents eroding line ends.
use crate::parameters::Parameters;
use crate::primitives::bool_matrix::BooleanMatrix;
use crate::primitives::int_point::IntPoint;
use crate::primitives::integers::Integers;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NeighborhoodType {
    Skeleton,
    Ending,
    Removable,
}

/// Precompute the 256-entry neighborhood classification table.
/// Mirrors .NET BinaryThinning.NeighborhoodTypes.
fn neighborhood_types() -> Vec<NeighborhoodType> {
    let mut types = vec![NeighborhoodType::Skeleton; 256];
    for mask in 0u32..256 {
        // Bit layout (mirroring .NET): TL=1, TC=2, TR=4, CL=8, CR=16, BL=32, BC=64, BR=128
        let tl = mask & 1 != 0;
        let tc = mask & 2 != 0;
        let tr = mask & 4 != 0;
        let cl = mask & 8 != 0;
        let cr = mask & 16 != 0;
        let bl = mask & 32 != 0;
        let bc = mask & 64 != 0;
        let br = mask & 128 != 0;
        let count = Integers::population_count(mask);
        let diagonal = !tc && !cl && tl || !cl && !bc && bl || !bc && !cr && br || !cr && !tc && tr;
        let horizontal = !tc && !bc && (tr || cr || br) && (tl || cl || bl);
        let vertical = !cl && !cr && (tl || tc || tr) && (bl || bc || br);
        let end = count == 1;
        if end {
            types[mask as usize] = NeighborhoodType::Ending;
        } else if !diagonal && !horizontal && !vertical {
            types[mask as usize] = NeighborhoodType::Removable;
        }
    }
    types
}

/// Whether an ending pixel's neighbor is itself a junction — such endings are
/// "false" and removable. Mirrors .NET BinaryThinning.IsFalseEnding.
fn is_false_ending(binary: &BooleanMatrix, ending: &IntPoint) -> bool {
    for relative_neighbor in IntPoint::CORNER_NEIGHBORS {
        let neighbor = *ending + *relative_neighbor;
        if binary.get_with_fallback(neighbor.x(), neighbor.y(), false) {
            let mut count = 0;
            for relative2 in IntPoint::CORNER_NEIGHBORS {
                let p = neighbor + *relative2;
                if binary.get_with_fallback(p.x(), p.y(), false) {
                    count += 1;
                }
            }
            return count > 2;
        }
    }
    false
}

/// Thin a binary image to 1-pixel-wide skeleton. Mirrors .NET BinaryThinning.Thin.
pub fn thin(input: &BooleanMatrix) -> BooleanMatrix {
    let neighborhood_types = neighborhood_types();
    let size = input.size();
    let mut mutable = BooleanMatrix::new(size.x() as usize, size.y() as usize);
    for y in 1..size.y() - 1 {
        for x in 1..size.x() - 1 {
            mutable.set(x as usize, y as usize, input.get(x as usize, y as usize));
        }
    }
    let mut thinned = BooleanMatrix::new(size.x() as usize, size.y() as usize);
    let mut removed_anything = true;
    let mut i = 0;
    while i < Parameters::THINNING_ITERATIONS && removed_anything {
        removed_anything = false;
        for even_y in 0..2i32 {
            for even_x in 0..2i32 {
                let mut y = 1 + even_y;
                while y < size.y() - 1 {
                    let mut x = 1 + even_x;
                    while x < size.x() - 1 {
                        let xu = x as usize;
                        let yu = y as usize;
                        if mutable.get(xu, yu)
                            && !thinned.get(xu, yu)
                            && !(mutable.get(xu, yu - 1)
                                && mutable.get(xu, yu + 1)
                                && mutable.get(xu - 1, yu)
                                && mutable.get(xu + 1, yu))
                        {
                            // Build the 8-neighbor bit mask (same bit layout as .NET).
                            let neighbors: u32 =
                                (if mutable.get(xu + 1, yu + 1) { 128 } else { 0 })
                                    | (if mutable.get(xu, yu + 1) { 64 } else { 0 })
                                    | (if mutable.get(xu - 1, yu + 1) { 32 } else { 0 })
                                    | (if mutable.get(xu + 1, yu) { 16 } else { 0 })
                                    | (if mutable.get(xu - 1, yu) { 8 } else { 0 })
                                    | (if mutable.get(xu + 1, yu - 1) { 4 } else { 0 })
                                    | (if mutable.get(xu, yu - 1) { 2 } else { 0 })
                                    | (if mutable.get(xu - 1, yu - 1) { 1 } else { 0 });
                            let t = neighborhood_types[neighbors as usize];
                            if t == NeighborhoodType::Removable
                                || (t == NeighborhoodType::Ending
                                    && is_false_ending(&mutable, &IntPoint::new(x, y)))
                            {
                                removed_anything = true;
                                mutable.set(xu, yu, false);
                            } else {
                                thinned.set(xu, yu, true);
                            }
                        }
                        x += 2;
                    }
                    y += 2;
                }
            }
        }
        i += 1;
    }
    thinned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thin_solid_block_to_skeleton() {
        let mut input = BooleanMatrix::new(20, 20);
        for y in 0..20 {
            for x in 0..20 {
                input.set(x, y, true);
            }
        }
        let thinned = thin(&input);
        // Thinned skeleton must be much smaller and non-empty.
        let count = (0..20)
            .map(|y| (0..20).filter(|&x| thinned.get(x, y)).count())
            .sum::<usize>();
        assert!(count > 0, "skeleton should not vanish");
        assert!(count < 400, "skeleton must be thinner than the block");
    }

    #[test]
    fn test_thin_preserves_single_line() {
        let mut input = BooleanMatrix::new(10, 3);
        for x in 0..10 {
            input.set(x, 1, true);
        }
        let thinned = thin(&input);
        // A 1px line survives (center pixels have 2 neighbors = skeleton).
        let count = (0..10).filter(|&x| thinned.get(x, 1)).count();
        assert!(count >= 3, "line core should survive, got {}", count);
    }

    #[test]
    fn test_neighborhood_table_basics() {
        let types = neighborhood_types();
        // mask 0 (no neighbors) → removable
        assert_eq!(types[0], NeighborhoodType::Removable);
        // mask 0b00000010 (only TC) → count=1 → Ending
        assert_eq!(types[0b00000010], NeighborhoodType::Ending);
        // mask 0b00011111 (5 neighbors) → removable per .NET rules (not a
        // diagonal/horizontal/vertical line, not an ending).
        assert_eq!(types[0b00011111], NeighborhoodType::Removable);
        // mask 255 is also "Removable" per the .NET table, but it can never be
        // reached: the thinning loop skips pixels whose 4 cross neighbors are
        // all set (that guard keeps solid regions intact).
    }
}
