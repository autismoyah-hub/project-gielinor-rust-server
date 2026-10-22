//! Regression for independent PG-DEF-003; exercise authoritative actions, not food helpers.
use super::*;
use crate::{consumable_clocks::{Kind, Policy, Rules}, inventory::Item};

fn fixture(native: bool, replacement: bool) -> (World, ActorId) {
    let mut world = World::default();
    world.collision.allocate(3200, 3200, 0);
    let mut stats = Stats::default();
    stats.set_base(3, 99);
    stats.set_current(3, 50);
    stats.set_base(0, 50);
    stats.set_current(0, 40);
    let mut data = PlayerData::new(1, "Food life".into(), CoordGrid::new(3200,3200,0).unwrap(), stats);
    for slot in 0..28 {
        data.inventory.restore_slot(slot, Some(Item { id: 1, count: 1, vars: 0 })).unwrap();
    }
    world.foods.insert(1, crate::food::Edible { id: 1, heal: 2, delay: 3,
        replacement: replacement.then_some(2), message: "Eat".into() });
    world.slot_preserving_foods.insert(1);
    if replacement {
        world.food_subtractions.insert(1, vec![crate::food::StatSubtraction { skill: 0, amount: 2 }]);
    }
    if native {
        world.consumable_rules = Some(Rules { foods: BTreeMap::from([(1, Policy {
            kind: Kind::Food, consume_delay: 3, combat_delay: 3, members: false,
        })]), potions: BTreeMap::new() });
    }
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    (world, actor)
}

fn eat(slot: usize, selected: bool) -> Action {
    if selected { Action::EatItemSlot { slot, item: 1 } } else { Action::EatSlot { slot } }
}

#[test]
fn food_life_refuses_zero_hp_and_unresolved_death_even_after_visible_hp_restoration() {
    let mut cases = 0;
    for native in [false, true] { for replacement in [false, true] {
        for selected in [false, true] { for slot in [0, 13, 27] { for phase in 0..3 {
            let (mut world, actor) = fixture(native, replacement);
            if phase == 0 {
                world.data_mut(actor).unwrap().stats.set_current(3, 0);
            } else {
                world.queue_hit(crate::hits::Target::Player(actor), crate::hits::Hit {
                    display: None, source: None, style: crate::hits::Style::Typeless, damage: 99,
                }, 0).unwrap();
                world.tick().unwrap();
                assert!(world.drain_events().iter().any(|event| matches!(event,
                    WorldEvent::DeathRequested(crate::hits::Target::Player(id)) if *id == actor)));
                if phase == 2 { world.data_mut(actor).unwrap().stats.set_current(3, 10); }
            }
            let before = world.data(actor).unwrap().clone();
            let clocks = world.players.get(actor.slot as usize).unwrap().consumables;
            world.submit(actor, eat(slot, selected)).unwrap();
            world.tick().unwrap();
            let player = world.players.get(actor.slot as usize).unwrap();
            assert_eq!(player.data.inventory.slots(), before.inventory.slots(), "case {cases}");
            assert_eq!(player.data.stats, before.stats, "case {cases}");
            assert_eq!(player.consumables, clocks, "case {cases}");
            assert!(!player.food_delay, "case {cases}");
            assert!(!world.drain_events().iter().any(|event| matches!(event,
                WorldEvent::Food { .. } | WorldEvent::FoodTransition { .. })), "case {cases}");
            cases += 1;
        }}}}
    }
    assert_eq!(cases, 72);
}

#[test]
fn food_life_live_controls_still_consume_selected_food_and_apply_replacement_effects() {
    for native in [false, true] { for replacement in [false, true] { for selected in [false, true] {
        let (mut world, actor) = fixture(native, replacement);
        world.submit(actor, eat(13, selected)).unwrap();
        world.tick().unwrap();
        let data = world.data(actor).unwrap();
        assert_eq!(data.inventory.get(13).map(|item| item.id), replacement.then_some(2));
        assert_eq!(data.inventory.get(0).unwrap().id, 1);
        assert_eq!(data.stats.current(3), 52);
        assert_eq!(data.stats.current(0), if replacement { 38 } else { 40 });
        assert!(world.drain_events().iter().any(|event| matches!(event, WorldEvent::Food { .. })));
    }}}
}
