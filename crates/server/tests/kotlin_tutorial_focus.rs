use gielinor_protocol::{framing::Packet, interfaces, varp};
use gielinor_server::tutorial_ui::{Config, Effect};
use gielinor_world::{
    tutorial_focus,
    tutorial_steps::Step,
    ui::{Component, State},
    variables::{Transmit, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;

#[test]
fn original_focus_repeated_steps_and_end_preserve_packet_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-focus.expected"
    ))
    .unwrap();
    let mut names = BTreeMap::new();
    for line in fixture.lines().filter(|l| l.starts_with("C|")) {
        let f: Vec<_> = line.split('|').collect();
        names.insert((f[1], f[2]), f[3].parse::<u32>().unwrap());
    }
    let id = |kind: &str, name: &str| names[&(kind, name)];
    let comp = |name: &str| id("component", name);
    let config = Config {
        overlay: id("interface", "tutorial_overlay") as u16,
        atmosphere: Component(comp("toplevel_osrs_stretch:overlay_atmosphere")),
        blocker: comp("tutorial_overlay:noclick"),
        texts: [
            comp("tutorial_overlay:hint_text_1"),
            comp("tutorial_overlay:hint_text_2"),
            comp("tutorial_overlay:hint_text_3"),
            comp("tutorial_overlay:hint_text_4"),
        ],
        mesoverlay: id("interface", "mesoverlay") as u16,
        chatmodal: Component(comp("chatbox:chatmodal")),
        panels: tutorial_focus::SIDE_PANELS
            .iter()
            .map(|p| {
                (
                    Component(comp(&format!("toplevel_osrs_stretch:side{}", p.side))),
                    id("interface", p.interface) as u16,
                    p.unlock,
                )
            })
            .collect(),
        flashside: 0,
        wind_strike: comp("magic_spellbook:wind_strike"),
        home_teleport: comp("magic_spellbook:teleport_home_standard"),
        npcs: Step::ALL
            .into_iter()
            .flat_map(|s| {
                [tutorial_focus::hint_npc(s), tutorial_focus::outline_npc(s)]
                    .into_iter()
                    .flatten()
            })
            .map(|name| (name, id("npc", name) as u16))
            .collect(),
        locations: Step::ALL
            .into_iter()
            .filter_map(tutorial_focus::outline_location)
            .map(|name| (name, id("loc", name)))
            .collect(),
    };
    let base = Varp {
        id: 1021,
        permanent: false,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let bit = Varbit {
        base: 1021,
        lsb: 0,
        msb: 3,
    };
    let mut cases = 0;
    for line in fixture.lines().filter(|l| l.starts_with("R|")) {
        let f: Vec<_> = line.split('|').collect();
        let step = Step::from_var(f[1].parse().unwrap());
        let mask: u32 = f[2].parse().unwrap();
        let logged = f[3] == "10";
        let mut ui = State::default();
        for (i, &(parent, interface, _)) in config.panels.iter().enumerate() {
            if mask & (1 << i) != 0 {
                ui.open_sub(parent, interface, true).unwrap();
            }
        }
        let mut vars = Variables::default();
        let mut packets = Vec::<Packet>::new();
        let effects = config
            .apply(step, Some(37))
            .unwrap()
            .into_iter()
            .chain(config.apply(step, Some(37)).unwrap())
            .chain(config.end().unwrap());
        for effect in effects {
            match effect {
                Effect::Packet(p) => packets.push(p),
                Effect::Overlay { parent, interface } => {
                    ui.open_sub(parent, interface, true).unwrap();
                    packets.push(interfaces::open_sub(
                        ui.translate(parent).0,
                        interface,
                        true,
                    ));
                }
                Effect::Panel {
                    parent,
                    interface,
                    open,
                } => {
                    let current = ui.overlays.parent(interface);
                    if open && current.is_none() {
                        ui.open_sub(parent, interface, true).unwrap();
                        packets.push(interfaces::open_sub(
                            ui.translate(parent).0,
                            interface,
                            true,
                        ));
                    } else if !open {
                        if let Some(current) = current {
                            for (p, _) in ui.close_sub(current) {
                                packets.push(interfaces::close_sub(ui.translate(p).0));
                            }
                        }
                    }
                }
                Effect::VariableBit { value, .. } => {
                    if let Some(update) = vars.set_bit(base, bit, value, logged).unwrap() {
                        packets.push(varp::update(update.id, update.value));
                    }
                }
            }
        }
        let encoded = packets
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
        assert_eq!(encoded, f[5], "step {step:?}, mask{mask}, logged{logged}");
        assert_eq!(vars.get(1021).to_string(), f[4]);
        for &(parent, interface, _) in &config.panels {
            assert_eq!(ui.overlays.get(parent), Some(interface));
        }
        cases += 1;
    }
    assert_eq!(cases, 324);
}
