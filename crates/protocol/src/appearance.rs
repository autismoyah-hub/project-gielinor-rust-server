//! Revision 239 appearance payload and common player extension blocks.
use crate::buffer::{decode_cp1252, DecodeError};

pub fn string(out: &mut Vec<u8>, text: &str) -> Result<(), DecodeError> {
    if text.contains('\0') {
        return Err(DecodeError::InvalidValue("embedded string terminator"));
    }
    for ch in text.chars() {
        let value = ch as u32;
        let byte = if (1..=127).contains(&value) || (160..=255).contains(&value) {
            value as u8
        } else {
            (128..=159)
                .find(|b| decode_cp1252(*b) == ch)
                .unwrap_or(b'?')
        };
        out.push(byte);
    }
    out.push(0);
    Ok(())
}
#[derive(Clone, Debug)]
pub struct Appearance {
    pub body_type: u8,
    pub skull: i8,
    pub overhead: i8,
    /// Prepacked values: zero, identikit + 256, or item + 2048.
    pub equipment: [u16; 12],
    pub base_equipment: [u16; 12],
    pub npc: Option<u16>,
    pub colours: [u8; 5],
    pub animations: [u16; 7],
    pub name: String,
    pub combat: u8,
    pub skill: u16,
    pub hidden: bool,
    pub force_refresh: bool,
    pub name_extras: [String; 3],
    pub pronoun: u8,
}
fn look(out: &mut Vec<u8>, value: u16) -> Result<(), DecodeError> {
    if value == 0 {
        out.push(0);
    } else if value < 256 || value == 65535 {
        return Err(DecodeError::InvalidValue("appearance equipment"));
    } else {
        out.extend(value.to_be_bytes());
    }
    Ok(())
}
impl Appearance {
    pub fn encode(&self) -> Result<Vec<u8>, DecodeError> {
        let mut out = vec![self.body_type, self.skull as u8, self.overhead as u8];
        if let Some(npc) = self.npc {
            out.extend([255, 255]);
            out.extend(npc.to_be_bytes());
        } else {
            for value in self.equipment {
                look(&mut out, value)?;
            }
        }
        for value in self.base_equipment {
            look(&mut out, value)?;
        }
        out.extend(self.colours);
        for value in self.animations {
            out.extend(value.to_be_bytes());
        }
        string(&mut out, &self.name)?;
        out.push(self.combat);
        out.extend(self.skill.to_be_bytes());
        out.push(self.hidden as u8);
        // Object recolour/retexture/model override blocks are a separate forthcoming extension.
        out.extend((if self.force_refresh { 32768u16 } else { 0 }).to_be_bytes());
        for text in &self.name_extras {
            string(&mut out, text)?;
        }
        out.push(self.pronoun);
        if out.len() > 255 {
            return Err(DecodeError::InvalidValue("appearance exceeds byte length"));
        }
        Ok(out)
    }
}

/// Player exact-movement block; offsets are relative to the synchronized landing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExactMove {
    pub offsets: [i8; 4],
    pub delays: [u16; 2],
    pub direction: u16,
}
impl ExactMove {
    fn encode(self) -> Result<Vec<u8>, DecodeError> {
        if self.delays[0] > self.delays[1] || self.direction >= 2048 {
            return Err(DecodeError::InvalidValue(
                "exact movement timing or direction",
            ));
        }
        let [a, b, c, d] = self.offsets;
        let mut out = vec![
            a as u8,
            (b as u8).wrapping_add(128),
            (c as u8).wrapping_neg(),
            128u8.wrapping_sub(d as u8),
        ];
        out.extend(self.delays[0].to_be_bytes());
        out.extend(self.delays[1].to_le_bytes());
        out.extend([
            (self.direction as u8).wrapping_add(128),
            (self.direction >> 8) as u8,
        ]);
        Ok(out)
    }
}

