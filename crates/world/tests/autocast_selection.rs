use gielinor_world::{
    autocast::{self, Event},
    autocast_select::{Access, Config},
    bank_actions::Binding,
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;
fn binding(id: u16) -> Binding {
    Binding {
        base: Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: id,
            lsb: 0,
            msb: 15,
        },
    }
}
fn config() -> (autocast::Config, Config) {
    let spells = [
        (
            1,
            autocast::Spell {
                item: 1001,
                requirements: vec![],
            },
        ),
        (
            2,
            autocast::Spell {
                item: 1002,
                requirements: vec![],
            },
        ),
    ]
    .into();
    (
        autocast::Config {
            enabled: binding(0),
            spell: binding(1),
            defensive: binding(2),
            staff: autocast::Saved {
                spell: binding(3),
                defensive: binding(4),
            },
            bladed: autocast::Saved {
                spell: binding(5),
                defensive: binding(6),
            },
            weapons: [
                (
                    100,
                    autocast::Weapon {
                        category: 18,
                        additional: Default::default(),
                    },
                ),
                (
                    101,
                    autocast::Weapon {
                        category: 21,
                        additional: [1002].into(),
                    },
                ),
            ]
            .into(),
            spells,
            autocast_items: [(1, 1001), (2, 1002)].into(),
            restricted: [1002].into(),
        },
        Config {
            book: binding(7),
            magic_stat: 6,
            spells: [
                (
                    1,
                    Access {
                        book: Some(0),
                        level: 50,
                    },
                ),
                (
                    2,
                    Access {
                        book: Some(1),
                        level: 20,
                    },
                ),
            ]
            .into(),
        },
    )
}
#[test]
fn selection_checks_book_staff_level_then_runes_without_changing_failed_selection() {
    let (autocast, selection) = config();
    let mut vars = Variables::default();
    binding(1).set(&mut vars, 9).unwrap();
    for (choice, weapon, book, level, expected) in [
        (2, 100, 0, 1, "spellbook"),
        (2, 100, 1, 1, "staff"),
        (1, 100, 0, 1, "Magic level"),
        (1, 100, 0, 50, "Missing rune"),
    ] {
        binding(7).set(&mut vars, book).unwrap();
        let mut called = false;
        let result = selection
            .select(
                &autocast,
                &mut vars,
                Some(weapon),
                choice,
                false,
                level,
                true,
                |_| {
                    called = true;
                    Ok(Some("Missing rune".into()))
                },
            )
            .unwrap();
        assert!(!result.restore_tab);
        assert_eq!(vars.get(1), 9);
        assert!(matches!(&result.events[..],[Event::Message(text)]if text.contains(expected)));
        assert_eq!(called, expected == "Missing rune");
    }
}
#[test]
fn successful_selection_saves_weapon_category_and_cancel_clears_only_that_category() {
    let (autocast, selection) = config();
    let mut vars = Variables::default();
    binding(7).set(&mut vars, 1).unwrap();
    binding(3).set(&mut vars, 9).unwrap();
    let result = selection
        .select(&autocast, &mut vars, Some(101), 2, true, 99, true, |_| {
            Ok(None)
        })
        .unwrap();
    assert!(result.restore_tab);
    assert_eq!(
        (
            vars.get(0),
            vars.get(1),
            vars.get(2),
            vars.get(5),
            vars.get(6)
        ),
        (1, 2, 1, 2, 1)
    );
    let order: Vec<_> = result
        .events
        .iter()
        .map(|e| match e {
            Event::Variable(v) => v.id,
            _ => panic!(),
        })
        .collect();
    assert_eq!(order, [5, 6, 1, 2, 0]);
    assert_eq!(vars.get(3), 9);
    let result = selection
        .select(&autocast, &mut vars, Some(101), 0, false, 0, true, |_| {
            panic!("cancel cannot check runes")
        })
        .unwrap();
    assert!(result.restore_tab);
    assert_eq!(
        (
            vars.get(0),
            vars.get(1),
            vars.get(2),
            vars.get(5),
            vars.get(6)
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(vars.get(3), 9);
}
#[test]
fn latest_protected_picker_request_waits_for_delay_then_preserves_defensive_choice() {
    use gielinor_world::{
        combat_wear,
        inventory::Item,
        magic, runes,
        simulation::{PlayerData, World, WorldEvent},
        stats::Stats,
        CoordGrid,
    };
    let (autocast, selection) = config();
    let mut world = World::default();
    world.combat_wear = Some(combat_wear::Config {
        autocast,
        runes: magic::Config {
            catalog: runes::Catalog::default(),
            fake: [binding(8); 2],
            fountain: binding(8),
            blighted: binding(8),
            regular_pouches: Default::default(),
            divine_pouches: Default::default(),
            pouch_types: [binding(8); 4],
            pouch_counts: [(8, binding(8)); 4],
            lms_pouch: 999,
            packs: BTreeMap::new(),
            names: magic::Names::default(),
        },
        level: binding(9),
        decimal: binding(10),
        combat_stats: [0, 1, 2, 3, 4, 6, 5],
    });
    world.autocast_selection = Some(selection);
    let mut data = PlayerData::new(
        1,
        "Caster".into(),
        CoordGrid::new(3200, 3200, 0).unwrap(),
        Stats::default(),
    );
    data.equipment
        .restore_slot(
            3,
            Some(Item {
                id: 100,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    data.stats.set_current(6, 99);
    world.queue_login(1, data).unwrap();
    world.tick().unwrap();
    let actor = world.actors().next().unwrap();
    world.drain_events();
    world.delay_player(actor, 2).unwrap();
    world.choose_autocast(actor, false).unwrap();
    world.choose_autocast(actor, true).unwrap();
    assert!(!world
        .drain_events()
        .iter()
        .any(|e| matches!(e, WorldEvent::AutocastPickerOpen { .. })));
    let mut openings = 0;
    for _ in 0..5 {
        world.tick().unwrap();
        openings += world
            .drain_events()
            .iter()
            .filter(|e| matches!(e, WorldEvent::AutocastPickerOpen { weapon: 100, .. }))
            .count();
    }
    assert_eq!(openings, 1);
    world.select_autocast(actor, 1).unwrap();
    assert_eq!(world.data(actor).unwrap().variables.get(2), 1);
    assert!(world.drain_events().iter().any(|e| matches!(
        e,
        WorldEvent::AutocastSelected {
            restore_tab: true,
            ..
        }
    )));
}
