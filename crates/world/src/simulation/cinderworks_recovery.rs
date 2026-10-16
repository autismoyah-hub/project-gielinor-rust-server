//! Sanctuary rescue uses normal movement and stat authorities. Equipment and
//! durable progress survive defeat; consumed supplies are never refunded.
use super::*;
impl World {
    pub(super) fn process_cinderworks_recovery(&mut self) {
        let Some(config) = self.cinderworks.clone() else {
            return;
        };
        let slots: Vec<_> = self
            .players
            .iter()
            .filter(|(_, p)| {
                !p.saving
                    && p.data.tower_run.id > 0
                    && crate::tower_layout::contains(config.origin, p.movement.coords)
                    && (p.data.stats.current(3) as u8 == 0 || p.cinderworks_recovery_at.is_some())
            })
            .map(|(slot, _)| slot)
            .collect();
        for slot in slots {
            let p = self.players.get_mut(slot).unwrap();
            let actor = p.id;
            if p.cinderworks_recovery_at.is_none() {
                p.cinderworks_recovery_at = Some(self.clock.deadline(4));
                p.death_queued = true;
                p.movement.rooted = true;
                p.movement.abort();
                p.protection.current_clock = self.clock.0;
                p.protection.delay(4);
                p.protection.suspended = false;
                p.queues.clear_weak();
                p.hits = Default::default();
                if let Some(queues) = self.player_hit_queues {
                    p.queues.remove_all(queues.standard);
                    p.queues.remove_all(queues.death);
                }
                p.npc_interaction = None;
                p.location_interaction = None;
                p.object_interaction = None;
                p.prayer = None;
                p.has_modal = false;
                self.events.push(WorldEvent::CloseModals { actor });
                self.events.push(WorldEvent::PlayerCombatAnimation {
                    actor,
                    sequence: config.death_animation,
                });
                self.events.push(WorldEvent::GameMessage {actor,text:"The heat overwhelms you. The sanctuary is drawing you back; your equipment and restoration progress are safe.".into()});
                continue;
            }
            if self.clock.0 < p.cinderworks_recovery_at.unwrap() {
                continue;
            }
            let destination = [(56, 56), (55, 56), (56, 55), (57, 56)]
                .into_iter()
                .filter_map(|(x, z)| config.origin.translate(x, z, 0).ok())
                .find(|&coords| {
                    p.movement
                        .teleport(&mut self.collision, coords, self.clock.0, true)
                        .is_ok()
                });
            let Some(destination) = destination else {
                p.cinderworks_recovery_at = Some(self.clock.deadline(1));
                continue;
            };
            p.data.coords = destination;
            p.data.stats.set_current(3, p.data.stats.base(3));
            p.movement.rooted = false;
            p.death_queued = false;
            p.cinderworks_recovery_at = None;
            p.protection.suspended = false;
            p.protection.delay_until = self.clock.0;
            p.aggressive_npc = None;
            p.retaliate_npcs.clear();
            p.last_npc_combat = -1000;
            p.logout_blocked_until = self.clock.0;
            p.hits = Default::default();
            if let Some(queues) = self.player_hit_queues {
                p.queues.remove_all(queues.standard);
                p.queues.remove_all(queues.death);
            }
            self.events.push(WorldEvent::ClearMapFlag(actor));
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: u16::MAX,
            });
            self.events.push(WorldEvent::GameMessage {actor,text:"You wake in the sanctuary. Consumed supplies stay spent; services, guardian unlocks and pending rewards remain. Prepare before returning to the court.".into()});
            self.events.push(WorldEvent::CinderworksProgress {
                actor,
                run: p.data.tower_run,
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cinderworks_encounter::Boss,
        hits::{Hit, Style, Target},
    };
    #[test]
    fn unsigned_boosted_health_is_not_mistaken_for_death() {
        let (mut world, actor, _) =
            super::super::cinderworks_encounter_actions::tests::setup(Boss::Imp);
        let coords = world.data(actor).unwrap().coords;
        world
            .data_mut(actor)
            .unwrap()
            .stats
            .set_current(3, 180u8 as i8);
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3) as u8, 180);
        assert_eq!(world.data(actor).unwrap().coords, coords);
        assert!(world
            .players
            .get(actor.slot as usize)
            .unwrap()
            .cinderworks_recovery_at
            .is_none());
    }
    #[test]
    fn lethal_hits_rescue_without_regranting_items_or_resetting_pending_rewards() {
        for victory in [false, true] {
            let (mut world, actor, target) =
                super::super::cinderworks_encounter_actions::tests::setup(Boss::Warden);
            if victory {
                assert!(world.data_mut(actor).unwrap().tower_run.defeat_warden());
            }
            let before = world.data(actor).unwrap().clone();
            world
                .queue_hit(
                    Target::Player(actor),
                    Hit {
                        source: None,
                        style: Style::Typeless,
                        damage: 30,
                        display: None,
                    },
                    0,
                )
                .unwrap();
            world.tick().unwrap();
            assert_eq!(world.data(actor).unwrap().stats.current(3), 0);
            world.tick().unwrap();
            assert!(world
                .prepare_movement_input(actor, CoordGrid::new(10424, 120, 0).unwrap(), false)
                .unwrap()
                .is_none());
            for _ in 0..5 {
                world.tick().unwrap();
            }
            let after = world.data(actor).unwrap();
            assert_eq!(after.coords, CoordGrid::new(10424, 120, 0).unwrap());
            assert_eq!(after.stats.current(3), 20);
            assert_eq!(after.tower_run, before.tower_run);
            assert_eq!(after.inventory, before.inventory);
            assert_eq!(after.equipment, before.equipment);
            assert!(world.movement_input_allowed(actor).unwrap());
            assert_eq!(world.npc(target).unwrap().stats[3], 42);
            assert!(world.cinderworks_encounters[1].is_none());
        }
    }
    #[test]
    fn a_saved_zero_health_projection_reconstructs_recovery_after_restart() {
        let (mut world, actor, _) =
            super::super::cinderworks_encounter_actions::tests::setup(Boss::Imp);
        world
            .queue_hit(
                Target::Player(actor),
                Hit {
                    source: None,
                    style: Style::Typeless,
                    damage: 30,
                    display: None,
                },
                0,
            )
            .unwrap();
        world.tick().unwrap();
        let projection = world.data(actor).unwrap().clone();
        assert_eq!(projection.stats.current(3), 0);
        let mut restarted = World::default();
        restarted.cinderworks = world.cinderworks.clone();
        for (x, z) in [(10424, 120), (projection.coords.x(), projection.coords.z())] {
            restarted.collision.allocate(x, z, 0);
        }
        restarted.queue_login(2, projection.clone()).unwrap();
        for _ in 0..7 {
            restarted.tick().unwrap();
        }
        let restored = restarted.data(restarted.actors().next().unwrap()).unwrap();
        assert_eq!(restored.stats.current(3), 20);
        assert_eq!(restored.coords, CoordGrid::new(10424, 120, 0).unwrap());
        assert_eq!(restored.tower_run, projection.tower_run);
        assert_eq!(restored.equipment, projection.equipment);
    }
}
