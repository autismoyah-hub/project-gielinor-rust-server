//! Shared NPC drop sub-tables: herb, gem, mega-rare and rare drop tables, in OpenRune
//! row order.
//!
//! Herb and mega-rare follow OpenRune `HerbDropTable.kt`/`MegaRareDropTable.kt` exactly
//! (both /128, as on the Wiki). Gem and rare follow the Wiki where OpenRune differs:
//! - Gem drop table (rev 15167578) is /128 with one location-dependent talisman row
//!   (chaos underground, nature overworld). OpenRune weights both talismans (/131) and
//!   drops the wrong-location one as nothing.
//! - Rare drop table (rev 15339986) has loop and tooth key halves at 20/128 each.
//!   OpenRune has 2 each, so its table totals 92 despite citing the Wiki's /128.
//!
//! Ring of wealth removes only shared gem/mega-rare Nothing rows at roll time.
//! Not modelled: the Legends' Quest talisman
//! replacing mega-rare via the gem table, and the F2P herb roll becoming 10 coins.
use gielinor_cache::symbols::Symbols;
use gielinor_world::npc_loot::{Outcome, Product, Row, Weighted};

/// A rev-239 `obj.sym` name, its id and the count.
pub type Item = (&'static str, u16, i32);

pub enum Spec {
    Nothing(u32),
    Item(u32, Item),
    Noted(u32, Item),
    /// Weight, overworld item, underground item.
    ByLocation(u32, Item, Item),
    Gem(u32),
    MegaRare(u32),
}
use Spec::*;

pub const HERB: &[Spec] = &[
    Item(32, ("unidentified_guam", 199, 1)),
    Item(24, ("unidentified_marentill", 201, 1)),
    Item(18, ("unidentified_tarromin", 203, 1)),
    Item(14, ("unidentified_harralander", 205, 1)),
    Item(11, ("unidentified_ranarr", 207, 1)),
    Item(8, ("unidentified_irit", 209, 1)),
    Item(6, ("unidentified_avantoe", 211, 1)),
    Item(5, ("unidentified_kwuarm", 213, 1)),
    Item(4, ("unidentified_cadantine", 215, 1)),
    Item(3, ("unidentified_lantadyme", 2485, 1)),
    Item(3, ("unidentified_dwarf_weed", 217, 1)),
];
pub const MEGA_RARE: &[Spec] = &[
    Nothing(113),
    Item(8, ("rune_spear", 1247, 1)),
    Item(4, ("dragonshield_a", 2366, 1)),
    Item(3, ("dragon_spear", 1249, 1)),
];
pub const GEM: &[Spec] = &[
    Nothing(63),
    Item(32, ("uncut_sapphire", 1623, 1)),
    Item(16, ("uncut_emerald", 1621, 1)),
    Item(8, ("uncut_ruby", 1619, 1)),
    ByLocation(3, ("nature_talisman", 1462, 1), ("chaos_talisman", 1452, 1)),
    Item(2, ("uncut_diamond", 1617, 1)),
    Item(1, ("rune_javelin", 830, 5)),
    Item(1, ("keyhalf2", 987, 1)),
    Item(1, ("keyhalf1", 985, 1)),
    MegaRare(1),
];
pub const RARE: &[Spec] = &[
    Item(21, ("coins", 995, 3000)),
    Gem(20),
    MegaRare(15),
    Item(5, ("runite_bar", 2363, 1)),
    Item(3, ("naturerune", 561, 67)),
    Item(3, ("rune_2h_sword", 1319, 1)),
    Item(3, ("rune_battleaxe", 1373, 1)),
    Item(2, ("adamant_javelin", 829, 20)),
    Item(2, ("deathrune", 560, 45)),
    Item(2, ("lawrune", 563, 45)),
    Item(2, ("rune_arrow", 892, 42)),
    Item(2, ("steel_arrow", 886, 150)),
    Item(2, ("rune_sq_shield", 1185, 1)),
    Item(20, ("keyhalf2", 987, 1)),
    Item(20, ("keyhalf1", 985, 1)),
    Item(2, ("dragonstone", 1615, 1)),
    Noted(2, ("cert_silver_ore", 443, 100)),
    Item(1, ("dragon_med_helm", 1149, 1)),
    Item(1, ("rune_kiteshield", 1201, 1)),
];

#[derive(Clone, Debug)]
pub struct SharedTables {
    pub herb: Weighted,
    pub mega_rare: Weighted,
    pub gem: Weighted,
    pub rare: Weighted,
}

fn product(objects: &Symbols, (name, id, count): Item) -> Result<Product, String> {
    let symbol = objects.get(name).copied();
    if symbol != Some(i32::from(id)) {
        return Err(format!(
            "shared drop tables: obj {name} is {symbol:?} in rev-239 symbols, expected {id}"
        ));
    }
    Ok(Product { item: id, count })
}

fn table(
    objects: &Symbols,
    specs: &[Spec],
    gem: Option<&Weighted>,
    mega_rare: Option<&Weighted>,
) -> Result<Weighted, String> {
    let mut rows = vec![];
    for spec in specs {
        let (weight, outcome) = match *spec {
            Nothing(w) => (w, Outcome::Nothing),
            Item(w, item) => (w, Outcome::Product(product(objects, item)?)),
            Noted(w, item) => (w, Outcome::Noted(product(objects, item)?)),
            ByLocation(w, over, under) => (
                w,
                Outcome::ByLocation {
                    overworld: product(objects, over)?,
                    underground: product(objects, under)?,
                },
            ),
            Gem(w) => (w, Outcome::Table(gem.ok_or("gem table not resolved")?.clone())),
            MegaRare(w) => (
                w,
                Outcome::Table(mega_rare.ok_or("mega-rare table not resolved")?.clone()),
            ),
        };
        rows.push(Row { weight, outcome });
    }
    let total = rows.iter().map(|r| r.weight).sum();
    Weighted::with_rows(total, rows).map_err(|e| format!("shared drop table: {e:?}"))
}

pub fn resolve(objects: &Symbols) -> Result<SharedTables, String> {
    let herb = table(objects, HERB, None, None)?;
    let mega_rare = table(objects, MEGA_RARE, None, None)?
        .with_wealth_without_nothing().map_err(|e| format!("wealth mega-rare: {e:?}"))?;
    let gem = table(objects, GEM, None, Some(&mega_rare))?
        .with_wealth_without_nothing().map_err(|e| format!("wealth gem: {e:?}"))?;
    let rare = table(objects, RARE, Some(&gem), Some(&mega_rare))?;
    Ok(SharedTables {
        herb,
        mega_rare,
        gem,
        rare,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        npc::RandomSource,
        npc_death::DropPolicy,
        npc_loot::{Location, Profile},
    };
    use std::collections::{BTreeMap, VecDeque};

    struct Script(VecDeque<u32>, Vec<u32>);
    impl RandomSource for Script {
        fn below(&mut self, bound: u32) -> u32 {
            self.1.push(bound);
            let draw = self.0.pop_front().expect("unscripted draw");
            assert!(draw < bound);
            draw
        }
    }
    fn items(specs: &[Spec]) -> Vec<Item> {
        specs
            .iter()
            .flat_map(|s| match *s {
                Item(_, i) | Noted(_, i) => vec![i],
                ByLocation(_, a, b) => vec![a, b],
                _ => vec![],
            })
            .collect()
    }
    fn symbols() -> Symbols {
        let mut s = Symbols::new();
        for (name, id, _) in [HERB, MEGA_RARE, GEM, RARE].into_iter().flat_map(items) {
            s.insert(name.into(), i32::from(id));
        }
        s
    }
    /// Expected products per row weight, expanding a sub-table row through `inner`.
    fn roll_all(table: &Weighted, inner: impl Fn(u32) -> Vec<u32>, at: Location) -> BTreeMap<Vec<(u16, i32)>, u32> {
        let p = Profile::new(DropPolicy::Imp, vec![], Some(table.clone())).unwrap();
        let mut out = BTreeMap::new();
        for draw in 0..table.total() {
            let nested = inner(draw);
            let scripts = if nested.is_empty() {
                vec![vec![draw]]
            } else {
                nested.into_iter().map(|d| vec![draw, d]).collect()
            };
            for script in scripts {
                let mut random = Script(script.into(), vec![]);
                let rolled = p.roll(&mut random, at);
                assert!(random.0.is_empty(), "every scripted draw is consumed");
                *out.entry(rolled.products).or_insert(0) += 1;
            }
        }
        out
    }
    fn weight(m: &BTreeMap<Vec<(u16, i32)>, u32>, item: u16, count: i32) -> u32 {
        m.get(&vec![(item, count)]).copied().unwrap_or(0)
    }

    #[test]
    fn herb_and_mega_rare_match_openrune_weights() {
        let t = resolve(&symbols()).unwrap();
        assert_eq!((t.herb.total(), t.mega_rare.total()), (128, 128));
        let herb = roll_all(&t.herb, |_| vec![], Location::Overworld);
        for (w, (_, id, n)) in HERB.iter().map(|s| match *s { Item(w, i) => (w, i), _ => unreachable!() }) {
            assert_eq!(weight(&herb, id, n), w, "herb {id}");
        }
        let mega = roll_all(&t.mega_rare, |_| vec![], Location::Overworld);
        assert_eq!(mega[&vec![]], 113);
        assert_eq!((weight(&mega, 1247, 1), weight(&mega, 2366, 1), weight(&mega, 1249, 1)), (8, 4, 3));
    }

    #[test]
    fn gem_table_is_wiki_128_with_a_location_talisman_and_nested_mega_rare() {
        let t = resolve(&symbols()).unwrap();
        assert_eq!(t.gem.total(), 128);
        // Draw 127 is the last row (mega-rare), which then rolls its own /128.
        let mega = |d| if d == 127 { (0..128).collect() } else { vec![] };
        for (at, wanted, other) in [(Location::Overworld, 1462, 1452), (Location::Underground, 1452, 1462)] {
            let gem = roll_all(&t.gem, mega, at);
            assert_eq!(weight(&gem, wanted, 1), 3);
            assert_eq!(weight(&gem, other, 1), 0);
            for (id, n, w) in [(1623, 1, 32), (1621, 1, 16), (1619, 1, 8), (1617, 1, 2), (830, 5, 1), (987, 1, 1), (985, 1, 1)] {
                assert_eq!(weight(&gem, id, n), w, "gem {id}");
            }
            // 63 direct nothing draws, plus 113 of the 128 mega-rare draws on the 1/128 row.
            assert_eq!(gem[&vec![]], 63 + 113);
            assert_eq!(weight(&gem, 1247, 1), 8, "rune spear via 1/128 then 8/128");
        }
        // OpenRune row order: nothing 0-62, sapphire 63-94, emerald, ruby, then the talisman at 119.
        let p = Profile::new(DropPolicy::Imp, vec![], Some(t.gem.clone())).unwrap();
        for (draw, item) in [(63, 1623), (95, 1621), (111, 1619), (119, 1462), (122, 1617), (126, 985)] {
            let mut random = Script([draw].into(), vec![]);
            assert_eq!(p.roll(&mut random, Location::Overworld).products, [(item, if item == 830 { 5 } else { 1 })]);
        }
    }

    #[test]
    fn rare_drop_table_is_wiki_128_with_gem_and_mega_rare_sub_tables() {
        let t = resolve(&symbols()).unwrap();
        assert_eq!(t.rare.total(), 128);
        let p = Profile::new(DropPolicy::Imp, vec![], Some(t.rare.clone())).unwrap();
        let mut direct = BTreeMap::new();
        let mut delegated = BTreeMap::new();
        for draw in 0..128 {
            let mut random = Script([draw, 63].into(), vec![]);
            let rolled = p.roll(&mut random, Location::Overworld);
            if random.1.len() == 1 {
                *direct.entry(rolled.products[0]).or_insert(0u32) += 1;
            } else {
                *delegated.entry(rolled.products.first().copied()).or_insert(0u32) += 1;
            }
        }
        for spec in RARE {
            if let Item(w, (_, id, n)) | Noted(w, (_, id, n)) = *spec {
                assert_eq!(direct[&(id, n)], w, "rare {id}");
            }
        }
        // Sub-table draw 63: gem's first sapphire row (20 draws), mega-rare's nothing (15).
        assert_eq!(delegated, [(None, 15), (Some((1623, 1)), 20)].into());
        // OpenRune row order: coins 0-20, gem 21-40, mega-rare 41-55.
        for (script, products) in [([20, 0], vec![(995, 3000)]), ([21, 63], vec![(1623, 1)]), ([41, 113], vec![(1247, 1)])] {
            let mut random = Script(script.into(), vec![]);
            assert_eq!(p.roll(&mut random, Location::Overworld).products, products);
        }
    }

    #[test]
    fn symbols_must_match_the_expected_ids() {
        let mut s = symbols();
        s.insert("keyhalf1".into(), 986);
        assert!(resolve(&s).unwrap_err().contains("keyhalf1"));
        s.remove("keyhalf1");
        assert!(resolve(&s).is_err());
    }

    #[test]
    fn wealth_shared_boundaries_remove_only_empty_weight_and_keep_location() {
        use gielinor_world::npc_loot::Context;
        let t = resolve(&symbols()).unwrap();
        for location in [Location::Overworld, Location::Underground] {
            let context = Context { location, wearing_wealth: true };
            assert_eq!((t.gem.total_with_context(context), t.mega_rare.total_with_context(context), t.rare.total_with_context(context)), (65, 15, 128));
            let gem = Profile::new(DropPolicy::Imp, vec![], Some(t.gem.clone())).unwrap();
            let boundaries = [(0,1623,1),(31,1623,1),(32,1621,1),(47,1621,1),
                (48,1619,1),(55,1619,1),(56,if location == Location::Underground {1452} else {1462},1),
                (58,if location == Location::Underground {1452} else {1462},1),
                (59,1617,1),(60,1617,1),(61,830,5),(62,987,1),(63,985,1)];
            for (draw,item,count) in boundaries {
                let mut random = Script([draw].into(), vec![]);
                assert_eq!(gem.roll_with_context(&mut random, context).products, [(item,count)]);
                assert_eq!(random.1, [65]);
            }
            let mega = Profile::new(DropPolicy::Imp, vec![], Some(t.mega_rare.clone())).unwrap();
            for (draw,item) in [(0,1247),(7,1247),(8,2366),(11,2366),(12,1249),(14,1249)] {
                let mut random = Script([draw].into(), vec![]);
                assert_eq!(mega.roll_with_context(&mut random, context).products, [(item,1)]);
                assert_eq!(random.1, [15]);
            }
        }
    }

    #[test]
    fn wealth_context_survives_rare_gem_mega_and_leaves_outer_denominator_unchanged() {
        use gielinor_world::npc_loot::Context;
        let t = resolve(&symbols()).unwrap();
        let p = Profile::new(DropPolicy::Imp, vec![], Some(t.rare)).unwrap();
        for (script, bounds, products) in [
            (vec![20],vec![128],vec![(995,3000)]),
            (vec![21,0],vec![128,65],vec![(1623,1)]),
            (vec![21,64,14],vec![128,65,15],vec![(1249,1)]),
            (vec![41,14],vec![128,15],vec![(1249,1)]),
        ] {
            let mut random = Script(script.into(), vec![]);
            let result = p.roll_with_context(&mut random, Context { location: Location::Overworld, wearing_wealth: true });
            assert_eq!(result.products, products);
            assert_eq!(random.1, bounds);
            assert!(random.0.is_empty());
        }
        let mut random = Script([21,0].into(), vec![]);
        assert!(p.roll(&mut random, Location::Overworld).products.is_empty());
        assert_eq!(random.1, [128,128], "normal Nothing remains with no wealth context");
    }

    #[test]
    fn wealth_leaves_every_existing_shared_herb_draw_unchanged() {
        use gielinor_world::npc_loot::Context;
        let table = resolve(&symbols()).unwrap().herb;
        let profile = Profile::new(DropPolicy::Imp, vec![], Some(table)).unwrap();
        for draw in 0..128 {
            let mut normal = Script([draw].into(), vec![]);
            let mut wealth = Script([draw].into(), vec![]);
            assert_eq!(profile.roll(&mut normal, Location::Overworld),
                profile.roll_with_context(&mut wealth, Context { location: Location::Overworld, wearing_wealth: true }));
            assert_eq!(normal.1, wealth.1);
            assert_eq!(wealth.1, [128]);
        }
    }
}
