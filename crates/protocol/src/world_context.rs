//! Root-world synchronization context. Dynamic world-entity lists are not encoded here.
use crate::{buffer::DecodeError, framing::Packet};

pub fn active_root(level: u8) -> Result<Packet, DecodeError> {
    if level > 3 {
        return Err(DecodeError::InvalidValue("Root world level"));
    }
    Ok(Packet {
        opcode: 47,
        payload: vec![0, 0, level],
    })
}
pub fn npc_origin(x: u8, z: u8) -> Packet {
    Packet {
        opcode: 116,
        payload: vec![x, z],
    }
}
/// Valid only for a world with no dynamic entities, including no retained entities.
pub fn empty_entities() -> Packet {
    Packet {
        opcode: 122,
        payload: vec![0],
    }
}
