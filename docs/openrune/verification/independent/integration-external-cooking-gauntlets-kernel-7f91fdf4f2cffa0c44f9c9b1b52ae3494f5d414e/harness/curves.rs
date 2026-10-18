//! Independent pinned-Wiki golden probability boundaries and actual transactions.
use gielinor_world::{
    cooking::{GauntletProfile, Gauntlets, Recipe},
    inventory::{Catalog, Inventory, Item, StackType},
    npc::RandomSource,
    stats::Stats,
};
// raw,cooked,burnt,required,guaranteed,low,high,XP,uniform successes at required/guaranteed-1.
const ROWS: [(i32, i32, i32, u8, u8, i32, i32, u16, u32, u32); 5] = [
    (377, 379, 381, 40, 64, 55, 368, 120, 181, 254),
    (371, 373, 375, 45, 80, 30, 310, 140, 157, 254),
    (7944, 7946, 7948, 62, 86, 24, 290, 150, 191, 253),
    (383, 385, 387, 80, 94, 15, 270, 210, 222, 255),
    (13439, 13441, 13443, 84, 97, 12, 260, 230, 223, 253),
];
struct Draw {
    value: u32,
    calls: usize,
}
impl RandomSource for Draw {
    fn below(&mut self, n: u32) -> u32 {
        assert_eq!(n, 256);
        self.calls += 1;
        self.value
    }
}
fn policy() -> Gauntlets {
    Gauntlets {
        item: 775,
        surfaces: [26185, 26181].into(),
        profiles: ROWS
            .into_iter()
            .map(|(raw, cooked, burnt, level, stop_burn, low, high, ..)| {
                (
                    raw,
                    GauntletProfile {
                        cooked,
                        burnt,
                        level,
                        stop_burn,
                        low,
                        high,
                    },
                )
            })
            .collect(),
    }
}
#[test]
fn five_wiki_curves_all_256_outcomes_on_both_surfaces_three_selected_slots_and_rates() {
    let g = policy();
    let items = Catalog::default();
    let mut worn = Inventory::new(14, StackType::Normal, false);
    worn.restore_slot(
        9,
        Some(Item {
            id: 775,
            count: 1,
            vars: 0,
        }),
    )
    .unwrap();
    let mut calls = 0;
    for (raw, cooked, burnt, required, guaranteed, low, high, xp, at_required, below_guarantee) in
        ROWS
    {
        let original = Recipe {
            raw,
            cooked,
            burnt,
            level: required,
            stop_burn: Some(99),
            low: 0,
            high: 0,
            xp,
            fine_xp_override: None,
            ticks: 4,
        };
        for surface in [26185, 26181] {
            let r = g.apply(original, surface, true, &worn, &items).unwrap();
            assert_eq!((r.low, r.high, r.stop_burn), (low, high, Some(guaranteed)));
            for (level, successes) in [
                (required, at_required),
                (guaranteed - 1, below_guarantee),
                (guaranteed, 256),
                (99, 256),
            ] {
                for slot in [0, 13, 27] {
                    for rate in [100, 125, 500] {
                        let mut passed = 0;
                        for value in 0..256 {
                            let mut inv = Inventory::new(28, StackType::Normal, false);
                            for s in 0..28 {
                                inv.restore_slot(
                                    s,
                                    Some(Item {
                                        id: raw,
                                        count: 1,
                                        vars: 0,
                                    }),
                                )
                                .unwrap();
                            }
                            let before = inv.clone();
                            let mut stats = Stats::default();
                            stats.set_base(7, 99);
                            stats.set_current(7, level as i8);
                            stats.set_fine_xp(7, 130344311).unwrap();
                            let old = stats.clone();
                            let mut rng = Draw { value, calls: 0 };
                            let result = r
                                .complete(slot, &mut inv, &items, &mut stats, 7, 99, rate, &mut rng)
                                .unwrap();
                            let success = value < successes;
                            assert_eq!(
                                result.burned, !success,
                                "raw{raw} surface{surface} level{level} roll{value}"
                            );
                            assert_eq!(result.output, if success { cooked } else { burnt });
                            assert_eq!(rng.calls, usize::from(level < guaranteed));
                            let mut expected = before;
                            expected
                                .restore_slot(
                                    slot,
                                    Some(Item {
                                        id: if success { cooked } else { burnt },
                                        count: 1,
                                        vars: 0,
                                    }),
                                )
                                .unwrap();
                            assert_eq!(inv, expected);
                            let mut expected = old;
                            if success {
                                expected
                                    .set_fine_xp(7, 130344311 + i32::from(xp) * 10 * rate / 100)
                                    .unwrap();
                                passed += 1;
                            }
                            assert_eq!(stats, expected);
                            calls += 1;
                        }
                        assert_eq!(passed, successes);
                    }
                }
            }
        }
    }
    assert_eq!(calls, 92160);
    println!("PG_GAUNTLET_CURVES92160 fivepinnedWikigoldenrequired/guarantee-1/guarantee/99 all256draws bothfire-range selected0/13/27 rates100/125/500 exactsuccesscounts/XP/fullbag zeroRNGatguarantee PASS");
}
#[test]
fn policy_does_not_hide_invalid_original_recipe_or_current_tool() {
    let mut cases = 0;
    for mode in 0..19 {
        let mut g = policy();
        let mut items = Catalog::default();
        let mut worn = Inventory::new(14, StackType::Normal, false);
        worn.restore_slot(
            9,
            Some(Item {
                id: 775,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut r = Recipe {
            raw: 377,
            cooked: 379,
            burnt: 381,
            level: 40,
            stop_burn: Some(99),
            low: 0,
            high: 0,
            xp: 120,
            fine_xp_override: None,
            ticks: 4,
        };
        let mut member = true;
        let mut surface = 26185;
        match mode {
            0 => member = false,
            1 => surface = 12345,
            2 => worn.restore_slot(9, None).unwrap(),
            3 => {
                worn.restore_slot(9, None).unwrap();
                worn.restore_slot(
                    8,
                    Some(Item {
                        id: 775,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            }
            4 => worn
                .restore_slot(
                    9,
                    Some(Item {
                        id: 775,
                        count: 2,
                        vars: 0,
                    }),
                )
                .unwrap(),
            5 => worn
                .restore_slot(
                    9,
                    Some(Item {
                        id: 775,
                        count: 1,
                        vars: 7,
                    }),
                )
                .unwrap(),
            6 => {
                items.dummy.insert(775);
            }
            7 => {
                items.stackable.insert(775);
            }
            8 => worn.placeholders = true,
            9 => worn.stack = StackType::Always,
            10 => r.low = -1,
            11 => r.high = -1,
            12 => r.high = 65536,
            13 => r.stop_burn = Some(39),
            14 => r.cooked = 315,
            15 => r.burnt = 323,
            16 => r.level = 39,
            17 => g.profiles.get_mut(&377).unwrap().stop_burn = 39,
            _ => g.profiles.get_mut(&377).unwrap().low = -1,
        }
        assert!(
            g.apply(r, surface, member, &worn, &items).is_none(),
            "mode{mode}"
        );
        cases += 1;
    }
    assert_eq!(cases, 19);
    println!("PG_GAUNTLET_POLICY19 member/surface/currenttool/slot/count/vars/catalog/wornshape andinvalidoriginal/profile preserveunmodifiedpolicy PASS");
}
