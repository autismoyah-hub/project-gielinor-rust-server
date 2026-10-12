//! Server map files in archive 5: terrain 0, locations 1, NPCs 100, objects 101.
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::collision::CollisionMap;
pub const BLOCK: u8 = 1;
pub const LINK_BELOW: u8 = 2;
pub const ROOF: u8 = 4;
pub const COLOURED: u8 = 8;
#[derive(Clone)]
pub struct Tiles {
    flags: Vec<u8>,
}
impl Tiles {
    pub fn get(&self, x: usize, z: usize, level: usize) -> u8 {
        self.flags[(z & 63) | ((x & 63) << 6) | ((level & 3) << 12)]
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut flags = vec![0; 16384];
        for tile in &mut flags {
            // Kotlin leaves the remaining tiles empty when EOF occurs at an opcode boundary.
            while r.remaining() > 0 {
                let opcode = r.u16()? as i16 as i32;
                match opcode {
                    0 => break,
                    1 => {
                        r.u8()?;
                        break;
                    }
                    i32::MIN..=49 => {
                        if r.u16()? != 0 {
                            *tile |= COLOURED;
                        }
                    }
                    50..=81 => *tile |= ((opcode - 49) as u8) & (BLOCK | LINK_BELOW | ROOF),
                    _ => {
                        if opcode - 81 != 0 {
                            *tile |= COLOURED;
                        }
                    }
                }
            }
        }
        Ok(Self { flags })
    }
    /// Preserve the reference's sparse allocation and level wrapping, including bridge tiles.
    pub fn apply_collision(&self, map: &mut CollisionMap, square: u16) {
        let base_x = ((square >> 8) as i32) * 64;
        let base_z = ((square & 255) as i32) * 64;
        for level in 0..4 {
            for x in 0..64 {
                for z in 0..64 {
                    let flags = self.get(x, z, level);
                    if flags == 0 {
                        continue;
                    }
                    let mut mask = 0;
                    if flags & BLOCK != 0 {
                        mask |= 0x200000
                    }
                    if flags & ROOF != 0 {
                        mask |= 0x80000000
                    }
                    let resolved = level as i32 - if flags & LINK_BELOW != 0 { 1 } else { 0 };
                    map.set(base_x + x as i32, base_z + z as i32, resolved, mask);
                }
            }
        }
    }
    pub fn visual_level(&self, loc: Location) -> Option<u8> {
        let below = self.get(loc.x as usize, loc.z as usize, loc.level as usize);
        let above = self.get(
            loc.x as usize,
            loc.z as usize,
            (loc.level + 1).min(3) as usize,
        );
        let flags = if above & LINK_BELOW != 0 {
            above
        } else {
            below
        };
        if flags & LINK_BELOW != 0 {
            loc.level.checked_sub(1)
        } else {
            Some(loc.level)
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub id: u32,
    pub x: u8,
    pub z: u8,
    pub level: u8,
    pub shape: u8,
    pub angle: u8,
}
pub fn locations(bytes: &[u8], max_spawns: usize) -> Result<Vec<Location>, DecodeError> {
    let mut r = Reader::new(bytes);
    let mut id = -1i32;
    let mut output = vec![];
    while r.remaining() > 0 {
        let mut increment = 0i32;
        loop {
            let part = r.smart()? as i32;
            increment = increment.wrapping_add(part);
            if part != 32767 {
                break;
            }
        }
        if increment == 0 {
            break;
        }
        id = id.wrapping_add(increment);
        let mut coordinate = 0i32;
        while r.remaining() > 0 {
            let delta = r.smart()? as i32;
            if delta == 0 {
                break;
            }
            coordinate = coordinate.wrapping_add(delta - 1);
            let attributes = r.u8()?;
            if output.len() >= max_spawns {
                return Err(DecodeError::InvalidValue("location count limit"));
            }
            output.push(Location {
                id: id as u32 & 0x1ffff,
                x: ((coordinate >> 6) & 63) as u8,
                z: (coordinate & 63) as u8,
                level: ((coordinate >> 12) & 3) as u8,
                shape: (attributes >> 2) & 31,
                angle: attributes & 3,
            });
        }
    }
    Ok(output)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spawn {
    pub id: u16,
    pub x: u8,
    pub z: u8,
    pub level: u8,
}
fn unpack(value: u32) -> Spawn {
    Spawn {
        id: value as u16,
        x: ((value >> 22) & 63) as u8,
        z: ((value >> 16) & 63) as u8,
        level: ((value >> 28) & 3) as u8,
    }
}
pub fn npcs(bytes: &[u8]) -> Result<Vec<Spawn>, DecodeError> {
    let mut r = Reader::new(bytes);
    let count = r.u16()? as usize;
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        result.push(unpack(r.u32()?));
    }
    Ok(result)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroundObject {
    pub spawn: Spawn,
    pub count: i32,
}
pub fn objects(bytes: &[u8]) -> Result<Vec<GroundObject>, DecodeError> {
    let mut r = Reader::new(bytes);
    let count = r.u16()? as usize;
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let packed = r.u64()?;
        result.push(GroundObject {
            spawn: unpack(packed as u32),
            count: (packed >> 30) as u32 as i32,
        });
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terrain_flags_accumulate_and_keep_sparse_zones() {
        let tiles = Tiles::decode(&[0, 82, 0, 50, 0, 53, 0, 0]).unwrap();
        assert_eq!(tiles.get(0, 0, 0), COLOURED | BLOCK | ROOF);
        let mut map = CollisionMap::default();
        tiles.apply_collision(&mut map, 0x3232);
        assert_eq!(map.get(3200, 3200, 0), 0x80200000);
        assert_eq!(map.get(3201, 3200, 0), 0);
        assert_eq!(map.get(3208, 3200, 0), u32::MAX);
    }
    #[test]
    fn delta_locations_and_packed_spawns() {
        let loc = locations(&[11, 66, 42, 0, 0], 10).unwrap();
        assert_eq!(
            loc,
            [Location {
                id: 10,
                x: 1,
                z: 1,
                level: 0,
                shape: 10,
                angle: 2
            }]
        );
        let packed = 123u32 | (2 << 16) | (3 << 22) | (1 << 28);
        let mut n = vec![0, 1];
        n.extend(packed.to_be_bytes());
        assert_eq!(
            npcs(&n).unwrap(),
            [Spawn {
                id: 123,
                x: 3,
                z: 2,
                level: 1
            }]
        );
        let mut o = vec![0, 1];
        o.extend(((packed as u64) | ((i32::MAX as u64) << 30)).to_be_bytes());
        assert_eq!(objects(&o).unwrap()[0].count, i32::MAX);
    }
}
