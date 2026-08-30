//! Unit tests for CircularList primitive type.
//! Mirrors SourceAFIS.Tests/Engine/Primitives/CircularListTest.cs

#[cfg(test)]
mod tests {
    use cearaafis::primitives::CircularList;

    #[test]
    fn test_circular_list_new() {
        let list: CircularList<i32> = CircularList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_circular_list_add() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 2);
        assert_eq!(*list.get(2), 3);
    }

    #[test]
    fn test_circular_list_get() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(42);
        assert_eq!(*list.get(0), 42);
    }

    #[test]
    fn test_circular_list_set() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.set(0, 2);
        assert_eq!(*list.get(0), 2);
    }

    #[test]
    fn test_circular_list_insert() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.add(3);
        list.insert(1, 2);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 2);
        assert_eq!(*list.get(2), 3);
    }

    #[test]
    fn test_circular_list_remove_at() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove_at(1); // remove 2
        assert_eq!(list.len(), 2);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 3);
    }

    #[test]
    fn test_circular_list_remove() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert!(list.remove(&2));
        assert_eq!(list.len(), 2);
        assert!(!list.remove(&2));
    }

    #[test]
    fn test_circular_list_clear() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.add(2);
        list.clear();
        assert!(list.is_empty());
    }

    #[test]
    fn test_circular_list_contains() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        assert!(list.contains(&1));
        assert!(!list.contains(&2));
    }

    #[test]
    fn test_circular_list_index_of() {
        let mut list: CircularList<i32> = CircularList::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.index_of(&2), Some(1));
        assert_eq!(list.index_of(&5), None);
    }
}
