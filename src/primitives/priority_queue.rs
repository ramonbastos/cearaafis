/// PriorityQueue<T>: heap-based priority queue — mirrors .NET PriorityQueue.cs.

pub struct PriorityQueue<T>
where
    T: Default + Ord,
{
    heap: Vec<T>,
    size: usize,
}

impl<T: Default + Ord> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default + Ord> PriorityQueue<T> {
    pub fn new() -> Self {
        Self {
            heap: Vec::new(),
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn clear(&mut self) {
        self.heap.truncate(0);
        self.size = 0;
    }

    pub fn add(&mut self, item: T) {
        self.heap.push(item);
        self.size += 1;
        self.bubble_up(self.size - 1);
    }

    /// Alias for add — matches C# Push
    pub fn push(&mut self, item: T) {
        self.add(item);
    }

    pub fn peek(&self) -> &T {
        assert!(self.size > 0, "PriorityQueue is empty");
        &self.heap[0]
    }

    pub fn pop(&mut self) -> T {
        assert!(self.size > 0, "PriorityQueue is empty");
        let result = std::mem::replace(&mut self.heap[0], Default::default());
        self.size -= 1;
        if self.size > 0 {
            self.heap[0] = std::mem::replace(&mut self.heap[self.size], Default::default());
            self.heap.truncate(self.size);
            self.bubble_down();
        }
        result
    }

    pub fn remove(&mut self) -> T {
        self.pop()
    }

    fn bubble_up(&mut self, mut bottom: usize) {
        while bottom > 0 {
            let parent = (bottom - 1) / 2;
            if self.heap[parent] < self.heap[bottom] {
                break;
            }
            self.heap.swap(parent, bottom);
            bottom = parent;
        }
    }

    fn bubble_down(&mut self) {
        let mut parent = 0;
        loop {
            let left = 2 * parent + 1;
            let right = 2 * parent + 2;
            if left >= self.size {
                break;
            }
            let mut child = left;
            if right < self.size && self.heap[right] < self.heap[left] {
                child = right;
            }
            if self.heap[parent] < self.heap[child] {
                break;
            }
            self.heap.swap(parent, child);
            parent = child;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let pq = PriorityQueue::<i32>::new();
        assert!(pq.is_empty());
        assert_eq!(pq.len(), 0);
    }

    #[test]
    fn test_add_peek() {
        let mut pq = PriorityQueue::new();
        pq.add(5);
        pq.add(3);
        pq.add(7);
        assert_eq!(*pq.peek(), 3);
    }

    #[test]
    fn test_remove_order() {
        let mut pq = PriorityQueue::new();
        pq.add(7);
        pq.add(3);
        pq.add(5);
        assert_eq!(pq.remove(), 3);
        assert_eq!(pq.remove(), 5);
        assert_eq!(pq.remove(), 7);
    }

    #[test]
    fn test_clear() {
        let mut pq = PriorityQueue::new();
        pq.add(1);
        pq.add(2);
        pq.clear();
        assert!(pq.is_empty());
    }

    #[test]
    fn test_single_element() {
        let mut pq = PriorityQueue::new();
        pq.add(42);
        assert_eq!(*pq.peek(), 42);
        assert_eq!(pq.remove(), 42);
        assert!(pq.is_empty());
    }

    #[test]
    #[should_panic(expected = "PriorityQueue is empty")]
    fn test_peek_empty() {
        let pq = PriorityQueue::<i32>::new();
        let _ = pq.peek();
    }

    #[test]
    #[should_panic(expected = "PriorityQueue is empty")]
    fn test_remove_empty() {
        let mut pq = PriorityQueue::<i32>::new();
        pq.remove();
    }
}
