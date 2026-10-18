use super::*;
use crate::runecrafting::{Config, Error, Pending, QUEUE};
fn refusal(events: &mut Vec<WorldEvent>, actor: ActorId, error: Error) {
    let text = match error {
        Error::Members => "You need to be a member to bind that essence.",
        Error::Level => "You need a higher Runecraft level to bind that essence.",
        Error::Empty => "You do not have any supported essence to bind.",
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
        let materials = match config.capture_for(recipe, &p.data.inventory) {
            Ok(v) => v,
            Err(e) => {
                refusal(&mut self.events, p.id, e);
                return;
            }
        };
        if let Err(e) = config.prepare_outcomes(
            recipe,
            &materials,
            &p.data.inventory,
            &p.data.equipment,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
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
    random: &mut impl RandomSource,
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
    match config.resolve(
        pending.recipe,
        &pending.materials,
        &p.data.inventory,
        &p.data.equipment,
        items,
        &p.data.stats,
        p.data.xp_rate_hundreds,
        p.member,
        random,
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
    const RECIPES: [(u32, i32, u8, u16, u8); 8] = [
        (34760, 556, 1, 50, 11),
        (34761, 558, 2, 55, 14),
        (34762, 555, 5, 60, 19),
        (34763, 557, 9, 65, 26),
        (34764, 554, 14, 70, 35),
        (34765, 559, 20, 75, 46),
        (34769, 562, 35, 85, 74),
        (34768, 561, 44, 90, 91),
    ];
    fn fixture(index: usize, kind: usize, member: bool) -> (World, ActorId, Location, Recipe) {
        let (id, output, level, xp_tenths, interval) = RECIPES[index];
        let recipe = Recipe {
            output,
            level,
            xp_tenths,
            interval,
            name: "rune",
            members: index >= 6,
            pure_only: index >= 6,
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
            blocked_held: [26879].into(),
            blocked_worn: [26850].into(),
            outfit: Default::default(),
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
                                id: 26879,
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
    #[test]
    fn daeyalt_actual_six_altars_full_and_mixed_priority_award_half_more_xp_not_runes() {
        for index in 0..6 {
            for mixed in [false, true] {
                let (mut w, a, loc, r) = fixture(index, 2, true);
                let count = if mixed { 7 } else { 28 };
                for slot in 0..count {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 24704,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                let xp = w.data(a).unwrap().stats.fine_xp(20);
                start(&mut w, a, loc);
                w.tick_with_random(&mut NoDraw).unwrap();
                let data = w.data(a).unwrap();
                assert_eq!(
                    data.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 24704)
                        .count(),
                    0
                );
                assert_eq!(
                    data.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| [1436, 7936].contains(&i.id))
                        .count(),
                    28 - count
                );
                assert_eq!(
                    data.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == r.output)
                        .map(|i| i.count)
                        .sum::<i32>(),
                    2 * count as i32
                );
                assert_eq!(
                    data.stats.fine_xp(20) - xp,
                    (count as i32 * i32::from(r.xp_tenths) * 3 * 125) / (2 * 100)
                );
                assert_eq!(data.inventory.free_space(), count - 1);
                if mixed {
                    start(&mut w, a, loc);
                    w.tick_with_random(&mut NoDraw).unwrap();
                    let data = w.data(a).unwrap();
                    assert_eq!(
                        data.inventory
                            .slots()
                            .iter()
                            .flatten()
                            .filter(|i| [1436, 7936].contains(&i.id))
                            .count(),
                        0
                    );
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
                    assert_eq!(
                        data.stats.fine_xp(20) - xp,
                        (7 * i32::from(r.xp_tenths) * 3 * 125) / (2 * 100)
                            + (21 * i32::from(r.xp_tenths) * 125) / 100
                    );
                }
            }
        }
    }
    #[test]
    fn daeyalt_f2p_and_late_member_or_captured_material_failure_never_fallback_to_pure() {
        for member in [false, true] {
            for case in 0..3 {
                let (mut w, a, loc, _) = fixture(1, 1, member);
                for slot in 0..7 {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(Item {
                                id: 24704,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(27, None)
                    .unwrap();
                let xp = w.data(a).unwrap().stats.fine_xp(20);
                start(&mut w, a, loc);
                if member {
                    match case {
                        0 => {
                            w.set_membership(a, false).unwrap();
                        }
                        1 => {
                            w.data_mut(a)
                                .unwrap()
                                .inventory
                                .restore_slot(
                                    3,
                                    Some(Item {
                                        id: 24704,
                                        count: 1,
                                        vars: 9,
                                    }),
                                )
                                .unwrap();
                        }
                        _ => {
                            w.data_mut(a)
                                .unwrap()
                                .inventory
                                .restore_slot(3, None)
                                .unwrap();
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
                    }
                }
                let before = w.data(a).unwrap().inventory.clone();
                for _ in 0..3 {
                    w.tick_with_random(&mut NoDraw).unwrap();
                }
                assert_eq!(w.data(a).unwrap().inventory, before);
                assert_eq!(w.data(a).unwrap().stats.fine_xp(20), xp);
            }
        }
    }
    fn outfit(w: &mut World, a: ActorId, pieces: usize, color: usize) {
        let config = w.runecrafting.as_mut().unwrap();
        config.blocked_worn.clear();
        for (group, (ids, slot)) in [
            ([26850, 26858, 26864, 26870], 0),
            ([26852, 26860, 26866, 26872], 4),
            ([26854, 26862, 26868, 26874], 7),
            ([26856; 4], 10),
        ]
        .into_iter()
        .enumerate()
        {
            for id in ids {
                config.outfit.insert(
                    id,
                    crate::runecrafting::OutfitBinding {
                        group: group as u8,
                        slot,
                    },
                );
            }
        }
        for (group, (ids, slot)) in [
            ([26850, 26858, 26864, 26870], 0),
            ([26852, 26860, 26866, 26872], 4),
            ([26854, 26862, 26868, 26874], 7),
            ([26856; 4], 10),
        ]
        .into_iter()
        .enumerate()
        {
            if group < pieces {
                w.data_mut(a)
                    .unwrap()
                    .equipment
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: ids[color],
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
        }
    }
    struct OutfitDraw {
        value: u32,
        calls: usize,
    }
    impl RandomSource for OutfitDraw {
        fn below(&mut self, bound: u32) -> u32 {
            assert_eq!(bound, 10);
            self.calls += 1;
            self.value
        }
    }
    #[test]
    fn rune_raiments_actual_six_altar_dispatch_full_outfit_both_rounding_branches_and_four_colors()
    {
        for index in 0..6 {
            for value in [0, 9] {
                let (mut w, a, loc, r) = fixture(index, 2, true);
                outfit(&mut w, a, 4, index % 4);
                let xp = w.data(a).unwrap().stats.fine_xp(20);
                let gear = w.data(a).unwrap().equipment.clone();
                start(&mut w, a, loc); // preflight uses NoDraw
                let mut rng = OutfitDraw { value, calls: 0 };
                w.tick_with_random(&mut rng).unwrap();
                let data = w.data(a).unwrap();
                assert_eq!(
                    data.inventory.get(0).unwrap().count,
                    if value == 0 { 92 } else { 86 }
                );
                assert_eq!(
                    data.stats.fine_xp(20) - xp,
                    28 * i32::from(r.xp_tenths) * 125 / 100
                );
                assert_eq!(data.equipment, gear);
                assert_eq!(rng.calls, 1);
                w.tick_with_random(&mut NoDraw).unwrap();
            }
        }
        for color in 0..4 {
            let (mut w, a, loc, _) = fixture(0, 0, true);
            outfit(&mut w, a, 1, color);
            start(&mut w, a, loc);
            let mut rng = OutfitDraw { value: 0, calls: 0 };
            w.tick_with_random(&mut rng).unwrap();
            assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().count, 62);
            assert_eq!(rng.calls, 1);
        }
    }
    #[test]
    fn rune_raiments_live_gear_member_credit_and_reward_failures_do_not_draw_or_consume() {
        for case in 0..4 {
            let (mut w, a, loc, _) = fixture(0, 0, true);
            outfit(&mut w, a, 4, 1);
            // Leave a slot for a late credit-capacity fault, while preserving captured essence.
            w.data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(27, None)
                .unwrap();
            start(&mut w, a, loc);
            match case {
                0 => w.players.get_mut(a.slot as usize).unwrap().member = false,
                1 => {
                    let d = w.data_mut(a).unwrap();
                    let mut i = d.equipment.get(0).unwrap();
                    i.vars = 1;
                    d.equipment.restore_slot(0, Some(i)).unwrap();
                }
                2 => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            27,
                            Some(Item {
                                id: 556,
                                count: i32::MAX - 84,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                3 => w.data_mut(a).unwrap().xp_rate_hundreds = 0,
                _ => unreachable!(),
            }
            let before = w.data(a).unwrap().clone();
            w.tick_with_random(&mut NoDraw).unwrap();
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().equipment, before.equipment);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
        // Gear is read live; a removed optional outfit yields ordinary runes, with no RNG.
        let (mut w, a, loc, _) = fixture(0, 0, true);
        outfit(&mut w, a, 1, 0);
        start(&mut w, a, loc);
        w.data_mut(a)
            .unwrap()
            .equipment
            .restore_slot(0, None)
            .unwrap();
        w.tick_with_random(&mut NoDraw).unwrap();
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().count, 56);
    }
    #[test]
    fn rune_member_chaos_nature_actual_dispatch_pure_only_and_live_authority() {
        for index in 6..8 {
            for kind in 0..3 {
                for member in [false, true] {
                    let (mut w, a, loc, r) = fixture(index, kind, member);
                    let before = w.data(a).unwrap().clone();
                    start(&mut w, a, loc);
                    w.tick_with_random(&mut NoDraw).unwrap();
                    let data = w.data(a).unwrap();
                    if !member || kind == 0 {
                        assert_eq!(data.inventory, before.inventory);
                        assert_eq!(data.stats, before.stats);
                    } else {
                        let consumed = if kind == 1 { 28 } else { 14 };
                        assert_eq!(
                            data.inventory
                                .slots()
                                .iter()
                                .flatten()
                                .find(|i| i.id == r.output)
                                .unwrap()
                                .count,
                            consumed * 2
                        );
                        assert_eq!(
                            data.inventory
                                .slots()
                                .iter()
                                .flatten()
                                .filter(|i| i.id == 1436)
                                .count(),
                            (28 - consumed) as usize
                        );
                        assert_eq!(
                            data.stats.fine_xp(20) - before.stats.fine_xp(20),
                            consumed * i32::from(r.xp_tenths) * 125 / 100
                        );
                    }
                }
            }
            let (mut w, a, loc, _) = fixture(index, 1, true);
            start(&mut w, a, loc);
            w.players.get_mut(a.slot as usize).unwrap().member = false;
            let before = w.data(a).unwrap().clone();
            w.tick_with_random(&mut NoDraw).unwrap();
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
    #[test]
    fn rune_member_chaos_nature_daeyalt_priority_and_outfit_remain_independent_of_ignored_rune() {
        for index in 6..8 {
            let (mut w, a, loc, r) = fixture(index, 0, true);
            outfit(&mut w, a, 4, 1);
            for slot in 0..14 {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: if slot < 7 { 24704 } else { 7936 },
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            let xp = w.data(a).unwrap().stats.fine_xp(20);
            start(&mut w, a, loc);
            let mut rng = OutfitDraw { value: 0, calls: 0 };
            w.tick_with_random(&mut rng).unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .find(|i| i.id == r.output)
                    .unwrap()
                    .count,
                26
            );
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == 7936)
                    .count(),
                7
            );
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == 1436)
                    .count(),
                14
            );
            assert_eq!(
                d.stats.fine_xp(20) - xp,
                (7.0 * f64::from(r.xp_tenths) * 1.5 * 1.25) as i32
            );
            assert_eq!(rng.calls, 1);
            let xp = d.stats.fine_xp(20);
            start(&mut w, a, loc);
            rng.calls = 0;
            w.tick_with_random(&mut rng).unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .find(|i| i.id == r.output)
                    .unwrap()
                    .count,
                52
            );
            assert_eq!(
                d.stats.fine_xp(20) - xp,
                7 * i32::from(r.xp_tenths) * 125 / 100
            );
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == 1436)
                    .count(),
                14
            );
            assert_eq!(rng.calls, 1);
            let before = d.clone();
            start(&mut w, a, loc);
            w.tick_with_random(&mut NoDraw).unwrap();
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
}
