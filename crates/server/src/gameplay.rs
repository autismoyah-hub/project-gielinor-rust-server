//! Packet-to-action boundary. Unknown content remains visible to the caller.
use gielinor_protocol::{buffer::DecodeError, framing::Packet, game};
use gielinor_world::{simulation::Action, CoordGrid};

#[derive(Debug)]
pub enum Request {
    Npc(gielinor_protocol::npc_input::Operation),
    Location(gielinor_protocol::loc_input::Operation),
    LocationTarget(gielinor_protocol::loc_input::Target),
    GroundObject(gielinor_protocol::obj_input::Operation),
    Action(Action),
    KeepAlive,
    WindowStatus(gielinor_protocol::game::WindowStatus),
    ResumeCount(i32),
    ResumePause(gielinor_protocol::dialog::Pause),
    Button(gielinor_protocol::buttons::Button),
    Drag(gielinor_protocol::buttons::Drag),
    Target(gielinor_protocol::buttons::Target),
    /// Requires protected-access teleport support before this can be executed.
    PrivilegedTelejump(CoordGrid),
    Unhandled,
}

/// Staff authority comes from the authenticated account, never packet fields.
pub fn decode(
    packet: &Packet,
    level: i32,
    staff_level: u8,
    development: bool,
) -> Result<Request, DecodeError> {
    let click = match packet.opcode {
        102 | 13 | 19 | 45 | 85 => {
            return Ok(Request::Npc(gielinor_protocol::npc_input::decode(
                packet.opcode,
                &packet.payload,
            )?))
        }
        115 => {
            return Ok(Request::ResumePause(gielinor_protocol::dialog::pause(
                &packet.payload,
            )?))
        }
        46 | 74 | 7 | 31 | 43 => {
            return Ok(Request::GroundObject(gielinor_protocol::obj_input::decode(
                packet.opcode,
                &packet.payload,
            )?))
        }
        73 | 103 | 104 | 15 | 116 => {
            return Ok(Request::Location(gielinor_protocol::loc_input::decode(
                packet.opcode,
                &packet.payload,
            )?))
        }
        62 => {
            return Ok(Request::LocationTarget(
                gielinor_protocol::loc_input::target(&packet.payload)?,
            ))
        }
        10 => return Ok(Request::WindowStatus(game::window_status(&packet.payload)?)),
        75 => {
            return Ok(Request::ResumeCount(gielinor_protocol::dialog::count(
                &packet.payload,
            )?));
        }
        47 => return game_button(&packet.payload),
        27 => {
            return Ok(Request::Target(gielinor_protocol::buttons::target(
                &packet.payload,
            )?))
        }
        48 => {
            return Ok(Request::Drag(gielinor_protocol::buttons::drag(
                &packet.payload,
            )?))
        }
        114 => game::move_game_click(&packet.payload)?,
        67 => game::move_minimap_click(&packet.payload)?.movement,
        89 => {
            if !packet.payload.is_empty() {
                return Err(DecodeError::InvalidValue("keepalive payload"));
            }
            return Ok(Request::KeepAlive);
        }
        _ => return Ok(Request::Unhandled),
    };
    let destination = CoordGrid::new(click.x as i32, click.z as i32, level)
        .map_err(|_| DecodeError::InvalidValue("movement coordinates"))?;
    if click.keys == 2 && (staff_level == 2 || development) {
        return Ok(Request::PrivilegedTelejump(destination));
    }
    Ok(Request::Action(Action::MoveClick {
        destination,
        control: click.keys == 1,
    }))
}
fn game_button(payload: &[u8]) -> Result<Request, DecodeError> {
    let button = gielinor_protocol::buttons::button(payload)?;
    if !(1..=10).contains(&button.op) {
        return Err(DecodeError::InvalidValue("Unhandled interface operation"));
    }
    Ok(Request::Button(button))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn target_operation_decodes_all_fields_and_rejects_wrong_lengths() {
        // target 149:0, selected slot 2, target item 1929, target slot 3,
        // selected item 1933, selected component 149:0 (middle-endian).
        let bytes = vec![0, 149, 0, 0, 0, 130, 9, 7, 3, 0, 7, 13, 0, 0, 0, 149];
        let packet = Packet {
            opcode: 27,
            payload: bytes.clone(),
        };
        let Request::Target(target) = decode(&packet, 0, 0, false).unwrap() else {
            panic!("target operation not dispatched");
        };
        assert_eq!(
            target,
            gielinor_protocol::buttons::Target {
                source: 149 << 16,
                source_slot: 2,
                source_object: 1933,
                target: 149 << 16,
                target_slot: 3,
                target_object: 1929,
            }
        );
        for size in 0..16 {
            assert!(decode(
                &Packet {
                    opcode: 27,
                    payload: bytes[..size].to_vec()
                },
                0,
                0,
                false
            )
            .is_err());
        }
        let mut extra = bytes;
        extra.push(0);
        assert!(decode(
            &Packet {
                opcode: 27,
                payload: extra
            },
            0,
            0,
            false
        )
        .is_err());
    }
    #[test]
    fn movement_uses_authoritative_plane_and_account_privileges() {
        let mut packet = Packet {
            opcode: 114,
            payload: vec![0x81, 0x0c, 0x82, 0x0c, 127],
        };
        match decode(&packet, 2, 0, false).unwrap() {
            Request::Action(Action::MoveClick {
                destination,
                control,
            }) => {
                assert_eq!(destination, CoordGrid::new(3201, 3202, 2).unwrap());
                assert!(control);
            }
            _ => panic!("missing movement action"),
        }
        packet.payload[4] = 126;
        assert!(matches!(
            decode(&packet, 0, 0, false).unwrap(),
            Request::Action(_)
        ));
        assert!(matches!(
            decode(&packet, 0, 2, false).unwrap(),
            Request::PrivilegedTelejump(_)
        ));
        assert!(matches!(
            decode(&packet, 0, 0, true).unwrap(),
            Request::PrivilegedTelejump(_)
        ));
        packet.payload.extend([0]);
        assert!(decode(&packet, 0, 0, false).is_err());
    }
    #[test]
    fn unimplemented_packets_and_malformed_keepalive_are_explicit() {
        assert!(matches!(
            decode(
                &Packet {
                    opcode: 51,
                    payload: vec![0; 4]
                },
                0,
                0,
                false
            )
            .unwrap(),
            Request::Unhandled
        ));
        assert!(decode(
            &Packet {
                opcode: 89,
                payload: vec![0]
            },
            0,
            0,
            false
        )
        .is_err());
    }
}
