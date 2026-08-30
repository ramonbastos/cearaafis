/// CircularList<T>: circular buffer wrapped as an IList<T> — mirrors .NET CircularList.cs.
use super::circular_array::CircularArray;

pub struct CircularList<T: Clone + PartialEq + Default> {
    inner: CircularArray<T>,
}

impl<T: Clone + PartialEq + Default> CircularList<T> {
    pub fn new() -> Self {
        Self {
            inner: CircularArray::new(16),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len
    }

    pub fn is_empty(&self) -> bool {
        self.inner.len == 0
    }

    pub fn get(&self, index: usize) -> &T {
        &self.inner.get(index)
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.inner.set(index, value);
    }

    pub fn index_of(&self, item: &T) -> Option<usize> {
        for i in 0..self.inner.len {
            if *self.inner.get(i) == *item {
                return Some(i);
            }
        }
        None
    }

    pub fn insert(&mut self, index: usize, item: T) {
        self.inner.insert(index, 1);
        self.inner.set(index, item);
    }

    pub fn remove_at(&mut self, index: usize) {
        self.inner.remove(index, 1);
    }

    pub fn add(&mut self, item: T) {
        self.inner.insert(self.inner.len, 1);
        self.inner.set(self.inner.len - 1, item);
    }

    pub fn clear(&mut self) {
        self.inner.remove(0, self.inner.len);
    }

    pub fn contains(&self, item: &T) -> bool {
        self.index_of(item).is_some()
    }

    pub fn remove(&mut self, item: &T) -> bool {
        if let Some(index) = self.index_of(item) {
            self.remove_at(index);
            true
        } else {
            false
        }
    }

    /// Alias for add — matches C# PushBack
    pub fn push_back(&mut self, item: T) {
        self.add(item);
    }

    /// Insert at beginning — matches C# PushFront
    pub fn push_front(&mut self, item: T) {
        self.insert(0, item);
    }

    /// Remove and return first element — matches C# PopFront
    pub fn pop_front(&mut self) -> Option<T> {
        if self.inner.len == 0 {
            return None;
        }
        let index = 0;
        let val = std::mem::replace(&mut self.inner.array[index], T::default());
        self.inner.remove(index, 1);
        Some(val)
    }

    /// Remove and return last element — matches C# PopBack
    pub fn pop_back(&mut self) -> Option<T> {
        if self.inner.len == 0 {
            return None;
        }
        let index = self.inner.len - 1;
        let val = std::mem::replace(&mut self.inner.array[index], T::default());
        self.inner.remove(index, 1);
        Some(val)
    }
}

impl<T: Clone + PartialEq + Default> Default for CircularList<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let list = CircularList::<i32>::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_add() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 2);
        assert_eq!(*list.get(2), 3);
    }

    #[test]
    fn test_get() {
        let mut list = CircularList::<i32>::new();
        list.add(42);
        assert_eq!(*list.get(0), 42);
    }

    #[test]
    fn test_set() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.set(0, 2);
        assert_eq!(*list.get(0), 2);
    }

    #[test]
    fn test_insert() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.add(3);
        list.insert(1, 2);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 2);
        assert_eq!(*list.get(2), 3);
    }

    #[test]
    fn test_remove_at() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove_at(1);
        assert_eq!(list.len(), 2);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 3);
    }

    #[test]
    fn test_remove() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert!(list.remove(&2));
        assert_eq!(list.len(), 2);
        assert!(!list.remove(&2));
    }

    #[test]
    fn test_clear() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.add(2);
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        assert!(list.contains(&1));
        assert!(!list.contains(&2));
    }

    #[test]
    fn test_index_of() {
        let mut list = CircularList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.index_of(&2), Some(1));
        assert_eq!(list.index_of(&5), None);
    }
}
