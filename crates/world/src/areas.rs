use crate::CoordGrid;
use std::collections::BTreeMap;

/// Original AreaIndex order is coordinate, zone, square; duplicates across scopes survive.
#[derive(Default, Clone)]
pub struct Areas {
    squares: BTreeMap<u16, Vec<u16>>,
    zones: BTreeMap<u32, Vec<u16>>,
    coords: BTreeMap<i32, Vec<u16>>,
}
fn insert<K: Ord>(
    map: &mut BTreeMap<K, Vec<u16>>,
    key: K,
    ids: impl IntoIterator<Item = u16>,
) -> Result<(), &'static str> {
    let mut ids: Vec<_> = ids.into_iter().collect();
    if ids.is_empty() || ids.len() > 5 || ids.iter().any(|id| *id >= 32767) {
        return Err("Invalid area membership");
    }
    if map.contains_key(&key) {
        return Err("Area key already registered");
    }
    // The pinned AreaIndex's overflow flag is bit 46, overlapping bit 1 of its
    // fourth 15-bit value (offset 45). Preserve that observable packing quirk.
    if ids.len() >= 4 {
        let fourth = ids[3] + 1;
        if ids.len() == 5 {
            ids[3] = (fourth | 2) - 1;
        } else if fourth & 2 != 0 {
            ids.push(u16::MAX);
        }
    }
    map.insert(key, ids);
    Ok(())
}
impl Areas {
    fn square(p: CoordGrid) -> u16 {
        (((p.x() >> 6) << 8) | (p.z() >> 6)) as u16
    }
    fn zone(p: CoordGrid) -> u32 {
        ((p.level() as u32) << 22) | ((p.x() as u32 >> 3) << 11) | (p.z() as u32 >> 3)
    }
    pub fn register_square(
        &mut self,
        key: u16,
        ids: impl IntoIterator<Item = u16>,
    ) -> Result<(), &'static str> {
        insert(&mut self.squares, key, ids)
    }
    pub fn register_zone(
        &mut self,
        p: CoordGrid,
        ids: impl IntoIterator<Item = u16>,
    ) -> Result<(), &'static str> {
        insert(&mut self.zones, Self::zone(p), ids)
    }
    pub fn register_coord(
        &mut self,
        p: CoordGrid,
        ids: impl IntoIterator<Item = u16>,
    ) -> Result<(), &'static str> {
        insert(&mut self.coords, p.packed(), ids)
    }
    pub fn at(&self, p: CoordGrid) -> Vec<u16> {
        let mut result = Vec::new();
        for values in [
            self.coords.get(&p.packed()),
            self.zones.get(&Self::zone(p)),
            self.squares.get(&Self::square(p)),
        ]
        .into_iter()
        .flatten()
        {
            result.extend(values.iter().copied());
        }
        result
    }
}
