//! Ordinary imp fixed main table; conditional tertiary tables remain separate.
use super::{Definitions, Symbols};
use gielinor_world::{
    npc_death::{Config, DropPolicy},
    npc_loot::{Entry, Product, Profile, Weighted},
};
const MAIN: [(&str, u16, &str, bool, [i8; 3], u32); 32] = [
    ("black_bead", 1474, "Black bead", false, [-1, -1, -1], 5),
    ("red_bead", 1470, "Red bead", false, [-1, -1, -1], 5),
    ("white_bead", 1476, "White bead", false, [-1, -1, -1], 5),
    ("yellow_bead", 1472, "Yellow bead", false, [-1, -1, -1], 5),
    ("bolt", 877, "Bronze bolts", true, [13, -1, -1], 8),
    ("bluewizhat", 579, "Blue wizard hat", false, [0, -1, -1], 8),
    ("egg", 1944, "Egg", false, [-1, -1, -1], 5),
    ("raw_chicken", 2138, "Raw chicken", false, [-1, -1, -1], 5),
    ("burnt_bread", 2311, "Burnt bread", false, [-1, -1, -1], 4),
    ("burnt_meat", 2146, "Burnt meat", false, [-1, -1, -1], 4),
    ("cabbage", 1965, "Cabbage", false, [-1, -1, -1], 2),
    ("bread_dough", 2307, "Bread dough", false, [-1, -1, -1], 2),
    ("bread", 2309, "Bread", false, [-1, -1, -1], 1),
    ("cooked_meat", 2142, "Cooked meat", false, [-1, -1, -1], 1),
    ("hammer", 2347, "Hammer", false, [-1, -1, -1], 8),
    ("tinderbox", 590, "Tinderbox", false, [-1, -1, -1], 5),
    ("shears", 1735, "Shears", false, [-1, -1, -1], 4),
    ("bucket_empty", 1925, "Bucket", false, [-1, -1, -1], 4),
    (
        "bucket_water",
        1929,
        "Bucket of water",
        false,
        [-1, -1, -1],
        2,
    ),
    ("jug_empty", 1935, "Jug", false, [-1, -1, -1], 2),
    ("jug_water", 1937, "Jug of water", false, [-1, -1, -1], 2),
    ("pot_empty", 1931, "Pot", false, [-1, -1, -1], 2),
    ("pot_flour", 1933, "Pot of flour", false, [-1, -1, -1], 2),
    ("ball_of_wool", 1759, "Ball of wool", false, [-1, -1, -1], 8),
    (
        "mind_talisman",
        1448,
        "Mind talisman",
        false,
        [-1, -1, -1],
        7,
    ),
    ("ashes", 592, "Ashes", false, [-1, -1, -1], 6),
    ("clay", 434, "Clay", false, [-1, -1, -1], 4),
    (
        "cadavaberries",
        753,
        "Cadava berries",
        false,
        [-1, -1, -1],
        4,
    ),
    ("grain", 1947, "Grain", false, [-1, -1, -1], 3),
    ("chefs_hat", 1949, "Chef's hat", false, [0, -1, -1], 2),
    ("flier", 956, "Flyer", false, [-1, -1, -1], 2),
    ("acne_potion", 195, "Potion", false, [-1, -1, -1], 1),
];
pub fn imp_drops(
    npcs: &Symbols,
    items: &Symbols,
    sequences: &Symbols,
    d: &Definitions,
    death: &Config,
) -> Result<Profile, String> {
    if npcs.get("imp") != Some(&5007)
        || npcs.get("ash_imp") == Some(&5007)
        || npcs.get("godwars_ancient_imp") == Some(&5007)
    {
        return Err("Conflicting ordinary imp identity".into());
    }
    let t = d.npcs.get(&5007).ok_or("Missing ordinary imp")?;
    let end = death
        .definitions
        .get(&5007)
        .ok_or("Missing imp lifecycle")?;
    if t.name != "Imp"
        || t.size != 1
        || t.stats != [1, 1, 1, 8, 1, 1]
        || t.ops != [None, Some("Attack".into()), None, None, None]
        || t.restrict != gielinor_world::npc::Restrict::Normal
        || !matches!(t.block_walk, gielinor_world::npc::BlockWalk::Npc)
        || t.content_group != -1
        || t.default_mode != 1
        || t.timer != -1
        || t.wander_range != 5
        || t.animations != [171, 65535, 168, 168, 65535, 65535, 65535]
        || !t.multi.is_empty()
        || t.multi_varp.is_some()
        || t.multi_varbit.is_some()
        || t.multi_default.is_some()
        || (end.animation, end.animation_ticks, end.sound) != (172, 3, Some(535))
    {
        return Err("Unattested ordinary imp lifecycle/profile".into());
    }
    for (name, id) in [("imp_ready", 171), ("imp_walk", 168), ("imp_death", 172)] {
        if sequences.get(name) != Some(&id) {
            return Err("Unattested imp sequence".into());
        }
    }
    for &(symbol, id, name, stack, wear, _) in MAIN.iter().chain(std::iter::once(&(
        "fiendish_ashes",
        25766,
        "Fiendish ashes",
        false,
        [-1, -1, -1],
        0,
    ))) {
        let t = d.items.get(&u32::from(id)).ok_or("Missing imp product")?;
        if items.get(symbol) != Some(&i32::from(id))
            || t.name != name
            || t.stackable != stack
            || t.members
            || t.wear != wear
            || t.dummy != -1
            || t.cert.template != 0
            || t.placeholder.template != 0
            || t.transform.template != 0
            || !t.objvars.is_empty()
        {
            return Err(format!("Unattested imp product {symbol}"));
        }
    }
    let entries = MAIN
        .iter()
        .map(|&(_, item, _, _, _, weight)| Entry {
            weight,
            product: Some(Product { item, count: 1 }),
        })
        .collect();
    let table = Weighted::new(128, entries).map_err(|e| format!("Invalid imp weights: {e:?}"))?;
    Profile::new(
        DropPolicy::Imp,
        vec![Product {
            item: 25766,
            count: 1,
        }],
        Some(table),
    )
    .map_err(|e| format!("Invalid imp profile: {e:?}"))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Symbols, Symbols, Symbols, Definitions, Config) {
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut t = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
        t.name = "Imp".into();
        t.size = 1;
        t.stats = [1, 1, 1, 8, 1, 1];
        t.ops[1] = Some("Attack".into());
        t.default_mode = 1;
        t.timer = -1;
        t.wander_range = 5;
        t.animations = [171, 65535, 168, 168, 65535, 65535, 65535];
        d.npcs.insert(5007, t);
        let mut symbols = Symbols::new();
        for &(s, id, name, stack, wear, _) in MAIN.iter().chain(std::iter::once(&(
            "fiendish_ashes",
            25766,
            "Fiendish ashes",
            false,
            [-1, -1, -1],
            0,
        ))) {
            let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            t.name = name.into();
            t.stackable = stack;
            t.wear = wear;
            d.items.insert(u32::from(id), t);
            symbols.insert(s.into(), i32::from(id));
        }
        (
            [
                ("imp".into(), 5007),
                ("ash_imp".into(), 999),
                ("godwars_ancient_imp".into(), 3134),
            ]
            .into(),
            symbols,
            [
                ("imp_ready".into(), 171),
                ("imp_walk".into(), 168),
                ("imp_death".into(), 172),
            ]
            .into(),
            d,
            Config {
                definitions: [(
                    5007,
                    gielinor_world::npc_death::Definition {
                        animation: 172,
                        animation_ticks: 3,
                        sound: Some(535),
                    },
                )]
                .into(),
                bones: 526,
                chickens: Default::default(),
            },
        )
    }
    #[test]
    fn all_main_weights_and_guaranteed_ashes_are_attested() {
        let (n, i, s, d, c) = fixture();
        let p = imp_drops(&n, &i, &s, &d, &c).unwrap();
        let mut draw = 0;
        for &(_, id, _, _, _, weight) in &MAIN {
            for _ in 0..weight {
                assert_eq!(p.products(Some(draw)), Some(vec![(25766, 1), (id, 1)]));
                draw += 1;
            }
        }
        assert_eq!(draw, 128);
        assert_eq!(p.products(Some(128)), None);
    }
    #[test]
    fn material_and_lifecycle_drift_fail_closed() {
        for id in MAIN.iter().map(|r| r.1).chain([25766]) {
            for case in 0..8 {
                let (n, mut i, s, mut d, c) = fixture();
                let t = d.items.get_mut(&u32::from(id)).unwrap();
                match case {
                    0 => t.name = "Unexpected".into(),
                    1 => t.stackable = !t.stackable,
                    2 => t.members = true,
                    3 => t.cert.template = 799,
                    4 => t.placeholder.template = 14401,
                    5 => t.transform.template = 1,
                    6 => t.dummy = 1,
                    _ => {
                        i.retain(|_, v| *v != i32::from(id));
                    }
                }
                assert!(imp_drops(&n, &i, &s, &d, &c).is_err(), "item{id}case{case}");
            }
        }
        for case in 0..8 {
            let (mut n, i, mut s, mut d, mut c) = fixture();
            match case {
                0 => {
                    n.insert("ash_imp".into(), 5007);
                }
                1 => {
                    n.insert("imp".into(), 3134);
                }
                2 => d.npcs.get_mut(&5007).unwrap().stats[3] = 10,
                3 => d.npcs.get_mut(&5007).unwrap().animations[0] = 65535,
                4 => {
                    s.insert("imp_death".into(), 836);
                }
                5 => c.definitions.get_mut(&5007).unwrap().animation_ticks = 2,
                6 => c.definitions.get_mut(&5007).unwrap().sound = None,
                _ => d.npcs.get_mut(&5007).unwrap().multi.push(Some(3134)),
            }
            assert!(imp_drops(&n, &i, &s, &d, &c).is_err(), "lifecycle{case}");
        }
    }
}
