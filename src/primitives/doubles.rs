/// Static helper methods for double arithmetic — mirrors .NET Doubles.cs.
pub struct Doubles;

impl Doubles {
    pub fn round_to_int(value: f64) -> i64 {
        (value + 0.5) as i64
    }

    pub fn sq(value: f64) -> f64 {
        value * value
    }

    pub fn interpolate(start: f64, end: f64, position: f64) -> f64 {
        start + position * (end - start)
    }

    pub fn interpolate_2d(
        topleft: f64,
        bottomleft: f64,
        topright: f64,
        bottomright: f64,
        x: f64,
        y: f64,
    ) -> f64 {
        let left = Self::interpolate(topleft, bottomleft, y);
        let right = Self::interpolate(topright, bottomright, y);
        Self::interpolate(left, right, x)
    }

    pub fn interpolate_exponential(start: f64, end: f64, position: f64) -> f64 {
        if start == 0.0 {
            return 0.0;
        }
        (end / start).powf(position) * start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_to_int() {
        assert_eq!(Doubles::round_to_int(3.2), 3);
        assert_eq!(Doubles::round_to_int(3.6), 4);
        assert_eq!(Doubles::round_to_int(0.5), 1);
        assert_eq!(Doubles::round_to_int(-0.5), 0);
    }

    #[test]
    fn test_sq() {
        assert_eq!(Doubles::sq(5.0), 25.0);
        assert_eq!(Doubles::sq(-3.0), 9.0);
        assert_eq!(Doubles::sq(0.0), 0.0);
    }

    #[test]
    fn test_interpolate() {
        assert_eq!(Doubles::interpolate(0.0, 10.0, 0.0), 0.0);
        assert_eq!(Doubles::interpolate(0.0, 10.0, 1.0), 10.0);
        assert_eq!(Doubles::interpolate(0.0, 10.0, 0.5), 5.0);
    }

    #[test]
    fn test_interpolate_2d() {
        // 0 at topleft, 10 at bottomleft, 1 at topright, 11 at bottomright
        // at x=0.5, y=0.5 should be 5.5
        let result = Doubles::interpolate_2d(0.0, 10.0, 1.0, 11.0, 0.5, 0.5);
        assert!((result - 5.5).abs() < 0.01);
    }

    #[test]
    fn test_interpolate_exponential() {
        assert_eq!(Doubles::interpolate_exponential(1.0, 10.0, 0.0), 1.0);
        assert_eq!(Doubles::interpolate_exponential(1.0, 10.0, 1.0), 10.0);
        assert_eq!(Doubles::interpolate_exponential(0.0, 10.0, 0.5), 0.0);
    }
}
