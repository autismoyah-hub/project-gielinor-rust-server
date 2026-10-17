//! Observer-specific revision 239.1 hitmark and headbar extension bodies.
use crate::buffer::DecodeError;

/// Supported NPC extension blocks in revision-specific writer order.
/// Bodies already contain the counts selected for this observer.
#[derive(Default)]
pub struct NpcUpdates<'a> {
    pub hitmarks: Option<&'a [u8]>,
    pub headbars: Option<&'a [u8]>,
    pub sequence: Option<(u16, u8)>,
    pub spot_animations: Option<&'a [u8]>,
    pub face: Option<&'a [u8]>,
    pub name: Option<&'a str>,
    pub combat: Option<u32>,
    pub transform: Option<u16>,
}
impl NpcUpdates<'_> {
    pub fn encode(&self) -> Vec<u8> {
        let mut mask = 0u32;
        let mut body = vec![];
        if let Some(level) = self.combat {
            mask |= 0x8000;
            body.extend(level.to_le_bytes());
        }
        if let Some(bytes) = self.hitmarks {
            mask |= 0x80000;
            body.extend(bytes);
        }
        if let Some(bytes) = self.headbars {
            mask |= 0x1000000;
            body.extend(bytes);
        }
        if let Some((id, delay)) = self.sequence {
            mask |= 128;
            body.extend(id.to_be_bytes());
            body.push(delay.wrapping_neg());
        }
        if let Some(spots) = self.spot_animations {
            mask |= 0x40000;
            body.extend(spots);
        }
        if let Some(name) = self.name {
            mask |= 0x4000;
            body.extend(name.as_bytes());
            body.push(0);
        }
        if let Some(id) = self.transform {
            mask |= 1;
            body.push((id as u8).wrapping_add(128));
            body.push((id >> 8) as u8);
        }
        if let Some(face) = self.face {
            mask |= 8;
            body.extend(face);
        }
        if mask == 0 {
            return vec![];
        }
        if mask > 255 {
            mask |= 64;
        }
        if mask > 65535 {
            mask |= 2048;
        }
        if mask > 16777215 {
            mask |= 2097152;
        }
        let mut out = vec![mask as u8];
        if mask & 64 != 0 {
            out.push((mask >> 8) as u8);
        }
        if mask & 2048 != 0 {
            out.push((mask >> 16) as u8);
        }
        if mask & 2097152 != 0 {
            out.push((mask >> 24) as u8);
        }
        out.extend(body);
        out
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Hitmark {
    pub source_index: i32,
    pub source_type: u16,
    pub self_type: u16,
    pub other_type: u16,
    pub value: u16,
    pub delay: u16,
    pub limit: u8,
}
#[derive(Clone, Copy, Debug)]
pub struct Headbar {
    pub source_index: i32,
    pub self_type: u16,
    pub other_type: u16,
    pub start_fill: u8,
    pub end_fill: u8,
    pub start_time: u16,
    pub end_time: u16,
}
fn smart(out: &mut Vec<u8>, value: u16) -> Result<(), DecodeError> {
    if value < 128 {
        out.push(value as u8);
    } else if value < 32768 {
        out.extend(value.wrapping_add(32768).to_be_bytes());
    } else {
        return Err(DecodeError::InvalidValue(
            "Combat smart value exceeds 32767",
        ));
    }
    Ok(())
}
pub fn hitmarks(
    npc: bool,
    observer: u16,
    target: u16,
    hits: &[Hitmark],
) -> Result<Vec<u8>, DecodeError> {
    let mut out = vec![0];
    let mut count = 0u8;
    for hit in hits {
        let source = observer as i32 == hit.source_index.wrapping_sub(65536);
        let kind = if npc {
            if source {
                hit.self_type
            } else {
                hit.other_type
            }
        } else if source {
            hit.source_type
        } else if observer == target {
            hit.self_type
        } else {
            hit.other_type
        };
        if kind == 65535 {
            continue;
        }
        for value in [kind, hit.value, hit.delay, hit.limit as u16] {
            smart(&mut out, value)?;
        }
        count += 1;
        if count == 255 {
            break;
        }
    }
    out[0] = if npc {
        count.wrapping_add(128)
    } else {
        128u8.wrapping_sub(count)
    };
    Ok(out)
}
pub fn headbars(
    npc: bool,
    observer: u16,
    target: u16,
    bars: &[Headbar],
) -> Result<Vec<u8>, DecodeError> {
    let mut out = vec![0];
    let mut count = 0u8;
    for bar in bars {
        let self_view =
            observer as i32 == bar.source_index.wrapping_sub(65536) || (!npc && observer == target);
        let kind = if self_view {
            bar.self_type
        } else {
            bar.other_type
        };
        if kind == 65535 {
            continue;
        }
        smart(&mut out, kind)?;
        smart(&mut out, bar.end_time)?;
        if bar.end_time != 32767 {
            smart(&mut out, bar.start_time)?;
            out.push(bar.start_fill.wrapping_add(128));
            if bar.end_time > 0 {
                out.push(if npc {
                    128u8.wrapping_sub(bar.end_fill)
                } else {
                    bar.end_fill.wrapping_neg()
                });
            }
        }
        count += 1;
        if count == 255 {
            break;
        }
    }
    out[0] = if npc {
        count.wrapping_neg()
    } else {
        count.wrapping_add(128)
    };
    Ok(out)
}

#[cfg(test)]
mod presentation_tests {
    use super::*;
    #[test]
    fn standard_npc_name_and_combat_masks_preserve_revision_writer_order() {
        let updates = NpcUpdates {
            name: Some("Ash Warden"),
            combat: Some(7),
            sequence: Some((4666, 0)),
            ..Default::default()
        };
        let bytes = updates.encode();
        // Extension marker + sequence, then name/combat mask byte. Decoder order
        // is combat LE, sequence BE + negated delay, then NUL-terminated name.
        assert_eq!(&bytes[..2], &[0xc0, 0xc0]);
        assert_eq!(&bytes[2..6], &7u32.to_le_bytes());
        assert_eq!(&bytes[6..9], &[0x12, 0x3a, 0]);
        assert_eq!(&bytes[9..], b"Ash Warden\0");
        assert!(NpcUpdates::default().encode().is_empty());
        assert_eq!(
            NpcUpdates {
                transform: Some(2075),
                ..Default::default()
            }
            .encode(),
            vec![1, 155, 8]
        );
    }
}
