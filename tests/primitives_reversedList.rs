//! Unit tests for ReversedList — mirrors .NET ReversedList.cs semantics.
//! C#: get(index) = inner[Count - index - 1], add = inner.Insert(0, item).

#[cfg(test)]
mod tests {
    use cearaafis::primitives::ReversedList;

    #[test]
    fn test_len_is_empty() {
        let list: ReversedList<i32> = ReversedList::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_add_order() {
        // C#: Add(item) => inner.Insert(0, item)
        // After add(1), add(2), add(3): inner=[3,2,1], get(0)=inner[2]=1 (most recent)
        let mut list = ReversedList::<i32>::new();
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
        let mut list = ReversedList::<i32>::new();
        list.add(42);
        assert_eq!(*list.get(0), 42);
    }

    #[test]
    fn test_set() {
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.set(0, 42);
        assert_eq!(*list.get(0), 42);
    }

    #[test]
    fn test_index_of() {
        // C#: IndexOf searches reversed order: get(0), get(1), get(2)...
        // add(1) → inner=[1], add(2) → inner=[2,1], add(3) → inner=[3,2,1]
        // get(0)=inner[2]=1, get(1)=inner[1]=2, get(2)=inner[0]=3
        // index_of(1)=0, index_of(2)=1, index_of(3)=2
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        assert_eq!(list.index_of(&1), Some(0));
        assert_eq!(list.index_of(&2), Some(1));
        assert_eq!(list.index_of(&3), Some(2));
        assert_eq!(list.index_of(&99), None);
    }

    #[test]
    fn test_is_empty_toggle() {
        let mut list = ReversedList::<i32>::new();
        assert!(list.is_empty());
        list.add(1);
        assert!(!list.is_empty());
    }

    #[test]
    fn test_insert() {
        // C#: Insert(1, 2) => inner.Insert(Count - 1, 2)
        // add(1) → inner=[1], add(3) → inner=[3,1]
        // Insert(1, 2) → inner.Insert(2, 2) → [3,1,2]
        // get(0)=inner[2]=2, get(1)=inner[1]=1, get(2)=inner[0]=3
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(3);
        // C#: Insert(1, 2) => inner.Insert(2-1, 2) = inner.Insert(1, 2)
        list.insert(1, 2);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.get(0), 1); // most recent added
        assert_eq!(*list.get(1), 2); // inserted
        assert_eq!(*list.get(2), 3); // oldest
    }

    #[test]
    fn test_remove_at() {
        // C#: RemoveAt(1) with inner=[3,2,1], get=[1,2,3]
        // inner.RemoveAt(3-1-1) = inner.RemoveAt(1) → removes 2 → inner=[3,1]
        // get(0)=inner[1]=1, get(1)=inner[0]=3
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        list.remove_at(1);
        assert_eq!(list.len(), 2);
        assert_eq!(*list.get(0), 1);
        assert_eq!(*list.get(1), 3);
    }

    #[test]
    fn test_clear() {
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.clear();
        assert_eq!(list.len(), 0);
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
        let mut list = ReversedList::<i32>::new();
        list.add(1);
        list.add(2);
        list.add(3);
        // C#: Remove finds index, then inner.RemoveAt(Count - index - 1)
        assert!(list.remove(&1));
        assert_eq!(list.len(), 2);
        assert!(!list.remove(&99));
    }
}
