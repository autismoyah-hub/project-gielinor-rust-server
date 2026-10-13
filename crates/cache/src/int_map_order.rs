//! Int2ObjectOpenHashMap's default-capacity insertion/iteration order. Kotlin's
//! DbTableResolver derives music IDs from this order, not ascending DBROW IDs.
fn hash(key: i32) -> usize {
    let h = (key as u32).wrapping_mul(0x9e3779b9);
    (h ^ (h >> 16)) as usize
}
fn slot(slots: &[Option<i32>], key: i32) -> usize {
    let mut p = hash(key) & (slots.len() - 1);
    while slots[p].is_some_and(|old| old != key) {
        p = (p + 1) & (slots.len() - 1);
    }
    p
}
pub fn order(keys: impl IntoIterator<Item = i32>) -> Vec<i32> {
    ordered(keys, 32)
}
/// Constructor taking a Map sizes the hash table to its entry count before putAll.
pub fn copied_map_order(keys: &[i32]) -> Vec<i32> {
    let needed = keys.len().saturating_mul(4).div_ceil(3).max(2);
    ordered(keys.iter().copied(), needed.next_power_of_two())
}
fn ordered(keys: impl IntoIterator<Item = i32>, capacity: usize) -> Vec<i32> {
    let mut slots = vec![None; capacity];
    let mut zero = false;
    let mut size = 0;
    for key in keys {
        let added = if key == 0 {
            let added = !zero;
            zero = true;
            added
        } else {
            let p = slot(&slots, key);
            slots[p].replace(key).is_none()
        };
        if added {
            let previous = size;
            size += 1;
            if previous >= (slots.len() * 3).div_ceil(4).min(slots.len() - 1) {
                let mut next = vec![None; slots.len() * 2];
                for old in slots.into_iter().rev().flatten() {
                    let p = slot(&next, old);
                    next[p] = Some(old);
                }
                slots = next;
            }
        }
    }
    let mut result = Vec::with_capacity(size);
    if zero {
        result.push(0);
    }
    result.extend(slots.into_iter().rev().flatten());
    result
}
