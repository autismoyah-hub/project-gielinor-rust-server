use crate::{appearance::string, buffer::DecodeError, framing::Packet};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Int(i32),
    String(String),
    Long(i64),
    IntArray(Vec<i32>),
    StringArray(Vec<String>),
}
fn varint(out: &mut Vec<u8>, mut value: u32) {
    while value > 127 {
        out.push(value as u8 | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
pub fn run(id: i32, args: &[Argument]) -> Result<Packet, DecodeError> {
    if args.len() > 65530 {
        return Err(DecodeError::InvalidValue("clientscript argument count"));
    }
    let mut payload: Vec<_> = args
        .iter()
        .map(|arg| match arg {
            Argument::Int(_) => b'i',
            Argument::String(_) => b's',
            Argument::Long(_) => 207,
            Argument::IntArray(_) => b'W',
            Argument::StringArray(_) => b'X',
        })
        .collect();
    payload.push(0);
    for arg in args.iter().rev() {
        match arg {
            Argument::Int(value) => payload.extend(value.to_be_bytes()),
            Argument::Long(value) => payload.extend(value.to_be_bytes()),
            Argument::String(value) => string(&mut payload, value)?,
            Argument::IntArray(values) => {
                if values.len() > 65535 {
                    return Err(DecodeError::InvalidValue("clientscript array length"));
                }
                varint(&mut payload, values.len() as u32);
                for &value in values {
                    varint(&mut payload, ((value as u32) << 1) ^ ((value >> 31) as u32));
                }
            }
            Argument::StringArray(values) => {
                if values.len() > 65535 {
                    return Err(DecodeError::InvalidValue("clientscript array length"));
                }
                varint(&mut payload, values.len() as u32);
                for value in values {
                    string(&mut payload, value)?;
                }
            }
        }
        if payload.len() > 65531 {
            return Err(DecodeError::InvalidValue("clientscript payload length"));
        }
    }
    payload.extend(id.to_be_bytes());
    Ok(Packet {
        opcode: 114,
        payload,
    })
}
