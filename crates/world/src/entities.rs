/// Rotating free-slot allocation matching Kotlin EntityList. Player capacity is
/// 2047 entries with slot 0 reserved: usable wire slots are 1 through 2046.
pub struct EntityList<T> {
    entries: Vec<Option<T>>,
    last: usize,
    padding: usize,
}
impl<T> EntityList<T> {
    pub fn new(capacity: usize, padding: usize) -> Self {
        assert!(capacity > 0 && padding < capacity);
        Self {
            entries: (0..capacity).map(|_| None).collect(),
            last: 0,
            padding,
        }
    }
    pub fn players() -> Self {
        Self::new(2047, 1)
    }
    pub fn next_free(&self) -> Option<usize> {
        (self.last + 1..self.entries.len())
            .chain(self.padding..self.last + 1)
            .find(|i| self.entries[*i].is_none())
    }
    pub fn insert(&mut self, value: T) -> Result<usize, T> {
        if let Some(slot) = self.next_free() {
            self.entries[slot] = Some(value);
            self.last = slot;
            Ok(slot)
        } else {
            Err(value)
        }
    }
    pub fn get(&self, slot: usize) -> Option<&T> {
        self.entries.get(slot).and_then(Option::as_ref)
    }
    pub fn get_mut(&mut self, slot: usize) -> Option<&mut T> {
        self.entries.get_mut(slot).and_then(Option::as_mut)
    }
    pub fn remove(&mut self, slot: usize) -> Option<T> {
        self.entries.get_mut(slot).and_then(Option::take)
    }
    pub fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.as_ref().map(|v| (i, v)))
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(i, v)| v.as_mut().map(|v| (i, v)))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn allocation_rotates_and_excludes_padding() {
        let mut list = EntityList::new(4, 1);
        assert_eq!(list.insert("a"), Ok(1));
        assert_eq!(list.insert("b"), Ok(2));
        list.remove(1);
        assert_eq!(list.insert("c"), Ok(3));
        assert_eq!(list.insert("d"), Ok(1));
        assert_eq!(list.insert("e"), Err("e"));
        assert_eq!(list.get(0), None);
    }
}
