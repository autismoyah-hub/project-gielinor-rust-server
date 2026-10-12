//! Revision 239 sequence metadata used by server animation scheduling.
use gielinor_protocol::buffer::{DecodeError, Reader};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceType {
    pub delays: Vec<i16>,
    pub priority: u8,
    pub duplicate_behaviour: u8,
    pub keyframe_set: i32,
    pub keyframe_range: (u16, u16),
    pub decoded_bytes: usize,
}
impl SequenceType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let mut t = Self {
            delays: vec![],
            priority: 5,
            duplicate_behaviour: 2,
            keyframe_set: -1,
            keyframe_range: (0, 0),
            decoded_bytes: 0,
        };
        while r.remaining() != 0 {
            match r.u8()? {
                0 => break,
                1 => {
                    let count = r.u16()? as usize;
                    t.delays = (0..count)
                        .map(|_| r.u16().map(|v| v as i16))
                        .collect::<Result<_, _>>()?;
                    r.take(count * 4)?;
                }
                2 | 6 | 7 => {
                    r.u16()?;
                }
                3 | 17 => {
                    let count = r.u8()? as usize;
                    r.take(count)?;
                }
                4 | 19 => (),
                5 => t.priority = r.u8()?,
                8 | 16 => {
                    r.u8()?;
                }
                9 | 10 => {
                    if r.u8()? > 2 {
                        return Err(DecodeError::InvalidValue("Sequence movement policy"));
                    }
                }
                11 => t.duplicate_behaviour = r.u8()?,
                12 => {
                    let count = r.u8()? as usize;
                    r.take(count * 4)?;
                }
                13 => t.keyframe_set = r.u32()? as i32,
                14 => {
                    let count = r.u16()? as usize;
                    r.take(count * 8)?;
                }
                15 => t.keyframe_range = (r.u16()?, r.u16()?),
                18 => {
                    r.string()?;
                }
                _ => return Err(DecodeError::InvalidValue("Unknown sequence opcode")),
            }
        }
        t.decoded_bytes = r.position();
        Ok(t)
    }
    /// UnpackedSeqType's duration uses signed delays, trims trailing values over 30,
    /// and adds a five-frame buffer when trimming. Keyframe ranges do not override it.
    pub fn tick_duration(&self) -> i32 {
        let end = self
            .delays
            .iter()
            .rposition(|d| *d <= 30)
            .map_or(0, |i| i + 1);
        let sum = self.delays[..end]
            .iter()
            .fold(0i32, |sum, d| sum.wrapping_add(*d as i32));
        let frames = sum.wrapping_add(if end != self.delays.len() { 5 } else { 0 });
        (frames.wrapping_mul(20) as f64 / 600.0).ceil() as i32
    }
}
