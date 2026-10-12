use gielinor_server::kronos_combat::{self, Catalog};
use std::collections::BTreeMap;
fn describe(catalog: &Catalog) -> String {
    catalog
        .by_id
        .iter()
        .map(|(id, info)| {
            format!(
                "{id},{},{},{},{},{},{},{},{},{}",
                info.stats[0],
                info.stats[1],
                info.stats[2],
                info.stats[3],
                info.stats[4],
                info.stats[5],
                (info.xp_modifier * 1000.0).round_ties_even() as i32,
                info.respawn_ticks,
                info.spawn_animation
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
#[test]
fn combat_json_and_runtime_stat_bind_match_actual_kotlin() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/kronos-combat.expected"
    ))
    .unwrap();
    assert_eq!(text.lines().count(), 2048);
    for (case, line) in text.lines().enumerate() {
        let (hex, expected) = line.split_once('|').unwrap();
        let bytes = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
            .collect();
        let result = kronos_combat::parse(&String::from_utf8(bytes).unwrap())
            .map(|infos| {
                let mut by_id = BTreeMap::new();
                for info in infos {
                    for &id in &info.ids {
                        by_id.insert(id, info.clone());
                    }
                }
                describe(&Catalog {
                    by_id,
                    failures: vec![],
                })
            })
            .unwrap_or_else(|_| "ERROR".into());
        assert_eq!(result, expected, "case {case}");
    }
}
#[test]
fn actual_local_combat_catalog_matches_reference_catalog() {
    let expected = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/kronos-combat.expected.catalog"
    ))
    .unwrap();
    let catalog = Catalog::load(std::path::Path::new(kronos_combat::DEFAULT_DIRECTORY));
    assert!(catalog.failures.is_empty(), "{:?}", catalog.failures);
    assert!(catalog.by_id.len() > 1000);
    assert_eq!(describe(&catalog).replace(';', "\n") + "\n", expected);
}
