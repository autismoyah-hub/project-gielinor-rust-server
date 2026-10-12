use crate::buffer::{DecodeError, Reader, Transform};
pub fn synth_sound(id: u16, loops: u8, delay: u16) -> crate::framing::Packet {
    let mut payload = Vec::with_capacity(5);
    payload.extend(id.to_be_bytes());
    payload.push(loops);
    payload.extend(delay.to_be_bytes());
    crate::framing::Packet {
        opcode: 77,
        payload,
    }
}
pub fn clear_map_flag() -> crate::framing::Packet {
    crate::framing::Packet {
        opcode: 143,
        payload: vec![255; 4],
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowStatus {
    pub mode: u8,
    pub width: u16,
    pub height: u16,
}
pub fn window_status(payload: &[u8]) -> Result<WindowStatus, DecodeError> {
    if payload.len() != 5 {
        return Err(DecodeError::InvalidValue("window status length"));
    }
    let mut r = Reader::new(payload);
    Ok(WindowStatus {
        mode: r.u8()?,
        width: r.u16()?,
        height: r.u16()?,
    })
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveClick {
    pub x: u16,
    pub z: u16,
    pub keys: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimapClick {
    pub movement: MoveClick,
    pub click_x: u8,
    pub click_z: u8,
    pub angle: u16,
    pub fine_x: u16,
    pub fine_z: u16,
}
fn click(r: &mut Reader<'_>) -> Result<MoveClick, DecodeError> {
    Ok(MoveClick {
        x: r.u16_le()?,
        z: r.u16_le()?,
        keys: r.transformed_u8(Transform::Sub128)?,
    })
}
pub fn move_game_click(payload: &[u8]) -> Result<MoveClick, DecodeError> {
    let mut r = Reader::new(payload);
    let movement = click(&mut r)?;
    r.finish()?;
    Ok(movement)
}
pub fn move_minimap_click(payload: &[u8]) -> Result<MinimapClick, DecodeError> {
    let mut r = Reader::new(payload);
    let movement = click(&mut r)?;
    let click_x = r.u8()?;
    let click_z = r.u8()?;
    let angle = r.u16()?;
    if r.take(4)? != [57, 0, 0, 89] {
        return Err(DecodeError::InvalidValue("minimap verification"));
    }
    let fine_x = r.u16()?;
    let fine_z = r.u16()?;
    if r.u8()? != 63 {
        return Err(DecodeError::InvalidValue("minimap trailer"));
    }
    r.finish()?;
    Ok(MinimapClick {
        movement,
        click_x,
        click_z,
        angle,
        fine_x,
        fine_z,
    })
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InventoryItem {
    pub id: u16,
    pub count: u32,
}
fn validate_item(item: InventoryItem) -> Result<(), DecodeError> {
    if item.id == u16::MAX || item.count > i32::MAX as u32 {
        return Err(DecodeError::InvalidValue("inventory object"));
    }
    Ok(())
}
pub fn run_energy(value: i32) -> Result<[u8; 2], DecodeError> {
    u16::try_from(value)
        .map(u16::to_be_bytes)
        .map_err(|_| DecodeError::InvalidValue("run energy"))
}
pub fn run_weight(kilograms: i32) -> [u8; 2] {
    (kilograms as i16).to_be_bytes()
}
pub fn inventory_stop(id: u16) -> [u8; 2] {
    let [high, low] = id.to_be_bytes();
    [high, low.wrapping_add(128)]
}
pub fn inventory_full(
    component: i32,
    inventory: u16,
    items: &[Option<InventoryItem>],
) -> Result<Vec<u8>, DecodeError> {
    let capacity =
        u16::try_from(items.len()).map_err(|_| DecodeError::InvalidValue("inventory capacity"))?;
    let mut out = component.to_be_bytes().to_vec();
    out.extend(inventory.to_be_bytes());
    out.extend(capacity.to_be_bytes());
    for item in items {
        if let Some(item) = item {
            validate_item(*item)?;
            out.push(128u8.wrapping_sub(item.count.min(255) as u8));
            if item.count >= 255 {
                let b = item.count.to_be_bytes();
                out.extend([b[1], b[0], b[3], b[2]]);
            }
            out.extend((item.id + 1).to_le_bytes());
        } else {
            out.extend([128, 0, 0]);
        }
    }
    Ok(out)
}
pub fn inventory_partial(
    component: i32,
    inventory: u16,
    items: &[(u16, Option<InventoryItem>)],
) -> Result<Vec<u8>, DecodeError> {
    let mut out = component.to_be_bytes().to_vec();
    out.extend(inventory.to_be_bytes());
    for (slot, item) in items {
        if *slot >= 32768 {
            return Err(DecodeError::InvalidValue("inventory slot"));
        }
        if *slot < 128 {
            out.push(*slot as u8)
        } else {
            out.extend((slot + 32768).to_be_bytes())
        }
        if let Some(item) = item {
            validate_item(*item)?;
            out.extend((item.id + 1).to_be_bytes());
            out.push(item.count.min(255) as u8);
            if item.count >= 255 {
                out.extend(item.count.to_be_bytes());
            }
        } else {
            out.extend([0, 0]);
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minimap_validates_sentinels_and_trailing_bytes() {
        let mut payload = vec![
            0x80, 0x0c, 0x81, 0x0c, 127, 5, 6, 0, 7, 57, 0, 0, 89, 0x10, 0x20, 0x30, 0x40, 63,
        ];
        assert_eq!(
            move_minimap_click(&payload).unwrap().movement,
            MoveClick {
                x: 3200,
                z: 3201,
                keys: 1
            }
        );
        payload[9] = 56;
        assert!(move_minimap_click(&payload).is_err());
    }
}
