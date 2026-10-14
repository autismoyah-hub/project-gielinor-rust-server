//! Revision239 PlayerSpotAnimEncoder: ascending slots, last write per slot.
use crate::buffer::DecodeError;
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Animation {
    pub slot: u8,
    pub id: u16,
    pub delay: u16,
    pub height: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn third_mask_byte_and_overwritten_slots_have_one_ordered_body() {
        let changes = [
            Animation {
                slot: 7,
                id: 99,
                delay: 1,
                height: 2,
            },
            Animation {
                slot: 1,
                id: 256,
                delay: 3,
                height: 2,
            },
            Animation {
                slot: 7,
                id: 1336,
                delay: 0,
                height: 0,
            },
        ];
        let updates = crate::appearance::Updates {
            spot_animations: Some(&changes),
            ..Default::default()
        };
        assert_eq!(
            updates.encode().unwrap(),
            [8, 8, 2, 2, 255, 1, 128, 0, 2, 0, 3, 249, 5, 184, 0, 0, 0, 0]
        );
        assert!(
            crate::appearance::Updates {
                spot_animations: Some(&[]),
                ..Default::default()
            }
            .encode()
            .unwrap()
            .is_empty()
        );
    }
}
pub fn encode(changes: &[Animation]) -> Result<Vec<u8>, DecodeError> {
    let slots: BTreeMap<_, _> = changes.iter().map(|a| (a.slot, a)).collect();
    let count =
        u8::try_from(slots.len()).map_err(|_| DecodeError::InvalidValue("spot animation count"))?;
    let mut bytes = vec![count];
    for (slot, animation) in slots {
        bytes.push(slot.wrapping_neg());
        let id = animation.id.to_be_bytes();
        bytes.extend([id[0], id[1].wrapping_add(128)]);
        bytes.extend(animation.height.to_be_bytes());
        bytes.extend(animation.delay.to_be_bytes());
    }
    Ok(bytes)
}

/// NpcSpotAnimEncoder uses a negated count, plain slots/ids, then delay and height.
pub fn encode_npc(changes: &[Animation]) -> Result<Vec<u8>, DecodeError> {
    let slots: BTreeMap<_, _> = changes.iter().map(|a| (a.slot, a)).collect();
    let count = u8::try_from(slots.len())
        .map_err(|_| DecodeError::InvalidValue("NPC spot animation count"))?;
    let mut bytes = vec![count.wrapping_neg()];
    for (slot, animation) in slots {
        bytes.push(slot);
        bytes.extend(animation.id.to_be_bytes());
        bytes.extend(animation.delay.to_be_bytes());
        bytes.extend(animation.height.to_be_bytes());
    }
    Ok(bytes)
}
