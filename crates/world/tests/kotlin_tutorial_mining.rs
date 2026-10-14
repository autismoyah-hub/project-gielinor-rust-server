use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    simulation::PlayerData,
    stats::Stats,
    tutorial_mining::{Config, Ore, Outcome},
    tutorial_progression::{Config as Progression, Effect},
    variables::{Transmit, Variables, Varp},
    CoordGrid,
};
#[test]
fn unchanged_tutorial_skills_mining_matches_guards_inventory_and_effects() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-mining.expected"
    ))
    .expect("Run scripts/tutorial-mining-parity.ps1");
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
        tin_rock: 1,
        copper_rock: 2,
        tin: 438,
        copper: 436,
        pickaxe: 1265,
        animation: 5,
        skill: 14,
        max_level: 99,
        global_xp_rate: 1.0,
    };
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 8);
        let num = |i: usize| f[i].parse::<i32>().unwrap();
        let mut p = PlayerData::new(
            1,
            "Oracle".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.inventory = Inventory::new(num(2) as usize, StackType::Normal, false);
        let mut slot = 0;
        for (bit, id) in [1265, 438, 436].into_iter().enumerate() {
            if num(3) & (1 << bit) != 0 {
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
                slot += 1;
            }
        }
        p.variables = Variables::restore([(1, num(0)), (2, num(1))].into());
        let result = config
            .mine(
                &mut p,
                &Catalog::default(),
                if num(4) == 0 { Ore::Tin } else { Ore::Copper },
                true,
            )
            .unwrap();
        let mut effects: Vec<String> = vec![];
        match result {
            Outcome::Denied(message) => effects.push(message.into()),
            Outcome::Mined {
                advancement,
                progression,
                message,
            } => {
                assert_eq!(advancement.added_xp, 17);
                assert_eq!(p.stats.fine_xp(14), 175);
                effects.extend(["anim:5".into(), "xp:14:17.5".into(), message.into()]);
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
        assert_eq!(p.variables.get(1), num(5), "{line}");
        assert_eq!(slots, f[6], "{line}");
        assert_eq!(effects.join("~"), f[7], "{line}");
        count += 1;
    }
    assert_eq!(count, 7168);
}
