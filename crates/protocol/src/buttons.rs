//! Revision 239 IF_BUTTONX and IF_BUTTOND. Item/slot presence is not authority;
//! handlers validate open interfaces and current inventory state separately.
use crate::buffer::{DecodeError, Order, Reader};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub component: u32,
    pub slot: i32,
    pub object: i32,
    pub op: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drag {
    pub source: u32,
    pub source_slot: i32,
    pub source_object: i32,
    pub target: u32,
    pub target_slot: i32,
    pub target_object: i32,
}
/// IF_BUTTONT (27), selected item/component onto a target component. This has
/// the same logical fields as a drag, but a distinct wire order and permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub source: u32,
    pub source_slot: i32,
    pub source_object: i32,
    pub target: u32,
    pub target_slot: i32,
    pub target_object: i32,
}
pub fn target(payload: &[u8]) -> Result<Target, DecodeError> {
    let mut r = Reader::new(payload);
    let target = r.u32()?;
    let source_slot = nullable(r.u16_add(false)?);
    let target_object = nullable(r.u16_add(true)?);
    let target_slot = nullable(r.u16_le()?);
    let source_object = nullable(r.u16_add(false)?);
    let source = r.u32_order(Order::Middle)?;
    r.finish()?;
    Ok(Target {
        source,
        source_slot,
        source_object,
        target,
        target_slot,
        target_object,
    })
}
fn nullable(value: u16) -> i32 {
    if value == u16::MAX {
        -1
    } else {
        value as i32
    }
}
pub fn button(payload: &[u8]) -> Result<Button, DecodeError> {
    let mut r = Reader::new(payload);
    let value = Button {
        component: r.u32()?,
        slot: nullable(r.u16()?),
        object: nullable(r.u16()?),
        op: r.u8()?,
    };
    r.finish()?;
    Ok(value)
}
pub fn drag(payload: &[u8]) -> Result<Drag, DecodeError> {
    let mut r = Reader::new(payload);
    let source = r.u32_order(Order::Little)?;
    let source_object = nullable(r.u16_le()?);
    let source_slot = nullable(r.u16_add(true)?);
    let target = r.u32()?;
    let target_object = nullable(r.u16_add(false)?);
    let target_slot = nullable(r.u16_add(false)?);
    r.finish()?;
    Ok(Drag {
        source,
        source_slot,
        source_object,
        target,
        target_slot,
        target_object,
    })
}
