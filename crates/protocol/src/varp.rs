use crate::framing::Packet;
/// VarpSync uses signed byte range, not the unsigned 0..255 range.
pub fn update(id: u16, value: i32) -> Packet {
    let [high, low] = id.to_be_bytes();
    if (-128..=127).contains(&value) {
        Packet {
            opcode: 97,
            payload: vec![(value as u8).wrapping_add(128), low.wrapping_add(128), high],
        }
    } else {
        let mut payload = vec![high, low.wrapping_add(128)];
        payload.extend(value.to_le_bytes());
        Packet {
            opcode: 12,
            payload,
        }
    }
}
pub fn reset() -> Packet {
    Packet {
        opcode: 44,
        payload: vec![],
    }
}
pub fn sync() -> Packet {
    Packet {
        opcode: 76,
        payload: vec![],
    }
}
