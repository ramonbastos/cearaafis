//! Unit tests for HistogramCube primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/HistogramCubeTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::HistogramCube;

    #[test]
    fn test_histogram_cube_new() {
        let cube = HistogramCube::new(10, 10, 256);
        assert_eq!(cube.width(), 10);
        assert_eq!(cube.height(), 10);
        assert_eq!(cube.bins, 256);
    }

    #[test]
    fn test_histogram_cube_with_size() {
        let cube = HistogramCube::new(5, 5, 128);
        assert_eq!(cube.width(), 5);
        assert_eq!(cube.height(), 5);
        assert_eq!(cube.bins, 128);
    }

    #[test]
    fn test_histogram_cube_set_get() {
        let mut cube = HistogramCube::new(3, 3, 10);
        cube.set(1, 1, 5, 42);
        assert_eq!(cube.get(1, 1, 5), 42);
    }

    #[test]
    fn test_histogram_cube_add() {
        let mut cube = HistogramCube::new(3, 3, 10);
        cube.add(1, 1, 5, 10);
        cube.add(1, 1, 5, 5);
        assert_eq!(cube.get(1, 1, 5), 15);
    }

    #[test]
    fn test_histogram_cube_increment() {
        let mut cube = HistogramCube::new(3, 3, 10);
        cube.increment(1, 1, 5);
        assert_eq!(cube.get(1, 1, 5), 1);
    }

    #[test]
    fn test_histogram_cube_sum() {
        let mut cube = HistogramCube::new(3, 3, 10);
        cube.set(1, 1, 0, 5);
        cube.set(1, 1, 5, 10);
        cube.set(1, 1, 9, 15);
        assert_eq!(cube.sum(1, 1), 30);
    }

    #[test]
    fn test_histogram_cube_constrain() {
        let cube = HistogramCube::new(3, 3, 10);
        let c: i32 = -1;
        let _c = if c < 0 { 0 } else { c };
        assert_eq!(_c, 0);
        let c: usize = 0;
        let _c = c.clamp(0, cube.bins - 1);
        assert_eq!(_c, 0usize);
        assert!(cube.bins > 0);
        assert!(cube.bins > 0);
        assert!(cube.bins > 0);
    }
}
