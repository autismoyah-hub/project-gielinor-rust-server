//! Ordinary Man and Woman drops: Wiki Template:Human drop table (rev 15374758, used by Man
//! rev 15374703 and Woman rev 15374728); behaviour from OpenRune `ManDropTable.kt` and
//! `woman.toml`, which share one /128 main table. Bones are guaranteed, then one main roll
//! (23/128 delegates to the shared herb table), then a beginner clue at 1/90. Bronze bolts
//! drop 2-12 (inclusive), drawn after the clue as OpenRune rolls each spawned count.
//!
//! Decisions where the sources differ:
//! - Coins follow 239.1 (cache objs end at Aug-2025 releases), as in OpenRune: 5 coins at 9.
//!   The Wiki's 19 Aug 2026 energy potion(3) row is later.
//! - man5 and woman4 (Wiki 14920/14921) are not in rev-239 symbols and are not registered.
//!
//! Not modelled: the Man looting bag (1/15, Wilderness men only; none of these ids), the
//! medium-clue key, the easy clue the Wiki lists for free-to-play worlds, F2P substitutions
//! (10 coins for herbs, nothing for bolts) and the one-beginner-clue-at-a-time rule.
use super::shared_drop_tables::Item;
use super::{Definitions, Symbols};
use gielinor_world::{
    npc_death::{Config, DropPolicy},
    npc_loot::{Outcome, Product, Profile, Row, Tertiary, Weighted},
};
use std::collections::BTreeMap;

/// OpenRune ManDropTable.kt npcs (Wiki Man ids), without man5.
pub const MEN: [(&str, u16); 17] = [
    ("al_kharid_man", 3261),
    ("death_man_indoors1", 6815),
    ("falador_man2", 3264),
    ("falador_man3", 3265),
    ("karamja_man", 3652),
    ("man", 3106),
    ("man2", 3107),
    ("man3", 3108),
    ("man4", 3109),
    ("man4_for_musa_point", 3110),
    ("man_indoor", 6818),
    ("shayzien_man_1", 11057),
    ("shayzien_man_2", 11058),
    ("varrock_man1", 3014),
    ("zeah_man", 6987),
    ("zeah_man2", 6988),
    ("zeah_man3", 6989),
];
/// OpenRune woman.toml npcs (Wiki Woman ids), without woman4.
pub const WOMEN: [(&str, u16); 10] = [
    ("varrock_woman1", 3015),
    ("woman", 3111),
    ("woman2", 3112),
    ("woman3", 3113),
    ("zeah_woman", 6990),
    ("zeah_woman2", 6991),
    ("zeah_woman3", 6992),
    ("zeah_woman_outside", 10728),
    ("shayzien_woman_1", 11053),
    ("shayzien_woman_2", 11054),
];
pub const BONES: Item = ("bones", 526, 1);

