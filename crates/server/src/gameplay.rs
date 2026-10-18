//! Packet-to-action boundary. Unknown content remains visible to the caller.
use gielinor_protocol::{buffer::DecodeError, framing::Packet, game};
use gielinor_world::{simulation::Action, CoordGrid};

#[derive(Debug)]
pub enum Request {
    Npc(gielinor_protocol::npc_input::Operation),
    NpcTarget(gielinor_protocol::npc_input::Target),
    Location(gielinor_protocol::loc_input::Operation),
    LocationTarget(gielinor_protocol::loc_input::Target),
    GroundObject(gielinor_protocol::obj_input::Operation),
    Action(Action),
    KeepAlive,
    /// Native CloseModal239.1: no component or client-selected continuation.
    Close,
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
        86 => {
            return Ok(Request::NpcTarget(gielinor_protocol::npc_input::target(
                &packet.payload,
            )?))
        }
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
        95 => {
            if !packet.payload.is_empty() {return Err(DecodeError::InvalidValue("close modal payload"));}
            return Ok(Request::Close);
        }
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
    fn npc_spell_target_routes_component_and_preserves_nullable_fields() {
        let bytes = vec![255, 255, 255, 0x12, 0xb4, 255, 255, 0, 218, 0, 5];
        let packet = Packet {
            opcode: 86,
            payload: bytes.clone(),
        };
        let Request::NpcTarget(t) = decode(&packet, 0, 0, false).unwrap() else {
            panic!("NPC target not dispatched");
        };
        assert_eq!(t.index, 0x1234);
        assert!(t.control);
        assert_eq!(t.source, (218 << 16) | 5);
        assert_eq!((t.source_slot, t.source_object), (-1, -1));
        for len in 0..11 {
            assert!(decode(
                &Packet {
                    opcode: 86,
                    payload: bytes[..len].to_vec()
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
                opcode: 86,
                payload: extra
            },
            0,
            0,
            false
        )
        .is_err());
    }
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

#[cfg(test)]mod native_close_tests{
 use super::*;
 use gielinor_world::{simulation::{World,PlayerData,WorldEvent},inventory::Item,CoordGrid};
 fn close_world(world:&mut World,a:gielinor_world::simulation::ActorId){let request=decode(&Packet{opcode:95,payload:vec![]},0,0,false).unwrap();assert!(matches!(request,Request::Close));world.close_dialogs(a).unwrap();}
 #[test]fn native_close_is_a_strict_empty_payload_request_without_privilege_gate(){for(staff,development)in[(0,false),(2,false),(0,true)]{assert!(matches!(decode(&Packet{opcode:95,payload:vec![]},0,staff,development).unwrap(),Request::Close));}for payload in [vec![0],vec![0,0]]{assert!(decode(&Packet{opcode:95,payload},0,0,false).is_err());}}
 #[test]fn decoded_native_close_invalidates_held_and_worn_chemistry_but_fresh_options_work(){for worn in [false,true]{let mut w=World::default();w.collision.allocate(3200,3200,0);w.finished_potions=Some(gielinor_world::finished_potions::Config{chemistry:[(175,gielinor_world::chemistry_modifier::Binding{amulet:21163,extra_output:2446})].into(),goggles:[29974,29976],recipes:Default::default(),quest_progress:80,quest_complete:4,animation:363});let mut data=PlayerData::new(1,"Close boundary".into(),CoordGrid::new(3200,3200,0).unwrap(),Default::default());data.chemistry=gielinor_world::chemistry::State::restore(2,0).unwrap();let slot=if worn{2}else{3};let inv=if worn{&mut data.equipment}else{&mut data.inventory};inv.restore_slot(slot,Some(Item{id:21163,count:1,vars:0})).unwrap();w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.drain_events();let open=Action::ChemistryItem{slot,item:21163,operation:4,worn};w.submit(a,open.clone()).unwrap();w.tick().unwrap();let token=w.drain_events().into_iter().find_map(|e|if let WorldEvent::ChemistryDialog{token:Some(token),..}=e{Some(token)}else{None}).unwrap();let before=w.data(a).unwrap().clone();close_world(&mut w,a);assert!(!w.protection(a).unwrap().suspended);w.submit(a,Action::ResumeChemistry{token,choice:2}).unwrap();w.tick().unwrap();let d=w.data(a).unwrap();assert_eq!(d.chemistry.remaining(),2);assert!(!d.chemistry.stop_on_crumble());assert_eq!(d.inventory,before.inventory);assert_eq!(d.equipment,before.equipment);assert_eq!(d.stats,before.stats);
  w.submit(a,open).unwrap();w.tick().unwrap();let fresh=w.drain_events().into_iter().find_map(|e|if let WorldEvent::ChemistryDialog{token:Some(token),..}=e{Some(token)}else{None}).unwrap();assert_ne!(fresh,token);w.submit(a,Action::ResumeChemistry{token:fresh,choice:2}).unwrap();w.tick().unwrap();assert!(w.data(a).unwrap().chemistry.stop_on_crumble());assert_eq!(w.data(a).unwrap().chemistry.remaining(),2);}}
 #[test]fn decoded_native_close_invalidates_quantity_continuation_without_blocking_new_dialog(){let mut w=World::default();w.queue_login(1,PlayerData::new(1,"Quantity close".into(),CoordGrid::new(3200,3200,0).unwrap(),Default::default())).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();let old=w.start_quantity_dialog(a).unwrap();close_world(&mut w,a);assert_eq!(w.quantity_dialog_token(a).unwrap(),None);assert!(!w.resume_quantity_dialog(a,old).unwrap());let fresh=w.start_quantity_dialog(a).unwrap();assert_ne!(fresh,old);assert!(!w.resume_quantity_dialog(a,old).unwrap());assert!(w.resume_quantity_dialog(a,fresh).unwrap());assert!(!w.protection(a).unwrap().suspended);}
}
