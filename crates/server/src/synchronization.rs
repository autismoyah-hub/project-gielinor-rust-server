//! Per-observer player synchronization; immutable world snapshots survive tick cleanup.
//! Appearance bytes are supplied by the content/type layer. NPC and world-entity packets
//! must be scheduled by the caller before this becomes a complete client cycle.
use gielinor_protocol::{
    appearance::Updates,
    buffer::DecodeError,
    framing::Packet,
    player_sync::{self, Position, State},
};
use gielinor_world::{
    simulation::{ActorId, MovementSnapshot},
    CoordGrid,
};
use std::collections::HashMap;

pub struct Avatar<'a> {
    pub combat: Option<&'a crate::combat_display::Display>,
    pub actor: ActorId,
    pub movement: MovementSnapshot,
    pub appearance: &'a [u8],
    pub appearance_rebuild: bool,
    pub sequence: Option<(u16, u8)>,
    pub spot_animations: Vec<gielinor_protocol::spot_animation::Animation>,
    pub face_angle: Option<u16>,
    pub face_npc: Option<u16>,
    pub face_changed: bool,
}
#[derive(Clone)]
struct Known {
    face_angle: Option<u16>,
    face_npc: Option<u16>,
    movement: MovementSnapshot,
    appearance: Vec<u8>,
}
#[derive(Clone)]
pub struct Observer {
    actor: ActorId,
    state: State,
    base: CoordGrid,
    radius: i32,
    known: HashMap<ActorId, Known>,
}
fn position(coords: CoordGrid) -> Position {
    Position {
        x: coords.x() as u16,
        z: coords.z() as u16,
        level: coords.level() as u8,
    }
}
fn face_angle(angle: u16) -> gielinor_protocol::appearance::Face {
    gielinor_protocol::appearance::Face {
        walk_mode: 0,
        instant: false,
        target: gielinor_protocol::appearance::FaceTarget::Angle(angle),
    }
}
fn face_npc(index: u16, angle: Option<u16>) -> gielinor_protocol::appearance::Face {
    gielinor_protocol::appearance::Face {
        walk_mode: 0,
        instant: false,
        target: gielinor_protocol::appearance::FaceTarget::Entity {
            entity_type: 1,
            index: Some(index as u32),
            fallback_angle: angle.unwrap_or(0),
        },
    }
}
fn region(coords: CoordGrid) -> u32 {
    ((coords.level() as u32) << 16) | ((coords.x() as u32 >> 13) << 8) | (coords.z() as u32 >> 13)
}
fn base(coords: CoordGrid) -> Result<CoordGrid, DecodeError> {
    // BuildAreaUtils uses checked ZoneKey.translate(-6, -6).
    CoordGrid::new(
        ((coords.x() >> 3) - 6) << 3,
        ((coords.z() >> 3) - 6) << 3,
        coords.level(),
    )
    .map_err(|_| DecodeError::InvalidValue("build area outside map bounds"))
}
fn needs_rebuild(coords: CoordGrid, area: CoordGrid) -> bool {
    let dx = coords.x() - area.x();
    let dz = coords.z() - area.z();
    dx < 16 || dz < 16 || dx >= 88 || dz >= 88
}
impl Observer {
    pub fn build_area_base(&self) -> CoordGrid {
        self.base
    }
    /// Radius is explicit until RSProt's dynamic viewing-distance policy is ported.
    pub fn initialize(
        actor: ActorId,
        avatars: &[Avatar<'_>],
        radius: i32,
    ) -> Result<(Self, Packet), DecodeError> {
        Self::initialize_positions(
            actor,
            &avatars
                .iter()
                .map(|p| (p.actor, p.movement.coords))
                .collect::<Vec<_>>(),
            radius,
        )
    }
    /// Login rebuilds need positions only. Content login hooks run after this
    /// rebuild but before the first appearance snapshots are constructed.
    pub fn initialize_positions(
        actor: ActorId,
        positions: &[(ActorId, CoordGrid)],
        radius: i32,
    ) -> Result<(Self, Packet), DecodeError> {
        if !(1..=127).contains(&radius) {
            return Err(DecodeError::InvalidValue("view radius"));
        }
        let local = positions
            .iter()
            .find(|p| p.0 == actor)
            .ok_or(DecodeError::InvalidValue("local avatar missing"))?;
        let mut regions = [0; 2048];
        let mut occupied = [false; 2048];
        for &(other, coords) in positions {
            let slot = other.slot as usize;
            if slot == 0 || slot >= 2048 || occupied[slot] || coords == CoordGrid::NULL {
                return Err(DecodeError::InvalidValue("avatar initialization"));
            }
            occupied[slot] = true;
            regions[slot] = region(coords);
        }
        let coords = local.1;
        let (state, gpi) =
            State::initialize(actor.slot, position(coords), actor.generation, &regions)?;
        let payload = player_sync::rebuild_login(
            &gpi,
            (coords.x() >> 3) as u16,
            (coords.z() >> 3) as u16,
            0,
        )?;
        Ok((
            Self {
                actor,
                state,
                base: base(coords)?,
                radius,
                known: HashMap::new(),
            },
            Packet {
                opcode: 49,
                payload,
            },
        ))
    }
    /// Encode atomically: invalid appearance/frame data cannot advance observer tracking.
    pub fn cycle(&mut self, avatars: &[Avatar<'_>]) -> Result<Vec<Packet>, DecodeError> {
        let mut next = self.clone();
        let packets = next.encode(avatars)?;
        *self = next;
        Ok(packets)
    }
    fn encode(&mut self, avatars: &[Avatar<'_>]) -> Result<Vec<Packet>, DecodeError> {
        let local = avatars
            .iter()
            .find(|p| p.actor == self.actor)
            .ok_or(DecodeError::InvalidValue("local avatar missing"))?;
        let center = local.movement.coords;
        let mut blocks = Vec::with_capacity(avatars.len());
        for avatar in avatars {
            if avatar.appearance.is_empty() {
                return Err(DecodeError::InvalidValue("missing appearance"));
            }
            let m = avatar.movement;
            let old = self.known.get(&avatar.actor);
            let changed =
                avatar.appearance_rebuild || old.is_none_or(|k| k.appearance != avatar.appearance);
            let speed_changed = old.is_none_or(|k| k.movement.cached_speed != m.cached_speed);
            let moved = old.is_none_or(|k| k.movement.coords != m.coords);
            let temporary = (moved && m.resolved_speed() != m.cached_speed)
                .then_some(m.resolved_speed().wire() as i8);
            let hits = avatar
                .combat
                .filter(|d| !d.hits.is_empty())
                .map(|d| {
                    gielinor_protocol::combat_updates::hitmarks(
                        false,
                        self.actor.slot,
                        avatar.actor.slot,
                        &d.hits,
                    )
                })
                .transpose()?;
            let bars = avatar
                .combat
                .filter(|d| !d.bars.is_empty())
                .map(|d| {
                    gielinor_protocol::combat_updates::headbars(
                        false,
                        self.actor.slot,
                        avatar.actor.slot,
                        &d.bars,
                    )
                })
                .transpose()?;
            let update = Updates {
                spot_animations: Some(&avatar.spot_animations),
                hitmarks: hits.as_deref(),
                headbars: bars.as_deref(),
                face: avatar
                    .face_npc
                    .filter(|npc| {
                        avatar.face_changed || old.is_none_or(|k| k.face_npc != Some(*npc))
                    })
                    .map(|npc| face_npc(npc, avatar.face_angle))
                    .or_else(|| {
                        avatar
                            .face_angle
                            .filter(|angle| {
                                avatar.face_npc.is_none()
                                    && (avatar.face_changed
                                        || old.is_none_or(|k| k.face_angle != Some(*angle)))
                            })
                            .map(face_angle)
                    })
                    .or_else(|| {
                        old.filter(|k| k.face_npc.is_some() && avatar.face_npc.is_none())
                            .map(|_| gielinor_protocol::appearance::Face {
                                walk_mode: 0,
                                instant: false,
                                target: gielinor_protocol::appearance::FaceTarget::Reset,
                            })
                    }),
                sequence: avatar.sequence,
                speed: speed_changed.then_some(m.cached_speed.wire() as i8),
                temporary_speed: temporary,
                appearance: changed.then_some(avatar.appearance),
                ..Default::default()
            }
            .encode()?;
            let initial = Updates {
                spot_animations: Some(&avatar.spot_animations),
                hitmarks: hits.as_deref(),
                headbars: bars.as_deref(),
                face: avatar
                    .face_npc
                    .map(|npc| face_npc(npc, avatar.face_angle))
                    .or_else(|| avatar.face_angle.map(face_angle)),
                sequence: avatar.sequence,
                speed: Some(m.cached_speed.wire() as i8),
                temporary_speed: temporary,
                appearance: Some(avatar.appearance),
                ..Default::default()
            }
            .encode()?;
            blocks.push((update, initial));
        }
        let players: Vec<_> = avatars
            .iter()
            .zip(&blocks)
            .map(|(avatar, (extended, initial))| {
                let m = avatar.movement;
                let visible = avatar.actor == self.actor
                    || (!m.hidden
                        && m.coords.level() == center.level()
                        && (m.coords.x() - center.x()).abs() <= self.radius
                        && (m.coords.z() - center.z()).abs() <= self.radius);
                player_sync::Player {
                    slot: avatar.actor.slot,
                    generation: avatar.actor.generation,
                    position: position(m.coords),
                    visible,
                    teleport: m.teleport || m.telejump,
                    extended,
                    initial_extended: initial,
                }
            })
            .collect();
        let payload = self.state.encode(&players)?;
        let mut packets = vec![];
        if needs_rebuild(center, self.base) {
            self.base = base(center)?;
            packets.push(Packet {
                opcode: 49,
                payload: player_sync::rebuild_normal(
                    (center.x() >> 3) as u16,
                    (center.z() >> 3) as u16,
                    0,
                )
                .to_vec(),
            });
        }
        packets.push(Packet {
            opcode: 28,
            payload,
        });
        self.known = avatars
            .iter()
            .map(|p| {
                (
                    p.actor,
                    Known {
                        face_angle: p.face_angle,
                        face_npc: p.face_npc,
                        movement: p.movement,
                        appearance: p.appearance.to_vec(),
                    },
                )
            })
            .collect();
        Ok(packets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::movement::Movement;
    #[test]
    fn player_hits_select_target_source_and_public_views_once() {
        use gielinor_protocol::combat_updates::{self, Headbar, Hitmark};
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        let mut avatars: Vec<_> = (1..=3)
            .map(|slot| Avatar {
                actor: ActorId {
                    slot,
                    generation: 1,
                },
                movement: MovementSnapshot::from(&Movement::new(coords)),
                appearance: &[0],
                appearance_rebuild: false,
                sequence: None,
                spot_animations: Vec::new(),
                face_angle: None,
                face_npc: None,
                face_changed: false,
                combat: None,
            })
            .collect();
        let mut observers: Vec<_> = avatars
            .iter()
            .map(|a| Observer::initialize(a.actor, &avatars, 15).unwrap().0)
            .collect();
        for observer in &mut observers {
            observer.cycle(&avatars).unwrap();
        }
        let display = crate::combat_display::Display {
            spot_animations: vec![],
            sequence: None,
            hits: vec![Hitmark {
                source_index: 65538,
                source_type: 43,
                self_type: 16,
                other_type: 17,
                value: 7,
                delay: 0,
                limit: 4,
            }],
            bars: vec![Headbar {
                source_index: 65538,
                self_type: 0,
                other_type: 0,
                start_fill: 9,
                end_fill: 9,
                start_time: 0,
                end_time: 0,
            }],
        };
        avatars[0].combat = Some(&display);
        avatars[0].spot_animations = vec![gielinor_protocol::spot_animation::Animation {
            slot: 0,
            id: 1336,
            delay: 0,
            height: 0,
        }];
        let mut blocks = vec![];
        for (index, observer) in observers.iter_mut().enumerate() {
            let hits = combat_updates::hitmarks(false, index as u16 + 1, 1, &display.hits).unwrap();
            let bars = combat_updates::headbars(false, index as u16 + 1, 1, &display.bars).unwrap();
            let block = Updates {
                hitmarks: Some(&hits),
                headbars: Some(&bars),
                spot_animations: Some(&avatars[0].spot_animations),
                ..Default::default()
            }
            .encode()
            .unwrap();
            assert!(observer.cycle(&avatars).unwrap()[0]
                .payload
                .ends_with(&block));
            blocks.push(block);
        }
        assert_ne!(blocks[0], blocks[1]);
        assert_ne!(blocks[1], blocks[2]);
        avatars[0].combat = None;
        avatars[0].spot_animations.clear();
        for (observer, block) in observers.iter_mut().zip(blocks) {
            assert!(!observer.cycle(&avatars).unwrap()[0]
                .payload
                .ends_with(&block));
        }
        // Creator setters mark a rebuild even when encoded appearance is equal.
        // Every observer must receive it before the tick clears that flag.
        avatars[0].appearance_rebuild = true;
        let repeated = Updates {
            appearance: Some(avatars[0].appearance),
            ..Default::default()
        }
        .encode()
        .unwrap();
        for observer in &mut observers {
            assert!(observer.cycle(&avatars).unwrap()[0]
                .payload
                .ends_with(&repeated));
        }
        avatars[0].appearance_rebuild = false;
        for observer in &mut observers {
            assert!(!observer.cycle(&avatars).unwrap()[0]
                .payload
                .ends_with(&repeated));
        }
    }
    #[test]
    fn initial_frame_rebuild_boundaries_and_failed_frame_rollback() {
        let actor = ActorId {
            slot: 1,
            generation: 1,
        };
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        let mut avatars = [Avatar {
            combat: None,
            actor,
            movement: MovementSnapshot::from(&Movement::new(coords)),
            appearance: &[0],
            appearance_rebuild: false,
            sequence: None,
            spot_animations: Vec::new(),
            face_angle: None,
            face_npc: None,
            face_changed: false,
        }];
        let (mut observer, login) = Observer::initialize(actor, &avatars, 15).unwrap();
        assert_eq!(login.opcode, 49);
        assert_eq!(login.payload.len(), 4614);
        let center = avatars[0].movement.coords;
        avatars[0].movement.coords = CoordGrid::new(0, 3200, 0).unwrap();
        assert!(Observer::initialize(actor, &avatars, 15).is_err());
        assert!(observer.cycle(&avatars).is_err());
        avatars[0].movement.coords = center;
        assert_eq!(observer.cycle(&avatars).unwrap().len(), 1);
        let mut expected = observer.clone();
        avatars[0].appearance = &[];
        assert!(observer.cycle(&avatars).is_err());
        avatars[0].appearance = &[0];
        assert_eq!(
            observer.cycle(&avatars).unwrap(),
            expected.cycle(&avatars).unwrap()
        );
        avatars[0].movement.coords = CoordGrid::new(3239, 3200, 0).unwrap();
        assert_eq!(observer.cycle(&avatars).unwrap().len(), 1);
        avatars[0].movement.coords = CoordGrid::new(3240, 3200, 0).unwrap();
        let packets = observer.cycle(&avatars).unwrap();
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0].opcode, 49);
    }
}
