use crate::definitions::{params, Param};
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::loc_collision::LocCollision;
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub struct LocationType {
    pub name: String,
    pub collision: LocCollision,
    pub active: i32,
    pub force_approach: u8,
    pub force_decor: bool,
    pub category: i32,
    pub content_group: i32,
    pub ops: [Option<String>; 5],
    pub multi_varbit: i32,
    pub multi_varp: i32,
    pub multi_default: i32,
    pub multi: Vec<i32>,
    pub params: BTreeMap<u32, Param>,
    pub encoded: Vec<u8>,
}
fn nullable(r: &mut Reader<'_>) -> Result<i32, DecodeError> {
    let value = r.u16()?;
    Ok(if value == 65535 { -1 } else { value as i32 })
}
impl LocationType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut d = Self {
            name: "null".into(),
            collision: LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            },
            active: -1,
            force_approach: 0,
            force_decor: false,
            category: -1,
            content_group: -1,
            ops: std::array::from_fn(|_| None),
            multi_varbit: -1,
            multi_varp: -1,
            multi_default: -1,
            multi: vec![],
            params: BTreeMap::new(),
            encoded: bytes.to_vec(),
        };
        while r.remaining() > 0 {
            let code = r.u8()?;
            match code {
                0 => break,
                1 => {
                    let n = r.u8()? as usize;
                    r.take(n * 3)?;
                }
                2 => d.name = r.string()?,
                3 => {
                    r.string()?;
                }
                5 => {
                    let n = r.u8()? as usize;
                    r.take(n * 2)?;
                }
                6 => {
                    let n = r.u8()? as usize;
                    r.take(n * 5)?;
                }
                7 => {
                    let n = r.u8()? as usize;
                    r.take(n * 4)?;
                }
                14 => d.collision.width = r.u8()?,
                15 => d.collision.length = r.u8()?,
                17 => {
                    d.collision.block_walk = 0;
                    d.collision.block_range = false;
                }
                18 => d.collision.block_range = false,
                19 => d.active = r.u8()? as i32,
                27 => d.collision.block_walk = 1,
                21 | 22 | 23 | 62 | 64 | 89 | 90 | 94 => {}
                24 | 65..=68 | 70..=72 | 82 => {
                    r.u16()?;
                }
                28 | 29 | 39 | 75 | 81 | 91 | 95 | 96 => {
                    r.u8()?;
                }
                30..=34 => d.ops[(code - 30) as usize] = Some(r.string()?),
                40 | 41 => {
                    let n = r.u8()? as usize;
                    r.take(n * 4)?;
                }
                61 => d.category = r.u16()? as i32,
                69 => d.force_approach = r.u8()?,
                73 => d.force_decor = true,
                74 => d.collision.break_route_finding = true,
                77 | 92 => {
                    d.multi_varbit = nullable(&mut r)?;
                    d.multi_varp = nullable(&mut r)?;
                    d.multi_default = if code == 92 { nullable(&mut r)? } else { -1 };
                    let n = r.u8()? as usize + 1;
                    d.multi.clear();
                    for _ in 0..n {
                        d.multi.push(nullable(&mut r)?);
                    }
                }
                78 => {
                    r.take(4)?;
                }
                79 => {
                    r.take(6)?;
                    let n = r.u8()? as usize;
                    r.take(n * 2)?;
                }
                93 => {
                    r.take(6)?;
                }
                100 => {
                    r.take(2)?;
                    r.string()?;
                }
                101 => {
                    r.take(13)?;
                    r.string()?;
                }
                102 => {
                    r.take(15)?;
                    r.string()?;
                }
                200 => d.content_group = r.u16()? as i32,
                249 => d.params = params(&mut r)?,
                _ => return Err(DecodeError::InvalidValue("unknown location opcode")),
            }
        }
        Ok(d)
    }
    pub fn transformed_id(&self, value: i32) -> Option<u32> {
        let id = if value >= 0 {
            self.multi
                .get(value as usize)
                .copied()
                .unwrap_or(self.multi_default)
        } else {
            self.multi_default
        };
        (id >= 0).then_some(id as u32)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collision_flags_and_transform_fallback() {
        let d = LocationType::decode(&[
            14, 2, 15, 3, 18, 74, 92, 255, 255, 0, 12, 0, 50, 1, 0, 10, 255, 255, 0,
        ])
        .unwrap();
        assert_eq!((d.collision.width, d.collision.length), (2, 3));
        assert!(!d.collision.block_range);
        assert!(d.collision.break_route_finding);
        assert_eq!(d.transformed_id(0), Some(10));
        assert_eq!(d.transformed_id(1), None);
        assert_eq!(d.transformed_id(2), Some(50));
    }
}
