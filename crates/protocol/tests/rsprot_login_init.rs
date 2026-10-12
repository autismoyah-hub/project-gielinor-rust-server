use gielinor_protocol::login_init::{self, Operations};
#[test]
fn original_login_initialization_encoders() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/login-init.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1538);
    for line in fixture.lines() {
        let f: Vec<_> = line.split('|').collect();
        let p = match f[0] {
            "chat" => login_init::chat_filters(f[1].parse().unwrap(), f[2].parse().unwrap()),
            "npc" => login_init::hide_operations(Operations::Npc, f[1] == "true"),
            "loc" => login_init::hide_operations(Operations::Location, f[1] == "true"),
            "obj" => login_init::hide_operations(Operations::Object, f[1] == "true"),
            "op" => login_init::player_operation(
                f[1].parse().unwrap(),
                f[2] == "true",
                (f[3] != "-").then_some(f[3]),
            )
            .unwrap(),
            "map" => login_init::minimap(f[1].parse().unwrap()),
            "camera" => login_init::camera_reset(),
            "anims" => login_init::reset_animations(),
            _ => panic!("Unknown fixture"),
        };
        assert_eq!(
            p.payload
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>(),
            *f.last().unwrap(),
            "{line}"
        );
    }
}
