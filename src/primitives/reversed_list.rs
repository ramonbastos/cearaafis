/// ReversedList<T>: a reversed view over an inner IList<T> — mirrors .NET ReversedList.cs.
/// get(n) maps to inner[Count - n - 1]; add prepends to inner.
use crate::primitives::circular_list::CircularList;

pub struct ReversedList<T: Clone + PartialEq + Default> {
    inner: CircularList<T>,
}

impl<T: Clone + PartialEq + Default> ReversedList<T> {
    pub fn new() -> Self {
        Self {
            inner: CircularList::new(),
        }
    }

    /// Number of elements in this reversed view.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get at reversed index: get(n) = inner[Count - n - 1].
    pub fn get(&self, index: usize) -> &T {
        let n = self.inner.len();
        assert!(index < n, "index out of bounds");
        self.inner.get(n - 1 - index)
    }

    /// Set at reversed index.
    pub fn set(&mut self, index: usize, value: T) {
        let n = self.inner.len();
        assert!(index < n, "index out of bounds");
        self.inner.set(n - 1 - index, value);
    }

    /// Search for item in reversed order.
    pub fn index_of(&self, item: &T) -> Option<usize> {
        let n = self.inner.len();
        (0..n).find(|&i| *self.inner.get(n - 1 - i) == *item)
    }

    /// Insert at reversed position: insert(index, item) = inner.Insert(Count - index, item).
    pub fn insert(&mut self, index: usize, item: T) {
        let n = self.inner.len();
        assert!(index <= n, "index out of bounds");
        self.inner.insert(n - index, item);
    }

    /// Remove at reversed index: remove_at(index) = inner.RemoveAt(Count - index - 1).
    pub fn remove_at(&mut self, index: usize) {
        let n = self.inner.len();
        assert!(index < n, "index out of bounds");
        self.inner.remove_at(n - 1 - index);
    }

    /// Add prepends to the inner list (like C# inner.Insert(0, item)).
    /// After add(1), add(2), add(3): inner = [3,2,1], get(0)=inner[2]=1, get(2)=inner[0]=3.
    pub fn add(&mut self, item: T) {
        self.inner.insert(0, item);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn contains(&self, item: &T) -> bool {
        self.inner.contains(item)
    }

    pub fn remove(&mut self, item: &T) -> bool {
        if let Some(index) = self.index_of(item) {
            self.remove_at(index);
            true
        } else {
            false
        }
    }
}

impl<T: Clone + PartialEq + Default> Default for ReversedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        let list = ReversedList::<i32>::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_add() {
        // C#: Add(item) => inner.Insert(0, item)
        // add(1)  → inner = [1]
        // add(2)  → inner = [2, 1]
        // add(3)  → inner = [3, 2, 1]
        // get(n) = inner[Count - n - 1]: get(0)=inner[2]=1, get(1)=inner[1]=2, get(2)=inner[0]=3
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1); // most recently added
        assert_eq!(*list.get(1), 2);
        assert_eq!(*list.get(2), 3); // oldest
    }

    #[test]
    fn test_get() {
        let mut list = ReversedList::<i32>::new();
        list.add(42);
        assert_eq!(*list.get(0), 42);
    }

    #[test]
    fn test_set() {
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        // inner = [2, 1], set(0) = inner[1] = 42 → inner = [2, 42]
        list.set(0, 42);
        assert_eq!(*list.get(0), 42);
    }

    #[test]
    fn test_index_of() {
        // inner = [3, 2, 1], get = [1, 2, 3]
        // index_of searches get(0), get(1), get(2)...
        // index_of(&3): i=0: get(0)=1≠3; i=1: get(1)=2≠3; i=2: get(2)=3=3 → Some(2)
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.index_of(&3), Some(2)); // oldest
        assert_eq!(list.index_of(&2), Some(1));
        assert_eq!(list.index_of(&1), Some(0)); // most recent
        assert_eq!(list.index_of(&99), None);
    }

    #[test]
    fn test_is_empty() {
        let mut list = ReversedList::<i32>::new();
        assert!(list.is_empty());
        list.add(1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_insert() {
        // add(1) → inner = [1]
        // add(3) → inner = [3, 1]
        // insert(1, 2) → inner.insert(2-1, 2) = inner.insert(1, 2) → [3, 2, 1]
        // get(0)=inner[2]=1, get(1)=inner[1]=2, get(2)=inner[0]=3
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(3);
        list.insert(1, 2);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1); // most recent
        assert_eq!(*list.get(1), 2); // inserted
        assert_eq!(*list.get(2), 3); // oldest
    }

    #[test]
    fn test_remove_at() {
        // add(1),add(2),add(3) → inner=[3,2,1]
        // remove_at(0) → inner.RemoveAt(3-0-1) = inner.RemoveAt(2) removes 1
        // inner = [3,2], get(0)=inner[1]=2, get(1)=inner[0]=3
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove_at(0);
        assert_eq!(list.len(), 2);
        assert_eq!(*list.get(0), 2);
        assert_eq!(*list.get(1), 3);
    }

    #[test]
    fn test_clear() {
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_contains() {
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        assert!(list.contains(&1));
        assert!(!list.contains(&2));
    }

    #[test]
    fn test_remove() {
        // index_of(&1) = 0, remove_at(0) → inner.RemoveAt(2) removes 1
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert!(list.remove(&1));
        assert_eq!(list.len(), 2);
        assert!(!list.remove(&99));
    }
}
