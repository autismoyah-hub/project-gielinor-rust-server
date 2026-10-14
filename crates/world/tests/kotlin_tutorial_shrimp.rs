use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    simulation::PlayerData,
    stats::Stats,
    tutorial_progression::{Config as Progression, Effect},
    tutorial_shrimp::Config,
    variables::{Transmit, Variables, Varp},
    CoordGrid,
};
#[test]
fn unchanged_tutorial_skills_shrimp_matches_guards_inventory_and_effects() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-shrimp.expected"
    ))
    .expect("Run scripts/tutorial-shrimp-parity.ps1");
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
        raw: 24652,
        cooked: 315,
        animation: 3,
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
                        id: 24652,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        p.variables = Variables::restore([(1, num(0)), (2, num(1))].into());
        let result = config.cook(&mut p, &Catalog::default(), true).unwrap();
        let mut effects: Vec<String> = vec![];
        if let Some(result) = result {
            assert_eq!(result.advancement.added_xp, 30);
            effects.extend([
                "anim:3".into(),
                "xp:7:30.0".into(),
                "You manage to cook some shrimp.".into(),
            ]);
            for effect in result.progression {
                if let Effect::Focus { step, .. } = effect {
                    effects.push(format!("focus:{}", step as i32));
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
