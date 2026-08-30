//! Unit tests for PriorityQueue primitive type.

#[cfg(test)]
mod tests {
    use cearaafis::primitives::PriorityQueue;

    #[test]
    fn test_priority_queue_new() {
        let pq: PriorityQueue<i32> = PriorityQueue::new();
        assert_eq!(pq.len(), 0);
        assert!(pq.is_empty());
    }

    #[test]
    fn test_priority_queue_add() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        pq.push(5);
        pq.push(10);
        pq.push(1);
        assert_eq!(pq.len(), 3);
    }

    #[test]
    fn test_priority_queue_peek() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        pq.push(5);
        pq.push(10);
        pq.push(1);
        // Min-heap: smallest element is at top (mirrors .NET)
        assert_eq!(*pq.peek(), 1);
    }

    #[test]
    fn test_priority_queue_pop() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        pq.push(1);
        pq.push(5);
        pq.push(10);
        let val = pq.pop();
        assert_eq!(val, 1); // min-heap: smallest first
        assert_eq!(pq.len(), 2);
    }

    #[test]
    fn test_priority_queue_order() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        for i in 0..10 {
            pq.push(i);
        }
        // Min-heap: pop in ascending order
        for i in 0..10 {
            let v = pq.pop();
            assert_eq!(v, i);
        }
    }

    #[test]
    fn test_priority_queue_clear() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        pq.push(1);
        pq.push(2);
        pq.clear();
        assert_eq!(pq.len(), 0);
    }

    #[test]
    fn test_priority_queue_empty_pop() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        pq.push(1);
        let v = pq.pop();
        assert_eq!(v, 1);
        assert!(pq.is_empty());
    }

    #[test]
    fn test_priority_queue_empty_peek() {
        let mut pq: PriorityQueue<i32> = PriorityQueue::new();
        pq.push(1);
        let ref_v = pq.peek();
        assert_eq!(*ref_v, 1);
    }
}
