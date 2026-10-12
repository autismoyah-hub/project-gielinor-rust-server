//! MESSAGE_GAME: smart chat type, optional sender, CP1252 message.
use crate::{appearance::string, buffer::DecodeError, framing::Packet};
pub fn game(kind: u16, name: Option<&str>, message: &str) -> Result<Packet, DecodeError> {
    if kind > 32767 {
        return Err(DecodeError::InvalidValue("chat type"));
    }
    let mut payload = Vec::new();
    if kind < 128 {
        payload.push(kind as u8);
    } else {
        payload.extend((kind + 32768).to_be_bytes());
    }
    payload.push(name.is_some() as u8);
    if let Some(name) = name {
        string(&mut payload, name)?;
    }
    string(&mut payload, message)?;
    if payload.len() > 255 {
        return Err(DecodeError::InvalidValue("chat payload length"));
    }
    Ok(Packet {
        opcode: 74,
        payload,
    })
}
