//! Server-relevant IF1/IF3 component metadata, with the complete original bytes
//! retained for consumers needing presentation fields or script hooks.
use gielinor_protocol::buffer::{DecodeError, Reader};
#[derive(Debug, Clone)]
pub struct ComponentType {
    pub v3: bool,
    pub kind: u8,
    pub button_type: u8,
    pub layer: Option<u32>,
    pub events: i32,
    pub operations: Vec<String>,
    pub encoded: Vec<u8>,
    pub decoded_bytes: usize,
}
fn parent(r: &mut Reader<'_>, id: u32) -> Result<Option<u32>, DecodeError> {
    let layer = r.u16()?;
    Ok((layer != u16::MAX).then_some((id & 0xffff0000) | layer as u32))
}
impl ComponentType {
    pub fn decode(id: u32, bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let first = r.u8()?;
        let mut component = Self {
            v3: first == 255,
            kind: first,
            button_type: 0,
            layer: None,
            events: 0,
            operations: vec![],
            encoded: bytes.to_vec(),
            decoded_bytes: 0,
        };
        if component.v3 {
            component.kind = r.u8()?;
            r.take(10)?; // Content type, x/y and width/height (signed for type 9).
            let width_mode = r.u8()?;
            let height_mode = r.u8()?;
            r.take(2)?; // x/y modes.
            component.layer = parent(&mut r, id)?;
            r.take(1)?;
            match component.kind {
                0 => {
                    r.take(5)?;
                }
                5 => {
                    r.take(15)?;
                }
                6 => {
                    r.take(21)?;
                    if width_mode != 0 {
                        r.take(2)?;
                    }
                    if height_mode != 0 {
                        r.take(2)?;
                    }
                }
                4 => {
                    r.take(2)?;
                    r.string()?;
                    r.take(8)?;
                }
                3 | 9 => {
                    r.take(6)?;
                }
                _ => (),
            }
            component.events = r.u24()? as i32;
            r.string()?; // opBase.
            let count = r.u8()?;
            for _ in 0..count {
                component.operations.push(r.string()?);
            }
            r.take(3)?;
            r.string()?; // Drag behavior and target verb.
            for _ in 0..18 {
                let count = r.u8()?;
                for _ in 0..count {
                    if r.u8()? == 0 {
                        r.take(4)?;
                    } else {
                        r.string()?;
                    }
                }
            }
            for _ in 0..3 {
                let count = r.u8()? as usize;
                r.take(count * 4)?;
            }
        } else {
            component.button_type = r.u8()?;
            r.take(11)?; // Content type, x/y, width/height, transparency.
            component.layer = parent(&mut r, id)?;
            r.take(2)?; // Mouse-over redirect.
            let count = r.u8()? as usize;
            r.take(count * 3)?;
            let count = r.u8()?;
            for _ in 0..count {
                let length = r.u16()? as usize;
                r.take(length * 2)?;
            }
            match component.kind {
                0 | 1 => {
                    r.take(3)?;
                }
                3 => {
                    r.take(1)?;
                }
                _ => (),
            }
            if component.kind == 4 || component.kind == 1 {
                r.take(6)?;
            }
            if component.kind == 4 {
                r.string()?;
                r.string()?;
            }
            if matches!(component.kind, 1 | 3 | 4) {
                r.take(4)?;
            }
            if matches!(component.kind, 3 | 4) {
                r.take(12)?;
            }
            if component.kind == 5 {
                r.take(8)?;
            }
            if component.kind == 6 {
                r.take(18)?;
            }
            if component.kind == 8 {
                r.string()?;
            }
            if component.button_type == 2 {
                r.string()?;
                r.string()?;
                let flags = (r.u16()? & 63) as i32;
                component.events = flags | (flags << 11);
            }
            if matches!(component.button_type, 1 | 4 | 5 | 6) {
                r.string()?;
            }
            if matches!(component.button_type, 1 | 4 | 5) {
                component.events |= 4194304;
            }
            if component.button_type == 6 {
                component.events |= 1;
            }
        }
        component.decoded_bytes = r.position();
        Ok(component)
    }
}
