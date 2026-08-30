/// BlockMap: two-layer block grid system — mirrors .NET BlockMap.cs.
use crate::primitives::{
    block_grid::BlockGrid, int_point::IntPoint, int_rect::IntRect, integers::Integers,
};

pub struct BlockMap {
    pub pixels: IntPoint,
    pub primary: BlockGrid,
    pub secondary: BlockGrid,
}

impl BlockMap {
    pub fn new(width: i32, height: i32, max_block_size: i32) -> Self {
        let pixels = IntPoint::new(width, height);
        let primary_blocks_x = Integers::round_up_div(width as i32, max_block_size as i32);
        let primary_blocks_y = Integers::round_up_div(height as i32, max_block_size as i32);
        let primary_size = IntPoint::new(primary_blocks_x, primary_blocks_y);
        let mut primary = BlockGrid::new(&primary_size);

        // Fill primary Y coordinates
        for y in 0..=primary_blocks_y {
            primary.y[y as usize] = y as i32 * height / primary_blocks_y;
        }

        // Fill primary X coordinates
        for x in 0..=primary_blocks_x {
            primary.x[x as usize] = x as i32 * width / primary_blocks_x;
        }

        // Create secondary grid from primary corners
        let mut secondary = BlockGrid::from_width_height(primary.corners.x(), primary.corners.y());

        // Initialize secondary Y
        secondary.y[0] = 0;
        for y in 0..primary_blocks_y {
            secondary.y[(y + 1) as usize] = primary.block(0, y).center().y();
        }
        secondary.y[primary_blocks_y as usize] = height;

        // Initialize secondary X
        secondary.x[0] = 0;
        for x in 0..primary_blocks_x {
            secondary.x[(x + 1) as usize] = primary.block(x, 0).center().x();
        }
        secondary.x[primary_blocks_x as usize] = width;

        Self {
            pixels,
            primary,
            secondary,
        }
    }

    /// Alias for primary.blocks
    pub fn primary_blocks(&self) -> IntPoint {
        self.primary.blocks
    }

    /// Alias for secondary.corners
    pub fn secondary_corners(&self) -> IntPoint {
        self.secondary.corners
    }

    pub fn pixels(&self) -> IntPoint {
        self.pixels
    }

    pub fn width(&self) -> i32 {
        self.pixels.x()
    }

    pub fn height(&self) -> i32 {
        self.pixels.y()
    }

    pub fn primary_block(&self, at_x: i32, at_y: i32) -> IntRect {
        self.primary.block(at_x, at_y)
    }

    pub fn secondary_block(&self, at_x: i32, at_y: i32) -> IntRect {
        self.secondary.block(at_x, at_y)
    }
}

impl Clone for BlockMap {
    fn clone(&self) -> Self {
        Self {
            pixels: self.pixels,
            primary: self.primary.clone(),
            secondary: self.secondary.clone(),
        }
    }
}

impl PartialEq for BlockMap {
    fn eq(&self, other: &Self) -> bool {
        self.pixels == other.pixels
            && self.primary == other.primary
            && self.secondary == other.secondary
    }
}

impl Eq for BlockMap {}

impl std::fmt::Debug for BlockMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockMap({}x{})", self.pixels.x(), self.pixels.y())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let bm = BlockMap::new(100, 100, 15);
        assert_eq!(bm.pixels.x(), 100);
        assert_eq!(bm.pixels.y(), 100);
        assert_eq!(bm.primary_blocks().x(), 7);
        assert_eq!(bm.primary_blocks().y(), 7);
    }

    #[test]
    fn test_pixels() {
        let bm = BlockMap::new(50, 30, 10);
        assert_eq!(bm.pixels.x(), 50);
        assert_eq!(bm.pixels.y(), 30);
    }
}
