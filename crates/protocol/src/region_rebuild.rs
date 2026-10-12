//! Revision 239 REBUILD_REGION_V2. Source map-square keys are counted once,
//! regardless of how many destination chunks reuse them.
use crate::{
    buffer::{BitWriter, DecodeError},
    framing::Packet,
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub x: u16,
    pub z: u16,
    pub level: u8,
    pub rotation: u8,
}
impl Zone {
    fn packed(self) -> Result<u32, DecodeError> {
        if self.x > 1023 || self.z > 2047 || self.level > 3 || self.rotation > 3 {
            return Err(DecodeError::InvalidValue("reference zone"));
        }
        Ok((self.level as u32) << 24
            | (self.x as u32) << 14
            | (self.z as u32) << 3
            | (self.rotation as u32) << 1)
    }
}

pub fn encode(
    center_x: u16,
    center_z: u16,
    reload: bool,
    mut source: impl FnMut(i32, i32, u8) -> Option<Zone>,
) -> Result<Packet, DecodeError> {
    let mut bits = BitWriter::default();
    let mut squares = BTreeSet::new();
    for level in 0..4 {
        for x in center_x as i32 - 6..=center_x as i32 + 6 {
            for z in center_z as i32 - 6..=center_z as i32 + 6 {
                match source(x, z, level) {
                    None => bits.write(1, 0)?,
                    Some(zone) => {
                        let packed = zone.packed()?;
                        bits.write(1, 1)?;
                        bits.write(26, packed)?;
                        squares.insert(((zone.x >> 3) << 8) | (zone.z >> 3));
                    }
                }
            }
        }
    }
    let mut payload = Vec::new();
    payload.extend(center_z.to_be_bytes());
    payload.extend(center_x.to_be_bytes());
    payload.push(128 + u8::from(reload));
    payload.extend((squares.len() as u16).to_be_bytes());
    payload.extend(bits.into_bytes());
    Ok(Packet {
        opcode: 125,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reused_source_square_is_counted_once_across_levels() {
        let packet = encode(1000, 10, true, |_, _, level| {
            Some(Zone {
                x: 404,
                z: 348,
                level,
                rotation: 3,
            })
        })
        .unwrap();
        assert_eq!(&packet.payload[..7], &[0, 10, 3, 232, 129, 0, 1]);
        assert_eq!(packet.payload.len(), 7 + (676usize * 27).div_ceil(8));
        assert!(encode(0, 0, false, |_, _, _| Some(Zone {
            x: 1024,
            z: 1,
            level: 0,
            rotation: 0
        }))
        .is_err());
    }
}
