//! Unit tests for Integers utility functions.
//! Replaces the duplicate file

#[cfg(test)]
mod tests {
    use cearaafis::primitives::Integers;

    #[test]
    fn test_sq() {
        assert_eq!(Integers::sq(3), 9);
        assert_eq!(Integers::sq(0), 0);
        assert_eq!(Integers::sq(-5), 25);
    }

    #[test]
    fn test_round_up_div() {
        assert_eq!(Integers::round_up_div(10, 3), 4);
        assert_eq!(Integers::round_up_div(10, 5), 2);
        assert_eq!(Integers::round_up_div(10, 10), 1);
    }

    #[test]
    fn test_population_count() {
        assert_eq!(Integers::population_count(0), 0);
        assert_eq!(Integers::population_count(1), 1);
        assert_eq!(Integers::population_count(255), 8);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(Integers::leading_zeros(0), 32);
        assert_eq!(Integers::leading_zeros(1), 31);
        assert_eq!(Integers::leading_zeros(0x80000000), 0);
    }
}
