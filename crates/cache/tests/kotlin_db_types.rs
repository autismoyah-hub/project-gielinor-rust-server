use gielinor_cache::db::{Row, Table, Value};
use std::collections::BTreeMap;
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn types(m: &BTreeMap<u8, Vec<u16>>) -> String {
    m.iter()
        .map(|(k, v)| {
            format!(
                "{k}:{}",
                v.iter().map(u16::to_string).collect::<Vec<_>>().join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
fn values(m: &BTreeMap<u8, Vec<Value>>) -> String {
    m.iter()
        .map(|(k, v)| {
            format!(
                "{k}:{}",
                v.iter()
                    .map(|v| match v {
                        Value::Int(n) => format!("I{n}"),
                        Value::String(s) => format!("S{}", hex(s.as_bytes())),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(";")
}
#[test]
fn original_kotlin_db_decoders_match_tuple_defaults_and_replacement() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/db-types.expected"
    ))
    .expect("Run scripts/db-types-parity.ps1");
    assert_eq!(fixture.lines().count(), 128);
    for (case, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split('|').collect();
        let bytes: Vec<u8> = (0..fields[1].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&fields[1][i..i + 2], 16).unwrap())
            .collect();
        let actual = match fields[0] {
            "R" => {
                let row = Row::decode(&bytes).unwrap();
                format!(
                    "{}|{}|{}|{}",
                    row.table,
                    row.column_count,
                    types(&row.types),
                    values(&row.data)
                )
            }
            "T" => {
                let table = Table::decode(&bytes).unwrap();
                format!(
                    "{}|{}|{}|{}",
                    table.column_count,
                    types(&table.types),
                    values(&table.defaults),
                    table
                        .attributes
                        .iter()
                        .map(|(k, v)| format!("{k}:{v}"))
                        .collect::<Vec<_>>()
                        .join(";")
                )
            }
            _ => panic!("Unknown reference vector"),
        };
        assert_eq!(actual, fields[2..].join("|"), "case {case}");
    }
}
