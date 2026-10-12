//! Revision 239 JS5 commands and RSProt's 512-byte response segmentation.
use crate::buffer::DecodeError;
use crate::framing::{PacketDef, Size};
pub static CLIENT: &[PacketDef] = &[
    PacketDef {
        opcode: 0,
        size: Size::Fixed(3),
        name: "PREFETCH_REQUEST",
    },
    PacketDef {
        opcode: 1,
        size: Size::Fixed(3),
        name: "URGENT_REQUEST",
    },
    PacketDef {
        opcode: 2,
        size: Size::Fixed(3),
        name: "PRIORITY_CHANGE_HIGH",
    },
    PacketDef {
        opcode: 3,
        size: Size::Fixed(3),
        name: "PRIORITY_CHANGE_LOW",
    },
    PacketDef {
        opcode: 4,
        size: Size::Fixed(3),
        name: "XOR_CHANGE",
    },
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handshake {
    pub revision: u32,
    pub seed: [u32; 4],
}
pub fn handshake(bytes: &[u8]) -> Result<Handshake, DecodeError> {
    if bytes.len() != 20 {
        return Err(DecodeError::InvalidValue("JS5 handshake length"));
    }
    let words: Vec<_> = bytes
        .chunks_exact(4)
        .map(|v| u32::from_be_bytes(v.try_into().unwrap()))
        .collect();
    Ok(Handshake {
        revision: words[0],
        seed: words[1..].try_into().unwrap(),
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Request {
        archive: u8,
        group: u16,
        urgent: bool,
    },
    Priority {
        high: bool,
    },
    Xor(u8),
}
pub fn command(opcode: u8, payload: &[u8]) -> Result<Command, DecodeError> {
    if payload.len() != 3 {
        return Err(DecodeError::InvalidValue("JS5 command length"));
    }
    match opcode {
        0 | 1 => Ok(Command::Request {
            archive: payload[0],
            group: u16::from_be_bytes([payload[1], payload[2]]),
            urgent: opcode == 1,
        }),
        2 | 3 => Ok(Command::Priority { high: opcode == 2 }),
        4 => Ok(Command::Xor(payload[0])),
        _ => Err(DecodeError::InvalidValue("Unknown JS5 command")),
    }
}
/// Input is a compressed JS5 container with any group-version trailer removed.
/// The original framing helper accepts arbitrary bytes; the store validates them.
pub fn response(archive: u8, group: u16, container: &[u8]) -> Vec<u8> {
    let first = container.len().min(509);
    let markers = container.len().saturating_sub(first).div_ceil(511);
    let mut bytes = Vec::with_capacity(3 + container.len() + markers);
    bytes.push(archive);
    bytes.extend(group.to_be_bytes());
    bytes.extend_from_slice(&container[..first]);
    for block in container[first..].chunks(511) {
        bytes.push(255);
        bytes.extend_from_slice(block);
    }
    bytes
}
pub fn xor(bytes: &mut [u8], key: u8) {
    for byte in bytes {
        *byte ^= key;
    }
}
