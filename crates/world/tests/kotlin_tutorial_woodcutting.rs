use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    simulation::PlayerData,
    stats::Stats,
    tutorial_progression::{Config as Progression, Effect},
    tutorial_woodcutting::{Config, Outcome, OAK},
    variables::{Transmit, Variables, Varp},
    CoordGrid,
};
#[test]
fn unchanged_tutorial_woodcutting_and_oak_match_progress_and_inventory_guards() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-woodcutting.expected"
    ))
    .expect("Run scripts/tutorial-woodcutting-parity.ps1");
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        progression: Progression {
            step: varp(1),
            completed: varp(2),
        },
        trees: [1, 2],
        oak: 3,
        logs: 24650,
        animation: 879,
        skill: 8,
        max_level: 99,
        global_xp_rate: 1.0,
    };
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 7);
        let num = |i: usize| f[i].parse::<i32>().unwrap();
        let mode = num(2);
        let mut p = PlayerData::new(
            1,
            "Oracle".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.inventory = Inventory::new(1, StackType::Normal, false);
        if mode != 0 {
            p.inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: if mode == 2 { 999 } else { 24650 },
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        p.variables = Variables::restore([(1, num(0)), (2, num(1))].into());
        let mut effects: Vec<String> = vec![];
        let result = if num(3) == 1 {
            Outcome::Denied(OAK)
        } else {
            config.chop(&mut p, &Catalog::default(), true).unwrap()
        };
        match result {
            Outcome::Denied(message) => effects.push(message.into()),
            Outcome::Chopped {
                advancement,
                progression,
            } => {
                assert_eq!(advancement.added_xp, 25);
                effects.extend([
                    "anim:879".into(),
                    "xp:8:25.0".into(),
                    "You manage to cut some logs.".into(),
                ]);
                for effect in progression {
                    if let Effect::Focus { step, .. } = effect {
                        effects.push(format!("focus:{}", step as i32));
                    }
                }
            }
        }
        let slots = p
            .inventory
            .slots()
            .iter()
            .map(|i| i.map_or(-1, |i| i.id).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(p.variables.get(1), num(4), "{line}");
        assert_eq!(slots, f[5], "{line}");
        assert_eq!(effects.join("~"), f[6], "{line}");
        count += 1;
    }
    assert_eq!(count, 1344);
}
