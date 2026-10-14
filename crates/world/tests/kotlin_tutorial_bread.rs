use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    simulation::PlayerData,
    stats::Stats,
    tutorial_bread::{Config, Outcome},
    tutorial_progression::{Config as Progression, Effect},
    variables::{Transmit, Variables, Varp},
    CoordGrid,
};
#[test]
fn unchanged_tutorial_skills_bread_matches_guards_inventory_and_effects() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-bread.expected"
    ))
    .expect("Run scripts/tutorial-bread-parity.ps1");
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
        location: 1,
        dough: 2307,
        bread: 2309,
        animation: 4,
        skill: 7,
        max_level: 99,
        global_xp_rate: 1.0,
    };
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 7);
        let num = |i: usize| f[i].parse::<i32>().unwrap();
        let mut p = PlayerData::new(
            1,
            "Oracle".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.inventory = Inventory::new(num(2) as usize, StackType::Normal, false);
        if num(3) == 1 {
            p.inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: 2307,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        p.variables = Variables::restore([(1, num(0)), (2, num(1))].into());
        let result = config.cook(&mut p, &Catalog::default(), true).unwrap();
        let mut effects: Vec<String> = vec![];
        match result {
            Outcome::Silent => {}
            Outcome::Missing => effects.push(gielinor_world::tutorial_bread::MISSING.into()),
            Outcome::Cooked {
                advancement,
                progression,
            } => {
                assert_eq!(advancement.added_xp, 40);
                effects.extend([
                    "anim:4".into(),
                    "xp:7:40.0".into(),
                    "You manage to bake some bread.".into(),
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
    assert_eq!(count, 896);
}
