//! Revision 239 client initialization packets used by LoginScript.
use crate::{buffer::DecodeError, framing::Packet};
pub fn chat_filters(public: u8, trade: u8) -> Packet {
    Packet {
        opcode: 124,
        payload: vec![public.wrapping_add(128), 128u8.wrapping_sub(trade)],
    }
}
#[derive(Clone, Copy)]
pub enum Operations {
    Npc,
    Location,
    Object,
}
pub fn hide_operations(kind: Operations, hidden: bool) -> Packet {
    Packet {
        opcode: match kind {
            Operations::Npc => 75,
            Operations::Location => 21,
            Operations::Object => 73,
        },
        payload: vec![hidden as u8],
    }
}
pub fn player_operation(
    slot: u8,
    priority: bool,
    operation: Option<&str>,
) -> Result<Packet, DecodeError> {
    if !(1..=8).contains(&slot) {
        return Err(DecodeError::InvalidValue("player operation slot"));
    }
    let mut payload = vec![priority as u8, slot.wrapping_add(128)];
    crate::appearance::string(&mut payload, operation.unwrap_or("null"))?;
    if payload.len() > 255 {
        return Err(DecodeError::InvalidValue("player operation length"));
    }
    Ok(Packet { opcode: 6, payload })
}
pub fn camera_reset() -> Packet {
    Packet {
        opcode: 3,
        payload: vec![],
    }
}
pub fn reset_animations() -> Packet {
    Packet {
        opcode: 92,
        payload: vec![],
    }
}
pub fn minimap(state: u8) -> Packet {
    Packet {
        opcode: 43,
        payload: vec![state],
    }
}
