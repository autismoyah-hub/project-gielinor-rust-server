use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    tutorial_dough::Config,
    tutorial_progression::{Config as Progression, Effect},
    variables::{Transmit, Variables, Varp},
};

#[test]
fn original_tutorial_skills_dough_matches_inventory_and_progression() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-dough.expected"
    ))
    .expect("Run scripts/tutorial-dough-parity.ps1");
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
        flour: 1933,
        water: 1929,
        pot: 1931,
        bucket: 1925,
        dough: 2307,
    };
    let mut count = 0;
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        assert_eq!(f.len(), 7);
        let num = |i: usize| f[i].parse::<i32>().unwrap();
        let mut inv = Inventory::new(num(2) as usize, StackType::Normal, false);
        for (bit, slot, id) in [(1, 0, 1933), (2, 1, 1929)] {
            if num(3) & bit != 0 {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
        }
        let mut vars = Variables::restore([(1, num(0)), (2, num(1))].into());
        let result = config.mix(&mut inv, &Catalog::default(), &mut vars, true);
        let mut messages = Vec::new();
        if let Some(result) = result {
            messages.push(result.message.to_string());
            for effect in result.progression {
                if let Effect::Focus { step, .. } = effect {
                    messages.push(format!("focus:{}", step as i32));
                }
            }
        }
        let slots = inv
            .slots()
            .iter()
            .map(|i| i.map_or(-1, |i| i.id).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(vars.get(1), num(4), "{line}");
        assert_eq!(slots, f[5], "{line}");
        assert_eq!(messages.join("~"), f[6], "{line}");
        count += 1;
    }
    assert_eq!(count, 2688);
}
