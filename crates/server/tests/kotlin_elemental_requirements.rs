use gielinor_server::elemental_requirements::{self, modern_names};
use std::{collections::BTreeMap, fs, path::PathBuf};
#[test]
fn original_elemental_requirement_loader_variants() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../reference/kronos-elemental");
    let book = fs::read_to_string(root.join("SpellBook.java")).unwrap();
    let runes = fs::read_to_string(root.join("rune/Rune.java")).unwrap();
    let symbols =
        gielinor_cache::symbols::parse(&fs::read_to_string(root.join("obj.sym")).unwrap(), None)
            .unwrap();
    let bodies: BTreeMap<_, _> = modern_names(&book)
        .into_iter()
        .map(|name| {
            let text = fs::read_to_string(root.join(format!("spells/modern/{name}.java"))).unwrap();
            (name, text)
        })
        .collect();
    let mut actual = String::new();
    for case in 0..8 {
        let mut text = book.clone();
        let mut bodies = bodies.clone();
        let mut symbols = symbols.clone();
        match case {
            1 => {
                symbols.remove("airrune");
            }
            2 => {
                bodies.remove("WindStrike");
            }
            3 => {
                let body = bodies.get_mut("WindStrike").unwrap();
                *body = body.replace("setLvlReq(1)", "setLvlReq( 1)");
            }
            4 => {
                let body = bodies.get_mut("WindStrike").unwrap();
                *body = body.replace("setRunes(", "setRUNES(");
            }
            5 => {
                let body = bodies.get_mut("WindStrike").unwrap();
                *body = body.replace("Rune.MIND", "Rune.UNKNOWN");
            }
            6 => {
                let body = bodies.get_mut("WindStrike").unwrap();
                *body = body
                    .replace("Rune.MIND", "Rune.UNKNOWN")
                    .replace("Rune.AIR", "Rune.UNKNOWN");
            }
            7 => text = text.replace("MODERN(", "OLD("),
            _ => {}
        }
        let result = elemental_requirements::parse(&text, &runes, &bodies, &symbols);
        actual.push_str(&format!("{case}|rows|{}\n", result.spells.len()));
        for (key, row) in result.spells {
            actual.push_str(&format!(
                "{case}|{key}|{}|{}\n",
                row.level,
                row.runes
                    .iter()
                    .map(|r| format!("{}:{}", r.item, r.count))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }
    let expected = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/elemental-requirements.expected"
    ))
    .expect("Run scripts/elemental-requirements-parity.ps1");
    assert_eq!(actual, expected.replace("\r\n", "\n"));
}
