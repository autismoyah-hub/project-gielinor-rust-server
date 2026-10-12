use gielinor_world::{
    npc_options::{Catalog, Definition},
    variables::{Varbit, Variables},
};
#[test]
fn compiled_npc_interactions_match_morph_precedence_and_operation_visibility() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/npc-options.expected"
    ))
    .expect("Run scripts/npc-options-parity.ps1");
    assert_eq!(fixture.lines().count(), 4096);
    let texts = [
        None,
        Some(""),
        Some(" "),
        Some("hidden"),
        Some("HiDdEn"),
        Some("Attack"),
        Some(" hidden "),
        Some("\u{a0}"),
        Some("\u{85}"),
        Some("\u{1c}"),
        Some("\u{2007}"),
        Some("\u{200b}"),
    ];
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let case: usize = f[0].parse().unwrap();
        let mode = case % 8;
        let value = ((case / 8) % 12) as i32 - 3;
        let text = (case / 96) % texts.len();
        let original = Definition {
            operations: [Some("Talk-to"), Some("Attack"), None, None, texts[text]]
                .map(|v| v.map(str::to_owned)),
            varp: Some(if [1, 3, 4, 6].contains(&mode) { 1 } else { 0 }),
            varbit: Some(if [2, 3, 4, 5, 7].contains(&mode) {
                2
            } else {
                0
            }),
            fallback: (case % 3 != 0).then_some(2),
            morphs: if case % 5 == 0 {
                vec![]
            } else {
                vec![Some(2), None, Some(3)]
            },
        };
        let second = Definition {
            operations: [
                texts[text],
                texts[(text + 1) % texts.len()],
                None,
                None,
                Some("Trade"),
            ]
            .map(|v| v.map(str::to_owned)),
            ..Default::default()
        };
        let third = Definition {
            operations: [Some("Bank"), texts[text], None, None, None].map(|v| v.map(str::to_owned)),
            ..Default::default()
        };
        let mut catalog = Catalog::default();
        catalog.types.extend([(1, original), (2, second)]);
        if case % 7 != 0 {
            catalog.types.insert(3, third);
        }
        if mode != 4 && mode != 6 {
            catalog.varps.insert(1);
        }
        if mode != 5 {
            catalog.varbits.insert(
                2,
                Varbit {
                    base: 1,
                    lsb: 1,
                    msb: 3,
                },
            );
        }
        let vars = Variables::restore([(1, value)].into());
        assert_eq!(
            catalog.morph(1, &vars).map(i32::from).unwrap_or(-1),
            f[1].parse::<i32>().unwrap(),
            "morph {case}"
        );
        let actual = (1..=5)
            .map(|op| {
                if catalog.has_operation(1, &vars, op) {
                    "1"
                } else {
                    "0"
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, f[2], "operations {case}");
    }
}
