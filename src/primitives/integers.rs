/// Static helper methods for integer arithmetic — mirrors .NET Integers.cs.
pub struct Integers;

impl Integers {
    pub fn sq(value: i32) -> i32 {
        value * value
    }

    /// Ceiling division: rounds up to the nearest multiple of divisor.
    pub fn round_up_div(dividend: i32, divisor: i32) -> i32 {
        (dividend + divisor - 1) / divisor
    }

    /// Population count: number of set bits in uint.
    /// https://stackoverflow.com/questions/10439242/count-leading-zeroes-in-an-int32
    pub fn population_count(x: u32) -> u32 {
        x.count_ones()
    }

    /// Number of leading zeros in a 32-bit unsigned int.
    pub fn leading_zeros(x: u32) -> u32 {
        x.leading_zeros()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sq() {
        assert_eq!(Integers::sq(0), 0);
        assert_eq!(Integers::sq(5), 25);
        assert_eq!(Integers::sq(-3), 9);
        assert_eq!(Integers::sq(100), 10_000);
    }

    #[test]
    fn test_round_up_div() {
        assert_eq!(Integers::round_up_div(10, 3), 4); // ceil(10/3) = 4
        assert_eq!(Integers::round_up_div(9, 3), 3); // exact
        assert_eq!(Integers::round_up_div(1, 10), 1);
        assert_eq!(Integers::round_up_div(10, 10), 1);
        assert_eq!(Integers::round_up_div(11, 10), 2);
    }

    #[test]
    fn test_population_count() {
        assert_eq!(Integers::population_count(0), 0);
        assert_eq!(Integers::population_count(1), 1);
        assert_eq!(Integers::population_count(0xFFFF_FFFF), 32);
        assert_eq!(Integers::population_count(0x00FF_00FF), 16);
        // Hamming weight of binary 1101_1010_1111_0011 = 10
        assert_eq!(Integers::population_count(0xD0F3), 9);
    }

    #[test]
    fn test_leading_zeros() {
        assert_eq!(Integers::leading_zeros(0), 32);
        assert_eq!(Integers::leading_zeros(1), 31);
        assert_eq!(Integers::leading_zeros(0x8000_0000), 0);
        assert_eq!(Integers::leading_zeros(0xFFFF_0000), 0);
    }
}
