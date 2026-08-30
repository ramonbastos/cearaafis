//! Unit tests for BlockGrid primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/BlockGridTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::{BlockGrid, IntPoint};

    #[test]
    fn test_block_grid_new() {
        let size = IntPoint::new(10, 10);
        let grid = BlockGrid::new(&size);
        assert_eq!(grid.blocks.x, 10);
        assert_eq!(grid.blocks.y, 10);
        assert_eq!(grid.corners.x, 11);
        assert_eq!(grid.corners.y, 11);
    }

    #[test]
    fn test_block_grid_corner() {
        let mut grid = BlockGrid::new(&IntPoint::new(2, 2));
        grid.x[0] = 0;
        grid.x[1] = 5;
        grid.x[2] = 10;
        grid.y[0] = 0;
        grid.y[1] = 5;
        grid.y[2] = 10;

        let corner = grid.corner(1, 1);
        assert_eq!(corner.x, 5);
        assert_eq!(corner.y, 5);
    }

    #[test]
    fn test_block_grid_block() {
        let mut grid = BlockGrid::new(&IntPoint::new(2, 2));
        grid.x[0] = 0;
        grid.x[1] = 5;
        grid.x[2] = 10;
        grid.y[0] = 0;
        grid.y[1] = 5;
        grid.y[2] = 10;

        let block = grid.block(0, 0);
        assert_eq!(block.left(), 0);
        assert_eq!(block.top(), 0);
        assert_eq!(block.right(), 5);
        assert_eq!(block.bottom(), 5);
    }

    #[test]
    fn test_block_grid_corner_at() {
        let mut grid = BlockGrid::new(&IntPoint::new(2, 2));
        grid.x[0] = 0;
        grid.x[1] = 5;
        grid.y[0] = 0;
        grid.y[1] = 5;

        let corner = grid.corner_at(&IntPoint::new(1, 1));
        assert_eq!(corner.x, 5);
        assert_eq!(corner.y, 5);
    }

    #[test]
    fn test_block_grid_block_at() {
        let mut grid = BlockGrid::new(&IntPoint::new(2, 2));
        grid.x[0] = 0;
        grid.x[1] = 5;
        grid.y[0] = 0;
        grid.y[1] = 5;

        grid.x[2] = 10;
        grid.y[2] = 10;

        let block = grid.block_at(&IntPoint::new(1, 1));
        assert_eq!(block.left(), 5);
        assert_eq!(block.top(), 5);
    }
}
