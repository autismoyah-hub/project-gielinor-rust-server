use gielinor_server::skill_multi::{Dialog, OptionItem, Presentation};
use gielinor_world::ui::Component;

#[test]
fn original_if_skill_multi_wire_order_and_arguments() {
    let mut cases = 0;
    for line in include_str!("fixtures/skill-multi.expected").lines() {
        let fields: Vec<_> = line.split('|').collect();
        let n = |i: usize| fields[i].parse::<i32>().unwrap();
        let dialog = Dialog {
            title: "Smelt".into(),
            options: (0..n(0))
                .map(|index| OptionItem {
                    object: 2349 + index,
                    label: format!("Option{index}"),
                })
                .collect(),
            max_count: n(1),
            selected_count: n(2),
            verb: n(3),
        };
        let p = Presentation {
            interface: 270,
            parent: Component::new(162, 0),
            setup_script: 2046,
        };
        let mut packets = vec![gielinor_protocol::interfaces::open_sub(
            p.parent.0,
            p.interface,
            false,
        )];
        packets.extend(p.setup_packets(&dialog).unwrap());
        let actual = packets
            .iter()
            .map(|packet| {
                format!(
                    "{}:{}",
                    packet.opcode,
                    packet
                        .payload
                        .iter()
                        .map(|v| format!("{v:02x}"))
                        .collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, fields[4], "{line}");
        cases += 1;
    }
    assert_eq!(cases, 36);
}
