use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::{areas::Areas, CoordGrid};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct MapAreas {
    pub square: Vec<u16>,
    pub zones: BTreeMap<u8, Vec<u16>>,
    pub coords: BTreeMap<u16, Vec<u16>>,
}
fn set(r: &mut Reader<'_>) -> Result<Vec<u16>, DecodeError> {
    let count = r.u8()?;
    let mut result = Vec::new();
    for _ in 0..count {
        let id = r.u16()?;
        if !result.contains(&id) {
            result.push(id);
        }
    }
    Ok(result)
}
impl MapAreas {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut d = Self {
            square: set(&mut r)?,
            ..Default::default()
        };
        let zones = r.u8()?;
        for _ in 0..zones {
            let key = r.u8()?;
            d.zones.insert(key, set(&mut r)?);
        }
        // InlineByteBuf.readShort is signed; Kotlin repeat performs no iterations if negative.
        let coords = r.u16()? as i16;
        for _ in 0..coords {
            let key = r.u16()?;
            d.coords.insert(key, set(&mut r)?);
        }
        Ok(d)
    }
    pub fn register(&self, square: u16, index: &mut Areas) -> Result<(), &'static str> {
        let x = (square >> 8) as i32 * 64;
        let z = (square & 255) as i32 * 64;
        if !self.square.is_empty() {
            index.register_square(square, self.square.iter().copied())?;
        }
        for (&key, ids) in &self.zones {
            let p = CoordGrid::new(
                x + ((key as i32 >> 3) & 7) * 8,
                z + (key as i32 & 7) * 8,
                (key >> 6) as i32,
            )
            .unwrap();
            index.register_zone(p, ids.iter().copied())?;
        }
        for (&key, ids) in &self.coords {
            let p = CoordGrid::new(
                x + ((key as i32 >> 6) & 63),
                z + (key as i32 & 63),
                ((key >> 12) & 3) as i32,
            )
            .unwrap();
            index.register_coord(p, ids.iter().copied())?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_scopes_duplicate_memberships_and_planes() {
        let bytes = [2, 0, 7, 0, 7, 1, 0, 1, 0, 8, 0, 1, 0, 65, 2, 0, 7, 0, 9];
        let d = MapAreas::decode(&bytes).unwrap();
        let mut index = Areas::default();
        d.register(0x3232, &mut index).unwrap();
        let at = |x, z, l| {
            index
                .at(CoordGrid::new(x, z, l).unwrap())
                .into_iter()
                .collect::<Vec<_>>()
        };
        assert_eq!(at(3201, 3201, 0), [7, 9, 8, 7]);
        assert_eq!(at(3201, 3202, 0), [8, 7]);
        assert_eq!(at(3201, 3201, 1), [7]);
        assert!(at(3264, 3201, 0).is_empty());
    }
}
