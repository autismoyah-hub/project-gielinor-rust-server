//! NPC movement snapshot adapter. The caller supplies the NPC update origin used by the client.
use gielinor_protocol::{
    buffer::DecodeError,
    framing::Packet,
    npc_sync::{self, Movement, Position, State},
};
use gielinor_world::{
    movement::Speed,
    simulation::{MovementSnapshot, NpcId},
    CoordGrid,
};
use std::collections::HashMap;
pub struct Avatar<'a> {
    pub actor: NpcId,
    pub type_id: u16,
    pub movement: MovementSnapshot,
    pub direction: u8,
    pub allocated_this_tick: bool,
    pub spawn_cycle: Option<u32>,
    pub extended: &'a [u8],
    pub initial_extended: &'a [u8],
}
#[derive(Clone, Default)]
pub struct Observer {
    state: State,
    known: HashMap<NpcId, CoordGrid>,
}
fn position(c: CoordGrid) -> Position {
    Position {
        x: c.x() as u16,
        z: c.z() as u16,
        level: c.level() as u8,
    }
}
fn step(a: CoordGrid, b: CoordGrid) -> Option<u8> {
    if a.level() != b.level() || a == CoordGrid::NULL || b == CoordGrid::NULL {
        return None;
    }
    npc_sync::direction(b.x() - a.x(), b.z() - a.z())
}
impl Observer {
    pub fn is_empty(&self) -> bool {
        self.known.is_empty()
    }
    pub fn actors(&self) -> impl Iterator<Item = NpcId> + '_ {
        self.known.keys().copied()
    }
    pub fn cycle(
        &mut self,
        origin: CoordGrid,
        large: bool,
        radius: i32,
        capacity: usize,
        avatars: &[Avatar<'_>],
    ) -> Result<Packet, DecodeError> {
        if origin == CoordGrid::NULL || !(1..=127).contains(&radius) {
            return Err(DecodeError::InvalidValue("NPC observer origin/radius"));
        }
        let mut npcs = vec![];
        for n in avatars {
            let m = n.movement;
            if m.coords == CoordGrid::NULL {
                return Err(DecodeError::InvalidValue("NPC null coordinate"));
            }
            let prior = self.known.get(&n.actor).copied();
            let movement = match (prior, m.steps) {
                (Some(prior), 1) => step(prior, m.coords).map(|d| {
                    if m.speed == Speed::Crawl {
                        Movement::Crawl(d)
                    } else {
                        Movement::Walk(d)
                    }
                }),
                (Some(prior), 2) => step(prior, m.before_last_step)
                    .zip(step(m.before_last_step, m.coords))
                    .map(|(a, b)| Movement::Run(a, b)),
                (_, 0) => Some(Movement::None),
                _ => None,
            };
            npcs.push(npc_sync::Npc {
                index: n.actor.slot,
                generation: n.actor.generation,
                type_id: n.type_id,
                position: position(m.coords),
                visible: !m.hidden
                    && m.coords.level() == origin.level()
                    && (m.coords.x() - origin.x()).abs() <= radius
                    && (m.coords.z() - origin.z()).abs() <= radius,
                teleport: m.teleport || m.telejump || movement.is_none(),
                jump: n.allocated_this_tick || !m.teleport || m.telejump,
                direction: n.direction,
                spawn_cycle: n.spawn_cycle,
                movement: movement.unwrap_or(Movement::None),
                extended: n.extended,
                initial_extended: n.initial_extended,
            });
        }
        let payload = self
            .state
            .encode(position(origin), large, &npcs, capacity)?;
        let included: std::collections::HashSet<_> = self.state.indices().collect();
        self.known = avatars
            .iter()
            .filter(|a| included.contains(&a.actor.slot))
            .map(|a| (a.actor, a.movement.coords))
            .collect();
        Ok(Packet {
            opcode: if large { 2 } else { 85 },
            payload,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::movement::Movement as WorldMovement;
    #[test]
    fn bent_running_steps_survive_world_cleanup() {
        let origin = CoordGrid::new(3200, 3200, 0).unwrap();
        let actor = NpcId {
            slot: 1,
            generation: 1,
        };
        let mut movement = WorldMovement::new(origin);
        let mut observer = Observer::default();
        let avatar = |movement| Avatar {
            actor,
            type_id: 1,
            movement,
            direction: 6,
            allocated_this_tick: false,
            spawn_cycle: None,
            extended: &[],
            initial_extended: &[],
        };
        observer
            .cycle(origin, false, 15, 255, &[avatar((&movement).into())])
            .unwrap();
        movement.pending_steps = 2;
        movement.speed = Speed::Run;
        movement.last_processed = CoordGrid::new(3201, 3200, 0).unwrap();
        movement.coords = CoordGrid::new(3201, 3201, 0).unwrap();
        let snapshot = (&movement).into();
        movement.finish_tick();
        // retained=1, changed=1, kind=2, running=1, east=4, north=1, no extension.
        assert_eq!(
            observer
                .cycle(origin, false, 15, 255, &[avatar(snapshot)])
                .unwrap()
                .payload,
            [1, 0xd8, 0x40]
        );
    }
}
