use super::*;
use crate::{
    locations::Location, tutorial_gates::Effect, tutorial_progression::Effect as Progress,
};

impl World {
    pub(super) fn process_tutorial_gate(&mut self, slot: usize, loc: Location) {
        let Some(config) = self.tutorial_gates.clone() else {
            return;
        };
        let Some(gate) = config.gates.get(&loc.id) else {
            return;
        };
        let p = self.players.get(slot).unwrap();
        let actor = p.id;
        let source = p.movement.coords;
        let effects = match gate.decide(
            config.progression.current(&p.data.variables),
            p.data.variables.get(config.progression.completed.id),
            source,
            loc,
        ) {
            Ok(effects) => effects,
            Err(_) => {
                self.events.push(WorldEvent::LocationError(
                    "Invalid tutorial gate destination",
                ));
                return;
            }
        };
        for effect in effects {
            match effect {
                Effect::Deny(message) => self
                    .events
                    .push(WorldEvent::LocationMessage { actor, message }),
                Effect::Advance(step) => {
                    let p = self.players.get_mut(slot).unwrap();
                    let logged = p.protection.processed_clock > 0;
                    let progress =
                        config
                            .progression
                            .advance(&mut p.data.variables, step, None, logged);
                    for effect in progress {
                        match effect {
                            Progress::Variable(update) => {
                                self.events.push(WorldEvent::SessionVariables {
                                    actor,
                                    updates: vec![update],
                                })
                            }
                            Progress::Focus { step, .. } => {
                                let hint = match config.hint_npcs.get(&step) {
                                    Some(&kind) => match self.find_npc(source, kind, 64) {
                                        Ok(hint) => hint,
                                        Err(error) => {
                                            self.events.push(WorldEvent::LocationError(error));
                                            return;
                                        }
                                    },
                                    None => None,
                                };
                                self.events.push(WorldEvent::TutorialGateFocus {
                                    actor,
                                    step,
                                    hint,
                                });
                            }
                            // No gate advances to Completed.
                            Progress::End => unreachable!("Tutorial gate completed tutorial"),
                        }
                    }
                }
                Effect::Teleport(to) | Effect::Telejump(to) => {
                    let p = self.players.get_mut(slot).unwrap();
                    let jump = matches!(effect, Effect::Telejump(_));
                    if p.movement
                        .teleport(&mut self.collision, to, self.clock.0, jump)
                        .is_err()
                    {
                        self.events.push(WorldEvent::ClearMapFlag(actor));
                        self.events.push(WorldEvent::EngineMessage {
                            actor,
                            message: "Invalid teleport!",
                        });
                    } else {
                        p.data.coords = to;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        npc::{BlockWalk, Npc, Restrict},
        tutorial_gates::{Config, Gate},
        tutorial_steps::Step,
        variables::{Transmit, Varp},
    };

    fn setup(step: Step, completed: i32, allocate_destination: bool) -> (World, ActorId, Location) {
        let mut world = World::default();
        let source = CoordGrid::new(1680, 6127, 0).unwrap();
        world.collision.allocate(source.x(), source.z(), 0);
        if allocate_destination {
            world.collision.allocate(1680, 12527, 0);
        }
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let progression = crate::tutorial_progression::Config {
            step: varp(1),
            completed: varp(2),
        };
        let mut data = PlayerData::new(1, "Gate".into(), source, Stats::default());
        data.variables.set(progression.step, step as i32, false);
        data.variables.set(progression.completed, completed, false);
        world.tutorial_gates = Some(Config {
            progression,
            gates: [(1, Gate::QuestLadder)].into(),
            hint_npcs: [(Step::MineTalk, 1)].into(),
        });
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        world.drain_events();
        (world, actor, Location::new(source, 1, 10, 0).unwrap())
    }
    fn spawn(world: &mut World, x: i32, z: i32) -> NpcId {
        world
            .spawn_npc(
                Npc::new(
                    1,
                    CoordGrid::new(x, z, 0).unwrap(),
                    1,
                    Restrict::NoMove,
                    BlockWalk::None,
                )
                .unwrap(),
            )
            .unwrap()
    }
    #[test]
    fn ladder_hint_uses_departure_and_persistence_uses_arrival() {
        let (mut world, actor, loc) = setup(Step::QuestLadder, 0, true);
        let departure = spawn(&mut world, 1681, 6127);
        let arrival = spawn(&mut world, 1681, 12527);
        world.process_tutorial_gate(actor.slot as usize, loc);
        assert_eq!(
            world.data(actor).unwrap().coords,
            CoordGrid::new(1680, 12527, 0).unwrap()
        );
        assert_eq!(
            world.data(actor).unwrap().variables.get(1),
            Step::MineTalk as i32
        );
        assert_eq!(
            world
                .find_npc(world.data(actor).unwrap().coords, 1, 64)
                .unwrap(),
            Some(arrival)
        );
        let events = world.drain_events();
        assert!(
            matches!(&events[0],WorldEvent::SessionVariables{updates,..} if updates[0].value==23)
        );
        assert!(
            matches!(&events[1],WorldEvent::TutorialGateFocus{hint:Some(id),..} if *id==departure)
        );
        world.request_logout(actor).unwrap();
        assert!(world.drain_events().iter().any(|event| matches!(event,WorldEvent::Save{data,..} if data.coords.z()==12527 && data.variables.get(1)==23)));
    }
    #[test]
    fn absent_departure_hint_does_not_acquire_arrival_npc() {
        let (mut world, actor, loc) = setup(Step::QuestLadder, 0, true);
        spawn(&mut world, 1681, 12527);
        world.process_tutorial_gate(actor.slot as usize, loc);
        assert!(world
            .drain_events()
            .iter()
            .any(|event| matches!(event, WorldEvent::TutorialGateFocus { hint: None, .. })));
    }
    #[test]
    fn denied_ladder_preserves_position_and_completed_ladder_skips_focus() {
        let (mut world, actor, loc) = setup(Step::CharCreate, 0, true);
        world.process_tutorial_gate(actor.slot as usize, loc);
        assert_eq!(world.data(actor).unwrap().coords, loc.coords);
        assert!(matches!(
            world.drain_events().as_slice(),
            [WorldEvent::LocationMessage { .. }]
        ));
        let done = world.tutorial_gates.as_ref().unwrap().progression.completed;
        world.data_mut(actor).unwrap().variables.set(done, 1, false);
        world.process_tutorial_gate(actor.slot as usize, loc);
        assert_eq!(world.data(actor).unwrap().coords.z(), 12527);
        assert_eq!(world.data(actor).unwrap().variables.get(1), 0);
        assert!(world.drain_events().is_empty());
    }
    #[test]
    fn ordinary_location_action_runs_gate_before_player_synchronization() {
        let (mut world, actor, loc) = setup(Step::QuestLadder, 0, true);
        world.location_types.insert(
            loc.id,
            crate::loc_collision::LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
        world.change_location(loc, true).unwrap();
        world.drain_events();
        world
            .submit(
                actor,
                Action::OperateLocation {
                    coords: loc.coords,
                    id: loc.id,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().coords.z(), 12527);
        let events = world.drain_events();
        let focus = events
            .iter()
            .position(|event| matches!(event, WorldEvent::TutorialGateFocus { .. }))
            .unwrap();
        let synchronization = events.iter().position(|event| matches!(event, WorldEvent::SynchronizePlayers { players, .. } if players.iter().any(|(id,movement)| *id == actor && movement.telejump))).unwrap();
        assert!(focus < synchronization);
    }
    #[test]
    fn invalid_destination_preserves_prior_progression_and_clears_map_flag() {
        let (mut world, actor, loc) = setup(Step::QuestLadder, 0, false);
        world.process_tutorial_gate(actor.slot as usize, loc);
        assert_eq!(world.data(actor).unwrap().coords, loc.coords);
        assert_eq!(world.data(actor).unwrap().variables.get(1), 23);
        let events = world.drain_events();
        assert!(matches!(&events[2],WorldEvent::ClearMapFlag(id) if *id==actor));
        assert!(matches!(
            &events[3],
            WorldEvent::EngineMessage {
                message: "Invalid teleport!",
                ..
            }
        ));
    }
}
