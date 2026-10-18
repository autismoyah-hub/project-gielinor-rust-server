use super::*;
use crate::runecrafting::{Config, Error, Pending, QUEUE};
fn refusal(events: &mut Vec<WorldEvent>, actor: ActorId, error: Error) {
    let text = match error {
        Error::Level => "You need a higher Runecraft level to bind that essence.",
        Error::Empty => "You do not have any rune or pure essence to bind.",
        Error::Unsupported => "You can't use that essence or equipment here yet.",
        _ => "You can't bind that essence right now.",
    };
    events.push(WorldEvent::GameMessage {
        actor,
        text: text.into(),
    });
}
impl World {
    pub(super) fn start_runecrafting(&mut self, slot: usize, location: crate::locations::Location) {
        let Some(config) = self.runecrafting.as_ref() else {
            return;
        };
        let Some(&recipe) = config.altars.get(&location.id) else {
            return;
        };
        let Some(revision) = self.locations.revision(location) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if p.pending_logout || p.saving || p.protection.access_protected(p.has_modal) {
            return;
        }
        let materials = match config.capture(&p.data.inventory) {
            Ok(v) => v,
            Err(e) => {
                refusal(&mut self.events, p.id, e);
                return;
            }
        };
        if let Err(e) = config.prepare(
            recipe,
            &materials,
            &p.data.inventory,
            &p.data.equipment,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
        ) {
            refusal(&mut self.events, p.id, e);
            return;
        }
        let pending = Pending {
            location,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            recipe,
            materials,
        };
        p.begin_held_action();
        p.queues.clear_weak();
        // Reached location callbacks run after this clock's queue pass.
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                1,
                Deferred::Runecrafting(pending),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: config.animation,
        });
        self.events.push(WorldEvent::PlayerSpotAnimation {
            actor: p.id,
            spot: config.spot,
            delay: 0,
            height: 100,
            slot: 0,
        });
    }
}
pub(super) fn complete(
    p: &mut Player,
    pending: Pending,
    config: Option<&Config>,
    items: &crate::inventory::Catalog,
    revision: Option<u64>,
    tutorial: bool,
    events: &mut Vec<WorldEvent>,
) {
    if revision != Some(pending.revision)
        || p.pending_logout
        || p.saving
        || p.has_modal
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || p.movement.coords != pending.origin
        || p.movement.last_movement != pending.last_movement
        || p.npc_interaction.is_some()
        || p.location_interaction.is_some()
        || p.object_interaction.is_some()
    {
        return;
    }
    let Some(config) =
        config.filter(|c| c.altars.get(&pending.location.id) == Some(&pending.recipe))
    else {
        return;
    };
    match config.prepare(
        pending.recipe,
        &pending.materials,
        &p.data.inventory,
        &p.data.equipment,
        items,
        &p.data.stats,
        p.data.xp_rate_hundreds,
    ) {
        Err(e) => refusal(events, p.id, e),
        Ok(prepared) => {
            p.data.inventory = prepared.inventory;
            p.data.stats = prepared.stats;
            if let Some(advancement) = prepared.advancement {
                if tutorial
                    && p.stat_events
                        .advance(&crate::tutorial_rules::bindings(), 20, advancement)
                        .is_err()
                {
                    p.pending_logout = true;
                    events.push(WorldEvent::ActionRejected {
                        actor: p.id,
                        error: ActionError::QueueFull,
                    });
                }
                events.push(WorldEvent::SkillAdvanced {
                    actor: p.id,
                    skill: 20,
                    advancement,
                });
            }
            events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: format!("You bind the essence into {}s.", pending.recipe.name),
            });
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item, loc_collision::LocCollision, locations::Location, runecrafting::Recipe,
    };
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Ordinary altar crafting must not draw RNG");
        }
    }
    const RECIPES: [(u32, i32, u8, u16, u8); 6] = [
        (34760, 556, 1, 50, 11),
        (34761, 558, 2, 55, 14),
        (34762, 555, 5, 60, 19),
        (34763, 557, 9, 65, 26),
        (34764, 554, 14, 70, 35),
        (34765, 559, 20, 75, 46),
    ];
    fn fixture(index: usize, kind: usize, member: bool) -> (World, ActorId, Location, Recipe) {
        let (id, output, level, xp_tenths, interval) = RECIPES[index];
        let recipe = Recipe {
            output,
            level,
            xp_tenths,
            interval,
            name: "rune",
        };
        let mut w = World::default();
        for x in (3192..3232).step_by(8) {
            for z in (3192..3232).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), id, 10, 0).unwrap();
        let ty = LocCollision {
            width: 3,
            length: 3,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(id, ty);
        w.locations.load(loc);
        crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        w.runecrafting = Some(Config {
            altars: [(id, recipe)].into(),
            blocked_held: [24704].into(),
            blocked_worn: [26850].into(),
            animation: 791,
            spot: 186,
        });
        w.object_types.stackable.insert(output);
        let mut stats = Stats::default();
        stats.set_base(20, interval as i8 - 1);
        stats.set_current(20, interval as i8);
        stats
            .set_xp(20, crate::skills::xp_from_level(interval - 1).unwrap())
            .unwrap();
        let mut data = PlayerData::new(
            1,
            "Runecrafter".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: match kind {
                            0 => 1436,
                            1 => 7936,
                            _ => {
                                if slot % 2 == 0 {
                                    1436
                                } else {
                                    7936
                                }
                            }
                        },
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, member).unwrap();
        w.drain_events();
        (w, a, loc, recipe)
    }
    fn start(w: &mut World, a: ActorId, loc: Location) {
        assert!(w.has_location_handler(loc.id, 1));
        assert!(!w.has_location_handler(loc.id, 2));
        w.submit(
            a,
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
    }
    #[test]
    fn rune_altars_actual_six_full_backpacks_rune_pure_mixed_both_world_types_one_tick() {
        for index in 0..6 {
            for kind in 0..3 {
                for member in [false, true] {
                    let (mut w, a, loc, r) = fixture(index, kind, member);
                    let xp = w.data(a).unwrap().stats.fine_xp(20);
                    let began = w.clock.0;
                    start(&mut w, a, loc);
                    assert_eq!(w.data(a).unwrap().stats.fine_xp(20), xp);
                    w.tick_with_random(&mut NoDraw).unwrap();
                    assert_eq!(w.clock.0 - 1, began + 1);
                    let data = w.data(a).unwrap();
                    assert_eq!(
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| i.id == r.output)
                            .map(|i| i.count)
                            .sum::<i32>(),
                        56
                    );
                    assert_eq!(data.inventory.free_space(), 27);
                    assert_eq!(
                        data.stats.fine_xp(20) - xp,
                        28 * i32::from(r.xp_tenths) * 125 / 100
                    );
                    let before = (data.inventory.clone(), data.stats.clone());
                    for _ in 0..3 {
                        w.tick_with_random(&mut NoDraw).unwrap();
                    }
                    assert_eq!(
                        (
                            w.data(a).unwrap().inventory.clone(),
                            w.data(a).unwrap().stats.clone()
                        ),
                        before
                    );
                }
            }
        }
    }
    #[test]
    fn rune_altars_recheck_captured_slots_context_modifiers_credit_level_and_rate() {
        for case in 0..9 {
            let (mut w, a, loc, r) = fixture(2, 2, false);
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(27, None)
                .unwrap();
            start(&mut w, a, loc);
            match case {
                0 => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            5,
                            Some(Item {
                                id: 7936,
                                count: 1,
                                vars: 9,
                            }),
                        )
                        .unwrap();
                }
                1 => {
                    w.locations.delete(loc);
                }
                2 => {
                    w.players.get_mut(a.slot as usize).unwrap().movement.coords =
                        CoordGrid::new(3200, 3201, 0).unwrap();
                }
                3 => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            27,
                            Some(Item {
                                id: 24704,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                4 => {
                    w.data_mut(a)
                        .unwrap()
                        .equipment
                        .restore_slot(
                            0,
                            Some(Item {
                                id: 26850,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                5 => {
                    w.data_mut(a).unwrap().stats.set_current(20, 4);
                }
                6 => {
                    w.object_types.dummy.insert(r.output);
                }
                7 => {
                    w.data_mut(a).unwrap().xp_rate_hundreds = 0;
                }
                8 => {
                    w.players.get_mut(a.slot as usize).unwrap().has_modal = true;
                }
                _ => {}
            }
            let inv = w.data(a).unwrap().inventory.clone();
            let xp = w.data(a).unwrap().stats.fine_xp(20);
            for _ in 0..4 {
                w.tick_with_random(&mut NoDraw).unwrap();
            }
            assert_eq!(w.data(a).unwrap().inventory, inv, "case{case}");
            assert_eq!(w.data(a).unwrap().stats.fine_xp(20), xp, "case{case}");
        }
        let (mut w, a, loc, r) = fixture(0, 0, false);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(27, None)
            .unwrap();
        let xp = w.data(a).unwrap().stats.fine_xp(20);
        start(&mut w, a, loc);
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(
                27,
                Some(Item {
                    id: 1436,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        let data = w.data(a).unwrap();
        assert_eq!(data.inventory.get(27).unwrap().id, 1436);
        assert_eq!(
            data.inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == r.output)
                .map(|i| i.count)
                .sum::<i32>(),
            54
        );
        assert_eq!(data.stats.fine_xp(20) - xp, 27 * 50 * 125 / 100);
    }
}
