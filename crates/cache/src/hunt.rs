//! Original HuntModeTypeDecoder (config archive group 2) and HuntModeTypeBuilder defaults.
use gielinor_protocol::buffer::{DecodeError, Reader};
use gielinor_world::hunt::{InvCondition, Kind, Mode, Operator, VarCondition, Vis};

fn nullable(r: &mut Reader<'_>) -> Result<Option<u16>, DecodeError> {
    let value = r.u16()?;
    Ok((value != 65535).then_some(value))
}
fn operator(r: &mut Reader<'_>) -> Result<Operator, DecodeError> {
    Operator::from_id(r.u8()?).ok_or(DecodeError::InvalidValue("Invalid hunt operator"))
}
pub fn decode(bytes: &[u8]) -> Result<Mode, DecodeError> {
    let mut mode = Mode::default();
    let mut r = Reader::new(bytes);
    while r.remaining() > 0 {
        match r.u8()? {
            0 => break,
            1 => {
                mode.kind = match r.u8()? {
                    0 => Kind::Off,
                    1 => Kind::Player,
                    2 => Kind::Npc,
                    3 => Kind::Obj,
                    4 => Kind::Scenery,
                    _ => return Err(DecodeError::InvalidValue("Invalid HuntType")),
                }
            }
            2 => {
                mode.vis = match r.u8()? {
                    0 => Vis::Off,
                    1 => Vis::LineOfSight,
                    2 => Vis::LineOfWalk,
                    _ => return Err(DecodeError::InvalidValue("Invalid HuntVis")),
                }
            }
            3 => {
                mode.not_too_strong = match r.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(DecodeError::InvalidValue("Invalid NotTooStrong")),
                }
            }
            4 => mode.not_busy = true,
            5 => mode.keep_hunting = true,
            6 => mode.find_new_mode = r.u8()?,
            7 => {
                mode.pause_when_nobody_near = match r.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(DecodeError::InvalidValue("Invalid NobodyNear")),
                }
            }
            8 => mode.not_combat = Some(r.u16()?),
            9 => mode.not_combat_self = Some(r.u16()?),
            10 => mode.check_afk = false,
            11 => mode.rate = i32::from(r.u16()?),
            12..=14 => {
                nullable(&mut r)?;
                nullable(&mut r)?;
                mode.has_target_filter = true;
            }
            code @ (15 | 16) => {
                let condition = InvCondition {
                    inv: r.u16()?,
                    key: r.u16()?,
                    operator: operator(&mut r)?,
                    required: r.u32()? as i32,
                };
                if code == 15 {
                    mode.inv_obj = Some(condition);
                } else {
                    mode.inv_param = Some(condition);
                }
            }
            code @ 17..=19 => {
                mode.vars[usize::from(code - 17)] = Some(VarCondition {
                    varp: r.u16()?,
                    operator: operator(&mut r)?,
                    required: r.u32()? as i32,
                });
            }
            _ => return Err(DecodeError::InvalidValue("Unknown hunt opcode")),
        }
    }
    Ok(mode)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builder_defaults_and_flag_opcodes() {
        assert_eq!(decode(&[0]).unwrap(), Mode::default());
        let mode = decode(&[1, 1, 2, 1, 3, 1, 4, 5, 6, 8, 7, 1, 8, 0, 43, 10, 11, 0, 3, 0]).unwrap();
        assert_eq!(mode.kind, Kind::Player);
        assert_eq!(mode.vis, Vis::LineOfSight);
        assert!(mode.not_too_strong && mode.not_busy && mode.keep_hunting);
        assert!(mode.pause_when_nobody_near && !mode.check_afk);
        assert_eq!((mode.find_new_mode, mode.not_combat, mode.rate), (8, Some(43), 3));
        let vars = decode(&[18, 0, 9, 2, 0, 0, 0, 5, 0]).unwrap();
        assert_eq!(
            vars.vars[1],
            Some(VarCondition {
                varp: 9,
                operator: Operator::Equals,
                required: 5
            })
        );
        assert!(decode(&[1, 9]).is_err());
        assert!(decode(&[99]).is_err());
    }
}
