use gielinor_world::{
    bonuses::{self, ItemBonuses},
    inventory::{Inventory, Item, StackType},
};
use std::collections::BTreeMap;
#[test]
fn compiled_worn_bonuses_match_quiver_sets_flags_and_integer_overflow() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/bonuses.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2048);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let mut types = BTreeMap::new();
        for item in f[0].split(';') {
            let parts: Vec<_> = item.split(':').collect();
            let id: i32 = parts[0].parse().unwrap();
            let category: i32 = parts[1].parse().unwrap();
            let values: Vec<i32> = parts[4].split(',').map(|v| v.parse().unwrap()).collect();
            types.insert(
                id,
                ItemBonuses {
                    values: values.try_into().unwrap(),
                    undead_melee_only: parts[2] == "1",
                    slayer_melee_only: parts[3] == "1",
                    ignore_quiver: category == 1 || category == 2,
                    bulwark: category == 3,
                    shadow: id == 10,
                    virtus_slots: match id / 10 {
                        2 => 1,
                        3 => 1 << 4,
                        4 => 1 << 7,
                        _ => 0,
                    },
                    elite_void_slots: match id / 10 {
                        5 => 1,
                        6 => 1 << 4,
                        7 => 1 << 7,
                        8 => 1 << 9,
                        _ => 0,
                    },
                },
            );
        }
        let mut worn = Inventory::new(16, StackType::Normal, false);
        for (slot, id) in f[1]
            .split(',')
            .map(|v| v.parse::<i32>().unwrap())
            .enumerate()
        {
            if id != 0 {
                worn.restore_slot(
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
        let b = bonuses::calculate(&worn, &types, f[2] == "0");
        let mut actual = b.values.to_vec();
        actual.extend([
            b.magic_additive,
            b.magic_multiplier,
            b.undead_melee_only as i32,
            b.slayer_melee_only as i32,
            b.displayed_magic_damage(),
        ]);
        let expected: Vec<i32> = f[3].split(',').map(|v| v.parse().unwrap()).collect();
        assert_eq!(actual, expected, "case {case}");
    }
}
