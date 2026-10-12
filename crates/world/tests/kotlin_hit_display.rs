use gielinor_world::hit_display::{self, Builder};
#[test]
fn actual_kotlin_hit_builder_and_healthbar_fill() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/hit-display.expected"
    ))
    .expect("Run scripts/hit-display-parity.ps1");
    assert_eq!(fixture.lines().count(), 8192);
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let a: Vec<i32> = f[1].split(',').map(|v| v.parse().unwrap()).collect();
        let output = if f[0] == "F" {
            hit_display::fill(a[0] as u8, a[1], a[2])
                .unwrap()
                .to_string()
        } else {
            let opt = |v: i32| (v >= 0).then_some(v as u16);
            let mark = Builder {
                target: a[1] as u16,
                source: a[2] as u16,
                public: opt(a[3]),
                zero_lit: opt(a[4]),
                zero_tint: opt(a[5]),
                max_lit: opt(a[6]),
                target_threshold: a[7],
                source_threshold: a[8],
                delay: a[9] as u16,
            }
            .build(a[0]);
            format!(
                "{},{},{},{}",
                mark.target,
                mark.source,
                mark.public.map(i32::from).unwrap_or(-1),
                mark.delay
            )
        };
        assert_eq!(output, f[2], "case {case}");
    }
}
