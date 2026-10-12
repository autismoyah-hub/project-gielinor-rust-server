//! Kotlin HeadbarTypeDecoder fields, including the segment count used by combat.
use gielinor_protocol::buffer::{DecodeError, Reader};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadbarType {
    pub unknown1: u16,
    pub show_priority: u8,
    pub hide_priority: u8,
    pub fadeout: i32,
    pub stick_time: u16,
    pub unknown6: u8,
    pub full: Option<u32>,
    pub empty: Option<u32>,
    pub segments: u8,
    pub padding: u8,
    pub decoded_bytes: usize,
}
fn nullable_large(r: &mut Reader<'_>) -> Result<Option<u32>, DecodeError> {
    let hi = r.u16()?;
    if hi & 0x8000 != 0 {
        Ok(Some(((hi as u32 & 0x7fff) << 16) | r.u16()? as u32))
    } else {
        Ok((hi != 32767).then_some(hi as u32))
    }
}
impl HeadbarType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut t = Self {
            unknown1: 0,
            show_priority: 255,
            hide_priority: 255,
            fadeout: -1,
            stick_time: 70,
            unknown6: 0,
            full: None,
            empty: None,
            segments: 30,
            padding: 0,
            decoded_bytes: 0,
        };
        while r.remaining() != 0 {
            match r.u8()? {
                0 => break,
                1 => t.unknown1 = r.u16()?,
                2 => t.show_priority = r.u8()?,
                3 => t.hide_priority = r.u8()?,
                4 => t.fadeout = 0,
                5 => t.stick_time = r.u16()?,
                6 => t.unknown6 = r.u8()?,
                7 => t.full = nullable_large(&mut r)?,
                8 => t.empty = nullable_large(&mut r)?,
                11 => t.fadeout = r.u16()? as i32,
                14 => t.segments = r.u8()?,
                15 => t.padding = r.u8()?,
                _ => return Err(DecodeError::InvalidValue("Unknown headbar opcode")),
            }
        }
        t.decoded_bytes = r.position();
        Ok(t)
    }
}
