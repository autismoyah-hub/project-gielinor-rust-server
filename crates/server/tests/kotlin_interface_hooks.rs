use gielinor_protocol::client_script::Argument;
use gielinor_server::interface_hooks::{Action, Hooks};
use gielinor_world::ui::{operation_mask, Component, State};
use std::collections::BTreeMap;
#[test]
fn compiled_interface_listeners_match_packets_and_permissions() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/interface-hooks.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 1024);
    for (case, line) in fixture.lines().enumerate() {
        let columns: Vec<_> = line.split('|').collect();
        let kind: usize = columns[0].parse().unwrap();
        let ids: Vec<u32> = columns[1].split(',').map(|v| v.parse().unwrap()).collect();
        let start: i32 = columns[2].parse().unwrap();
        let end: i32 = columns[3].parse().unwrap();
        let names = [
            "prayerbook:filtermenu",
            "account_summary_sidepanel:summary_click_layer",
            "emote:contents",
            "settings_side:music_bobble_container",
            "settings_side:sound_bobble_container",
            "settings_side:areasounds_bobble_container",
            "settings_side:master_bobble_container",
            "settings_side:attack_priority_player_buttons",
            "settings_side:attack_priority_npc_buttons",
            "settings_side:display_dynamic_setting_1_buttons",
            "settings_side:brightness_bobble_container",
            "settings_side:zoom_slider",
        ];
        let components = names
            .into_iter()
            .zip(&ids)
            .map(|(n, &id)| (n.into(), id as i32))
            .collect();
        let interfaces = [
            "prayerbook",
            "account_summary_sidepanel",
            "emote",
            "settings_side",
        ]
        .into_iter()
        .enumerate()
        .map(|(i, n)| (n.into(), i as i32))
        .collect();
        let mut encoded = vec![1, b'i', 2, b's', 5, 0, 2];
        for key in [start, end] {
            encoded.extend(key.to_be_bytes());
            encoded.extend(b"emote\0");
        }
        encoded.push(0);
        let table = gielinor_cache::enums::EnumType::decode(&encoded)
            .unwrap()
            .unwrap();
        let hooks = Hooks::standard(
            &interfaces,
            &components,
            &BTreeMap::from([("emote_names".into(), 0)]),
            &BTreeMap::from([(0, table)]),
        )
        .unwrap();
        let mut ui = State::default();
        let packets = hooks.opened(kind as u16, &mut ui).unwrap();
        let actual = packets
            .iter()
            .map(|p| {
                format!(
                    "{}:{}",
                    p.opcode,
                    p.payload
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, columns[4], "case {case}");
        for action in &hooks.opens[&(kind as u16)] {
            if let Action::Events {
                component,
                start,
                end,
                mask,
            } = action
            {
                assert_eq!(ui.events.get(*component, *start), Some(*mask));
                assert_eq!(ui.events.get(*component, *end), Some(*mask));
                assert_eq!(ui.events.get(*component, end + 1), None);
            }
        }
    }
}
#[test]
fn failed_listener_does_not_publish_partial_permissions() {
    let component = Component::new(1, 2);
    let hooks = Hooks {
        opens: BTreeMap::from([(
            1,
            vec![
                Action::Events {
                    component,
                    start: 0,
                    end: 4,
                    mask: operation_mask(1).unwrap(),
                },
                Action::Script {
                    id: 1,
                    arguments: vec![Argument::String("embedded\0nul".into())],
                },
            ],
        )]),
    };
    let mut ui = State::default();
    assert!(hooks.opened(1, &mut ui).is_err());
    assert_eq!(ui.events.get(component, 0), None);
}
