//! Ordinary goblin drops: Wiki Goblin "Drop table 2" (rev 15374683) and Goblin (Goblin
//! Village) (rev 15374680); behaviour from OpenRune `GoblinDropTable.kt` and
//! `GoblinGoblinVillageDropTable.kt`. Bones are guaranteed, then one /128 main roll (two
//! rows delegate to the shared herb table), then independent tertiaries.
//!
//! Decisions where the sources differ:
//! - Goblin mail colour follows the dying NPC: plain goblin mail for drop table 2, red or
//!   green goblin mail for the matching Goblin Village soldiers (Wiki). OpenRune weights
//!   both village colours with an always-true condition and no nothing row, so its village
//!   table totals 130; the Wiki's 8/128 nothing row brings each colour back to 128.
//! - Coins follow 239.1 (cache objs end at Aug-2025 releases), as in OpenRune: 1 coin at
//!   34 and 3 coins at 13. The Wiki's Aug/Sep 2026 energy potion and 8-coin rows are later.
//!
//! Not modelled: Rag and Bone Man goblin skull, the goblin champion scroll (needs 32 quest
//! points), the easy clue the Wiki lists for free-to-play worlds, F2P substitutions (10
//! coins for herbs, nothing for members items), and the one-beginner-clue-at-a-time rule.
use super::{Definitions, Symbols};
use super::shared_drop_tables::Item;
use gielinor_world::{
    npc_death::{Config, DropPolicy},
    npc_loot::{Outcome, Product, Profile, Row, Tertiary, Weighted},
};
use std::collections::BTreeMap;

/// Drop table 2 (Wiki ids 5192,5193,5204-5208; 3045,3073-3076; 3046), OpenRune order.
pub const TABLE_2: [(&str, u16); 13] = [
    ("fai_barbarian_goblin_armed_1", 3073),
    ("fai_barbarian_goblin_armed_2", 3074),
    ("fai_barbarian_goblin_armed_3", 3075),
    ("fai_barbarian_goblin_armed_4", 3076),
    ("goblin_armed", 3045),
    ("goblin_armed_mcannon_1", 5204),
    ("goblin_armed_mcannon_2", 5205),
    ("goblin_armed_mcannon_3", 5206),
    ("goblin_armed_mcannon_4", 5207),
    ("goblin_armed_mcannon_5", 5208),
    ("goblin_helmet", 3046),
    ("mcannon_goblin1", 5192),
    ("mcannon_goblin_guard", 5193),
];
/// Goblin Village soldiers 2-8 (Wiki red 655-661, green 662-668). Soldier 1 is not listed.
pub const RED: [(&str, u16); 7] = [
    ("goblin_red_soldier_2", 655),
    ("goblin_red_soldier_3", 656),
    ("goblin_red_soldier_4", 657),
    ("goblin_red_soldier_5", 658),
    ("goblin_red_soldier_6", 659),
    ("goblin_red_soldier_7", 660),
    ("goblin_red_soldier_8", 661),
];
pub const GREEN: [(&str, u16); 7] = [
    ("goblin_green_soldier_2", 662),
    ("goblin_green_soldier_3", 663),
    ("goblin_green_soldier_4", 664),
    ("goblin_green_soldier_5", 665),
    ("goblin_green_soldier_6", 666),
    ("goblin_green_soldier_7", 667),
    ("goblin_green_soldier_8", 668),
];
pub const BONES: Item = ("bones", 526, 1);
pub const MAIL: Item = ("goblin_armour", 288, 1);
pub const RED_MAIL: Item = ("goblin_armour_red", 9054, 1);
pub const GREEN_MAIL: Item = ("goblin_armour_green", 9057, 1);

pub enum Spec {
    Nothing(u32),
    Item(u32, Item),
    /// The goblin mail matching the dying goblin.
    Mail(u32),
    Herb(u32),
}
use Spec::*;

