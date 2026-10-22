use super::*;
use gielinor_world::{
    facility_crafting::Request,
    inventory::Item,
    locations::Location,
    simulation::{Action, ActorId, PlayerData, World, WorldEvent},
    stats::Stats,
    CoordGrid,
};
const JOINS: [(i32, i32, i32); 6] = [
    (11818, 11820, 11794),
    (11818, 11822, 11796),
    (11820, 11822, 11800),
    (11818, 11800, 11798),
    (11820, 11796, 11798),
    (11822, 11794, 11798),
];
struct NoDraw;
impl gielinor_world::npc::RandomSource for NoDraw {
    fn below(&mut self, _: u32) -> u32 {
        panic!("Blade forging cannot roll RNG")
    }
}
fn fixture(items: &[i32], tool: usize, rate: i32, full: bool) -> (World, ActorId, Location) {
    let (n, s, t, c, d, a) = super::tests::prepared();
    let f = godsword_forging(&n, &[("anvil".into(), 2097)].into(), &s, &t, &c, &d, &a).unwrap();
    let mut w = World::default();
    for x in (3192..3224).step_by(8) {
        for z in (3192..3224).step_by(8) {
            w.collision.allocate(x, z, 0);
        }
    }
    let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 2097, 10, 0).unwrap();
    let ty = d.locations[&2097].collision;
    w.location_types.insert(2097, ty);
    w.locations.load(loc);
    gielinor_world::loc_collision::toggle(
        &mut w.collision,
        loc.coords,
        loc.shape,
        loc.angle,
        ty,
        true,
    )
    .unwrap();
    w.object_types = d.inventory_catalog();
    w.facility_crafting.insert(2097, f.facility);
    w.facility_policies.insert(2097, f.policy);
    w.smithing = Some(gielinor_world::smithing_make::Config {
        queue: 900,
        skill: 13,
        max_level: 99,
        xp_rate: 1.0,
        tutorial: None,
    });
    for &id in &f.parts {
        w.held_use_types
            .insert(id, gielinor_world::held_use::Type { id, content: -1 });
    }
    for first in 0..f.parts.len() {
        for second in first + 1..f.parts.len() {
            w.held_use_handlers
                .register(
                    gielinor_world::held_use::Scope::Types,
                    f.parts[first],
                    f.parts[second],
                    gielinor_world::held_use::Handler::Message(
                        "These blade pieces need to be joined on an anvil.",
                    ),
                )
                .unwrap();
        }
    }
    let mut stats = Stats::default();
    stats.set_base(3, 10);
    stats.set_current(3, 10);
    stats.set_base(13, 80);
    stats.set_current(13, 80);
    let mut p = PlayerData::new(
        1,
        "Blade smith".into(),
        CoordGrid::new(3200, 3200, 0).unwrap(),
        stats,
    );
    p.xp_rate_hundreds = rate;
    if full {
        for slot in 0..28 {
            p.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 1511,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
    }
    for (&id, slot) in items.iter().zip([4, 11, 18]) {
        p.inventory
            .restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    let (id, worn) = match tool {
        0 => (2347, None),
        1 => (25644, None),
        2 => (29775, None),
        3 => (25644, Some(3)),
        _ => (29775, Some(5)),
    };
    if let Some(slot) = worn {
        p.equipment
            .restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    } else {
        p.inventory
            .restore_slot(
                27,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    w.queue_login(1, p).unwrap();
    w.tick().unwrap();
    let a = w.actors().next().unwrap();
    w.set_membership(a, true).unwrap();
    w.drain_events();
    (w, a, loc)
}
fn amount(w: &World, a: ActorId, id: i32) -> usize {
    w.data(a)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .flatten()
        .filter(|i| i.id == id)
        .count()
}
fn request(w: &mut World, a: ActorId, loc: Location, slot: usize, id: i32) -> Option<Request> {
    w.drain_events();
    w.submit(
        a,
        Action::UseLocation {
            coords: loc.coords,
            id: loc.id,
            slot: slot as i32,
            object: id,
            control: false,
        },
    )
    .unwrap();
    w.tick().unwrap();
    w.drain_events().into_iter().find_map(|e| {
        if let WorldEvent::FacilityCraftingRequested { request, .. } = e {
            Some(request)
        } else {
            None
        }
    })
}
fn finish(w: &mut World) {
    for _ in 0..8 {
        w.tick_with_random(&mut NoDraw).unwrap();
    }
}
#[test]
fn godsword_all_six_joins_have_exact_rewards_and_retain_five_hammer_forms_in_full_bags() {
    for (first, second, output) in JOINS {
        for tool in 0..5 {
            for rate in [100, 250, 500] {
                for full in [false, true] {
                    let (mut w, a, loc) = fixture(&[first, second], tool, rate, full);
                    let gear = w.data(a).unwrap().equipment.clone();
                    let before = w.data(a).unwrap().stats.clone();
                    let r = request(&mut w, a, loc, 4, first)
                        .expect("Reached blade piece must offer valid join");
                    let index = r.recipes.iter().position(|r| r.output == output).unwrap();
                    w.start_facility_crafting(a, r, index, 1).unwrap();
                    finish(&mut w);
                    assert_eq!(amount(&w, a, first), 0);
                    assert_eq!(amount(&w, a, second), 0);
                    assert_eq!(amount(&w, a, output), 1);
                    assert_eq!(w.data(a).unwrap().stats.fine_xp(13), 1000 * rate / 100);
                    for skill in 0..=22 {
                        if skill != 13 {
                            assert_eq!(
                                w.data(a).unwrap().stats.fine_xp(skill),
                                before.fine_xp(skill)
                            );
                        }
                    }
                    assert_eq!(w.data(a).unwrap().equipment, gear);
                    let hammer = if tool == 0 {
                        2347
                    } else if matches!(tool, 1 | 3) {
                        25644
                    } else {
                        29775
                    };
                    assert_eq!(amount(&w, a, hammer), usize::from(tool < 3));
                    assert_eq!(
                        amount(&w, a, 1511),
                        if full {
                            if tool < 3 {
                                25
                            } else {
                                26
                            }
                        } else {
                            0
                        }
                    );
                }
            }
        }
    }
}
#[test]
fn godsword_three_loose_shards_need_two_actual_joins_for_two_hundred_base_xp() {
    let (mut w, a, loc) = fixture(&[11818, 11820, 11822], 0, 500, false);
    let r = request(&mut w, a, loc, 4, 11818).unwrap();
    assert!(!r.recipes.iter().any(|r| r.output == 11798));
    let index = r.recipes.iter().position(|r| r.output == 11794).unwrap();
    w.start_facility_crafting(a, r, index, 1).unwrap();
    finish(&mut w);
    assert_eq!(amount(&w, a, 11794), 1);
    assert_eq!(amount(&w, a, 11822), 1);
    assert_eq!(amount(&w, a, 11798), 0);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(13), 5000);
    let slot = w
        .data(a)
        .unwrap()
        .inventory
        .slots()
        .iter()
        .position(|i| i.is_some_and(|i| i.id == 11794))
        .unwrap();
    let r = request(&mut w, a, loc, slot, 11794).unwrap();
    assert_eq!(r.recipes.len(), 1);
    w.start_facility_crafting(a, r, 0, 1).unwrap();
    finish(&mut w);
    assert_eq!(amount(&w, a, 11798), 1);
    assert_eq!(amount(&w, a, 11794) + amount(&w, a, 11822), 0);
    assert_eq!(w.data(a).unwrap().stats.fine_xp(13), 10000);
}
#[test]
fn godsword_single_partial_duplicate_and_overlapping_pieces_cannot_reforge_themselves() {
    for items in [
        vec![11818],
        vec![11794],
        vec![11796],
        vec![11800],
        vec![11794, 11796],
        vec![11818, 11794],
        vec![11820, 11794],
        vec![11818, 11818],
    ] {
        let (mut w, a, loc) = fixture(&items, 0, 100, false);
        let before = w.data(a).unwrap().clone();
        assert!(request(&mut w, a, loc, 4, items[0]).is_none());
        finish(&mut w);
        assert_eq!(w.data(a).unwrap().inventory, before.inventory);
        assert_eq!(w.data(a).unwrap().stats, before.stats);
    }
}
#[test]
fn godsword_requires_current_smithing_eighty_and_membership_but_allows_a_real_boost() {
    for case in 0..3 {
        let (mut w, a, loc) = fixture(&[11818, 11820], 0, 100, false);
        w.data_mut(a).unwrap().stats.set_base(13, 79);
        w.data_mut(a)
            .unwrap()
            .stats
            .set_current(13, if case == 0 { 79 } else { 80 });
        w.data_mut(a).unwrap().stats.set_current(12, 99);
        if case == 2 {
            w.set_membership(a, false).unwrap();
        }
        let before = w.data(a).unwrap().clone();
        let r = request(&mut w, a, loc, 4, 11818).unwrap();
        w.start_facility_crafting(a, r, 0, 1).unwrap();
        finish(&mut w);
        if case == 1 {
            assert_eq!(amount(&w, a, 11794), 1);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(13), 1000);
        } else {
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
}
#[test]
fn godsword_seven_tick_reward_and_late_tools_items_level_or_membership_are_authoritative() {
    for case in 0..6 {
        let (mut w, a, loc) = fixture(&[11818, 11820], 0, 100, false);
        let r = request(&mut w, a, loc, 4, 11818).unwrap();
        w.start_facility_crafting(a, r, 0, 1).unwrap();
        let start = w.clock.0;
        w.tick().unwrap();
        match case {
            0 => {}
            1 => w
                .data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(27, None)
                .unwrap(),
            2 => w
                .data_mut(a)
                .unwrap()
                .inventory
                .restore_slot(11, None)
                .unwrap(),
            3 => w.data_mut(a).unwrap().stats.set_current(13, 79),
            4 => w.set_membership(a, false).unwrap(),
            _ => {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(27, None)
                    .unwrap();
                w.data_mut(a)
                    .unwrap()
                    .equipment
                    .restore_slot(
                        5,
                        Some(Item {
                            id: 29775,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
        }
        let before = w.data(a).unwrap().clone();
        for _ in 1..7 {
            w.tick_with_random(&mut NoDraw).unwrap();
            assert_eq!(amount(&w, a, 11794), 0);
        }
        w.tick_with_random(&mut NoDraw).unwrap();
        assert_eq!(w.clock.0 - 1, start + 7);
        if matches!(case, 0 | 5) {
            assert_eq!(amount(&w, a, 11794), 1);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(13), 1000);
        } else {
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
        }
    }
}
#[test]
fn godsword_held_partial_feedback_is_directional_current_slot_verified_and_has_no_reward() {
    let parts = [11818, 11820, 11822, 11794, 11796, 11800];
    for first in 0..6 {
        for second in first + 1..6 {
            for reverse in [false, true] {
                let (mut w, a, _) = fixture(&[parts[first], parts[second]], 0, 100, false);
                let request = gielinor_world::held_use::Request {
                    selected_slot: if reverse { 11 } else { 4 },
                    selected_id: if reverse { parts[second] } else { parts[first] },
                    target_slot: if reverse { 4 } else { 11 },
                    target_id: if reverse { parts[first] } else { parts[second] },
                };
                let before = w.data(a).unwrap().clone();
                w.prepare_held_use(a, request).unwrap().unwrap();
                w.finish_held_use(a, request).unwrap();
                assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="These blade pieces need to be joined on an anvil.")));
                assert_eq!(w.data(a).unwrap().inventory, before.inventory);
                assert_eq!(w.data(a).unwrap().stats, before.stats);
            }
        }
    }
}

#[test]
fn godsword_failed_output_stale_material_or_life_cannot_debit_or_award() {
    for case in 0..4 {
        let (mut w, a, loc) = fixture(&[11818, 11820], 0, 500, false);
        let r = request(&mut w, a, loc, 4, 11818).unwrap();
        w.start_facility_crafting(a, r, 0, 1).unwrap();
        w.tick().unwrap();
        match case {
            0 => {
                w.object_types.dummy.insert(11794);
            }
            1 => {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        11,
                        Some(Item {
                            id: 11820,
                            count: 2,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            2 => {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        11,
                        Some(Item {
                            id: 11820,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap();
            }
            _ => w.data_mut(a).unwrap().stats.set_current(3, 0),
        }
        let before = w.data(a).unwrap().clone();
        finish(&mut w);
        if let Ok(after) = w.data(a) {
            assert_eq!(after.inventory, before.inventory, "case{case}");
            assert_eq!(after.stats, before.stats);
        }
        assert!(!w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::SmithingProductMade { .. } | WorldEvent::SkillAdvanced { .. }
        )));
    }
}
