//! Byte2IntOpenHashMap storage order used by Kotlin's ZoneLocMap.
//! Retaining this order matters when persistent zone messages are rebuilt.
#[derive(Clone)]
pub(crate) struct ByteMap<T: Copy> {
    slots: Vec<Option<(u8, T)>>,
    zero: Option<T>,
    size: usize,
}
impl<T: Copy> Default for ByteMap<T> {
    fn default() -> Self {
        Self {
            slots: vec![None; 32],
            zero: None,
            size: 0,
        }
    }
}
fn hash(key: u8) -> usize {
    let h = (key as i8 as i32 as u32).wrapping_mul(0x9e3779b9);
    (h ^ (h >> 16)) as usize
}
impl<T: Copy> ByteMap<T> {
    fn index(&self, key: u8) -> usize {
        let mut pos = hash(key) & (self.slots.len() - 1);
        while let Some((k, _)) = self.slots[pos] {
            if k == key {
                break;
            }
            pos = (pos + 1) & (self.slots.len() - 1);
        }
        pos
    }
    pub fn get(&self, key: u8) -> Option<T> {
        if key == 0 {
            self.zero
        } else {
            self.slots[self.index(key)].map(|(_, v)| v)
        }
    }
    pub fn insert(&mut self, key: u8, value: T) -> Option<T> {
        let old = if key == 0 {
            self.zero.replace(value)
        } else {
            let p = self.index(key);
            self.slots[p].replace((key, value)).map(|(_, v)| v)
        };
        if old.is_none() {
            let before = self.size;
            self.size += 1;
            if before >= self.slots.len() * 3 / 4 {
                self.rehash(self.slots.len() * 2);
            }
        }
        old
    }
    fn rehash(&mut self, n: usize) {
        let old = std::mem::replace(&mut self.slots, vec![None; n]);
        for entry in old.into_iter().rev().flatten() {
            let p = self.index(entry.0);
            self.slots[p] = Some(entry);
        }
    }
    pub fn remove(&mut self, key: u8) -> Option<T> {
        let value = if key == 0 {
            self.zero.take()?
        } else {
            let mut last = self.index(key);
            let value = self.slots[last]?.1;
            loop {
                let mut pos = (last + 1) & (self.slots.len() - 1);
                loop {
                    let Some((key, _)) = self.slots[pos] else {
                        self.slots[last] = None;
                        break;
                    };
                    let slot = hash(key) & (self.slots.len() - 1);
                    if if last <= pos {
                        last >= slot || slot > pos
                    } else {
                        last >= slot && slot > pos
                    } {
                        break;
                    }
                    pos = (pos + 1) & (self.slots.len() - 1);
                }
                if self.slots[pos].is_none() {
                    break;
                }
                self.slots[last] = self.slots[pos];
                last = pos;
            }
            value
        };
        self.size -= 1;
        if self.slots.len() > 32 && self.size < (self.slots.len() * 3 / 4) / 4 {
            self.rehash(self.slots.len() / 2);
        }
        Some(value)
    }
    pub fn entries(&self) -> impl Iterator<Item = (u8, T)> + '_ {
        self.zero
            .into_iter()
            .map(|v| (0, v))
            .chain(self.slots.iter().rev().filter_map(|&e| e))
    }
}