pub enum Spec {
    Nothing(u32),
    Item(u32, Item),
    /// Weight, obj and the inclusive count range.
    Range(u32, (&'static str, u16), i32, i32),
    Herb(u32),
}
use Spec::*;

pub const MAIN: &[Spec] = &[
    Item(2, ("bronze_med_helm", 1139, 1)),
    Item(1, ("iron_dagger", 1203, 1)),
    Range(22, ("bolt", 877), 2, 12),
    Item(3, ("bronze_arrow", 882, 7)),
    Item(2, ("earthrune", 557, 4)),
    Item(2, ("firerune", 554, 6)),
    Item(2, ("mindrune", 558, 9)),
    Item(1, ("chaosrune", 562, 2)),
    Item(38, ("coins", 995, 3)),
    Item(9, ("coins", 995, 5)),
    Item(4, ("coins", 995, 15)),
    Item(1, ("coins", 995, 25)),
    Nothing(8),
    Item(5, ("fishing_bait", 313, 1)),
    Item(2, ("copper_ore", 436, 1)),
    Item(2, ("earth_talisman", 1440, 1)),
    Item(1, ("cabbage", 1965, 1)),
    Herb(23),
];
pub const CLUE: (u32, u32, Item) = (1, 90, ("trail_clue_beginner", 23182, 1));

fn product(objects: &Symbols, (name, id, count): Item) -> Result<Product, String> {
    let symbol = objects.get(name).copied();
    if symbol != Some(i32::from(id)) {
        return Err(format!(
            "man/woman drops: obj {name} is {symbol:?} in rev-239 symbols, expected {id}"
        ));
    }
    Ok(Product { item: id, count })
}

fn profile(objects: &Symbols, policy: DropPolicy, herb: &Weighted) -> Result<Profile, String> {
    let mut rows = vec![];
    for spec in MAIN {
        let (weight, outcome) = match *spec {
            Nothing(w) => (w, Outcome::Nothing),
            Item(w, item) => (w, Outcome::Product(product(objects, item)?)),
            Range(w, (name, id), min, max) => {
                let item = product(objects, (name, id, min))?.item;
                (w, Outcome::Range { item, min, max })
            }
            Herb(w) => (w, Outcome::Table(herb.clone())),
        };
        rows.push(Row { weight, outcome });
    }
    let main =
        Weighted::with_rows(128, rows).map_err(|e| format!("man/woman main table: {e:?}"))?;
    let (chance, out_of, clue) = CLUE;
    let clue = Tertiary { chance, out_of, outcome: Outcome::Product(product(objects, clue)?) };
    Profile::new(policy, vec![product(objects, BONES)?], Some(main))
        .and_then(|p| p.with_tertiary(vec![clue]))
        .map_err(|e| format!("man/woman profile: {e:?}"))
}

/// Profiles by NPC id. Each must be a cache "Man"/"Woman" with Attack and a timed death.
pub fn man_woman_drops(
    npcs: &Symbols,
    objects: &Symbols,
    d: &Definitions,
    death: &Config,
    herb: &Weighted,
) -> Result<BTreeMap<u16, Profile>, String> {
    if death.bones != BONES.1 || herb.total() != 128 {
        return Err("man/woman drops: unattested bones or herb table".into());
    }
    let mut profiles = BTreeMap::new();
    for (group, name, policy) in
        [(&MEN[..], "Man", DropPolicy::Man), (&WOMEN[..], "Woman", DropPolicy::Woman)]
    {
        let p = profile(objects, policy, herb)?;
        for &(symbol, id) in group {
            let t = d.npcs.get(&u32::from(id));
            if npcs.get(symbol) != Some(&i32::from(id))
                || t.is_none_or(|t| {
                    t.name != name || !t.ops.iter().any(|o| o.as_deref() == Some("Attack"))
                })
                || !death.definitions.get(&id).is_some_and(|d| d.animation_ticks > 0)
                || death.chickens.contains_key(&id)
            {
                return Err(format!("man/woman drops: unattested {symbol} ({id})"));
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

    /// Replays scripted draws (or a seeded stream) and records each (bound, draw).
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
        for (group, name) in [(&MEN[..], "Man"), (&WOMEN[..], "Woman")] {
            for &(symbol, id) in group {
                let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
                t.name = name.into();
                t.ops[1] = Some("Attack".into());
                d.npcs.insert(u32::from(id), t);
                n.insert(symbol.into(), i32::from(id));
                deaths.insert(id, Definition { animation: 836, animation_ticks: 2, sound: None });
            }
        }
        let mut o = Symbols::new();
        let mut items = vec![BONES, CLUE.2];
        for spec in MAIN {
            match *spec {
                Item(_, i) => items.push(i),
                Range(_, (name, id), min, _) => items.push((name, id, min)),
                _ => {}
            }
        }
        for spec in [shared::HERB, shared::MEGA_RARE, shared::GEM, shared::RARE].into_iter().flatten() {
            match *spec {
                shared::Spec::Item(_, i) | shared::Spec::Noted(_, i) => items.push(i),
                shared::Spec::ByLocation(_, a, b) => items.extend([a, b]),
                _ => {}
            }
        }
        for (name, id, _) in items {
            o.insert(name.into(), i32::from(id));
        }
        let herb = shared::resolve(&o).unwrap().herb;
        let config = Config { definitions: deaths, bones: 526, chickens: Default::default() };
        (n, o, d, config, herb)
    }
    fn profiles() -> BTreeMap<u16, Profile> {
        let (n, o, d, c, herb) = fixture();
        man_woman_drops(&n, &o, &d, &c, &herb).unwrap()
    }
    /// Wiki Human drop table rows as (low, high inclusive, product); None is nothing or
    /// herb, and the bolt row's count is its minimum (draw 0 of 11).
    fn wiki_rows() -> Vec<(u32, u32, Option<(u16, i32)>)> {
        let rows: [(u32, Option<(u16, i32)>); 18] = [
            (2, Some((1139, 1))),
            (1, Some((1203, 1))),
            (22, Some((877, 2))),
            (3, Some((882, 7))),
            (2, Some((557, 4))),
            (2, Some((554, 6))),
            (2, Some((558, 9))),
            (1, Some((562, 2))),
            (38, Some((995, 3))),
            (9, Some((995, 5))),
            (4, Some((995, 15))),
            (1, Some((995, 25))),
            (8, None),
            (5, Some((313, 1))),
            (2, Some((436, 1))),
            (2, Some((1440, 1))),
            (1, Some((1965, 1))),
            (23, None),
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
    const BOLTS: std::ops::RangeInclusive<u32> = 3..=24;
    const HERB: std::ops::RangeInclusive<u32> = 105..=127;

    #[test]
    fn men_and_women_get_guaranteed_bones_and_one_main_roll() {
        let p = profiles();
        assert_eq!(p.len(), 27);
        for (group, policy) in [(&MEN[..], DropPolicy::Man), (&WOMEN[..], DropPolicy::Woman)] {
            for &(_, id) in group {
                let profile = &p[&id];
                assert_eq!((profile.bound(), profile.policy()), (Some(128), policy), "npc {id}");
                // Main 0 (bronze med helm), clue miss: one main draw, then 1/90.
                let mut random = script(&[0, 1]);
                let rolled = profile.roll(&mut random, Location::Overworld);
                assert_eq!((rolled.draw, rolled.products), (Some(0), vec![(526, 1), (1139, 1)]));
                assert_eq!(random.1, [(128, 0), (90, 1)]);
                // Nothing row (draws 89-96): bones alone.
                let mut random = script(&[89, 1]);
                assert_eq!(profile.roll(&mut random, Location::Overworld).products, [(526, 1)]);
            }
        }
        for missing in [14920, 14921] {
            assert!(!p.contains_key(&missing), "post-239 id {missing}");
        }
    }

    #[test]
    fn every_draw_matches_the_wiki_human_drop_table() {
        let p = profiles();
        let mut weights = BTreeMap::<Option<(u16, i32)>, u32>::new();
        for (low, high, expected) in wiki_rows() {
            for draw in low..=high {
                // Herb rows draw the herb table; bolt rows draw their count after the clue.
                let draws = if HERB.contains(&draw) {
                    vec![draw, 0, 1]
                } else if BOLTS.contains(&draw) {
                    vec![draw, 1, 0]
                } else {
                    vec![draw, 1]
                };
                let mut random = script(&draws);
                let rolled = p[&3106].roll(&mut random, Location::Overworld);
                let bounds: Vec<u32> = random.1.iter().map(|b| b.0).collect();
                let main = if HERB.contains(&draw) {
                    assert_eq!(bounds, [128, 128, 90]);
                    Some((199, 1))
                } else if BOLTS.contains(&draw) {
                    assert_eq!(bounds, [128, 90, 11]);
                    expected
                } else {
                    assert_eq!(bounds, [128, 90]);
                    expected
                };
                let mut want = vec![(526, 1)];
                want.extend(main);
                assert_eq!(rolled.products, want, "draw {draw}");
                *weights.entry(expected).or_default() += 1;
            }
        }
        assert_eq!(weights[&None], 8 + 23);
        assert_eq!(weights[&Some((877, 2))], 22);
        assert_eq!(weights[&Some((995, 3))], 38);
        assert_eq!(weights[&Some((995, 5))], 9);
        assert_eq!(weights.values().sum::<u32>(), 128);
        // The woman table is the same table.
        for draw in 0..128 {
            let draws = if HERB.contains(&draw) { vec![draw, 0, 1] } else { vec![draw, 1, 0] };
            let man = p[&3106].roll(&mut script(&draws), Location::Overworld);
            let woman = p[&3111].roll(&mut script(&draws), Location::Overworld);
            assert_eq!(man, woman, "draw {draw}");
        }
    }

    #[test]
    fn bolt_counts_are_inclusive_two_to_twelve_and_drawn_last() {
        let p = profiles();
        let mut counts = vec![];
        for count in 0..11 {
            // Main bolts (draw 3), clue hit, then the count.
            let mut random = script(&[3, 0, count]);
            let rolled = p[&3014].roll(&mut random, Location::Overworld);
            assert_eq!(random.1, [(128, 3), (90, 0), (11, count)]);
            assert_eq!(rolled.products[0], (526, 1));
            assert_eq!(rolled.products[2], (23182, 1), "the clue is batched after the bolts");
            counts.push(rolled.products[1]);
        }
        assert_eq!(counts, (2..=12).map(|n| (877, n)).collect::<Vec<_>>());
    }

    #[test]
    fn herb_rows_delegate_to_the_shared_herb_table() {
        let p = profiles();
        let mut herbs = BTreeMap::new();
        for main in HERB {
            for herb in 0..128 {
                let rolled = p[&11053].roll(&mut script(&[main, herb, 1]), Location::Overworld);
                assert_eq!(rolled.products.len(), 2);
                *herbs.entry(rolled.products[1]).or_insert(0u32) += 1;
            }
        }
        // 23/128 x 32/128 guam ... 3/128 dwarf weed, per HerbDropTable.
        assert_eq!(herbs[&(199, 1)], 23 * 32);
        assert_eq!(herbs[&(207, 1)], 23 * 11);
        assert_eq!(herbs[&(217, 1)], 23 * 3);
        assert_eq!(herbs.values().sum::<u32>(), 23 * 128);
    }

    #[test]
    fn fixed_seed_rolls_follow_the_pinned_draw_order() {
        let p = profiles();
        let rows = wiki_rows();
        let mut random = Record(SeededRandom(45), vec![]);
        let mut counts = [0u32; 18];
        let mut bolts = [0u32; 13];
        let mut clues = 0;
        let kills = 128_000;
        for _ in 0..kills {
            random.1.clear();
            let rolled = p[&3108].roll(&mut random, Location::Overworld);
            // Decode the recorded draws independently: main, optional herb, 1/90, optional count.
            let mut draws = random.1.iter().copied();
            let (bound, main) = draws.next().unwrap();
            assert_eq!((bound, rolled.draw), (128, Some(main)));
            let index = rows.iter().position(|r| (r.0..=r.1).contains(&main)).unwrap();
            let mut want = vec![(526, 1)];
            if HERB.contains(&main) {
                let (bound, _) = draws.next().unwrap();
                assert_eq!(bound, 128);
                want.push(rolled.products[1]);
            } else {
                want.extend(rows[index].2);
            }
            let (bound, clue) = draws.next().unwrap();
            assert_eq!(bound, 90);
            if clue == 0 {
                want.push((23182, 1));
                clues += 1;
            }
            if BOLTS.contains(&main) {
                let (bound, count) = draws.next().unwrap();
                assert_eq!(bound, 11);
                want[1] = (877, 2 + count as i32);
                bolts[2 + count as usize] += 1;
            }
            assert!(draws.next().is_none());
            assert_eq!(rolled.products, want);
            counts[index] += 1;
        }
        println!("MAN_FIXED_SEED seed=45 kills={kills} clues={clues} rows={counts:?} bolts={bolts:?}");
        let near = |got: u32, p: f64| {
            (got as f64 - kills as f64 * p).abs() < 4.0 * (kills as f64 * p * (1.0 - p)).sqrt()
        };
        for (index, weight) in [(2, 22), (8, 38), (12, 8), (17, 23)] {
            assert!(near(counts[index], weight as f64 / 128.0), "row {index}: {counts:?}");
        }
        assert!(near(clues, 1.0 / 90.0), "{clues}");
        // Golden counts for seed 45: any change to row order, weights or draw order moves them.
        assert_eq!(
            counts,
            [2004, 976, 22012, 2940, 2013, 2015, 2029, 1013, 38158, 8927, 4101, 1049, 8013, 4964, 1969, 2028, 954, 22835]
        );
        assert_eq!(bolts, [0, 0, 1959, 2012, 2091, 2038, 1965, 1959, 1944, 2015, 1981, 2003, 2045]);
        assert_eq!(clues, 1431);
    }

    #[test]
    fn identity_and_lifecycle_drift_fail_closed() {
        for case in 0..9 {
            let (mut n, mut o, mut d, mut c, herb) = fixture();
            match case {
                0 => {
                    n.insert("man".into(), 3107);
                }
                1 => d.npcs.get_mut(&3111).unwrap().name = "Man".into(),
                2 => d.npcs.get_mut(&3261).unwrap().ops[1] = None,
                3 => c.definitions.get_mut(&10728).unwrap().animation_ticks = 0,
                4 => {
                    c.definitions.remove(&6815);
                }
                5 => {
                    o.insert("bolt".into(), 9140);
                }
                6 => {
                    o.remove("trail_clue_beginner");
                }
                7 => c.bones = 532,
                _ => {
                    d.npcs.remove(&11054);
                }
            }
            assert!(man_woman_drops(&n, &o, &d, &c, &herb).is_err(), "drift{case}");
        }
    }
}
