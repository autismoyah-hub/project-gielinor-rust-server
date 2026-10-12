use crate::buffer::{DecodeError, Reader};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pause {
    pub component: u32,
    pub subcomponent: i32,
}
pub fn pause(payload: &[u8]) -> Result<Pause, DecodeError> {
    let mut reader = Reader::new(payload);
    let component = reader.u32_order(crate::buffer::Order::InverseMiddle)?;
    let sub = reader.u16()?;
    let value = Pause {
        component,
        subcomponent: if sub == u16::MAX { -1 } else { sub as i32 },
    };
    reader.finish()?;
    Ok(value)
}

/// RESUME_P_COUNTDIALOG carries a signed, untransformed big-endian integer.
pub fn count(payload: &[u8]) -> Result<i32, DecodeError> {
    let mut reader = Reader::new(payload);
    let value = reader.u32()? as i32;
    reader.finish()?;
    Ok(value)
}
