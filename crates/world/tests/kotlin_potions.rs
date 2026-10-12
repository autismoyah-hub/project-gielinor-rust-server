use gielinor_world::potions::{self, Effect};
#[test]
fn compiled_kotlin_potion_catalog_and_recovery_arithmetic_match() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/potions.expected"
    ))
    .unwrap();
    let doses = potions::catalog();
    let mut catalogs = 0;
    let mut formulas = 0;
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        match f[0] {
            "C" => {
                let dose = &doses[catalogs];
                catalogs += 1;
                assert_eq!(
                    (dose.name, dose.symbol, dose.next, dose.left.to_string()),
                    (f[1], f[2], f[3], f[4].into()),
                    "dose {case}"
                );
                let effects = dose
                    .effects
                    .iter()
                    .map(|e| match e {
                        Effect::Boost {
                            skill,
                            constant,
                            percent,
                        } => format!("B:{skill}:{constant}:{percent}"),
                        Effect::Recover {
                            skill,
                            constant,
                            percent,
                        } => format!("R:{skill}:{constant}:{percent}"),
                        Effect::Energy(percent) => format!("E:{percent}"),
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                assert_eq!(effects, f[5], "effects {case}");
                assert_eq!(potions::sip(dose.name), f[6]);
                assert_eq!(potions::remaining(dose.left), f[7]);
            }
            "R" => {
                formulas += 1;
                let n: Vec<i32> = f[1..].iter().map(|n| n.parse().unwrap()).collect();
                assert_eq!(
                    potions::restored_level(n[0], n[1], n[2], n[3]),
                    n[4],
                    "level {case}"
                );
                assert_eq!(
                    potions::restored_energy(n[0], n[3], n[1]),
                    n[5],
                    "energy {case}"
                );
            }
            _ => panic!("Unknown fixture row"),
        }
    }
    assert_eq!(catalogs, 76);
    assert_eq!(doses.len(), catalogs);
    assert_eq!(formulas, 4096);
}
