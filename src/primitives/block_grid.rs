/// BlockGrid: grid of blocks with corner tracking — mirrors .NET BlockGrid.cs.
use crate::primitives::{int_point::IntPoint, int_rect::IntRect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockGrid {
    pub blocks: IntPoint,
    pub corners: IntPoint,
    pub x: Vec<i32>,
    pub y: Vec<i32>,
}

impl BlockGrid {
    pub fn new(size: &IntPoint) -> Self {
        let blocks = size;
        let corners = IntPoint::new(blocks.x() + 1, blocks.y() + 1);
        let x = vec![0; (blocks.x() + 1) as usize];
        let y = vec![0; (blocks.y() + 1) as usize];
        Self {
            blocks: *blocks,
            corners,
            x,
            y,
        }
    }

    pub fn from_width_height(width: i32, height: i32) -> Self {
        Self::new(&IntPoint::new(width, height))
    }

    pub fn corner(&self, at_x: i32, at_y: i32) -> IntPoint {
        IntPoint::new(self.x[at_x as usize], self.y[at_y as usize])
    }

    pub fn corner_at(&self, at: &IntPoint) -> IntPoint {
        self.corner(at.x(), at.y())
    }

    pub fn block(&self, at_x: i32, at_y: i32) -> IntRect {
        IntRect::between(
            &self.corner(at_x, at_y),
            &self.corner(at_x + 1, at_y + 1),
        )
    }

    pub fn block_at(&self, at: &IntPoint) -> IntRect {
        self.block(at.x(), at.y())
    }
}

impl Default for BlockGrid {
    fn default() -> Self {
        Self::new(&IntPoint::new(0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let grid = BlockGrid::new(&IntPoint::new(5, 3));
        assert_eq!(grid.blocks.x(), 5);
        assert_eq!(grid.blocks.y(), 3);
        assert_eq!(grid.corners.x(), 6);
        assert_eq!(grid.corners.y(), 4);
    }

    #[test]
    fn test_from_width_height() {
        let grid = BlockGrid::from_width_height(10, 5);
        assert_eq!(grid.blocks.x(), 10);
        assert_eq!(grid.blocks.y(), 5);
    }

    #[test]
    fn test_corner() {
        let mut grid = BlockGrid::new(&IntPoint::new(1, 1));
        grid.x[0] = 0;
        grid.x[1] = 100;
        grid.y[0] = 0;
        grid.y[1] = 200;
        assert_eq!(grid.corner(0, 0), IntPoint::new(0, 0));
        assert_eq!(grid.corner(1, 1), IntPoint::new(100, 200));
    }

    #[test]
    fn test_block() {
        let mut grid = BlockGrid::new(&IntPoint::new(1, 1));
        grid.x[0] = 0;
        grid.x[1] = 100;
        grid.y[0] = 0;
        grid.y[1] = 100;
        let block = grid.block(0, 0);
        assert_eq!(block.left(), 0);
        assert_eq!(block.top(), 0);
        assert_eq!(block.width(), 100);
        assert_eq!(block.height(), 100);
    }
}
