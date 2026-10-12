use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::variables::{Transmit, Varbit, Varp};
pub fn varp(id: u16, bytes: &[u8]) -> Result<Varp, DecodeError> {
    let mut r = Reader::new(bytes);
    let mut d = Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    while r.remaining() > 0 {
        match r.u8()? {
            0 => break,
            5 => d.client_code = r.u16()? as i32,
            202 => d.bit_protect = true,
            203 => {
                d.permanent = match r.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(DecodeError::InvalidValue("varp lifetime")),
                }
            }
            204 => {
                d.transmit = match r.u8()? {
                    0 => Transmit::Never,
                    1 => Transmit::Different,
                    2 => Transmit::Always,
                    _ => return Err(DecodeError::InvalidValue("varp transmit")),
                }
            }
            _ => return Err(DecodeError::InvalidValue("unknown varp opcode")),
        }
    }
    Ok(d)
}
pub fn varbit(bytes: &[u8]) -> Result<Varbit, DecodeError> {
    let mut r = Reader::new(bytes);
    let mut definition = None;
    while r.remaining() > 0 {
        match r.u8()? {
            0 => break,
            1 => {
                let d = Varbit {
                    base: r.u16()?,
                    lsb: r.u8()?,
                    msb: r.u8()?,
                };
                if d.lsb > d.msb || d.msb > 31 {
                    return Err(DecodeError::InvalidValue("varbit bit range"));
                }
                definition = Some(d);
            }
            _ => return Err(DecodeError::InvalidValue("unknown varbit opcode")),
        }
    }
    definition.ok_or(DecodeError::InvalidValue("missing varbit definition"))
}