pub const MAIN: &[Spec] = &[
    Item(3, ("bronze_axe", 1351, 1)),
    Item(1, ("bronze_scimitar", 1321, 1)),
    Item(9, ("bronze_spear", 1237, 1)),
    Item(3, ("bronze_arrow", 882, 7)),
    Item(3, ("mindrune", 558, 2)),
    Item(3, ("earthrune", 557, 4)),
    Item(3, ("bodyrune", 559, 2)),
    Item(2, ("bronze_javelin", 825, 5)),
    Item(1, ("chaosrune", 562, 1)),
    Item(1, ("naturerune", 561, 1)),
    Item(34, ("coins", 995, 1)),
    Item(13, ("coins", 995, 3)),
    Item(8, ("coins", 995, 5)),
    Item(7, ("coins", 995, 16)),
    Item(3, ("coins", 995, 24)),
    Nothing(8),
    Item(9, ("hammer", 2347, 1)),
    Item(2, ("slice_goblin_history_book", 10999, 1)),
    Mail(10),
    Item(1, ("grapes", 1987, 1)),
    Item(1, ("red_cape", 1007, 1)),
    Item(1, ("tin_ore", 438, 1)),
    Herb(2),
];
/// Independent tertiaries in OpenRune order, after the main roll: (chance, out of, item).
pub const TERTIARY: [(u32, u32, Item); 2] = [
    (1, 30, ("arceuus_corpse_goblin", 13448, 1)),
    (1, 80, ("trail_clue_beginner", 23182, 1)),
];

fn product(objects: &Symbols, (name, id, count): Item) -> Result<Product, String> {
    let symbol = objects.get(name).copied();
    if symbol != Some(i32::from(id)) {
        return Err(format!(
            "goblin drops: obj {name} is {symbol:?} in rev-239 symbols, expected {id}"
        ));
    }
    Ok(Product { item: id, count })
}

fn profile(objects: &Symbols, mail: Item, herb: &Weighted) -> Result<Profile, String> {
    let mut rows = vec![];
    for spec in MAIN {
        let (weight, outcome) = match *spec {
            Nothing(w) => (w, Outcome::Nothing),
            Item(w, item) => (w, Outcome::Product(product(objects, item)?)),
            Mail(w) => (w, Outcome::Product(product(objects, mail)?)),
            Herb(w) => (w, Outcome::Table(herb.clone())),
        };
        rows.push(Row { weight, outcome });
    }
    let main = Weighted::with_rows(128, rows).map_err(|e| format!("goblin main table: {e:?}"))?;
    let tertiary = TERTIARY
        .iter()
        .map(|&(chance, out_of, item)| {
            Ok(Tertiary { chance, out_of, outcome: Outcome::Product(product(objects, item)?) })
        })
        .collect::<Result<_, String>>()?;
    Profile::new(DropPolicy::Goblin, vec![product(objects, BONES)?], Some(main))
        .and_then(|p| p.with_tertiary(tertiary))
        .map_err(|e| format!("goblin profile: {e:?}"))
}

