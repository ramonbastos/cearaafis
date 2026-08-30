//! Unit tests for BlockMap primitive type.
//! Replaces the duplicate file

#[cfg(test)]
mod tests {
    use cearaafis::primitives::BlockMap;

    #[test]
    fn test_block_map_new() {
        let blocks = BlockMap::new(100, 100, 15);
        assert_eq!(blocks.pixels().x, 100);
        assert_eq!(blocks.pixels().y, 100);
    }

    #[test]
    fn test_block_map_primary_blocks() {
        let blocks = BlockMap::new(100, 100, 15);
        let primary = blocks.primary_blocks();
        assert!(primary.x >= 1);
        assert!(primary.y >= 1);
    }

    #[test]
    fn test_block_map_primary_block() {
        let blocks = BlockMap::new(100, 100, 15);
        let block = blocks.primary_block(0, 0);
        assert!(block.left() >= 0);
        assert!(block.top() >= 0);
    }

    #[test]
    fn test_block_map_secondary_corners() {
        let blocks = BlockMap::new(100, 100, 15);
        let corners = blocks.secondary_corners();
        assert!(corners.x > 0);
        assert!(corners.y > 0);
    }

    #[test]
    fn test_block_map_secondary_block() {
        let blocks = BlockMap::new(100, 100, 15);
        let block = blocks.secondary_block(0, 0);
        assert!(block.left() >= 0);
        assert!(block.top() >= 0);
    }
}
