use gielinor_world::{
    anvil::{Config, Open},
    anvil_products,
    inventory::{Catalog, Inventory, Item, StackType},
    stats::Stats,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn original_anvil_tables_and_best_tier() {
    let mut config = Config {
        hammer: 2347,
        skill: 13,
        animation: 898,
        bar_by_tier: anvil_products::TIERS.iter().copied().collect(),
        products: BTreeMap::new(),
        quantity: BTreeMap::new(),
        bars: BTreeMap::new(),
        level: BTreeMap::new(),
        xp_per_bar: anvil_products::XP_PER_BAR.iter().copied().collect(),
        names: BTreeMap::new(),
    };
    for &(bar, slot, product) in anvil_products::PRODUCTS {
        config
            .products
            .entry(bar)
            .or_default()
            .insert(slot, product);
    }
    let bar_ids: Vec<_> = config.bar_by_tier.values().copied().collect();
    let products: BTreeSet<_> = config
        .products
        .values()
        .flat_map(|grid| grid.values().copied())
        .collect();
    let mut tiers = BTreeMap::new();
    let mut grid = BTreeMap::<i32, BTreeMap<u16, i32>>::new();
    let mut xp = BTreeMap::new();
    let mut cases = 0;
    for line in include_str!("fixtures/anvil-tier.expected").lines() {
        let fields: Vec<_> = line.split('|').collect();
        let n = |i: usize| fields[i].parse::<i32>().unwrap();
        match fields[0] {
            "T" => {
                assert!(tiers.insert(n(1), n(2)).is_none());
            }
            "G" => {
                assert!(grid
                    .entry(n(1))
                    .or_default()
                    .insert(n(2) as u16, n(3))
                    .is_none());
            }
            "X" => {
                assert!(xp.insert(n(1), fields[2].parse::<f64>().unwrap()).is_none());
            }
            "B" => {
                config.names = bar_ids
                    .iter()
                    .filter(|&&id| n(3) == 0 || id != 13354)
                    .map(|&id| (id, format!("bar{id}")))
                    .collect();
                config.level = products
                    .iter()
                    .filter(|&&id| n(4) == 0 || id % 3 != 0)
                    .map(|&id| (id, id % 100))
                    .collect();
                let mut inv = Inventory::new(8, StackType::Normal, false);
                for (index, &bar) in bar_ids.iter().enumerate() {
                    if n(1) & (1 << index) != 0 {
                        inv.restore_slot(
                            index,
                            Some(Item {
                                id: bar,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                    }
                }
                // The oracle isolates bestTier; prepare performs its hammer guard first.
                inv.restore_slot(
                    7,
                    Some(Item {
                        id: 2347,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
                let mut stats = Stats::default();
                stats.set_current(13, n(2) as i8);
                let actual = match config.prepare(&inv, &Catalog::default(), &stats, None) {
                    Open::Tier(tier) => tier,
                    Open::Denied(message) => {
                        assert_eq!(message, "You don't have any bars to smith with.");
                        -1
                    }
                };
                assert_eq!(actual, n(5), "{line}");
                cases += 1;
            }
            _ => panic!("Unknown oracle row"),
        }
    }
    assert_eq!(cases, 4096);
    assert_eq!(tiers, config.bar_by_tier);
    assert_eq!(grid, config.products);
    assert_eq!(xp, config.xp_per_bar);
    assert_eq!(grid.values().map(|v| v.len()).sum::<usize>(), 188);
}