/// Profiles by NPC id. Every goblin must be a cache "Goblin" with Attack and a timed death.
pub fn goblin_drops(
    npcs: &Symbols,
    objects: &Symbols,
    d: &Definitions,
    death: &Config,
    herb: &Weighted,
) -> Result<BTreeMap<u16, Profile>, String> {
    if death.bones != BONES.1 || herb.total() != 128 {
        return Err("goblin drops: unattested bones or herb table".into());
    }
    let mut profiles = BTreeMap::new();
    for (group, mail) in [(&TABLE_2[..], MAIL), (&RED[..], RED_MAIL), (&GREEN[..], GREEN_MAIL)] {
        let p = profile(objects, mail, herb)?;
        for &(symbol, id) in group {
            let t = d.npcs.get(&u32::from(id));
            if npcs.get(symbol) != Some(&i32::from(id))
                || t.is_none_or(|t| {
                    t.name != "Goblin" || !t.ops.iter().any(|o| o.as_deref() == Some("Attack"))
                })
                || !death.definitions.get(&id).is_some_and(|d| d.animation_ticks > 0)
                || death.chickens.contains_key(&id)
            {
                return Err(format!("goblin drops: unattested goblin {symbol} ({id})"));
            }
            profiles.insert(id, p.clone());
        }
    }
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::shared_drop_tables as shared;
    use gielinor_world::{
        npc::{RandomSource, SeededRandom},
        npc_death::Definition,
        npc_loot::Location,
    };
    use std::collections::VecDeque;

    /// Replays scripted draws (or a seeded stream) and records each exclusive bound.
    struct Record<R>(R, Vec<(u32, u32)>);
    impl<R: RandomSource> RandomSource for Record<R> {
        fn below(&mut self, bound: u32) -> u32 {
            let draw = self.0.below(bound);
            assert!(draw < bound);
            self.1.push((bound, draw));
            draw
        }
    }
    struct Script(VecDeque<u32>);
    impl RandomSource for Script {
        fn below(&mut self, _: u32) -> u32 {
            self.0.pop_front().expect("unscripted draw")
        }
    }
    fn script(draws: &[u32]) -> Record<Script> {
        Record(Script(draws.iter().copied().collect()), vec![])
    }

    fn all_items() -> Vec<Item> {
        let mut items = vec![BONES, MAIL, RED_MAIL, GREEN_MAIL];
        items.extend(MAIN.iter().filter_map(|s| if let Item(_, i) = *s { Some(i) } else { None }));
        items.extend(TERTIARY.iter().map(|t| t.2));
        items
    }
    fn fixture() -> (Symbols, Symbols, Definitions, Config, Weighted) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut n = Symbols::new();
        let mut deaths = BTreeMap::new();
        for &(symbol, id) in TABLE_2.iter().chain(&RED).chain(&GREEN) {
            let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            t.name = "Goblin".into();
            t.ops[1] = Some("Attack".into());
            d.npcs.insert(u32::from(id), t);
            n.insert(symbol.into(), i32::from(id));
            deaths.insert(id, Definition { animation: 6182, animation_ticks: 3, sound: None });
        }
        let mut o = Symbols::new();
        for (name, id, _) in all_items() {
            o.insert(name.into(), i32::from(id));
        }
        // resolve() checks every shared-table obj; only its herb table is used here.
        let mut all = o.clone();
        for spec in [shared::HERB, shared::MEGA_RARE, shared::GEM, shared::RARE].into_iter().flatten() {
            let items = match *spec {
                shared::Spec::Item(_, i) | shared::Spec::Noted(_, i) => vec![i],
                shared::Spec::ByLocation(_, a, b) => vec![a, b],
                _ => vec![],
            };
            for (name, id, _) in items {
                all.insert(name.into(), i32::from(id));
            }
        }
        let herb = shared::resolve(&all).unwrap().herb;
        let config = Config { definitions: deaths, bones: 526, chickens: Default::default() };
        (n, o, d, config, herb)
    }
    fn profiles() -> BTreeMap<u16, Profile> {
        let (n, o, d, c, herb) = fixture();
        goblin_drops(&n, &o, &d, &c, &herb).unwrap()
    }
    /// Expected main-roll product for a draw, decoded from the Wiki rows independently of
    /// `Weighted`: (low draw, high draw inclusive, product or None for nothing / herb).
    fn wiki_rows(mail: u16) -> Vec<(u32, u32, Option<(u16, i32)>)> {
        let rows: [(u32, Option<(u16, i32)>); 23] = [
            (3, Some((1351, 1))),
            (1, Some((1321, 1))),
            (9, Some((1237, 1))),
            (3, Some((882, 7))),
            (3, Some((558, 2))),
            (3, Some((557, 4))),
            (3, Some((559, 2))),
            (2, Some((825, 5))),
            (1, Some((562, 1))),
            (1, Some((561, 1))),
            (34, Some((995, 1))),
            (13, Some((995, 3))),
            (8, Some((995, 5))),
            (7, Some((995, 16))),
            (3, Some((995, 24))),
            (8, None),
            (9, Some((2347, 1))),
            (2, Some((10999, 1))),
            (10, Some((mail, 1))),
            (1, Some((1987, 1))),
            (1, Some((1007, 1))),
            (1, Some((438, 1))),
            (2, None),
        ];
        let mut at = 0;
        rows.iter()
            .map(|&(w, p)| {
                let row = (at, at + w - 1, p);
                at += w;
                row
            })
            .collect()
    }
    const HERB_GUAM: (u16, i32) = (199, 1);

    #[test]
    fn goblins_get_guaranteed_bones_and_one_main_roll() {
        let p = profiles();
        let ids: Vec<u16> = p.keys().copied().collect();
        let mut wanted: Vec<u16> = TABLE_2.iter().chain(&RED).chain(&GREEN).map(|g| g.1).collect();
        wanted.sort();
        assert_eq!(ids, wanted);
        assert_eq!(ids.len(), 27);
        for (id, profile) in &p {
            assert_eq!(profile.bound(), Some(128), "goblin {id}");
            assert_eq!(profile.policy(), DropPolicy::Goblin);
            // Main 0 (bronze axe) and both tertiaries miss: one main draw, then 1/30, 1/80.
            let mut random = script(&[0, 1, 1]);
            let rolled = profile.roll(&mut random, Location::Overworld);
            assert_eq!(rolled.draw, Some(0));
            assert_eq!(rolled.products, [(526, 1), (1351, 1)]);
            assert_eq!(random.1, [(128, 0), (30, 1), (80, 1)]);
            // Nothing row (draws 94-101): bones alone.
            let mut random = script(&[94, 29, 79]);
            assert_eq!(profile.roll(&mut random, Location::Overworld).products, [(526, 1)]);
        }
    }

    #[test]
    fn every_draw_matches_the_wiki_rows_and_colour() {
        let p = profiles();
        for (group, mail) in [(&TABLE_2[..], 288), (&RED[..], 9054), (&GREEN[..], 9057)] {
            for &(_, id) in group {
                let mut weights = BTreeMap::<Option<(u16, i32)>, u32>::new();
                for (low, high, expected) in wiki_rows(mail) {
                    for draw in low..=high {
                        let herb = draw >= 126;
                        let draws = if herb { vec![draw, 0, 1, 1] } else { vec![draw, 1, 1] };
                        let mut random = script(&draws);
                        let rolled = p[&id].roll(&mut random, Location::Overworld);
                        let bounds: Vec<u32> = random.1.iter().map(|b| b.0).collect();
                        assert_eq!(bounds, if herb { vec![128, 128, 30, 80] } else { vec![128, 30, 80] });
                        let main = if herb { Some(HERB_GUAM) } else { expected };
                        let mut want = vec![(526, 1)];
                        want.extend(main);
                        assert_eq!(rolled.products, want, "goblin {id} draw {draw}");
                        *weights.entry(expected).or_default() += 1;
                    }
                }
                // Wiki weights: 8 nothing + 2 herb, 34+13+8+7+3 coins by count, 10 mail.
                assert_eq!(weights[&None], 10);
                assert_eq!(weights[&Some((995, 1))], 34);
                assert_eq!(weights[&Some((995, 3))], 13);
                assert_eq!(weights[&Some((mail, 1))], 10);
                for other in [288, 9054, 9057].into_iter().filter(|&m| m != mail) {
                    assert!(!weights.contains_key(&Some((other, 1))), "goblin {id} drops only its own mail");
                }
                assert_eq!(weights.values().sum::<u32>(), 128);
            }
        }
    }

    #[test]
    fn herb_rows_delegate_to_the_shared_herb_table() {
        let p = profiles();
        let mut herbs = BTreeMap::new();
        for main in [126, 127] {
            for herb in 0..128 {
                let mut random = script(&[main, herb, 1, 1]);
                let rolled = p[&3045].roll(&mut random, Location::Overworld);
                assert_eq!(rolled.products.len(), 2);
                *herbs.entry(rolled.products[1]).or_insert(0u32) += 1;
            }
        }
        // 2/128 x 32/128 guam ... 3/128 dwarf weed, per HerbDropTable.
        assert_eq!(herbs[&(199, 1)], 64);
        assert_eq!(herbs[&(207, 1)], 22);
        assert_eq!(herbs[&(217, 1)], 6);
        assert_eq!(herbs.values().sum::<u32>(), 256);
    }

    #[test]
    fn tertiaries_roll_after_the_main_roll_in_openrune_order() {
        let p = profiles();
        // Main nothing, ensouled head hit (0 of 30), clue hit (0 of 80).
        let mut random = script(&[94, 0, 0]);
        let rolled = p[&655].roll(&mut random, Location::Overworld);
        assert_eq!(rolled.products, [(526, 1), (13448, 1), (23182, 1)]);
        assert_eq!(random.1, [(128, 94), (30, 0), (80, 0)]);
        // Herb then clue only: the herb draw comes before both tertiaries.
        let mut random = script(&[127, 127, 1, 0]);
        let rolled = p[&662].roll(&mut random, Location::Overworld);
        assert_eq!(rolled.products, [(526, 1), (217, 1), (23182, 1)]);
        assert_eq!(random.1, [(128, 127), (128, 127), (30, 1), (80, 0)]);
    }

    #[test]
    fn fixed_seed_rolls_follow_the_pinned_draw_order() {
        let p = profiles();
        let rows = wiki_rows(9054);
        let mut random = Record(SeededRandom(44), vec![]);
        let mut counts = [0u32; 23];
        let (mut heads, mut clues) = (0, 0);
        let kills = 128_000;
        for _ in 0..kills {
            random.1.clear();
            let rolled = p[&655].roll(&mut random, Location::Overworld);
            // Decode the recorded draws independently: main, optional herb, 1/30, 1/80.
            let mut draws = random.1.iter().copied();
            let (bound, main) = draws.next().unwrap();
            assert_eq!(bound, 128);
            assert_eq!(rolled.draw, Some(main));
            let index = rows.iter().position(|r| (r.0..=r.1).contains(&main)).unwrap();
            let row = rows[index];
            let mut want = vec![(526, 1)];
            if main >= 126 {
                let (bound, herb) = draws.next().unwrap();
                assert_eq!(bound, 128);
                assert!(herb < 128);
                want.push(rolled.products[1]);
            } else {
                want.extend(row.2);
            }
            let (b30, head) = draws.next().unwrap();
            let (b80, clue) = draws.next().unwrap();
            assert_eq!((b30, b80), (30, 80));
            assert!(draws.next().is_none());
            if head == 0 {
                want.push((13448, 1));
                heads += 1;
            }
            if clue == 0 {
                want.push((23182, 1));
                clues += 1;
            }
            assert_eq!(rolled.products, want);
            counts[index] += 1;
        }
        // Within four standard deviations of weight/128 (coins 1, nothing, mail, herb rows).
        let near = |got: u32, p: f64| (got as f64 - kills as f64 * p).abs() < 4.0 * (kills as f64 * p * (1.0 - p)).sqrt();
        for (index, weight) in [(10, 34), (15, 8), (18, 10), (22, 2)] {
            assert!(near(counts[index], weight as f64 / 128.0), "row {index}: {counts:?}");
        }
        assert!(near(heads, 1.0 / 30.0), "{heads}");
        assert!(near(clues, 1.0 / 80.0), "{clues}");
        // Golden counts for seed 44: any change to row order, weights or draw order moves them.
        assert_eq!(
            counts,
            [2999, 1049, 9030, 2888, 2908, 3080, 2965, 1991, 944, 987, 34074, 12984, 7979, 7017, 2978, 7978, 9136, 1991, 10066, 994, 981, 992, 1989]
        );
        assert_eq!((heads, clues), (4243, 1546));
        println!("GOBLIN_FIXED_SEED seed=44 kills={kills} heads={heads} clues={clues} rows={counts:?}");
    }

    #[test]
    fn identity_and_lifecycle_drift_fail_closed() {
        for case in 0..9 {
            let (mut n, mut o, mut d, mut c, herb) = fixture();
            match case {
                0 => {
                    n.insert("goblin_armed".into(), 3029);
                }
                1 => d.npcs.get_mut(&3046).unwrap().name = "Hobgoblin".into(),
                2 => d.npcs.get_mut(&655).unwrap().ops[1] = None,
                3 => c.definitions.get_mut(&668).unwrap().animation_ticks = 0,
                4 => {
                    c.definitions.remove(&5193);
                }
                5 => {
                    o.insert("goblin_armour_green".into(), 9058);
                }
                6 => {
                    o.remove("trail_clue_beginner");
                }
                7 => c.bones = 532,
                _ => {
                    d.npcs.remove(&5208);
                }
            }
            assert!(goblin_drops(&n, &o, &d, &c, &herb).is_err(), "drift{case}");
        }
    }
}
