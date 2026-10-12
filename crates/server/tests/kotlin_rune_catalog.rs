use gielinor_cache::{definitions::ItemType, enums::EnumType, game::Definitions};
use std::collections::BTreeMap;
fn records(text: &str) -> Vec<(u32, Vec<u8>)> {
    text.split(';')
        .map(|s| {
            let (id, hex) = s.split_once(':').unwrap();
            (
                id.parse().unwrap(),
                hex.as_bytes()
                    .chunks_exact(2)
                    .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                    .collect(),
            )
        })
        .collect()
}
fn map_text<'a>(map: impl IntoIterator<Item = (&'a i32, &'a i32)>) -> String {
    map.into_iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join("/")
}
fn lists<'a, T: IntoIterator<Item = &'a i32>>(
    map: impl IntoIterator<Item = (&'a i32, T)>,
) -> String {
    map.into_iter()
        .map(|(k, v)| {
            format!(
                "{k}:{}",
                v.into_iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join("/")
}
#[test]
fn cache_rune_catalog_matches_actual_kotlin_repositories_and_combo_order() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rune-catalog.expected"
    ))
    .expect("Run scripts/rune-catalog-parity.ps1");
    assert_eq!(rows.lines().count(), 256);
    let symbols = [
        "rune_compact_ids",
        "fake_runes",
        "rune_staves",
        "unlimited_runes_hiprio",
        "unlimited_runes_loprio",
        "rune_substitutes",
        "staff_substitutes",
        "combo_runes",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, s)| (s.to_owned(), i as i32 + 1))
    .collect();
    for (case, line) in rows.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let defs = Definitions {
            items: records(f[0])
                .into_iter()
                .map(|(id, b)| (id, ItemType::decode(&b).unwrap()))
                .collect(),
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            stats: BTreeMap::new(),
            varps: BTreeMap::new(),
            varbits: BTreeMap::new(),
        };
        let tables = records(f[1])
            .into_iter()
            .map(|(id, b)| (id, EnumType::decode(&b).unwrap().unwrap()))
            .collect();
        let c = gielinor_server::rune_catalog::resolve(&symbols, &tables, &defs).unwrap();
        let actual = [
            map_text(&c.compact),
            map_text(&c.fake),
            lists(&c.high),
            lists(&c.low),
            lists(&c.substitutes),
            lists(&c.staff_substitutes),
            c.combos
                .iter()
                .map(|(a, b, c)| format!("{a},{b},{c}"))
                .collect::<Vec<_>>()
                .join("/"),
        ];
        for (field, value) in actual.iter().enumerate() {
            assert_eq!(value, f[field + 2], "case {case}, field {field}");
        }
    }
}
