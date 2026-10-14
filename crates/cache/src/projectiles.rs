//! Original ProjAnimTypeDecoder and required/default fields of ProjAnimTypeBuilder.
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::projectile::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectileType {
    pub style: Style,
    pub decoded_bytes: usize,
}
impl ProjectileType {
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut r = Reader::new(bytes);
        let (mut start, mut end, mut delay, mut angle) = (None, None, None, None);
        let (mut length_adjustment, mut progress, mut step_multiplier) = (0, 11, 5);
        while r.remaining() > 0 {
            match r.u8()? {
                0 => break,
                1 => start = Some(i32::from(r.u8()?)),
                2 => end = Some(i32::from(r.u8()?)),
                3 => delay = Some(i32::from(r.u8()?)),
                4 => angle = Some(i32::from(r.u8()?)),
                5 => length_adjustment = i32::from(r.u8()? as i8),
                6 => progress = i32::from(r.u8()?),
                7 => step_multiplier = i32::from(r.u8()?),
                8 => start = Some(i32::from(r.u16()?)),
                9 => end = Some(i32::from(r.u16()?)),
                _ => return Err(DecodeError::InvalidValue("Unknown projectile opcode")),
            }
        }
        Ok(Self {
            style: Style {
                start_height: start
                    .ok_or(DecodeError::InvalidValue("Missing projectile start height"))?,
                end_height: end
                    .ok_or(DecodeError::InvalidValue("Missing projectile end height"))?,
                delay: delay.ok_or(DecodeError::InvalidValue("Missing projectile delay"))?,
                angle: angle.ok_or(DecodeError::InvalidValue("Missing projectile angle"))?,
                length_adjustment,
                progress,
                step_multiplier,
            },
            decoded_bytes: r.position(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extended_heights_signed_adjustment_and_last_opcode_win() {
        let bytes = [
            1, 172, 2, 124, 3, 51, 4, 16, 5, 251, 8, 1, 44, 9, 255, 255, 6, 64, 0, 255,
        ];
        let value = ProjectileType::decode(&bytes).unwrap();
        assert_eq!(
            value.style,
            Style {
                start_height: 300,
                end_height: 65535,
                delay: 51,
                angle: 16,
                length_adjustment: -5,
                progress: 64,
                step_multiplier: 5
            }
        );
        assert_eq!(value.decoded_bytes, bytes.len() - 1);
        let defaults = ProjectileType::decode(&[1, 0, 2, 0, 3, 0, 4, 0]).unwrap();
        assert_eq!(
            (
                defaults.style.length_adjustment,
                defaults.style.progress,
                defaults.style.step_multiplier
            ),
            (0, 11, 5)
        );
    }
    #[test]
    fn truncated_fields_unknown_opcodes_and_missing_required_fields_fail() {
        let valid = [1, 172, 2, 124, 3, 51, 4, 16];
        for end in 0..valid.len() {
            assert!(ProjectileType::decode(&valid[..end]).is_err());
        }
        for suffix in [&[8, 1][..], &[9, 1], &[5], &[10], &[255]] {
            let mut bytes = valid.to_vec();
            bytes.extend(suffix);
            assert!(ProjectileType::decode(&bytes).is_err());
        }
    }
}
