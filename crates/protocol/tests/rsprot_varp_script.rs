use gielinor_protocol::{
    client_script::{self, Argument},
    varp,
};
#[test]
fn rsprot_varps_and_all_clientscript_argument_types() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/varp-script.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 2049);
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let (packet, expected) = match f[0] {
            "varp" => {
                let p = varp::update(f[1].parse().unwrap(), f[2].parse().unwrap());
                assert_eq!(p.opcode, f[3].parse().unwrap());
                (p, f[4])
            }
            "script" => {
                let ints = if f[5] == "-" {
                    vec![]
                } else {
                    f[5].split(',').map(|s| s.parse().unwrap()).collect()
                };
                let strings = if f[6] == "-" {
                    vec![]
                } else {
                    f[6].split(',').map(str::to_owned).collect()
                };
                (
                    client_script::run(
                        f[1].parse().unwrap(),
                        &[
                            Argument::Int(f[2].parse().unwrap()),
                            Argument::String(f[4].into()),
                            Argument::Long(f[3].parse().unwrap()),
                            Argument::IntArray(ints),
                            Argument::StringArray(strings),
                        ],
                    )
                    .unwrap(),
                    f[7],
                )
            }
            "empty" => (client_script::run(0, &[]).unwrap(), f[1]),
            _ => panic!(),
        };
        let hex: String = packet.payload.iter().map(|v| format!("{v:02x}")).collect();
        assert_eq!(hex, expected, "{line}");
    }
}
