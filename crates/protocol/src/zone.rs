//! Revision 239 desktop enclosed ground-object messages and zone headers.
use crate::{buffer::DecodeError, framing::Packet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Add {
    pub id: u16,
    pub count: i32,
    pub x: u8,
    pub z: u8,
    pub ops: u8,
    pub until_public: u16,
    pub until_despawn: u16,
    pub ownership: u8,
    pub never_public: bool,
}
impl Add {
    pub fn visible(id: u16, count: i32, x: u8, z: u8) -> Self {
        Self {
            id,
            count,
            x,
            z,
            ops: 255,
            until_public: 0,
            until_despawn: 0,
            ownership: 0,
            never_public: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Object {
    Graphic {id:u16,height:u8,delay:u16,x:u8,z:u8},
    SoundArea(SoundArea),
    Projectile(Projectile),
    Add(Add),
    Delete {
        id: u16,
        count: i32,
        x: u8,
        z: u8,
    },
    Count {
        id: u16,
        old: i32,
        new: i32,
        x: u8,
        z: u8,
    },
    LocAdd {
        id: u16,
        x: u8,
        z: u8,
        shape: u8,
        angle: u8,
        ops: u8,
    },
    LocDelete {
        x: u8,
        z: u8,
        shape: u8,
        angle: u8,
    },
    LocAnimate {
        sequence: u16,
        x: u8,
        z: u8,
        shape: u8,
        angle: u8,
    },
}
/// Revision239 enclosed wire type14 (internal RSProt protId9).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoundArea {
    pub id: u16,
    pub delay: u8,
    pub loops: u8,
    pub range: u8,
    pub drop_off_range: u8,
    pub x: u8,
    pub z: u8,
}
/// MAP_PROJANIM_V2 uses enclosed wire type 13, with no standalone opcode.
/// RSProt's internal protId is not the encoded enclosure discriminator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projectile {
    pub id: u16,
    pub start_height: u16,
    pub end_height: u16,
    pub start_time: u16,
    pub end_time: u16,
    pub angle: u8,
    pub progress: u16,
    pub source_index: i32,
    pub target_index: i32,
    pub x: u8,
    pub z: u8,
    pub end: i32,
}
fn coord(x: u8, z: u8) -> Result<u8, DecodeError> {
    if x > 7 || z > 7 {
        return Err(DecodeError::InvalidValue("zone coordinate"));
    }
    Ok((x << 4) | z)
}
impl Object {
    pub fn encode(self) -> Result<Vec<u8>, DecodeError> {
        let mut out = Vec::new();
        match self {
            Self::Graphic{id,height,delay,x,z}=>{
                // Existing revision-239 MapAnim enclosed record, not a new opcode.
                out.extend([5,height]);out.extend(id.to_le_bytes());out.extend(delay.to_le_bytes());out.push(coord(x,z)?);
            }
            Self::SoundArea(v) => {
                out.extend([
                    14,
                    128u8.wrapping_sub(coord(v.x, v.z)?),
                    v.drop_off_range.wrapping_neg(),
                    v.range,
                    v.delay.wrapping_neg(),
                    v.loops.wrapping_add(128),
                ]);
                out.extend(v.id.to_be_bytes());
            }
            Self::Projectile(v) => {
                out.push(13);
                out.extend(v.start_time.to_be_bytes());
                out.push(coord(v.x, v.z)?);
                for value in [v.progress, v.id] {
                    let [high, low] = value.to_be_bytes();
                    out.extend([high, low.wrapping_add(128)]);
                }
                out.extend(v.end_time.to_le_bytes());
                out.extend(v.end.to_le_bytes());
                let [high, low] = v.end_height.to_be_bytes();
                out.extend([low.wrapping_add(128), high]);
                let [_, a, b, c] = v.source_index.to_be_bytes();
                out.extend([a, c, b]);
                out.push(v.angle.wrapping_neg());
                out.extend(v.start_height.to_be_bytes());
                out.extend(&v.target_index.to_be_bytes()[1..]);
            }
            Self::LocAdd {
                id,
                x,
                z,
                shape,
                angle,
                ops,
            } => {
                let prop = properties(shape, angle)?;
                let [high, low] = id.to_be_bytes();
                out.extend([
                    7,
                    128,
                    128u8.wrapping_sub(ops),
                    prop.wrapping_add(128),
                    coord(x, z)?.wrapping_neg(),
                    low.wrapping_add(128),
                    high,
                ]);
            }
            Self::LocDelete { x, z, shape, angle } => {
                out.extend([0, properties(shape, angle)?, coord(x, z)?.wrapping_neg()])
            }
            Self::LocAnimate {
                sequence,
                x,
                z,
                shape,
                angle,
            } => {
                let [high, low] = sequence.to_be_bytes();
                out.extend([
                    9,
                    properties(shape, angle)?,
                    128u8.wrapping_sub(coord(x, z)?),
                    low.wrapping_add(128),
                    high,
                ]);
            }
            Self::Add(v) => {
                out.extend([8, coord(v.x, v.z)?.wrapping_add(128)]);
                out.extend(v.until_public.to_be_bytes());
                out.push(v.ownership);
                let [high, low] = v.id.to_be_bytes();
                out.extend([high, low.wrapping_add(128)]);
                out.extend([u8::from(v.never_public), v.ops.wrapping_neg()]);
                let [high, low] = v.until_despawn.to_be_bytes();
                out.extend([low.wrapping_add(128), high]);
                out.extend(v.count.to_le_bytes());
            }
            Self::Delete { id, count, x, z } => {
                out.push(10);
                out.extend(id.to_be_bytes());
                out.push(coord(x, z)?);
                let [a, b, c, d] = count.to_be_bytes();
                out.extend([c, d, a, b]);
            }
            Self::Count { id, old, new, x, z } => {
                out.push(4);
                let [a, b, c, d] = old.to_be_bytes();
                out.extend([b, a, d, c]);
                out.push(coord(x, z)?.wrapping_add(128));
                out.extend(new.to_le_bytes());
                let [high, low] = id.to_be_bytes();
                out.extend([low.wrapping_add(128), high]);
            }
        }
        Ok(out)
    }
}
fn properties(shape: u8, angle: u8) -> Result<u8, DecodeError> {
    if shape > 31 || angle > 3 {
        return Err(DecodeError::InvalidValue("location shape or angle"));
    }
    Ok(shape << 2 | angle)
}
pub fn reset(x: u8, z: u8, level: u8) -> Packet {
    Packet {
        opcode: 54,
        payload: vec![128u8.wrapping_sub(z), x.wrapping_neg(), level],
    }
}
pub fn enclosed(x: u8, z: u8, level: u8, objects: &[Object]) -> Result<Packet, DecodeError> {
    let mut payload = vec![
        level.wrapping_neg(),
        z.wrapping_add(128),
        x.wrapping_add(128),
    ];
    for object in objects {
        let bytes = object.encode()?;
        if payload.len() + bytes.len() > 40_000 {
            return Err(DecodeError::InvalidValue(
                "zone enclosure exceeds 40000 bytes",
            ));
        }
        payload.extend(bytes);
    }
    Ok(Packet {
        opcode: 105,
        payload,
    })
}


#[cfg(test)]
mod graphic_tests {
    use super::*;
    #[test]
    fn original_rsprot_mapanim_vector_and_coordinate_bounds() {
        // Independently emitted by cached RSProt239 Kotlin desktop encoder.
        assert_eq!(Object::Graphic{id:0x1234,height:7,delay:0x5678,x:3,z:6}.encode().unwrap(),[5,7,0x34,0x12,0x78,0x56,0x36]);
        assert!(Object::Graphic{id:1,height:0,delay:0,x:8,z:0}.encode().is_err());
        assert!(Object::Graphic{id:1,height:0,delay:0,x:0,z:8}.encode().is_err());
    }
}
