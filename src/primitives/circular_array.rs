/// CircularArray<T>: array with wrap-around cursor — mirrors .NET CircularArray.cs.
pub struct CircularArray<T> {
    pub array: Vec<T>,
    pub head: usize,
    pub len: usize,
}

impl<T: Clone + Default> CircularArray<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            array: vec![T::default(); capacity],
            head: 0,
            len: 0,
        }
    }

    pub fn size(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len == self.array.len()
    }

    fn validate_item_index(&self, _index: usize) {}

    fn validate_cursor_index(&self, _index: usize) {}

    fn location(&self, index: usize) -> usize {
        let loc = self.head + index;
        if loc < self.array.len() {
            loc
        } else {
            loc - self.array.len()
        }
    }

    fn enlarge(&mut self) {
        let mut enlarged = vec![T::default(); 2 * self.array.len()];
        for i in 0..self.len {
            let loc = self.location(i);
            enlarged[i] = std::mem::replace(&mut self.array[loc], T::default());
        }
        self.array = enlarged;
        self.head = 0;
    }

    fn move_items(&mut self, from: usize, to: usize, length: usize) {
        for i in 0..length {
            let from_loc = self.location(from + i);
            let to_loc = self.location(to + i);
            self.array[to_loc] = std::mem::replace(&mut self.array[from_loc], T::default());
        }
    }

    pub fn insert(&mut self, index: usize, amount: usize) {
        self.validate_cursor_index(index);
        assert!(amount > 0, "amount must be > 0");
        while self.len + amount > self.array.len() {
            self.enlarge();
        }
        if 2 * index >= self.len {
            self.len += amount;
            self.move_items(index, index + amount, self.len - index - amount);
        } else {
            if self.head >= amount {
                self.head -= amount;
            } else {
                self.head = self.array.len().saturating_sub(amount);
            }
            self.len += amount;
            self.move_items(amount, 0, index);
        }
        for i in 0..amount {
            let loc = self.location(index + i);
            self.array[loc] = T::default();
        }
    }

    pub fn remove(&mut self, index: usize, amount: usize) {
        self.validate_cursor_index(index);
        assert!(amount > 0, "amount must be > 0");
        self.validate_cursor_index(index + amount);
        if 2 * index >= self.len - amount {
            self.move_items(index + amount, index, self.len - amount - index);
            self.len -= amount;
        } else {
            self.move_items(0, amount, index);
            self.head = (self.head + amount) % self.array.len();
            self.len -= amount;
        }
    }

    pub fn get(&self, index: usize) -> &T {
        self.validate_item_index(index);
        &self.array[self.location(index)]
    }

    pub fn set(&mut self, index: usize, value: T) {
        self.validate_item_index(index);
        let loc = self.location(index);
        self.array[loc] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let arr = CircularArray::<i32>::new(10);
        assert_eq!(arr.size(), 0);
        assert!(arr.is_empty());
        assert!(!arr.is_full());
    }

    #[test]
    fn test_set_get() {
        let mut arr = CircularArray::<i32>::new(10);
        arr.set(0, 5);
        arr.set(1, 10);
        arr.len = 2;
        assert_eq!(*arr.get(0), 5);
        assert_eq!(*arr.get(1), 10);
    }

    #[test]
    fn test_insert() {
        let mut arr = CircularArray::<i32>::new(4);
        arr.set(0, 1);
        arr.len = 1;
        arr.insert(0, 2);
        assert_eq!(arr.size(), 3);
    }

    #[test]
    fn test_remove() {
        let mut arr = CircularArray::<i32>::new(4);
        arr.set(0, 1);
        arr.len = 1;
        arr.remove(0, 1);
        assert_eq!(arr.size(), 0);
        assert!(arr.is_empty());
    }
}
