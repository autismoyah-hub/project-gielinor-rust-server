//! Revision 239.1 NPC operations, preserving the V2 suboperation byte.
use crate::buffer::{DecodeError, Reader, Transform};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Operation {
    pub index: u16,
    pub operation: u8,
    pub suboperation: u8,
    pub control: bool,
}
pub fn decode(opcode: u16, bytes: &[u8]) -> Result<Operation, DecodeError> {
    let mut r = Reader::new(bytes);
    let (index, operation, suboperation, control) = match opcode {
        102 | 13 => {
            let index = if opcode == 102 {
                r.u16_le()?
            } else {
                r.u16_add(true)?
            };
            let sub = r.transformed_u8(Transform::Negate)?;
            let control = r.transformed_u8(Transform::Sub128)? == 1;
            (index, if opcode == 102 { 1 } else { 2 }, sub, control)
        }
        19 => {
            let control = r.transformed_u8(Transform::Negate)? == 1;
            let sub = r.transformed_u8(Transform::Sub128)?;
            (r.u16_add(false)?, 3, sub, control)
        }
        45 => {
            let sub = r.transformed_u8(Transform::Add128)?;
            let index = r.u16()?;
            let control = r.transformed_u8(Transform::Negate)? == 1;
            (index, 4, sub, control)
        }
        85 => {
            let sub = r.transformed_u8(Transform::Add128)?;
            let control = r.transformed_u8(Transform::Negate)? == 1;
            (r.u16_add(false)?, 5, sub, control)
        }
        _ => return Err(DecodeError::InvalidValue("NPC operation opcode")),
    };
    r.finish()?;
    Ok(Operation {
        index,
        operation,
        suboperation,
        control,
    })
}
