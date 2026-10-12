//! Revision 239 ground-object operations (distinct from map location operations).
use crate::buffer::{DecodeError, Reader, Transform};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Operation {
    pub id: u16,
    pub x: u16,
    pub z: u16,
    pub operation: u8,
    pub suboperation: u8,
    pub control: bool,
}
pub fn decode(opcode: u16, bytes: &[u8]) -> Result<Operation, DecodeError> {
    let mut r = Reader::new(bytes);
    let result = match opcode {
        46 => {
            let id = r.u16_add(false)?;
            let suboperation = r.transformed_u8(Transform::Sub128)?;
            let z = r.u16_add(false)?;
            let control = r.transformed_u8(Transform::Sub128)? == 1;
            let x = r.u16()?;
            Operation {
                id,
                x,
                z,
                operation: 1,
                suboperation,
                control,
            }
        }
        74 => {
            let control = r.transformed_u8(Transform::Add128)? == 1;
            let suboperation = r.transformed_u8(Transform::Sub128)?;
            let z = r.u16()?;
            let id = r.u16_le()?;
            let x = r.u16_add(true)?;
            Operation {
                id,
                x,
                z,
                operation: 2,
                suboperation,
                control,
            }
        }
        7 => {
            let control = r.transformed_u8(Transform::Add128)? == 1;
            let x = r.u16_le()?;
            let id = r.u16_add(true)?;
            let suboperation = r.transformed_u8(Transform::Add128)?;
            let z = r.u16_add(false)?;
            Operation {
                id,
                x,
                z,
                operation: 3,
                suboperation,
                control,
            }
        }
        31 => {
            let suboperation = r.transformed_u8(Transform::Sub128)?;
            let id = r.u16()?;
            let x = r.u16()?;
            let z = r.u16_le()?;
            let control = r.transformed_u8(Transform::Negate)? == 1;
            Operation {
                id,
                x,
                z,
                operation: 4,
                suboperation,
                control,
            }
        }
        43 => {
            let suboperation = r.transformed_u8(Transform::Negate)?;
            let x = r.u16_add(false)?;
            let id = r.u16()?;
            let z = r.u16_add(true)?;
            let control = r.transformed_u8(Transform::Add128)? == 1;
            Operation {
                id,
                x,
                z,
                operation: 5,
                suboperation,
                control,
            }
        }
        _ => return Err(DecodeError::InvalidValue("not a ground object operation")),
    };
    r.finish()?;
    Ok(result)
}