#[derive(Default)]
pub struct Updates<'a> {
    /// Encoded observer-specific extension bodies (including each list count).
    pub hitmarks: Option<&'a [u8]>,
    pub headbars: Option<&'a [u8]>,
    pub reset: Option<u8>,
    pub speed: Option<i8>,
    pub spot_animations: Option<&'a [crate::spot_animation::Animation]>,
    pub face: Option<Face>,
    pub say: Option<&'a str>,
    pub sequence: Option<(u16, u8)>,
    pub exact_move: Option<ExactMove>,
    pub temporary_speed: Option<i8>,
    pub appearance: Option<&'a [u8]>,
}
impl Updates<'_> {
    pub fn encode(&self) -> Result<Vec<u8>, DecodeError> {
        let mut mask = 0u32;
        let mut body = vec![];
        if let Some(bytes) = self.hitmarks {
            mask |= 0x40000;
            body.extend(bytes);
        }
        if let Some(value) = self.reset {
            mask |= 2;
            body.push(value.wrapping_neg());
        }
        if let Some(value) = self.face {
            mask |= 1;
            body.extend(value.encode()?);
        }
        if let Some(value) = self.speed {
            mask |= 0x400;
            body.push((value as u8).wrapping_neg());
        }
        if let Some(spots) = self.spot_animations.filter(|spots| !spots.is_empty()) {
            mask |= 0x20000;
            body.extend(crate::spot_animation::encode(spots)?);
        }
        if let Some(bytes) = self.headbars {
            mask |= 0x10000;
            body.extend(bytes);
        }
        if let Some(text) = self.say {
            mask |= 4;
            string(&mut body, text)?;
        }
        if let Some((id, delay)) = self.sequence {
            mask |= 64;
            let b = id.to_be_bytes();
            body.extend([b[0], b[1].wrapping_add(128), delay]);
        }
        if let Some(value) = self.temporary_speed {
            mask |= 0x1000;
            body.push(value as u8);
        }
        if let Some(bytes) = self.appearance {
            let len = u8::try_from(bytes.len())
                .map_err(|_| DecodeError::InvalidValue("appearance length"))?;
            mask |= 32;
            body.push(128u8.wrapping_sub(len));
            body.extend(bytes.iter().map(|b| b.wrapping_add(128)));
        }
        if let Some(exact) = self.exact_move {
            mask |= 0x4000;
            body.extend(exact.encode()?);
        }
        if mask == 0 {
            return Ok(vec![]);
        }
        if mask > 255 {
            mask |= 8;
        }
        if mask > 65535 {
            mask |= 0x800;
        }
        let mut out = vec![mask as u8];
        if mask & 8 != 0 {
            out.push((mask >> 8) as u8);
        }
        if mask & 0x800 != 0 {
            out.push((mask >> 16) as u8);
        }
        out.extend(body);
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_flags_and_transforms() {
        let updates = Updates {
            speed: Some(1),
            appearance: Some(&[0, 255, 128]),
            ..Default::default()
        };
        assert_eq!(updates.encode().unwrap(), [40, 4, 255, 125, 128, 127, 0]);
        assert!(Face {
            walk_mode: 8,
            instant: false,
            target: FaceTarget::Reset
        }
        .encode()
        .is_err());
    }
    #[test]
    fn player_exact_move_wire_matches_target_offsets_delays_direction_and_rejects_invalid_controls()
    {
        let e = ExactMove {
            offsets: [-1, 2, 0, 0],
            delays: [30, 94],
            direction: 1536,
        };
        assert_eq!(
            Updates {
                exact_move: Some(e),
                ..Default::default()
            }
            .encode()
            .unwrap(),
            [8, 64, 255, 130, 0, 128, 0, 30, 94, 0, 128, 6]
        );
        for invalid in [
            ExactMove {
                delays: [95, 94],
                ..e
            },
            ExactMove {
                direction: 2048,
                ..e
            },
        ] {
            assert!(Updates {
                exact_move: Some(invalid),
                ..Default::default()
            }
            .encode()
            .is_err());
        }
        let combined = Updates {
            sequence: Some((839, 30)),
            appearance: Some(&[0]),
            exact_move: Some(e),
            ..Default::default()
        }
        .encode()
        .unwrap();
        assert_eq!(&combined[..8], &[104, 64, 3, 199, 30, 127, 128, 255]);
        assert_eq!(&combined[7..], &[255, 130, 0, 128, 0, 30, 94, 0, 128, 6]);
    }

    #[test]
    fn cp1252_strings_reject_embedded_terminators() {
        let mut out = vec![];
        string(&mut out, "€ßŸ").unwrap();
        assert_eq!(out, [128, 223, 159, 0]);
        assert!(string(&mut out, "bad\0name").is_err());
    }
}

#[derive(Clone, Copy, Debug)]
pub enum FaceTarget {
    Entity {
        entity_type: u16,
        index: Option<u32>,
        fallback_angle: u16,
    },
    Location {
        x: u16,
        z: u16,
        width: u8,
        length: u8,
    },
    Angle(u16),
    Reset,
}
#[derive(Clone, Copy, Debug)]
pub struct Face {
    pub walk_mode: u8,
    pub instant: bool,
    pub target: FaceTarget,
}
fn smart(out: &mut Vec<u8>, value: u16) -> Result<(), DecodeError> {
    if value >= 32768 {
        return Err(DecodeError::InvalidValue("facing smart value"));
    }
    if value < 128 {
        out.push(value as u8);
    } else {
        out.extend((value + 32768).to_be_bytes());
    }
    Ok(())
}
impl Face {
    pub fn encode_npc(self) -> Result<Vec<u8>, DecodeError> {
        let mut bytes = self.encode()?;
        // NPC flags use p1Alt1 (+128); players use p1Alt2 (negation).
        bytes[0] = bytes[0].wrapping_neg().wrapping_add(128);
        Ok(bytes)
    }
    pub fn encode(self) -> Result<Vec<u8>, DecodeError> {
        if self.walk_mode > 7 {
            return Err(DecodeError::InvalidValue("facing walk mode"));
        }
        let kind = match self.target {
            FaceTarget::Entity { .. } => 0,
            FaceTarget::Location { .. } => 1,
            FaceTarget::Angle(_) => 2,
            FaceTarget::Reset => 3,
        };
        let mut out =
            vec![(self.walk_mode | (kind << 3) | if self.instant { 64 } else { 0 }).wrapping_neg()];
        match self.target {
            FaceTarget::Reset => (),
            FaceTarget::Angle(angle) => smart(&mut out, angle)?,
            FaceTarget::Entity {
                entity_type,
                index,
                fallback_angle,
            } => {
                smart(&mut out, entity_type)?;
                match index {
                    None => out.extend(32767u16.to_be_bytes()),
                    Some(i) if i < 32767 => out.extend((i as u16).to_be_bytes()),
                    Some(i) if i <= i32::MAX as u32 => out.extend((i | 0x80000000).to_be_bytes()),
                    _ => return Err(DecodeError::InvalidValue("facing entity index")),
                }
                smart(&mut out, fallback_angle)?;
            }
            FaceTarget::Location {
                x,
                z,
                width,
                length,
            } => {
                if width > 15 || length > 15 {
                    return Err(DecodeError::InvalidValue("facing location size"));
                }
                smart(&mut out, x)?;
                smart(&mut out, z)?;
                smart(&mut out, (length as u16) << 4 | width as u16)?;
            }
        }
        Ok(out)
    }
}
