//! Target239 camera V2 commands, using absolute world tiles.
use crate::{buffer::DecodeError, framing::Packet};
fn position(x: u16, z: u16) -> Result<(), DecodeError> {
    if x >= 16384 || z >= 16384 {
        Err(DecodeError::InvalidValue("camera tile outside world"))
    } else {
        Ok(())
    }
}
pub fn move_to(
    x: u16,
    z: u16,
    height: u16,
    rate: u8,
    proportional: u8,
) -> Result<Packet, DecodeError> {
    position(x, z)?;
    let mut payload = vec![];
    payload.extend(x.to_le_bytes());
    payload.push(rate.wrapping_neg());
    payload.extend(height.to_le_bytes());
    payload.push(proportional.wrapping_neg());
    payload.extend(z.to_be_bytes());
    Ok(Packet {
        opcode: 146,
        payload,
    })
}
pub fn look_at(
    x: u16,
    z: u16,
    height: u16,
    rate: u8,
    proportional: u8,
) -> Result<Packet, DecodeError> {
    position(x, z)?;
    let mut payload = vec![];
    payload.extend(z.to_be_bytes());
    let h = height.to_be_bytes();
    payload.extend([
        h[0],
        h[1].wrapping_add(128),
        proportional,
        128u8.wrapping_sub(rate),
    ]);
    payload.extend(x.to_le_bytes());
    Ok(Packet {
        opcode: 139,
        payload,
    })
}
pub fn reset() -> Packet {
    Packet {
        opcode: 3,
        payload: vec![],
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_v2_absolute_tiles_match_target_wire_and_refuse_out_of_world_before_packet() {
        assert_eq!(
            move_to(3200, 3194, 1500, 2, 10).unwrap().payload,
            [128, 12, 254, 220, 5, 246, 12, 122]
        );
        assert_eq!(
            look_at(3200, 3203, 450, 2, 10).unwrap().payload,
            [12, 131, 1, 66, 10, 126, 128, 12]
        );
        assert!(move_to(16384, 0, 0, 0, 0).is_err());
        assert!(look_at(0, 16384, 0, 0, 0).is_err());
        assert_eq!((reset().opcode, reset().payload.len()), (3, 0));
    }
}
