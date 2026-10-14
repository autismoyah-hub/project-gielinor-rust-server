use crate::{buffer::DecodeError, framing::Packet};
/// Revision 239 dialogue heads and expressions (RSProt interface encoders).
pub fn npc_head(component: u32, npc: u16) -> Packet {
    let mut payload = component.to_be_bytes().to_vec();
    let [hi, lo] = npc.to_be_bytes();
    payload.extend([lo.wrapping_add(128), hi]);
    Packet {
        opcode: 84,
        payload,
    }
}
pub fn player_head(component: u32) -> Packet {
    let [a, b, c, d] = component.to_be_bytes();
    Packet {
        opcode: 120,
        payload: vec![c, d, a, b],
    }
}
pub fn animation(component: u32, sequence: Option<u16>) -> Packet {
    let mut payload = component.to_le_bytes().to_vec();
    payload.extend(sequence.unwrap_or(u16::MAX).to_le_bytes());
    Packet {
        opcode: 59,
        payload,
    }
}
pub fn object(component: u32, id: u16, zoom_or_count: i32) -> Packet {
    let [a, b, c, d] = component.to_be_bytes();
    let [hi, lo] = id.to_be_bytes();
    let mut payload = vec![c, d, a, b, hi, lo.wrapping_add(128)];
    payload.extend(zoom_or_count.to_be_bytes());
    Packet {
        opcode: 106,
        payload,
    }
}
pub fn text(component: u32, text: &str) -> Result<Packet, DecodeError> {
    let mut payload = Vec::new();
    crate::appearance::string(&mut payload, text)?;
    let [a, b, c, d] = component.to_be_bytes();
    payload.extend([c, d, a, b]);
    if payload.len() > u16::MAX as usize {
        return Err(DecodeError::InvalidValue("interface text length"));
    }
    Ok(Packet {
        opcode: 80,
        payload,
    })
}
/// IF_SETHIDE: combined-id alt3 followed by the boolean byte plus128.
pub fn hide(component: u32, hidden: bool) -> Packet {
    let [a, b, c, d] = component.to_be_bytes();
    Packet {
        opcode: 63,
        payload: vec![b, a, d, c, 128 + hidden as u8],
    }
}
pub fn open_top(interface: u16) -> Packet {
    let [a, b] = interface.to_be_bytes();
    Packet {
        opcode: 96,
        payload: vec![a, b.wrapping_add(128)],
    }
}
pub fn open_sub(parent: u32, interface: u16, overlay: bool) -> Packet {
    let [a, b] = interface.to_be_bytes();
    let [p, q, r, s] = parent.to_be_bytes();
    Packet {
        opcode: 7,
        payload: vec![b.wrapping_add(128), a, q, p, s, r, overlay as u8],
    }
}
pub fn close_sub(parent: u32) -> Packet {
    Packet {
        opcode: 23,
        payload: parent.to_be_bytes().to_vec(),
    }
}
pub fn move_sub(source: u32, destination: u32) -> Packet {
    let mut payload = destination.to_le_bytes().to_vec();
    payload.extend(source.to_be_bytes());
    Packet {
        opcode: 123,
        payload,
    }
}
pub fn events(component: u32, start: i32, end: i32, mask: u64) -> Result<Packet, DecodeError> {
    if !(-1..=65534).contains(&start) || !(-1..=65534).contains(&end) {
        return Err(DecodeError::InvalidValue("interface event slot"));
    }
    let [a, b] = (end as u16).to_be_bytes();
    let [p, q, r, s] = (mask as u32).to_be_bytes();
    let [w, x, y, z] = component.to_be_bytes();
    let mut payload = vec![a, b.wrapping_add(128)];
    payload.extend(((mask >> 32) as u32).to_be_bytes());
    payload.extend([r, s, p, q]);
    payload.extend((start as u16).to_be_bytes());
    payload.extend([x, w, z, y]);
    Ok(Packet {
        opcode: 108,
        payload,
    })
}
