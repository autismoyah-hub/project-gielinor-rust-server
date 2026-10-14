//! Revision 239 OpLocV2 input payloads. Suboperations are retained for dispatch.
use crate::buffer::{DecodeError, Reader, Transform};
/// OPLOCT (62), used for selected interface items and spells targeting a loc.
/// Nullable selected fields are retained; authority is checked by the UI/world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub id: u16,
    pub x: u16,
    pub z: u16,
    pub control: bool,
    pub source: u32,
    pub source_slot: i32,
    pub source_object: i32,
}
pub fn target(bytes: &[u8]) -> Result<Target, DecodeError> {
    let mut r = Reader::new(bytes);
    let control = r.transformed_u8(Transform::Negate)? == 1;
    let source_slot = r.u16_add(false)?;
    let id = r.u16_le()?;
    let source = r.u32_order(crate::buffer::Order::Little)?;
    let z = r.u16_add(true)?;
    let x = r.u16_le()?;
    let source_object = r.u16_le()?;
    r.finish()?;
    let nullable = |v: u16| if v == u16::MAX { -1 } else { v as i32 };
    Ok(Target {
        id,
        x,
        z,
        control,
        source,
        source_slot: nullable(source_slot),
        source_object: nullable(source_object),
    })
}
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
        73 => {
            let z = r.u16_add(false)?;
            let x = r.u16_le()?;
            let control = r.transformed_u8(Transform::Add128)? == 1;
            let suboperation = r.transformed_u8(Transform::Add128)?;
            let id = r.u16()?;
            Operation {
                id,
                x,
                z,
                operation: 1,
                suboperation,
                control,
            }
        }
        103 => {
            let z = r.u16_add(true)?;
            let id = r.u16_add(true)?;
            let x = r.u16()?;
            let suboperation = r.transformed_u8(Transform::Negate)?;
            let control = r.u8()? == 1;
            Operation {
                id,
                x,
                z,
                operation: 2,
                suboperation,
                control,
            }
        }
        104 => {
            let id = r.u16_add(false)?;
            let suboperation = r.transformed_u8(Transform::Sub128)?;
            let control = r.u8()? == 1;
            let z = r.u16_add(true)?;
            let x = r.u16_add(false)?;
            Operation {
                id,
                x,
                z,
                operation: 3,
                suboperation,
                control,
            }
        }
        15 => {
            let control = r.transformed_u8(Transform::Add128)? == 1;
            let x = r.u16_le()?;
            let id = r.u16_add(true)?;
            let suboperation = r.transformed_u8(Transform::Negate)?;
            let z = r.u16()?;
            Operation {
                id,
                x,
                z,
                operation: 4,
                suboperation,
                control,
            }
        }
        116 => {
            let z = r.u16_add(true)?;
            let x = r.u16_le()?;
            let control = r.transformed_u8(Transform::Add128)? == 1;
            let suboperation = r.transformed_u8(Transform::Negate)?;
            let id = r.u16()?;
            Operation {
                id,
                x,
                z,
                operation: 5,
                suboperation,
                control,
            }
        }
        _ => return Err(DecodeError::InvalidValue("not a location operation")),
    };
    r.finish()?;
    Ok(result)
}
