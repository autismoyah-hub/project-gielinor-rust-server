#![cfg(windows)]
use gielinor_cache::{
    definitions::{InventoryType, StatType},
    game::Definitions,
};
use gielinor_protocol::rsa::PrivateKey;
use gielinor_server::{
    account_worker::{Completion, Request},
    authentication::Authenticated,
    characters::Policy,
    runtime::{AccountBackend, Event, Runtime},
    transport::{Limits, Transport},
};
use gielinor_storage::{accounts::Accounts, sqlite::Database};
use gielinor_world::{simulation::World, CoordGrid};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    io::{Read, Write},
    net::TcpStream,
    rc::Rc,
    sync::mpsc::{TryRecvError, TrySendError},
    time::{Duration, Instant},
};

/// Gameplay assertions ignore tick delimiters, but this adapter independently
/// enforces exactly one delimiter between consecutive player-info batches.
struct TickDecoder {
    inner: gielinor_protocol::framing::Decoder,
    tick_ends: usize,
    player_info: bool,
}
impl TickDecoder {
    fn feed(&mut self, bytes: &[u8]) -> Result<(), gielinor_protocol::framing::FrameError> {
        self.inner.feed(bytes)
    }
    fn next_packet(
        &mut self,
    ) -> Result<Option<gielinor_protocol::framing::Packet>, gielinor_protocol::framing::FrameError>
    {
        while let Some(packet) = self.inner.next_packet()? {
            if packet.opcode == 83 {
                assert!(packet.payload.is_empty());
                assert!(
                    self.player_info,
                    "Duplicate or premature server tick delimiter"
                );
                self.player_info = false;
                self.tick_ends += 1;
                continue;
            }
            if packet.opcode == 28 {
                assert!(!self.player_info, "Missing server tick delimiter");
                self.player_info = true;
            }
            return Ok(Some(packet));
        }
        Ok(None)
    }
}

#[derive(Default)]
struct Queues {
    accept: Cell<bool>,
    requests: RefCell<VecDeque<Request>>,
    completed: RefCell<VecDeque<Completion>>,
}
#[derive(Clone)]
struct Backend(Rc<Queues>);
impl AccountBackend for Backend {
    fn submit(&self, r: Request) -> Result<(), TrySendError<Request>> {
        if !self.0.accept.get() {
            return Err(TrySendError::Full(r));
        }
        self.0.requests.borrow_mut().push_back(r);
        Ok(())
    }
    fn poll(&self) -> Result<Completion, TryRecvError> {
        self.0
            .completed
            .borrow_mut()
            .pop_front()
            .ok_or(TryRecvError::Empty)
    }
}
fn hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn pump(runtime: &mut Runtime<Backend>) -> Vec<Event> {
    runtime
        .pump(1791770523, "2026-10-12T01:02:03", || Ok(0x1020304050607080))
        .unwrap()
}
#[track_caller]
fn read(runtime: &mut Runtime<Backend>, client: &mut TcpStream, bytes: &mut [u8]) {
    client.set_nonblocking(true).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut offset = 0;
    while offset < bytes.len() {
        pump(runtime);
        match client.read(&mut bytes[offset..]) {
            Ok(0) => panic!("peer closed"),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => (),
            Err(e) => panic!("{e}"),
        };
        assert!(
            Instant::now() < deadline,
            "TCP read timed out after {offset}/{} bytes",
            bytes.len()
        );
        if offset < bytes.len() {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    client.set_nonblocking(false).unwrap();
}
// Context switches are high priority and no longer delimit a completed tick.
fn finish_tick(
    runtime: &mut Runtime<Backend>,
    client: &mut TcpStream,
    decoder: &mut TickDecoder,
) -> Vec<gielinor_protocol::framing::Packet> {
    let end = decoder.tick_ends + 1;
    let mut packets = Vec::new();
    while decoder.tick_ends < end {
        if let Some(packet) = decoder.next_packet().unwrap() {
            packets.push(packet);
        } else if decoder.tick_ends < end {
            let mut byte = [0];
            read(runtime, client, &mut byte);
            decoder.feed(&byte).unwrap();
        }
    }
    packets
}
fn reference_sorted<const N: usize>(mut opcodes: [u16; N]) -> [u16; N] {
    reference_order(&mut opcodes);
    opcodes
}
fn reference_order(opcodes: &mut [u16]) {
    let categories = include_str!("../../protocol/tests/fixtures/outgoing-categories.expected");
    opcodes.sort_by_key(|opcode| {
        let row = categories
            .lines()
            .find(|row| {
                row.split_whitespace()
                    .next()
                    .and_then(|op| op.parse::<u16>().ok())
                    == Some(*opcode)
            })
            .unwrap();
        match row.split_whitespace().nth(2).unwrap() {
            // Original post-tick process emits inventory after info/zone updates.
            "HIGH_PRIORITY_PROT" if matches!(*opcode, 18 | 22) => 1,
            "HIGH_PRIORITY_PROT" => 0,
            "LOW_PRIORITY_PROT" => 2,
            value => panic!("Unexpected original category {value}"),
        }
    });
}
#[test]
fn tcp_login_world_admission_and_save_backpressure_keep_account_reserved() {
    exercise_lifecycle(false, false);
}
#[test]
fn configured_login_initializes_vars_camera_gameframe_menus_and_minimap_over_tcp() {
    exercise_lifecycle(true, false);
}
#[test]
fn tutorial_completed_login_preserves_visible_appearance_and_standard_packet_order() {
    exercise_lifecycle(true, true);
}
fn exercise_lifecycle(with_login: bool, completed_tutorial: bool) {
    let mut defs = Definitions {
        inventories: [93, 94, 95]
            .into_iter()
            .map(|id| (id, InventoryType::decode(&[2, 0, 28, 200, 1, 0]).unwrap()))
            .collect(),
        items: BTreeMap::from([
            (
                7,
                gielinor_cache::definitions::ItemType::decode(&[35, b'E', b'a', b't', 0, 0])
                    .unwrap(),
            ),
            (
                8,
                gielinor_cache::definitions::ItemType::decode(&[
                    13, 0, 36, b'W', b'e', b'a', b'r', 0, 0,
                ])
                .unwrap(),
            ),
        ]),
        npcs: BTreeMap::new(),
        locations: BTreeMap::new(),
        varps: BTreeMap::from([(
            1000,
            gielinor_world::variables::Varp {
                id: 1000,
                permanent: true,
                transmit: gielinor_world::variables::Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
        )]),
        varbits: BTreeMap::from([(
            2000,
            gielinor_world::variables::Varbit {
                base: 1000,
                lsb: 4,
                msb: 7,
            },
        )]),
        stats: BTreeMap::from([(3, StatType::decode(&[4, 10, 0]).unwrap())]),
    };
    defs.inventories.get_mut(&95).unwrap().stack = gielinor_world::inventory::StackType::Always;
    for id in [
        1933, 1929, 1931, 1925, 2307, 2309, 436, 438, 2349, 2347, 1205, 1265,
    ] {
        defs.items.insert(
            id,
            gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
        );
    }
    defs.stats.insert(7, StatType::decode(&[0]).unwrap());
    defs.stats.insert(13, StatType::decode(&[0]).unwrap());
    defs.varps.insert(
        7000,
        gielinor_world::variables::Varp {
            id: 7000,
            permanent: true,
            transmit: gielinor_world::variables::Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
    );
    let mut bar_varp = defs.varps[&7000];
    bar_varp.id = 7002;
    defs.varps.insert(7002, bar_varp);
    defs.varbits.insert(
        7100,
        gielinor_world::variables::Varbit {
            base: 7002,
            lsb: 0,
            msb: 3,
        },
    );
    defs.locations.insert(
        60002,
        gielinor_cache::locations::LocationType::decode(&[0]).unwrap(),
    );
    defs.locations.insert(
        60001,
        gielinor_cache::locations::LocationType::decode(&[0]).unwrap(),
    );
    if with_login {
        for id in 100..115 {
            defs.varps.insert(
                id,
                gielinor_world::variables::Varp {
                    id: id as u16,
                    permanent: true,
                    transmit: gielinor_world::variables::Transmit::Always,
                    bit_protect: false,
                    client_code: -1,
                },
            );
        }
    }
    let bank_names: Vec<String> = (1..=9)
        .map(|i| format!("bank_tab_{i}"))
        .chain(
            [
                "bank_tab_main",
                "bank_currenttab",
                "bank_capacity",
                "bank_withdrawnotes",
                "bank_leaveplaceholders",
                "bank_quantity_type",
                "bank_requestedquantity",
                "bank_insertmode",
                "bank_fillermode",
                "bank_showincinerator",
                "bank_hidebanktut",
                "bank_hidesideops",
                "bank_hidedepositinv",
                "bank_hidedepositworn",
                "bank_tab_display",
            ]
            .map(str::to_owned),
        )
        .collect();
    let bank_vars = bank_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| {
            let base = 1100 + i as u16;
            defs.varps.insert(
                base as u32,
                gielinor_world::variables::Varp {
                    id: base,
                    permanent: true,
                    transmit: gielinor_world::variables::Transmit::Always,
                    bit_protect: false,
                    client_code: -1,
                },
            );
            defs.varbits.insert(
                2100 + i as u32,
                gielinor_world::variables::Varbit {
                    base,
                    lsb: 0,
                    msb: 15,
                },
            );
            (name, 2100 + i as i32)
        })
        .collect();
    let policy = Policy {
        world: 1,
        inventory: 93,
        equipment: 94,
        bank: 95,
        hitpoints: 3,
        spawn: CoordGrid::new(3200, 3200, 0).unwrap(),
        base_xp_rate_hundreds: 100,
        auto_display_names: true,
        dev_modlevel: None,
        modlevels: BTreeMap::new(),
    };
    let key = PrivateKey::from_be_bytes(
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-exponent.hex"
        )),
        &hex(include_str!(
            "../../../reference/protocol/login-fixture/rsa-modulus.hex"
        )),
    )
    .unwrap();
    let crcs = std::array::from_fn(|i| 0x80102030u32.wrapping_add(i as u32 * 0x010203));
    let transport =
        Transport::bind("127.0.0.1:0".parse().unwrap(), key, crcs, Limits::default()).unwrap();
    let queues = Rc::new(Queues::default());
    queues.accept.set(true);
    let mut runtime = Runtime::new(
        World::default(),
        transport,
        defs,
        policy,
        Backend(queues.clone()),
    );
    let component = |events| gielinor_cache::components::ComponentType {
        v3: true,
        kind: 0,
        button_type: 0,
        layer: None,
        events,
        operations: vec![],
        encoded: vec![],
        decoded_bytes: 0,
    };
    let bank_control_symbols: BTreeMap<String, i32> = [
        "swap",
        "insert",
        "item",
        "note",
        "quantity1",
        "quantity5",
        "quantity10",
        "quantityall",
        "bank_filler_1",
        "bank_filler_10",
        "bank_filler_50",
        "bank_filler_x",
        "bank_filler_all",
        "placeholder",
        "incinerator_toggle",
        "banktut_toggle",
        "sideops_toggle",
        "depositinv_toggle",
        "depositworn_toggle",
        "dropdown_content",
        "quantityx",
        "tabs",
        "depositinv",
        "depositworn",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, name)| (format!("bankmain:{name}"), (12 << 16) | (100 + i as i32)))
    .collect();
    let mut components = BTreeMap::from([
        ((548 << 16) | 20, component(0)),
        ((548 << 16) | 21, component(0)),
        ((548 << 16) | 22, component(0)),
        ((548 << 16) | 23, component(0)),
        (193 << 16, component(0)),
        ((193 << 16) | 1, component(0)),
        ((193 << 16) | 2, component(0)),
        (219 << 16, component(0)),
        ((219 << 16) | 1, component(0)),
        (584 << 16, component(0)),
        ((155 << 16) | 5, component(2)),
        ((155 << 16) | 6, component(2)),
        (201 << 16, component(0)),
        (12 << 16, component(0)),
        (15 << 16, component(0)),
        (149 << 16, component(4)),
        ((149 << 16) | 1, component(2)),
        (150 << 16, component(0)),
        ((150 << 16) | 1, component(2)),
        ((150 << 16) | 2, component(2)),
        ((150 << 16) | 3, component(2)),
        (151 << 16, component(0)),
        ((151 << 16) | 1, component(0)),
        (152 << 16, component(0)),
        (153 << 16, component(0)),
        (154 << 16, component(0)),
        (600 << 16, component(2)),
        ((229 << 16) | 4, component(1)),
        (679 << 16, component(2)),
        ((679 << 16) | 1, component(2)),
        ((679 << 16) | 2, component(2)),
        ((679 << 16) | 3, component(2)),
        (929 << 16, component(0)),
        ((929 << 16) | 1, component(0)),
        ((929 << 16) | 2, component(0)),
        (162 << 16, component(2)),
        ((548 << 16) | 24, component(0)),
        ((548 << 16) | 25, component(0)),
    ]);
    components.extend((1..=5).map(|child| (((153u32 << 16) | child), component(0))));
    components.extend(
        bank_control_symbols
            .values()
            .map(|&id| (id as u32, component(6))),
    );
    let bonus_components = std::array::from_fn(|i| (12u32 << 16) | (200 + i as u32));
    let bonus_tooltip = (12u32 << 16) | 218;
    components.extend(
        bonus_components
            .iter()
            .chain(std::iter::once(&bonus_tooltip))
            .map(|&id| (id, component(0))),
    );
    for top in [549u32, 550] {
        for child in [20, 21, 22] {
            components.insert((top << 16) | child, component(0));
        }
    }
    for child in 0..5 {
        components.insert((155 << 16) | child, component(2));
    }
    for child in 0..6 {
        components.insert(
            (614 << 16) | child,
            component(if child == 0 { 2 } else { 0 }),
        );
    }
    for interface in [217, 231] {
        for child in 0..7 {
            components.insert(
                (interface << 16) | child,
                component(if child == 5 { 1 } else { 0 }),
            );
        }
    }
    for child in 15..33 {
        components.insert((270 << 16) | child, component(0));
    }
    for child in 3..=9 {
        components.insert((312 << 16) | child, component(2));
    }
    runtime.configure_interfaces(components).unwrap();
    runtime
        .configure_smelting(
            gielinor_server::smelting::Config {
                presentation: gielinor_server::skill_multi::Presentation {
                    interface: 270,
                    parent: gielinor_world::ui::Component::new(548, 20),
                    setup_script: 2046,
                },
                recipes: vec![gielinor_world::smelting::Recipe {
                    bar: 2349,
                    label: "Bronze".into(),
                    level: 1,
                    xp: 6.2,
                    ingredients: vec![(436, 1), (438, 1)],
                }],
                types: [(436, "Copper ore"), (438, "Tin ore"), (2349, "Bronze bar")]
                    .into_iter()
                    .map(|(id, name)| {
                        (
                            id,
                            gielinor_world::smelting::ItemType {
                                name: name.into(),
                                certificate: false,
                                placeholder: false,
                            },
                        )
                    })
                    .collect(),
                quantity_varp: 7000,
                skill: 13,
                animation: 899,
            },
            gielinor_world::smithing_make::Config {
                queue: 900,
                skill: 13,
                max_level: 99,
                xp_rate: 1.0,
                tutorial: None,
            },
        )
        .unwrap();
    runtime
        .configure_furnaces(
            BTreeMap::from([(
                60002,
                gielinor_world::smelting::Furnace {
                    operations: 1 << 1,
                    item_use: true,
                    ore_ids: None,
                    tutorial: None,
                },
            )]),
            [436, 438].into_iter().collect(),
        )
        .unwrap();
    runtime
        .configure_anvil(gielinor_server::anvil::Config {
            rules: gielinor_world::anvil::Config {
                hammer: 2347,
                skill: 13,
                animation: 898,
                bar_by_tier: BTreeMap::from([(1, 2349)]),
                products: BTreeMap::from([(2349, BTreeMap::from([(9, 1205)]))]),
                quantity: BTreeMap::new(),
                bars: BTreeMap::new(),
                level: BTreeMap::from([(1205, 0)]),
                xp_per_bar: BTreeMap::from([(2349, 12.5)]),
                names: [(2349, "Bronze bar"), (1205, "Bronze dagger")]
                    .into_iter()
                    .map(|(id, name)| (id, name.into()))
                    .collect(),
            },
            interface: 312,
            parent: gielinor_world::ui::Component::new(548, 20),
            bar_type: gielinor_world::bank_actions::Binding {
                base: runtime.definitions.varps[&7002],
                bit: runtime.definitions.varbits[&7100],
            },
            bar_type_id: 7100,
            quantity: runtime.definitions.varps[&7000],
            quantity_buttons: [
                (3, Some(1)),
                (4, Some(5)),
                (5, Some(10)),
                (6, None),
                (7, Some(28)),
            ]
            .into_iter()
            .map(|(slot, count)| ((312 << 16) | slot, count))
            .collect(),
            locations: Default::default(),
            bars: [2349].into_iter().collect(),
        })
        .unwrap();
    {
        use gielinor_server::player_design::{Bindings, Config, Control, Kits};
        use gielinor_world::{
            bank_actions::Binding,
            variables::{Transmit, Varbit, Varp},
        };
        let binding = |id| Binding {
            base: Varp {
                id,
                permanent: false,
                transmit: Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: id,
                lsb: 0,
                msb: 1,
            },
        };
        runtime
            .configure_player_design(Bindings {
                config: Config {
                    kits: Kits::default(),
                    body: binding(5000),
                    pronoun: binding(5001),
                },
                buttons: [
                    (679 << 16, Control::Body(1)),
                    ((679 << 16) | 1, Control::Body(0)),
                ]
                .into(),
                pronoun_component: (679 << 16) | 2,
            })
            .unwrap();
        for id in 5000..=5005 {
            runtime
                .definitions
                .varps
                .insert(id as u32, binding(id).base);
            runtime.definitions.varbits.insert(
                id as u32,
                gielinor_world::variables::Varbit {
                    base: id,
                    lsb: 0,
                    msb: if id == 5002 { 3 } else { 1 },
                },
            );
        }
        use gielinor_world::ui::Component;
        runtime
            .configure_tutorial(gielinor_server::tutorial_character::Config {
                panels: [
                    (162 << 16, gielinor_world::tutorial_panels::Panel::Worn),
                    (614 << 16, gielinor_world::tutorial_panels::Panel::Settings),
                ]
                .into(),
                progression: gielinor_world::tutorial_progression::Config {
                    step: binding(5003).base,
                    completed: binding(5004).base,
                },
                focus: gielinor_server::tutorial_ui::Config {
                    overlay: 614,
                    atmosphere: Component::new(548, 24),
                    blocker: (614 << 16) | 1,
                    texts: [2, 3, 4, 5].map(|child| (614 << 16) | child),
                    mesoverlay: 162,
                    chatmodal: Component::new(548, 25),
                    panels: vec![],
                    flashside: 5002,
                    wind_strike: (679 << 16) | 3,
                    home_teleport: 0,
                    npcs: gielinor_world::tutorial_steps::Step::ALL
                        .into_iter()
                        .flat_map(|step| {
                            [
                                gielinor_world::tutorial_focus::hint_npc(step),
                                gielinor_world::tutorial_focus::outline_npc(step),
                            ]
                            .into_iter()
                            .flatten()
                        })
                        .map(|name| (name, 1))
                        .collect(),
                    locations: gielinor_world::tutorial_steps::Step::ALL
                        .into_iter()
                        .filter_map(gielinor_world::tutorial_focus::outline_location)
                        .map(|name| (name, 1))
                        .collect(),
                },
                mainmodal: Component::new(548, 23),
                creator: 679,
                confirm: (679 << 16) | 3,
                pronouns: Component::new(679, 2),
                experience: 929,
                experience_buttons: [0, 1, 2].map(|child| Component::new(929, child)),
                body_bit: 5000,
                pronoun_bit: 5001,
                adventure_bit: 5005,
            })
            .unwrap();
    }
    runtime
        .configure_tutorial_gates(
            [(60000, gielinor_world::tutorial_gates::Gate::SurvivalEntry)].into(),
        )
        .unwrap();
    runtime
        .configure_tutorial_dough(1933, 1929, 1931, 1925, 2307)
        .unwrap();
    runtime
        .configure_tutorial_bread(60001, 2307, 2309, 897, 7)
        .unwrap();
    runtime
        .configure_dialogue(
            gielinor_server::dialogue::Presentation {
                parent: gielinor_world::ui::Component::new(548, 25),
                unclamp: 2000,
                player: 217,
                npc: 231,
                choice: 219,
                animations: [[0; 4], [0; 4], [0; 4], [0; 4], [6000, 6001, 6002, 6003]],
                font: gielinor_world::dialogue_text::Font {
                    advances: [6; 256],
                    kerning: vec![],
                },
            },
            -999,
            -1,
        )
        .unwrap();
    {
        use gielinor_world::{
            bank_actions::Binding,
            drop_actions::{Config, Kind, Rule},
            ui::Component,
            variables::Varbit,
        };
        let binding = |lsb, msb| Binding {
            base: runtime.definitions.varps[&1000],
            bit: Varbit {
                base: 1000,
                lsb,
                msb,
            },
        };
        let config = Config {
            warning_enabled: binding(0, 0),
            warning_threshold: binding(8, 23),
            sound: 2739,
            rules: [(
                8,
                Rule {
                    kind: Kind::Drop,
                    name: "Test item".into(),
                    market_price: 100,
                    destroy_title: "Destroy?".into(),
                    destroy_description: "Cannot undo".into(),
                    release_title: None,
                    release_message: Some("Released.".into()),
                },
            )]
            .into(),
        };
        runtime
            .configure_dropping(
                gielinor_server::drop_dialog::Presentation {
                    parent: Component::new(548, 23),
                    unclamp: 2000,
                    object_interface: 193,
                    object_button: Component::new(193, 0),
                    object_model: Component::new(193, 1),
                    object_text: Component::new(193, 2),
                    choice_interface: 219,
                    choice_button: Component::new(219, 0),
                    destroy_interface: 584,
                    destroy_button: Component::new(584, 0),
                },
                config,
            )
            .unwrap();
    }
    if with_login {
        use gielinor_world::{
            bank_actions::Binding,
            gameframe::{Frame, Frames},
            ui::Component,
            variables::Varbit,
        };
        let binding = |id, lsb, msb| Binding {
            base: runtime.definitions.varps[&(id as u32)],
            bit: Varbit { base: id, lsb, msb },
        };
        let filter = binding(104, 2, 2);
        let minimap = binding(105, 0, 3);
        runtime
            .configure_login(gielinor_server::login_init::Presentation {
                variables: gielinor_server::login_init::Config {
                    chatbox: binding(104, 0, 0),
                    roofs: binding(104, 1, 1),
                    volumes: std::array::from_fn(|i| runtime.definitions.varps[&(100 + i as u32)]),
                    music_mode: None,
                    transmit: runtime.definitions.varps.values().copied().collect(),
                },
                frames: Frames {
                    top: binding(106, 0, 15),
                    stones: binding(107, 0, 0),
                    ordered: (0..3)
                        .map(|i| Frame {
                            top: 548 + i,
                            overlays: vec![
                                (Component::new(548, 20), 149),
                                (Component::new(548, 21), 150),
                                (Component::new(548, 22), 155),
                            ],
                            mappings: (20..=22)
                                .map(|child| {
                                    (Component::new(548, child), Component::new(548 + i, child))
                                })
                                .collect(),
                            client_mode: i as i32,
                            resizable: i != 0,
                            default: i == 0,
                            stones: i == 2,
                        })
                        .collect(),
                },
                welcome: None,
                broadcast: None,
            })
            .unwrap();
        runtime
            .configure_gameframe_moves(gielinor_server::gameframe_move::Config {
                button: 149 << 16,
                choices: [548, 549, 550],
                targets: vec![Component::new(548, 20), Component::new(548, 21)],
                events: BTreeMap::new(),
                xp_drops: Component::new(548, 21),
                ehc_listener: Component::new(548, 20),
                orbs: (Component::new(548, 22), 154),
                resync: [runtime.definitions.varps[&1000]; 2],
                minimap,
            })
            .unwrap();
        runtime
            .configure_interface_hooks(gielinor_server::interface_hooks::Hooks {
                opens: BTreeMap::from([
                    (
                        149,
                        vec![gielinor_server::interface_hooks::Action::Events {
                            component: Component::new(149, 1),
                            start: 0,
                            end: 4,
                            mask: gielinor_world::ui::operation_mask(1).unwrap(),
                        }],
                    ),
                    (
                        151,
                        vec![gielinor_server::interface_hooks::Action::Events {
                            component: Component::new(151, 1),
                            start: 3,
                            end: 7,
                            mask: 15u64 << 32,
                        }],
                    ),
                ]),
            })
            .unwrap();
        runtime
            .configure_interface_controls(BTreeMap::from([(
                (149 << 16) | 1,
                gielinor_server::interface_controls::Control::ToggleSlot(vec![filter; 5]),
            )]))
            .unwrap();
    }
    runtime
        .configure_bonuses(
            gielinor_server::bonuses::Config {
                items: runtime
                    .definitions
                    .items
                    .keys()
                    .map(|&id| (id as i32, gielinor_world::bonuses::ItemBonuses::default()))
                    .collect(),
                speeds: runtime
                    .definitions
                    .items
                    .keys()
                    .map(|&id| (id as i32, 4))
                    .collect(),
                attack_style_varp: 9000,
            },
            gielinor_server::bonuses::Styles {
                packed: BTreeMap::from([(0, 0x7652)]),
            },
            Some(gielinor_server::bonuses::BankDisplay {
                components: bonus_components,
                tooltip: bonus_tooltip,
            }),
        )
        .unwrap();
    runtime
        .configure_foods(
            149 << 16,
            BTreeMap::from([(
                7,
                gielinor_world::food::Edible {
                    id: 7,
                    heal: 3,
                    delay: 1,
                    replacement: None,
                    message: "You eat the shrimp.".into(),
                },
            )]),
            829,
        )
        .unwrap();
    let params = [
        "statreq1_skill",
        "statreq1_level",
        "statreq2_skill",
        "statreq2_level",
        "statreq_failmessage1",
        "statreq_failmessage2",
    ]
    .into_iter()
    .enumerate()
    .map(|(i, s)| (s.to_owned(), i as i32 + 100))
    .collect();
    runtime
        .configure_equipment(149 << 16, BTreeMap::from([((149 << 16) | 1, 0)]), &params)
        .unwrap();
    runtime
        .configure_banking(
            gielinor_server::runtime::BankPresentation {
                main_interface: 12,
                side_interface: 15,
                main_components: vec![12 << 16],
                side_component: 15 << 16,
                default_capacity: 28,
            },
            &bank_vars,
            &BTreeMap::from([("no_bank".to_owned(), 106)]),
        )
        .unwrap();
    runtime
        .configure_bank_controls(&bank_control_symbols, &bank_vars)
        .unwrap();
    let npc = runtime
        .world
        .spawn_npc(
            gielinor_world::npc::Npc::new(
                1,
                CoordGrid::new(3201, 3200, 0).unwrap(),
                1,
                gielinor_world::npc::Restrict::NoMove,
                gielinor_world::npc::BlockWalk::None,
            )
            .unwrap(),
        )
        .unwrap();
    runtime
        .configure_bankers(
            &[],
            (
                gielinor_world::ui::Component::new(548, 21),
                gielinor_world::ui::Component::new(548, 22),
            ),
        )
        .unwrap();
    for id in [61000, 61001] {
        runtime.world.location_types.insert(
            id,
            gielinor_world::loc_collision::LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
    }
    runtime.configure_tutorial_bank(61000, 61001, 8).unwrap();
    runtime
        .enable_player_synchronization(gielinor_server::runtime::Presentation {
            animations: gielinor_server::appearance::AnimationPolicy {
                human: [808, 823, 819, 820, 821, 822, 824],
                weapon_params: None,
            },
            combat_stats: [0, 1, 2, 3, 4, 6, 5],
            radius: 15,
        })
        .unwrap();
    if with_login {
        use gielinor_world::{bank_actions::Binding, ui::Component, variables::Varbit};
        runtime
            .configure_journal(gielinor_server::journal::Journal {
                diary: Some(gielinor_server::journal::Diary {
                    header: (153 << 16) | 1,
                    hidden: [2, 3, 4, 5].map(|id| (153 << 16) | id),
                    // Synthetic cache stat identifiers; production resolves symbols.
                    skills: [7, 13, 3],
                }),
                interface: 150,
                tabs: [151, 152, 153],
                parent: Component::new(150, 0),
                contents: Component::new(151, 0),
                click_layer: Component::new(151, 1),
                selected: Binding {
                    base: runtime.definitions.varps[&107],
                    bit: Varbit {
                        base: 107,
                        lsb: 1,
                        msb: 2,
                    },
                },
                playtime: 1000,
                switching: Some(gielinor_server::journal::Switching {
                    buttons: (0..3)
                        .map(|i| (((150u32 << 16) | (i + 1)), i as usize))
                        .collect(),
                    collection: [runtime.definitions.varps[&1000]; 12],
                }),
            })
            .unwrap();
        let binding = |id, msb| Binding {
            base: runtime.definitions.varps[&(id as u32)],
            bit: Varbit {
                base: id,
                lsb: 0,
                msb,
            },
        };
        runtime
            .configure_combat_tab(gielinor_server::combat_tab::Config {
                presentation: gielinor_server::combat_tab::Presentation {
                    interface: 155,
                    category: binding(111, 5),
                    level: binding(112, 7),
                    decimal: binding(113, 7),
                },
                stance: gielinor_world::combat_stance::Config {
                    stance: runtime.definitions.varps[&108],
                    melee: runtime.definitions.varps[&109],
                    retaliate: runtime.definitions.varps[&110],
                    saved: BTreeMap::from([(0, binding(114, 1))]),
                    styles: BTreeMap::from([(0, 0x4321)]),
                    weapons: BTreeMap::new(),
                },
                buttons: (0..5)
                    .map(|i| ((155 << 16) | i, if i < 4 { Some(i as u8) } else { None }))
                    .collect(),
            })
            .unwrap();
        configure_autocast(&mut runtime);
    }
    runtime.enable_ground_synchronization().unwrap();
    let mut client = TcpStream::connect(runtime.transport.address().unwrap()).unwrap();
    client.write_all(&[14]).unwrap();
    let mut challenge = [0; 9];
    read(&mut runtime, &mut client, &mut challenge);
    assert_eq!(challenge, [0, 16, 32, 48, 64, 80, 96, 112, 128]);
    let payload = hex(include_str!(
        "../../../reference/protocol/login-fixture/login.hex"
    ));
    let mut bytes = vec![16];
    bytes.extend((payload.len() as u16).to_be_bytes());
    bytes.extend(payload);
    client.write_all(&bytes).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let connection = loop {
        pump(&mut runtime);
        if let Some(Request::Authenticate {
            connection, login, ..
        }) = queues.requests.borrow_mut().pop_front()
        {
            assert_eq!(login.username, "parity_actor");
            break connection;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    let mut accounts = Accounts::from_database(Database::memory().unwrap());
    accounts.initialize().unwrap();
    accounts.create("parity_actor", "fixture-hash", 1).unwrap();
    let mut account = accounts.load("parity_actor", 1).unwrap().unwrap();
    if completed_tutorial {
        account.varps_json = "{\"5004\":1}".into();
    }
    let observer_id = account.character_id;
    queues
        .completed
        .borrow_mut()
        .push_back(Completion::Authenticated {
            connection,
            result: Ok(Authenticated {
                account,
                new_account: true,
                authenticator_code: None,
            }),
        });
    pump(&mut runtime);
    assert_eq!(runtime.world.actors().count(), 0);
    let events = runtime.tick().unwrap();
    let actor = events
        .iter()
        .find_map(|event| match event {
            Event::PlayerReady { actor, .. } => Some(*actor),
            _ => None,
        })
        .unwrap();
    assert_eq!(runtime.world.data(actor).unwrap().stats.base(3), 10);
    assert_eq!(
        runtime
            .world
            .banking
            .as_ref()
            .unwrap()
            .capacity
            .get(&runtime.world.data(actor).unwrap().variables)
            .unwrap(),
        if with_login { 28 } else { 0 }
    );
    let mut accepted = [0; 36];
    read(&mut runtime, &mut client, &mut accepted);
    assert_eq!(&accepted[..2], &[2, 37]);
    let mut outgoing = TickDecoder {
        inner: gielinor_protocol::framing::Decoder::new(
            gielinor_protocol::tables::SERVER,
            Some(gielinor_protocol::isaac::Isaac::new(&[0x80000032, 289, 51, 49]).unwrap()),
            gielinor_protocol::framing::OpcodeWidth::Smart,
            65535,
            131072,
        ),
        tick_ends: 0,
        player_info: false,
    };
    let mut inventory_ids = vec![];
    let mut initial_opcodes = if with_login {
        vec![
            49, 124, 75, 21, 73, 44, 97, 97, 97, 97, 97, 97, 3, 114, 114, 96, 7, 108, 7, 114, 114,
            7, 108, 7, 114, 114, 114, 114, 114, 6, 6, 6, 6, 6, 114, 46, 31, 64, 92, 43, 22, 22, 47,
            116, 122, 28, 85, 47,
        ]
    } else {
        vec![49, 22, 22, 46, 31, 64, 47, 116, 122, 28, 85, 47]
    };
    if completed_tutorial {
        initial_opcodes.insert(12, 97);
    }
    // Ordered fixture stats: Hitpoints3, Cooking7, Smithing13.
    let first_stat = initial_opcodes.iter().position(|&op| op == 46).unwrap();
    initial_opcodes.insert(first_stat + 1, 46);
    initial_opcodes.insert(first_stat + 2, 46);
    // Build hook-order expectations from original fixtures, then apply the independently
    // extracted RSProt category schedule. Payload checks remain exact.
    let mut initial: Vec<(u16, Option<Vec<u8>>)> =
        initial_opcodes.into_iter().map(|op| (op, None)).collect();
    if with_login && !completed_tutorial {
        let fixture = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/tutorial-bootstrap.expected"
        ))
        .unwrap();
        let row = fixture
            .lines()
            .find(|row| row.starts_with("0|0|0|-1|"))
            .unwrap();
        let bootstrap = row.split('|').nth(6).unwrap().split(',').map(|encoded| {
            let (op, payload) = encoded.split_once(':').unwrap();
            (op.parse::<u16>().unwrap(), Some(hex(payload)))
        });
        initial.splice(26..26, bootstrap);
        assert!(runtime.appearance_design(actor).unwrap().hidden);
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .contains_interface(679));
    }
    if with_login {
        use gielinor_protocol::{
            client_script::{self, Argument},
            interfaces,
        };
        let header = (153 << 16) | 1;
        let packets = [2, 3, 4, 5]
            .map(|id| interfaces::hide((153 << 16) | id, true))
            .into_iter()
            .chain(std::iter::once(
                client_script::run(600, &[0, 0, 10, header as i32].map(Argument::Int)).unwrap(),
            ))
            .chain(std::iter::once(
                interfaces::text(
                    header,
                    &gielinor_world::tower_diary_ui::body(Default::default()),
                )
                .unwrap(),
            ));
        let position = 26 + usize::from(completed_tutorial);
        initial.splice(
            position..position,
            packets.map(|packet| (packet.opcode, Some(packet.payload))),
        );
    }
    for x in (24..=72).step_by(8) {
        for z in (24..=72).step_by(8) {
            let packet = gielinor_protocol::zone::reset(x, z, 0);
            initial.push((packet.opcode, Some(packet.payload)));
        }
    }
    let categories = include_str!("../../protocol/tests/fixtures/outgoing-categories.expected");
    initial.sort_by_key(|(opcode, _)| {
        let row = categories
            .lines()
            .find(|row| {
                row.split_whitespace()
                    .next()
                    .and_then(|op| op.parse::<u16>().ok())
                    == Some(*opcode)
            })
            .unwrap();
        match row.split_whitespace().nth(2).unwrap() {
            "HIGH_PRIORITY_PROT" if matches!(*opcode, 18 | 22) => 1,
            "HIGH_PRIORITY_PROT" => 0,
            "LOW_PRIORITY_PROT" => 2,
            value => panic!("Unexpected original category {value}"),
        }
    });
    let first_stat = initial.iter().position(|(op, _)| *op == 46).unwrap();
    for (index, (expected, payload)) in initial.into_iter().enumerate() {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, expected, "initial packet {index}");
        if let Some(payload) = payload {
            assert_eq!(packet.payload, payload, "initial reference payload {index}");
            continue;
        }
        if expected == 49 {
            assert!(packet.payload.len() > 4600);
        } else if expected == 28 {
            let name: Vec<_> = runtime
                .world
                .data(actor)
                .unwrap()
                .display_name
                .bytes()
                .map(|b| b.wrapping_add(128))
                .collect();
            assert!(packet
                .payload
                .windows(name.len())
                .any(|bytes| bytes == name));
            let name_start = packet
                .payload
                .windows(name.len())
                .position(|bytes| bytes == name)
                .unwrap();
            assert_eq!(
                packet.payload[name_start + name.len() + 4],
                128 + u8::from(with_login && !completed_tutorial),
                "hidden bit in first appearance"
            );
        } else if expected == 22 {
            let id = u16::from_be_bytes([packet.payload[4], packet.payload[5]]);
            inventory_ids.push(id);
            assert_eq!(&packet.payload[..4], &(-(1234 + id as i32)).to_be_bytes());
            assert_eq!(&packet.payload[6..], &[0, 0]);
        } else if expected == 46 {
            assert_eq!(
                &packet.payload[..3],
                if index == first_stat {
                    &[10, 10, 131]
                } else if index == first_stat + 1 {
                    &[1, 1, 135]
                } else {
                    &[1, 1, 141]
                }
            );
        } else if expected == 31 {
            assert_eq!(packet.payload, [0, 0]);
        } else if expected == 64 {
            assert_eq!(packet.payload, 10000u16.to_be_bytes());
        } else if expected == 47 {
            assert_eq!(packet.payload, [0, 0, 0]);
        } else if expected == 116 {
            assert_eq!(packet.payload, [48, 48]);
        } else if expected == 122 {
            assert_eq!(packet.payload, [0]);
        } else if expected == 85 {
            let mut bit = 0usize;
            let mut bits = |count: usize| {
                let mut value = 0u32;
                for _ in 0..count {
                    value = (value << 1) | ((packet.payload[bit / 8] >> (7 - bit % 8)) & 1) as u32;
                    bit += 1;
                }
                value
            };
            assert_eq!(bits(8), 0);
            assert_eq!(bits(16), npc.slot as u32);
            assert_eq!(bits(1), 0); // Spawn cycle zero has no optional field.
            assert_eq!(bits(1), 0); // No extension block.
            assert_eq!(bits(6), 1);
            assert_eq!(bits(3), 6);
            assert_eq!(bits(6), 0);
            assert_eq!(bits(1), 1);
            assert_eq!(bits(14), 1);
        }
    }
    assert_eq!(inventory_ids, [93, 94]);
    while outgoing.tick_ends == 0 {
        assert!(outgoing.next_packet().unwrap().is_none());
        if outgoing.tick_ends == 0 {
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        }
    }
    assert_eq!(outgoing.tick_ends, 1);
    if with_login {
        let data = runtime.world.data(actor).unwrap();
        assert_eq!(
            (100..105)
                .map(|id| data.variables.get(id))
                .collect::<Vec<_>>(),
            vec![100, 20, 45, 25, 3]
        );
        assert_eq!(data.variables.get(106), 549);
        if !completed_tutorial {
            // Finish inspecting the first-login UI before the independent overlay
            // and gameplay scenarios below. Modal input must not leak into them.
            for child in [23, 24, 25] {
                runtime
                    .close_interface(actor, gielinor_world::ui::Component::new(548, child))
                    .unwrap();
                let packet = loop {
                    if let Some(packet) = outgoing.next_packet().unwrap() {
                        break packet;
                    }
                    let mut byte = [0];
                    read(&mut runtime, &mut client, &mut byte);
                    outgoing.feed(&byte).unwrap();
                };
                assert_eq!(packet.opcode, 23);
            }
            let mut design = runtime.appearance_design(actor).unwrap().clone();
            design.hidden = false;
            runtime.set_appearance(actor, design).unwrap();
        }
        // Kotlin suppresses immediate writes until processedMapClock is positive.
        runtime.tick().unwrap();
        let mut root_contexts = 0;
        while root_contexts < 2 {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            if packet.opcode == 47 {
                root_contexts += 1;
            }
        }
        let tail = finish_tick(&mut runtime, &mut client, &mut outgoing);
        assert!(tail.iter().all(|packet| matches!(packet.opcode, 18 | 22)
            || gielinor_protocol::outgoing_category::category(packet.opcode)
                == Some(gielinor_protocol::outgoing_category::Category::Low)));
        // A dynamically granted permission dispatches the overlay listener over ISAAC.
        let mut cipher =
            Some(gielinor_protocol::isaac::Isaac::new(&[0x80000000, 239, 1, u32::MAX]).unwrap());
        let button = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|def| def.opcode == 47)
            .unwrap();
        let mut payload = ((149u32 << 16) | 1).to_be_bytes().to_vec();
        payload.extend(3u16.to_be_bytes());
        payload.extend(u16::MAX.to_be_bytes());
        payload.push(1);
        let bytes = gielinor_protocol::framing::encode(
            button,
            &payload,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            pump(&mut runtime);
            if runtime.world.data(actor).unwrap().variables.get(104) == 7 {
                break;
            }
            assert!(Instant::now() < deadline, "overlay filter did not execute");
            std::thread::sleep(Duration::from_millis(1));
        }
        // Reopening the interface replays its listener instead of retaining stale ranges.
        runtime
            .open_interface(
                actor,
                Some(gielinor_world::ui::Component::new(548, 20)),
                149,
                true,
            )
            .unwrap();
        for expected in [97, 7, 108] {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, expected);
        }
        let mut payload = ((151u32 << 16) | 1).to_be_bytes().to_vec();
        payload.extend(3u16.to_be_bytes());
        payload.extend(u16::MAX.to_be_bytes());
        payload.push(1);
        let bytes = gielinor_protocol::framing::encode(
            button,
            &payload,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            pump(&mut runtime);
            if runtime.world.data(actor).unwrap().variables.get(107) == 2 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "summary quest shortcut did not execute"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        for expected in [97, 23, 114, 7] {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, expected);
            if expected == 114 {
                assert_eq!(
                    packet.payload,
                    gielinor_protocol::client_script::run(
                        828,
                        &[gielinor_protocol::client_script::Argument::Int(0)]
                    )
                    .unwrap()
                    .payload
                );
            }
        }
        runtime.world.delay_player(actor, 3).unwrap();
        for child in [1, 3, 4] {
            let target = (155u32 << 16) | child;
            let mut payload = target.to_be_bytes().to_vec();
            payload.extend(u16::MAX.to_be_bytes());
            payload.extend(u16::MAX.to_be_bytes());
            payload.push(1);
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        button,
                        &payload,
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                if pump(&mut runtime).iter().any(
                    |e| matches!(e,Event::InterfaceButton {button,..} if button.component==target),
                ) {
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(108), 0);
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(110), 0);
        let mut applied = false;
        for _ in 0..8 {
            runtime.tick().unwrap();
            let mut contexts = 0;
            let mut opcodes = Vec::new();
            while contexts < 2 {
                let packet = loop {
                    if let Some(packet) = outgoing.next_packet().unwrap() {
                        break packet;
                    }
                    let mut byte = [0];
                    read(&mut runtime, &mut client, &mut byte);
                    outgoing.feed(&byte).unwrap();
                };
                if packet.opcode == 47 {
                    contexts += 1;
                }
                opcodes.push(packet.opcode);
            }
            opcodes.extend(
                finish_tick(&mut runtime, &mut client, &mut outgoing)
                    .into_iter()
                    .map(|packet| packet.opcode),
            );
            if runtime.world.data(actor).unwrap().variables.get(108) == 3 {
                assert_eq!(&opcodes[..4], &[97, 97, 97, 97]);
                applied = true;
                break;
            }
        }
        assert!(applied);
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(109), 3);
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(114), 3);
        assert_eq!(
            runtime.world.data(actor).unwrap().variables.get(110),
            0,
            "protected retaliate was discarded"
        );
        let mut payload = ((155u32 << 16) | 4).to_be_bytes().to_vec();
        payload.extend(u16::MAX.to_be_bytes());
        payload.extend(u16::MAX.to_be_bytes());
        payload.push(1);
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    button,
                    &payload,
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.data(actor).unwrap().variables.get(110) != 1 {
            pump(&mut runtime);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, 97);
        runtime
            .set_interface_events(
                actor,
                gielinor_world::ui::Component::new(149, 0),
                1,
                3,
                1u64 << 32,
            )
            .unwrap();
        let permission = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(permission.opcode, 108);
        // The encrypted login fixture requests resizing. WindowStatus reports a client-side
        // change before the server's queued toplevel move; this path needs only one tick.
        let window = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 10)
            .unwrap();
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    window,
                    &[1, 3, 32, 2, 88],
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.interface_state(actor).unwrap().frame_resizable {
            pump(&mut runtime);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        runtime.move_gameframe(actor, 548).unwrap();
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, 12);
        runtime.tick().unwrap();
        let mut contexts = 0;
        while contexts < 2 {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            if packet.opcode == 47 {
                contexts += 1;
            }
        }
        let tail = finish_tick(&mut runtime, &mut client, &mut outgoing);
        assert!(tail.iter().all(|packet| matches!(packet.opcode, 18 | 22)
            || gielinor_protocol::outgoing_category::category(packet.opcode)
                == Some(gielinor_protocol::outgoing_category::Category::Low)));
        assert_eq!(runtime.interface_state(actor).unwrap().top_level, Some(548));
        runtime.world.delay_player(actor, 5).unwrap();
        let mut payload = (149u32 << 16).to_be_bytes().to_vec();
        payload.extend(3u16.to_be_bytes());
        payload.extend(u16::MAX.to_be_bytes());
        payload.push(1);
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    button,
                    &payload,
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.data(actor).unwrap().variables.get(106) != 550 {
            pump(&mut runtime);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(runtime.interface_state(actor).unwrap().top_level, Some(548));
        for expected in [114, 12] {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, expected);
        }
        for cycle in 0..2 {
            runtime.tick().unwrap();
            let mut opcodes = Vec::new();
            let mut contexts = 0;
            while contexts < 2 {
                let packet = loop {
                    if let Some(packet) = outgoing.next_packet().unwrap() {
                        break packet;
                    }
                    let mut byte = [0];
                    read(&mut runtime, &mut client, &mut byte);
                    outgoing.feed(&byte).unwrap();
                };
                if packet.opcode == 47 {
                    contexts += 1;
                }
                opcodes.push(packet.opcode);
            }
            opcodes.extend(
                finish_tick(&mut runtime, &mut client, &mut outgoing)
                    .into_iter()
                    .map(|packet| packet.opcode),
            );
            if cycle == 0 {
                assert!(!opcodes.contains(&96));
                assert_eq!(runtime.interface_state(actor).unwrap().top_level, Some(548));
            } else {
                let mut expected = vec![
                    97, 97, 97, 96, 123, 114, 114, 123, 7, 96, 123, 114, 114, 123, 7, 7, 43, 47,
                    116, 122, 28, 85, 47,
                ];
                reference_order(&mut expected);
                assert_eq!(opcodes, expected);
                assert_eq!(runtime.interface_state(actor).unwrap().top_level, Some(550));
            }
        }
        assert!(runtime.world.protection(actor).unwrap().delayed());
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(107), 3);
        let window = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 10)
            .unwrap();
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    window,
                    &[1, 3, 32, 2, 88],
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.interface_state(actor).unwrap().frame_resizable {
            pump(&mut runtime);
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(runtime.interface_state(actor).unwrap().frame_width, 800);
        assert_eq!(runtime.interface_state(actor).unwrap().frame_height, 600);
        return;
    }
    assert!(runtime.world.npc(npc).unwrap().viewed);
    // A queued world hit reaches the real encrypted NPC synchronization stream once.
    {
        use gielinor_server::combat_display::{Bar, Config};
        use gielinor_world::{
            hit_display::Builder,
            hits::{Hit, Style, Target},
        };
        let type_id = runtime.world.npc(npc).unwrap().type_id;
        let health = runtime.world.npc(npc).unwrap().stats[3];
        let max_health = runtime.world.npc(npc).unwrap().base_stats[3];
        runtime.combat_display = Some(Config {
            regular: Builder {
                target: 16,
                source: 16,
                public: Some(17),
                zero_lit: Some(12),
                zero_tint: Some(13),
                max_lit: Some(43),
                target_threshold: i32::MAX,
                source_threshold: i32::MAX,
                delay: 0,
            },
            zero: (12, Some(13)),
            player_bar: Bar {
                id: 0,
                segments: 30,
            },
            npc_bars: BTreeMap::from([(
                type_id,
                Bar {
                    id: 0,
                    segments: 30,
                },
            )]),
        });
        let hit = Hit {
            source: Some(Target::Player(actor)),
            style: Style::Melee,
            damage: 1,
            display: None,
        };
        runtime.world.queue_hit(Target::Npc(npc), hit, 1).unwrap();
        let impact = hit.impact(health, true);
        let fill = (impact.health * 30 / max_health) as u8;
        let hits = gielinor_protocol::combat_updates::hitmarks(
            true,
            actor.slot,
            npc.slot,
            &[gielinor_protocol::combat_updates::Hitmark {
                source_index: 65536 + actor.slot as i32,
                source_type: 16,
                self_type: 16,
                other_type: 17,
                value: impact.displayed as u16,
                delay: 0,
                limit: 4,
            }],
        )
        .unwrap();
        let bars = gielinor_protocol::combat_updates::headbars(
            true,
            actor.slot,
            npc.slot,
            &[gielinor_protocol::combat_updates::Headbar {
                source_index: 65536 + actor.slot as i32,
                self_type: 0,
                other_type: 0,
                start_fill: fill,
                end_fill: fill,
                start_time: 0,
                end_time: 0,
            }],
        )
        .unwrap();
        let block = gielinor_protocol::combat_updates::NpcUpdates {
            hitmarks: Some(&hits),
            headbars: Some(&bars),
            sequence: None,
            face: None,
            spot_animations: None,
        }
        .encode();
        for cycle in 0..2 {
            let events = runtime.tick().unwrap();
            assert!(!events
                .iter()
                .any(|e| matches!(e, Event::SynchronizationFailed { .. })));
            let mut contexts = 0;
            let mut saw_npc = false;
            while contexts < 2 {
                let packet = loop {
                    if let Some(packet) = outgoing.next_packet().unwrap() {
                        break packet;
                    }
                    let mut byte = [0];
                    read(&mut runtime, &mut client, &mut byte);
                    outgoing.feed(&byte).unwrap();
                };
                if packet.opcode == 47 {
                    contexts += 1;
                }
                if packet.opcode == 85 {
                    saw_npc = true;
                    assert_eq!(packet.payload.ends_with(&block), cycle == 0);
                }
            }
            let tail = finish_tick(&mut runtime, &mut client, &mut outgoing);
            assert!(tail
                .iter()
                .all(
                    |packet| gielinor_protocol::outgoing_category::category(packet.opcode)
                        == Some(gielinor_protocol::outgoing_category::Category::Low)
                ));
            assert!(saw_npc);
        }
        assert_eq!(runtime.world.npc(npc).unwrap().stats[3], impact.health);
        runtime.combat_display = None;
    }
    runtime
        .world
        .data_mut(actor)
        .unwrap()
        .inventory
        .restore_slot(
            2,
            Some(gielinor_world::inventory::Item {
                id: 7,
                count: 300,
                vars: 0,
            }),
        )
        .unwrap();
    runtime.world.despawn_npc(npc).unwrap();
    runtime.tick().unwrap();
    for expected in reference_sorted([18, 47, 116, 122, 28, 85, 47]) {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, expected);
        if expected == 85 {
            assert_eq!(packet.payload, [0]);
        }
        if expected == 18 {
            assert_eq!(&packet.payload[6..], &[2, 0, 8, 255, 0, 0, 1, 44]);
        }
    }
    assert!(runtime
        .world
        .data(actor)
        .unwrap()
        .inventory
        .modified()
        .is_empty());
    let mut cipher =
        Some(gielinor_protocol::isaac::Isaac::new(&[0x80000000, 239, 1, u32::MAX]).unwrap());
    let parent = gielinor_world::ui::Component::new(548, 20);
    let inventory_component = gielinor_world::ui::Component::new(149, 0);
    runtime.open_interface(actor, None, 548, false).unwrap();
    runtime
        .open_interface(actor, Some(parent), 149, true)
        .unwrap();
    runtime
        .set_interface_events(
            actor,
            inventory_component,
            0,
            27,
            gielinor_world::ui::operation_mask(2).unwrap()
                | gielinor_world::ui::operation_mask(3).unwrap()
                | gielinor_world::ui::operation_mask(7).unwrap()
                | (1 << 20),
        )
        .unwrap();
    for expected in [96, 7, 108] {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, expected);
    }
    let button = *gielinor_protocol::tables::CLIENT
        .iter()
        .find(|def| def.opcode == 47)
        .unwrap();
    let mut payload = inventory_component.0.to_be_bytes().to_vec();
    payload.extend(2u16.to_be_bytes());
    payload.extend(7u16.to_be_bytes());
    payload.push(2);
    let bytes = gielinor_protocol::framing::encode(
        button,
        &payload,
        &mut cipher,
        gielinor_protocol::framing::OpcodeWidth::Byte,
    )
    .unwrap();
    client.write_all(&bytes).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if pump(&mut runtime).iter().any(|event| matches!(event, Event::InterfaceButton { button, route: gielinor_world::ui::ButtonRoute::Overlay, .. } if button.slot == 2 && button.object == 7 && button.op == 2)) { break; }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .get(2)
            .unwrap()
            .count,
        300
    );
    runtime
        .world
        .data_mut(actor)
        .unwrap()
        .stats
        .set_current(3, 2);
    for cycle in 0..2 {
        runtime.tick().unwrap();
        let expected: &[u16] = if cycle == 0 {
            &[74, 18, 46, 47, 116, 122, 28, 47]
        } else {
            &[47, 116, 122, 28, 47]
        };
        let mut expected = expected.to_vec();
        reference_order(&mut expected);
        for opcode in expected {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, opcode);
            if opcode == 74 {
                assert_eq!(packet.payload, b"\0\0You eat the shrimp.\0");
            }
            if opcode == 28 && cycle == 0 {
                assert!(packet.payload.ends_with(&[64, 3, 189, 0]));
            }
        }
    }
    assert_eq!(
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .get(2)
            .unwrap()
            .count,
        299
    );
    assert_eq!(runtime.world.data(actor).unwrap().stats.current(3), 5);
    assert!(!runtime.world.protection(actor).unwrap().delayed());
    runtime
        .world
        .data_mut(actor)
        .unwrap()
        .inventory
        .restore_slot(
            2,
            Some(gielinor_world::inventory::Item {
                id: 7,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    // Object IDs are untrusted; only the current authoritative slots are used.
    let mut drag = inventory_component.0.to_le_bytes().to_vec();
    drag.extend(666u16.to_le_bytes());
    drag.extend([130, 0]); // source slot 2, LE-add
    drag.extend(inventory_component.0.to_be_bytes());
    drag.extend([255, 127]); // null target object, BE-add
    drag.extend([0, 133]); // target slot 5, BE-add
    let drag_def = *gielinor_protocol::tables::CLIENT
        .iter()
        .find(|d| d.opcode == 48)
        .unwrap();
    let bytes = gielinor_protocol::framing::encode(
        drag_def,
        &drag,
        &mut cipher,
        gielinor_protocol::framing::OpcodeWidth::Byte,
    )
    .unwrap();
    client.write_all(&bytes).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if pump(&mut runtime)
            .iter()
            .any(|e| matches!(e, Event::InterfaceDrag { .. }))
        {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    runtime.tick().unwrap();
    for opcode in reference_sorted([18, 47, 116, 122, 28, 47]) {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, opcode);
    }
    assert!(runtime
        .world
        .data(actor)
        .unwrap()
        .inventory
        .get(2)
        .is_none());
    assert_eq!(
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .get(5)
            .unwrap()
            .id,
        7
    );
    runtime
        .world
        .data_mut(actor)
        .unwrap()
        .inventory
        .restore_slot(
            6,
            Some(gielinor_world::inventory::Item {
                id: 8,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    // Exercise the new wear-position packet adapter over encrypted TCP. This
    // fixture wears a hat: sound and player-op5 follow high-priority inventory
    // synchronization; hand-slot variables are covered by the Kotlin oracle.
    let wear_binding = gielinor_world::bank_actions::Binding {
        base: runtime.definitions.varps[&1000],
        bit: gielinor_world::variables::Varbit {
            base: 1000,
            lsb: 0,
            msb: 7,
        },
    };
    runtime.world.wearpos = Some(gielinor_world::wearpos::Config {
        special: wear_binding.base,
        stance: wear_binding.base,
        category: wear_binding,
        level: wear_binding,
        decimal: wear_binding,
        combat_stats: [0, 1, 2, 3, 4, 6, 5],
        items: BTreeMap::from([(
            8,
            gielinor_world::wearpos::Item {
                sound: Some(2238),
                ..Default::default()
            },
        )]),
    });
    for (component, slot, op, equipping) in [
        (149u32 << 16, 6u16, 3u8, true),
        ((149u32 << 16) | 1, 65535u16, 1u8, false),
    ] {
        let mut request = component.to_be_bytes().to_vec();
        request.extend(slot.to_be_bytes());
        request.extend(12345u16.to_be_bytes());
        request.push(op);
        let bytes = gielinor_protocol::framing::encode(
            button,
            &request,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if pump(&mut runtime)
                .iter()
                .any(|e| matches!(e, Event::InterfaceButton { .. }))
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let events = runtime.tick().unwrap();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::World(gielinor_world::simulation::WorldEvent::EquipmentResult {
                result: Ok(_),
                ..
            })
        )));
        for opcode in reference_sorted([77, 6, 18, 18, 47, 116, 122, 28, 47]) {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, opcode);
            if opcode == 77 {
                assert_eq!(packet, gielinor_protocol::game::synth_sound(2238, 1, 0));
            } else if opcode == 6 {
                assert_eq!(
                    packet,
                    gielinor_protocol::login_init::player_operation(5, false, None).unwrap()
                );
            }
        }
        assert_eq!(
            runtime
                .world
                .data(actor)
                .unwrap()
                .equipment
                .get(0)
                .is_some(),
            equipping
        );
    }
    assert_eq!(
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .get(0)
            .unwrap()
            .id,
        8
    );
    runtime.world.wearpos = None;
    runtime.set_varp(actor, 1000, 127).unwrap();
    runtime.set_varp(actor, 1000, 127).unwrap();
    runtime.set_varp(actor, 1000, 128).unwrap();
    runtime.set_varbit(actor, 2000, 15).unwrap();
    assert!(runtime.set_varbit(actor, 2000, 16).is_err());
    assert_eq!(runtime.world.data(actor).unwrap().variables.get(1000), 240);
    runtime.resynchronize_variables(actor, true).unwrap();
    let args = [
        gielinor_protocol::client_script::Argument::Int(93),
        gielinor_protocol::client_script::Argument::String("Inventory".into()),
    ];
    runtime.run_client_script(actor, 239, &args).unwrap();
    let expected = [
        gielinor_protocol::varp::update(1000, 127),
        gielinor_protocol::varp::update(1000, 127),
        gielinor_protocol::varp::update(1000, 128),
        gielinor_protocol::varp::update(1000, 240),
        gielinor_protocol::varp::reset(),
        gielinor_protocol::varp::update(1000, 240),
        gielinor_protocol::client_script::run(239, &args).unwrap(),
    ];
    for expected in expected {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet, expected);
    }
    let bank_parent = gielinor_world::ui::Component::new(548, 21);
    let side_parent = gielinor_world::ui::Component::new(548, 22);
    runtime.open_bank(actor, bank_parent, side_parent).unwrap();
    let expected_bonus = gielinor_server::bonuses::BankDisplay {
        components: bonus_components,
        tooltip: bonus_tooltip,
    }
    .packets(&gielinor_world::bonuses::Bonuses::default(), 4, 4)
    .unwrap();
    assert_eq!(
        runtime.equipment_bonuses(actor).unwrap(),
        gielinor_world::bonuses::Bonuses::default()
    );
    let mut bank_open_opcodes = if with_login {
        vec![97, 7, 7]
    } else {
        vec![97, 97, 7, 7]
    };
    let bonus_start = bank_open_opcodes.len();
    bank_open_opcodes.extend(expected_bonus.iter().map(|packet| packet.opcode));
    bank_open_opcodes.extend([108; 6]);
    for (index, opcode) in bank_open_opcodes.into_iter().enumerate() {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, opcode);
        if (bonus_start..bonus_start + expected_bonus.len()).contains(&index) {
            assert_eq!(packet, expected_bonus[index - bonus_start]);
        }
    }
    for deposit in [true, false] {
        let component = if deposit { 15u32 << 16 } else { 12u32 << 16 };
        let mut request = component.to_be_bytes().to_vec();
        request.extend(0u16.to_be_bytes());
        request.extend(60000u16.to_be_bytes());
        request.push(if deposit { 3 } else { 2 });
        let bytes = gielinor_protocol::framing::encode(
            button,
            &request,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if pump(&mut runtime)
                .iter()
                .any(|e| matches!(e, Event::InterfaceButton { .. }))
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let events = runtime.tick().unwrap();
        assert!(events.iter().any(|e| matches!(
            e,
            Event::World(gielinor_world::simulation::WorldEvent::Bank { result: Ok(_), .. })
        )));
        let expected: &[u16] = if deposit {
            &[97, 18, 22, 47, 116, 122, 28, 47]
        } else {
            &[
                97, 18, 18, 47, 116, 122, 28, 47, 108, 108, 108, 108, 108, 108,
            ]
        };
        let mut expected = expected.to_vec();
        reference_order(&mut expected);
        for opcode in expected {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, opcode);
        }
        assert_eq!(
            runtime.world.data(actor).unwrap().bank.get(0).is_some(),
            deposit
        );
    }
    assert_eq!(
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .get(0)
            .unwrap()
            .id,
        8
    );
    // Authenticated bank control clicks update the exact persistent setting varps.
    for (name, varp, value) in [
        ("note", 1112, 1),
        ("item", 1112, 0),
        ("placeholder", 1113, 1),
        ("placeholder", 1113, 0),
        ("quantity5", 1114, 1),
        ("quantity1", 1114, 0),
        ("insert", 1116, 1),
    ] {
        let component = bank_control_symbols[&format!("bankmain:{name}")] as u32;
        let mut request = component.to_be_bytes().to_vec();
        request.extend(u16::MAX.to_be_bytes());
        request.extend(u16::MAX.to_be_bytes());
        request.push(1);
        let bytes = gielinor_protocol::framing::encode(
            button,
            &request,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump(&mut runtime)
            .iter()
            .any(|e| matches!(e, Event::InterfaceButton { .. }))
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        runtime.tick().unwrap();
        for opcode in reference_sorted([97, 47, 116, 122, 28, 47]) {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, opcode);
            if opcode == 97 {
                assert_eq!(packet, gielinor_protocol::varp::update(varp, value));
            }
        }
    }
    // Deposit/withdraw X suspend until a signed count response arrives.
    for deposit in [true, false] {
        let component = if deposit { 15u32 << 16 } else { 12u32 << 16 };
        let mut request = component.to_be_bytes().to_vec();
        request.extend(0u16.to_be_bytes());
        request.extend(60000u16.to_be_bytes());
        request.push(if deposit { 7 } else { 6 });
        let bytes = gielinor_protocol::framing::encode(
            button,
            &request,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump(&mut runtime)
            .iter()
            .any(|e| matches!(e, Event::InterfaceButton { .. }))
        {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        runtime.tick().unwrap();
        assert!(runtime.world.protection(actor).unwrap().suspended);
        for opcode in reference_sorted([114, 47, 116, 122, 28, 47]) {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, opcode);
            if opcode == 114 {
                assert_eq!(
                    packet,
                    gielinor_protocol::client_script::run(
                        108,
                        &[gielinor_protocol::client_script::Argument::String(
                            "Enter amount:".into()
                        )]
                    )
                    .unwrap()
                );
            }
        }
        let count_def = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 75)
            .unwrap();
        let bytes = gielinor_protocol::framing::encode(
            count_def,
            &(-1i32).to_be_bytes(),
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump(&mut runtime).iter().any(|e| {
            matches!(
                e,
                Event::CountDialogInput {
                    count: -1,
                    accepted: true,
                    ..
                }
            )
        }) {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        runtime.tick().unwrap();
        assert!(!runtime.world.protection(actor).unwrap().suspended);
        for opcode in reference_sorted([97, 97, 18, 18, 47, 116, 122, 28, 47]) {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet.opcode, opcode);
        }
        assert_eq!(
            runtime.world.data(actor).unwrap().bank.get(0).is_some(),
            deposit
        );
    }
    // Closing the captured bank modal aborts a pending prompt.
    let mut request = (15u32 << 16).to_be_bytes().to_vec();
    request.extend(0u16.to_be_bytes());
    request.extend(60000u16.to_be_bytes());
    request.push(7);
    let bytes = gielinor_protocol::framing::encode(
        button,
        &request,
        &mut cipher,
        gielinor_protocol::framing::OpcodeWidth::Byte,
    )
    .unwrap();
    client.write_all(&bytes).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !pump(&mut runtime)
        .iter()
        .any(|e| matches!(e, Event::InterfaceButton { .. }))
    {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    runtime.tick().unwrap();
    for opcode in reference_sorted([114, 47, 116, 122, 28, 47]) {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, opcode);
    }
    runtime.close_interface(actor, bank_parent).unwrap();
    runtime.close_interface(actor, side_parent).unwrap();
    assert_eq!(runtime.world.count_dialog_token(actor).unwrap(), None);
    assert!(!runtime.world.protection(actor).unwrap().suspended);
    for opcode in [23, 117, 45, 114, 23] {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, opcode);
    }
    // Drink through the encrypted held-op path. Output ordering is immediate
    // energy, sip/remaining chat, then the normal inventory/animation cycle.
    runtime.world.foods.remove(&7);
    runtime.world.potions.insert(
        7,
        gielinor_world::potions::Potion {
            id: 7,
            next: 8,
            vial: 8,
            sip: "You drink some of your super energy.".into(),
            remaining: "You have 3 doses of potion left.".into(),
            effects: vec![gielinor_world::potions::ResolvedEffect::Energy(20)],
        },
    );
    let data = runtime.world.data_mut(actor).unwrap();
    data.energy = 9000;
    data.inventory
        .restore_slot(
            0,
            Some(gielinor_world::inventory::Item {
                id: 7,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    let mut sip = inventory_component.0.to_be_bytes().to_vec();
    sip.extend(0u16.to_be_bytes());
    sip.extend(7u16.to_be_bytes());
    sip.push(2);
    client
        .write_all(
            &gielinor_protocol::framing::encode(
                button,
                &sip,
                &mut cipher,
                gielinor_protocol::framing::OpcodeWidth::Byte,
            )
            .unwrap(),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !pump(&mut runtime)
        .iter()
        .any(|e| matches!(e,Event::InterfaceButton{button,..} if button.slot==0&&button.object==7))
    {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    for cycle in 0..4 {
        let events = runtime.tick().unwrap();
        assert!(!events
            .iter()
            .any(|e| matches!(e, Event::SynchronizationFailed { .. })));
        let mut packets = Vec::new();
        let mut contexts = 0;
        while contexts < 2 {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            if packet.opcode == 47 {
                contexts += 1;
            }
            packets.push(packet);
        }
        let tail = finish_tick(&mut runtime, &mut client, &mut outgoing);
        assert!(tail.iter().all(|packet| matches!(packet.opcode, 18 | 22)
            || gielinor_protocol::outgoing_category::category(packet.opcode)
                == Some(gielinor_protocol::outgoing_category::Category::Low)));
        packets.extend(tail);
        if cycle == 0 {
            assert_eq!(
                packets[..2].iter().map(|p| p.opcode).collect::<Vec<_>>(),
                [74, 74]
            );
            let energies: Vec<_> = packets.iter().filter(|p| p.opcode == 64).collect();
            assert_eq!(energies.len(), 1);
            assert_eq!(
                energies[0].payload,
                gielinor_protocol::game::run_energy(10000).unwrap()
            );
            assert_eq!(
                packets[0],
                gielinor_protocol::chat::game(0, None, "You drink some of your super energy.")
                    .unwrap()
            );
            assert_eq!(
                packets[1],
                gielinor_protocol::chat::game(0, None, "You have 3 doses of potion left.").unwrap()
            );
            assert!(packets.iter().any(|p| p.opcode == 18));
            assert_eq!(
                runtime
                    .world
                    .data(actor)
                    .unwrap()
                    .inventory
                    .get(0)
                    .unwrap()
                    .id,
                8
            );
        }
    }
    assert!(!runtime.world.protection(actor).unwrap().delayed());
    // Real encrypted output: private updates precede shared updates, hidden objects
    // stay absent, and revealing somebody else's object makes it visible next cycle.
    {
        use gielinor_protocol::zone::{self, Add, Object as WireObject};
        use gielinor_world::ground::{Object, Scope};
        let coords = runtime.world.data(actor).unwrap().coords;
        let clock = runtime.world.clock;
        let mut catalog = gielinor_world::inventory::Catalog::default();
        catalog.stackable.insert(8);
        let object = |x, scope, receiver| Object {
            coords: CoordGrid::new(x, coords.z(), coords.level()).unwrap(),
            item: 8,
            count: 1,
            scope,
            created: clock.0,
            receiver,
            owner: None,
        };
        let public = runtime
            .world
            .objects
            .add(
                object(3200, Scope::Temporary, None),
                &catalog,
                clock,
                i32::MAX,
                100,
            )
            .unwrap()
            .remove(0);
        runtime
            .world
            .objects
            .add(
                object(3201, Scope::Private, Some(observer_id)),
                &catalog,
                clock,
                i32::MAX,
                100,
            )
            .unwrap();
        runtime
            .world
            .objects
            .add(
                object(3202, Scope::Private, Some(observer_id + 100)),
                &catalog,
                clock,
                200,
                2,
            )
            .unwrap();
        for cycle in 0..2 {
            if cycle == 1 {
                assert!(runtime
                    .world
                    .objects
                    .delete(&public, runtime.world.clock, i32::MAX));
            }
            let events = runtime.tick().unwrap();
            assert!(!events
                .iter()
                .any(|event| matches!(event, Event::SynchronizationFailed { .. })));
            let mut roots = 0;
            while roots < 2 {
                let packet = loop {
                    if let Some(packet) = outgoing.next_packet().unwrap() {
                        break packet;
                    }
                    let mut byte = [0];
                    read(&mut runtime, &mut client, &mut byte);
                    outgoing.feed(&byte).unwrap();
                };
                if packet.opcode == 47 {
                    roots += 1;
                }
            }
            let expected = if cycle == 0 {
                vec![
                    zone::enclosed(48, 48, 0, &[WireObject::Add(Add::visible(8, 1, 1, 0))])
                        .unwrap(),
                    zone::enclosed(48, 48, 0, &[WireObject::Add(Add::visible(8, 1, 0, 0))])
                        .unwrap(),
                ]
            } else {
                vec![
                    zone::enclosed(48, 48, 0, &[WireObject::Add(Add::visible(8, 1, 2, 0))])
                        .unwrap(),
                    zone::enclosed(
                        48,
                        48,
                        0,
                        &[WireObject::Delete {
                            id: 8,
                            count: 1,
                            x: 0,
                            z: 0,
                        }],
                    )
                    .unwrap(),
                ]
            };
            for expected in expected {
                let packet = loop {
                    if let Some(packet) = outgoing.next_packet().unwrap() {
                        break packet;
                    }
                    let mut byte = [0];
                    read(&mut runtime, &mut client, &mut byte);
                    outgoing.feed(&byte).unwrap();
                };
                assert_eq!(packet, expected);
            }
        }
    }
    runtime.close_interface(actor, parent).unwrap();
    let packet = loop {
        if let Some(packet) = outgoing.next_packet().unwrap() {
            break packet;
        }
        let mut byte = [0];
        read(&mut runtime, &mut client, &mut byte);
        outgoing.feed(&byte).unwrap();
    };
    assert_eq!(packet.opcode, 23);
    assert_eq!(packet.payload, parent.0.to_be_bytes());
    // The same valid wire click is ignored after closing its interface.
    let bytes = gielinor_protocol::framing::encode(
        button,
        &payload,
        &mut cipher,
        gielinor_protocol::framing::OpcodeWidth::Byte,
    )
    .unwrap();
    client.write_all(&bytes).unwrap();
    // Exercise encrypted gameplay through the actual socket/session/action boundary.
    runtime.world.collision.allocate(3200, 3200, 0);
    // A real movement packet must close a modal and cancel weak production,
    // rather than merely replacing the destination while the job continues.
    runtime
        .open_interface(actor, Some(parent), 149, false)
        .unwrap();
    runtime.world.smithing = Some(gielinor_world::smithing_make::Config {
        queue: 900,
        skill: 13,
        max_level: 99,
        xp_rate: 1.0,
        tutorial: None,
    });
    let smithing_before = runtime.world.data(actor).unwrap().stats.fine_xp(13);
    runtime
        .world
        .start_smithing(
            actor,
            std::sync::Arc::new(gielinor_world::smithing_make::Request {
                product: 8,
                produced: 1,
                consumed: vec![],
                xp: 6.2,
                animation: 899,
                cycles: 5,
            }),
            28,
        )
        .unwrap();
    let movement = *gielinor_protocol::tables::CLIENT
        .iter()
        .find(|d| d.opcode == 114)
        .unwrap();
    let bytes = gielinor_protocol::framing::encode(
        movement,
        &[0x83, 0x0c, 0x80, 0x0c, 127],
        &mut cipher,
        gielinor_protocol::framing::OpcodeWidth::Byte,
    )
    .unwrap();
    client.write_all(&bytes).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.data(actor).unwrap().coords.x() != 3203 {
        let events = pump(&mut runtime);
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::PacketRejected { .. }
                | Event::Packet { .. }
                | Event::SynchronizationFailed { .. }
                | Event::InterfaceButton { .. }
        )));
        runtime.tick().unwrap();
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime
        .interface_state(actor)
        .unwrap()
        .modals
        .get(parent)
        .is_none());
    for _ in 0..6 {
        runtime.tick().unwrap();
    }
    assert_eq!(
        runtime.world.data(actor).unwrap().stats.fine_xp(13),
        smithing_before
    );
    // SkillMulti uses pause-button subcomponents as quantities, not item slots.
    // Reject quantity29 at the permission boundary, then accept the second row7.
    {
        use gielinor_server::skill_multi::{Dialog, OptionItem, Presentation, Selection};
        use gielinor_world::ui::Component;
        let presentation = Presentation {
            interface: 270,
            parent,
            setup_script: 2046,
        };
        let dialog = Dialog {
            title: "Smelt".into(),
            options: vec![
                OptionItem {
                    object: 2349,
                    label: "Bronze".into(),
                },
                OptionItem {
                    object: 2351,
                    label: "Iron".into(),
                },
            ],
            verb: 13,
            max_count: 28,
            selected_count: 1,
        };
        runtime
            .open_skill_multi(actor, presentation, dialog.clone())
            .unwrap();
        let token = runtime.world.quantity_dialog_token(actor).unwrap().unwrap();
        let pause = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 115)
            .unwrap();
        let component = Component::new(270, 16).0.to_be_bytes();
        for count in [29u16, 7] {
            let payload = [
                component[1],
                component[0],
                component[3],
                component[2],
                (count >> 8) as u8,
                count as u8,
            ];
            let bytes = gielinor_protocol::framing::encode(
                pause,
                &payload,
                &mut cipher,
                gielinor_protocol::framing::OpcodeWidth::Byte,
            )
            .unwrap();
            client.write_all(&bytes).unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let events = pump(&mut runtime);
            let selected: Vec<_> = events
                .iter()
                .filter_map(|e| match e {
                    Event::SkillMultiSelected { selection, .. } => Some(*selection),
                    _ => None,
                })
                .collect();
            if !selected.is_empty() {
                assert_eq!(selected, vec![Some(Selection { index: 1, count: 7 })]);
                break;
            }
            assert!(Instant::now() < deadline, "SkillMulti resume timed out");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(runtime.world.quantity_dialog_token(actor).unwrap(), None);
        assert!(!runtime.world.resume_quantity_dialog(actor, token).unwrap());
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .get(parent)
            .is_none());
        runtime
            .open_skill_multi(actor, presentation, dialog)
            .unwrap();
        runtime.world.delay_player(actor, 2).unwrap();
        let payload = [component[1], component[0], component[3], component[2], 0, 1];
        let bytes = gielinor_protocol::framing::encode(
            pause,
            &payload,
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime
            .world
            .quantity_dialog_token(actor)
            .unwrap()
            .is_some()
        {
            let events = pump(&mut runtime);
            assert!(!events
                .iter()
                .any(|e| matches!(e, Event::SkillMultiSelected { .. })));
            assert!(
                Instant::now() < deadline,
                "Delayed SkillMulti did not cancel"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .close_queue
            .contains(&parent));
        for _ in 0..3 {
            runtime.tick().unwrap();
        }
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .get(parent)
            .is_none());
    }
    // Encrypted OpLocV2 must reach the authoritative door action and observer zone stream.
    // First exercise furnace reach -> skillmulti -> weak production over TCP.
    {
        use gielinor_world::{
            inventory::{Catalog, Delete, Insert},
            loc_collision::LocCollision,
            locations::Location,
        };
        let coords = runtime
            .world
            .data(actor)
            .unwrap()
            .coords
            .translate(1, 0, 0)
            .unwrap();
        let furnace = Location::new(coords, 60002, 10, 0).unwrap();
        runtime.world.location_types.insert(
            60002,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: false,
                break_route_finding: false,
            },
        );
        runtime.world.change_location(furnace, true).unwrap();
        let before = runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .slots()
            .to_vec();
        let xp = runtime.world.data(actor).unwrap().stats.fine_xp(13);
        for ore in [436, 438] {
            runtime
                .world
                .data_mut(actor)
                .unwrap()
                .inventory
                .insert(&Catalog::default(), Insert::new(ore, 1))
                .unwrap();
        }
        let op = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 73)
            .unwrap();
        let payload = [
            (coords.z() >> 8) as u8,
            (coords.z() as u8).wrapping_add(128),
            coords.x() as u8,
            (coords.x() >> 8) as u8,
            128,
            128,
            (60002u16 >> 8) as u8,
            60002u16 as u8,
        ];
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    op,
                    &payload,
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime
            .world
            .quantity_dialog_token(actor)
            .unwrap()
            .is_none()
        {
            pump(&mut runtime);
            runtime.tick().unwrap();
            assert!(Instant::now() < deadline, "Furnace did not open SkillMulti");
            std::thread::sleep(Duration::from_millis(1));
        }
        let pause = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 115)
            .unwrap();
        let component = gielinor_world::ui::Component::new(270, 15).0.to_be_bytes();
        let payload = [
            component[1],
            component[0],
            component[3],
            component[2],
            0,
            28,
        ];
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    pause,
                    &payload,
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.data(actor).unwrap().stats.fine_xp(13) == xp {
            pump(&mut runtime);
            runtime.tick().unwrap();
            assert!(Instant::now() < deadline, "Furnace did not produce bronze");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            runtime.world.data(actor).unwrap().stats.fine_xp(13),
            xp + 62
        );
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(7000), 28);
        assert!(!runtime.world.protection(actor).unwrap().delayed());
        runtime
            .world
            .data_mut(actor)
            .unwrap()
            .inventory
            .insert(&Catalog::default(), Insert::new(2347, 1))
            .unwrap();
        runtime.open_anvil(actor, None).unwrap();
        let background = gielinor_protocol::client_script::run(
            2524,
            &[
                gielinor_protocol::client_script::Argument::Int(-1),
                gielinor_protocol::client_script::Argument::Int(-1),
            ],
        )
        .unwrap();
        let opened = gielinor_protocol::interfaces::open_sub(
            runtime.interface_state(actor).unwrap().translate(parent).0,
            312,
            false,
        );
        let mut saw_background = false;
        let mut map_flags = 0;
        loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                if packet.opcode == 143 {
                    assert_eq!(packet, gielinor_protocol::game::clear_map_flag());
                    map_flags += 1;
                }
                if packet == background {
                    saw_background = true;
                }
                if packet == opened {
                    assert!(
                        saw_background,
                        "Anvil must prepare the main modal before opening it"
                    );
                    break;
                }
            } else {
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            }
        }
        assert_eq!(
            runtime.interface_state(actor).unwrap().modals.get(parent),
            Some(312)
        );
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(7002), 1);
        let count_def = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 75)
            .unwrap();
        for (input, expected) in [(-5i32, 5), (i32::MIN, 1)] {
            let mut payload = ((312u32 << 16) | 6).to_be_bytes().to_vec();
            payload.extend(u16::MAX.to_be_bytes());
            payload.extend(u16::MAX.to_be_bytes());
            payload.push(1);
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        button,
                        &payload,
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime
                .world
                .quantity_dialog_token(actor)
                .unwrap()
                .is_none()
            {
                pump(&mut runtime);
                assert!(Instant::now() < deadline, "Make X did not suspend");
                std::thread::sleep(Duration::from_millis(1));
            }
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        count_def,
                        &input.to_be_bytes(),
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime
                .world
                .quantity_dialog_token(actor)
                .unwrap()
                .is_some()
            {
                pump(&mut runtime);
                assert!(Instant::now() < deadline, "Make X did not resume");
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(
                runtime.world.data(actor).unwrap().variables.get(7000),
                expected
            );
            assert_eq!(
                runtime.interface_state(actor).unwrap().modals.get(parent),
                Some(312)
            );
        }
        // A second valid modal button cancels Make X before its protection
        // check. A late numeric reply must not change the new literal quantity.
        for child in [6u32, 4] {
            let mut payload = ((312u32 << 16) | child).to_be_bytes().to_vec();
            payload.extend(u16::MAX.to_be_bytes());
            payload.extend(u16::MAX.to_be_bytes());
            payload.push(1);
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        button,
                        &payload,
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.data(actor).unwrap().variables.get(7000) != 5 {
            pump(&mut runtime);
            assert!(
                Instant::now() < deadline,
                "Literal button did not cancel Make X"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(runtime.world.quantity_dialog_token(actor).unwrap(), None);
        assert!(!runtime.interface_state(actor).unwrap().input_dialog_open);
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    count_def,
                    &99i32.to_be_bytes(),
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !pump(&mut runtime).iter().any(|event| {
            matches!(
                event,
                Event::CountDialogInput {
                    count: 99,
                    accepted: false,
                    ..
                }
            )
        }) {
            assert!(Instant::now() < deadline, "Stale count input not rejected");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(7000), 5);
        for child in [7u32, 9] {
            let mut payload = ((312u32 << 16) | child).to_be_bytes().to_vec();
            payload.extend(u16::MAX.to_be_bytes());
            payload.extend(u16::MAX.to_be_bytes());
            payload.push(1);
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        button,
                        &payload,
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.data(actor).unwrap().stats.fine_xp(13) == xp + 62 {
            pump(&mut runtime);
            runtime.tick().unwrap();
            assert!(Instant::now() < deadline, "Anvil did not produce dagger");
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            runtime.world.data(actor).unwrap().stats.fine_xp(13),
            xp + 62 + 125
        );
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(7000), 28);
        assert_eq!(
            runtime.interface_state(actor).unwrap().modals.get(parent),
            None
        );
        for item in [2347, 1205] {
            runtime
                .world
                .data_mut(actor)
                .unwrap()
                .inventory
                .delete(&Catalog::default(), Delete::new(item, 1))
                .unwrap();
        }
        assert_eq!(runtime.world.data(actor).unwrap().inventory.slots(), before);
        runtime.world.change_location(furnace, false).unwrap();
        // Publish furnace teardown in its own tick. Otherwise its LocDelete
        // coalesces with the next door update in the same zone, invalidating
        // that scenario's exact packet assertion. Drain through the delimiter
        // so the door's map-flag assertion measures only its own action.
        runtime.tick().unwrap();
        let completed_ticks = runtime.world.clock.0 as usize;
        while outgoing.tick_ends < completed_ticks {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                if packet.opcode == 143 {
                    assert_eq!(packet, gielinor_protocol::game::clear_map_flag());
                    map_flags += 1;
                }
            } else if outgoing.tick_ends < completed_ticks {
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            }
        }
        assert_eq!(map_flags, 1, "furnace reach clears its map flag");
    }
    {
        use gielinor_world::{
            doors::{Door, Kind},
            loc_collision::LocCollision,
            locations::Location,
            simulation::WorldEvent,
        };
        let loc = Location::new(
            gielinor_world::CoordGrid::new(3204, 3200, 0).unwrap(),
            1,
            0,
            0,
        )
        .unwrap();
        runtime.world.locations.load(loc);
        runtime.world.doors.insert(
            1,
            Door {
                kind: Kind::ClosedSingle,
                next: 2,
                sound: 62,
            },
        );
        for id in [1, 2] {
            runtime.world.location_types.insert(
                id,
                LocCollision {
                    width: 1,
                    length: 1,
                    block_walk: 0,
                    block_range: false,
                    break_route_finding: false,
                },
            );
        }
        let definition = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 73)
            .unwrap();
        let bytes = gielinor_protocol::framing::encode(
            definition,
            &[12, 0, 132, 12, 128, 128, 0, 1],
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut events = pump(&mut runtime);
            events.extend(runtime.tick().unwrap());
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::PacketRejected { .. }
                    | Event::Packet { .. }
                    | Event::SynchronizationFailed { .. }
            )));
            if events.iter().any(|e| matches!(e, Event::World(WorldEvent::DoorSound { actor: who, sound: 62 }) if *who == actor)) { break; }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let expected = gielinor_protocol::zone::enclosed(
            48,
            48,
            0,
            &[
                gielinor_protocol::zone::Object::LocDelete {
                    x: 4,
                    z: 0,
                    shape: 0,
                    angle: 0,
                },
                gielinor_protocol::zone::Object::LocAdd {
                    id: 2,
                    x: 3,
                    z: 0,
                    shape: 0,
                    angle: 1,
                    ops: 255,
                },
            ],
        )
        .unwrap();
        let mut sound = false;
        let mut resets = 0;
        loop {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            if packet.opcode == 77 {
                assert_eq!(packet, gielinor_protocol::game::synth_sound(62, 1, 0));
                sound = true;
            }
            if packet.opcode == 143 {
                assert_eq!(packet, gielinor_protocol::game::clear_map_flag());
                resets += 1;
            }
            if packet == expected {
                break;
            }
        }
        for packet in finish_tick(&mut runtime, &mut client, &mut outgoing) {
            if packet.opcode == 77 {
                assert_eq!(packet, gielinor_protocol::game::synth_sound(62, 1, 0));
                assert!(!sound, "Duplicate door sound");
                sound = true;
            }
            if packet.opcode == 143 {
                assert_eq!(packet, gielinor_protocol::game::clear_map_flag());
                resets += 1;
            }
        }
        assert!(sound);
        assert_eq!(resets, 2);
        assert!(runtime.world.locations.find_type(loc.coords, 1).is_none());
    }
    {
        use gielinor_world::{
            ground::{Object, Scope},
            ground_transfer::{Effect, PickupEffects},
            simulation::WorldEvent,
        };
        runtime.world.pickup_effects = Some(PickupEffects {
            sound: 2582,
            animation: 832,
        });
        let coords = runtime.world.data(actor).unwrap().coords;
        let count = |runtime: &Runtime<Backend>| {
            runtime
                .world
                .data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .filter(|item| item.id == 8)
                .map(|item| item.count)
                .sum::<i32>()
        };
        let before = count(&runtime);
        runtime
            .world
            .objects
            .add(
                Object {
                    coords,
                    item: 8,
                    count: 1,
                    scope: Scope::Temporary,
                    created: runtime.world.clock.0,
                    receiver: None,
                    owner: None,
                },
                &runtime.world.object_types,
                runtime.world.clock,
                i32::MAX,
                100,
            )
            .unwrap();
        let definition = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 7)
            .unwrap();
        let bytes = gielinor_protocol::framing::encode(
            definition,
            &[128, 131, 12, 136, 0, 128, 12, 0],
            &mut cipher,
            gielinor_protocol::framing::OpcodeWidth::Byte,
        )
        .unwrap();
        client.write_all(&bytes).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut events = pump(&mut runtime);
            events.extend(runtime.tick().unwrap());
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::PacketRejected { .. }
                    | Event::Packet { .. }
                    | Event::SynchronizationFailed { .. }
            )));
            if events.iter().any(|e|matches!(e,Event::World(WorldEvent::GroundAction { actor: who, effect: Effect::Sound(2582) })if *who==actor)){break;}
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(count(&runtime), before + 1);
        assert!(!runtime
            .world
            .objects
            .at(coords)
            .any(|h| h.snapshot().item == 8));
        let mut sound = false;
        // Add and delete can be coalesced in one tick, or sent on consecutive ticks
        // when the socket input reaches the world after the first update.
        loop {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            if packet.opcode == 77 {
                assert_eq!(packet, gielinor_protocol::game::synth_sound(2582, 1, 0));
                sound = true;
            }
            let deletion = gielinor_protocol::zone::Object::Delete {
                id: 8,
                count: 1,
                x: 3,
                z: 0,
            }
            .encode()
            .unwrap();
            if packet.opcode == 105 && packet.payload.ends_with(&deletion) {
                break;
            }
        }
        for packet in finish_tick(&mut runtime, &mut client, &mut outgoing) {
            if packet.opcode == 77 {
                assert_eq!(packet, gielinor_protocol::game::synth_sound(2582, 1, 0));
                assert!(!sound, "Duplicate pickup sound");
                sound = true;
            }
        }
        assert!(sound);
    }
    // Exercise all generic held-op 5 confirmation paths over encrypted TCP.
    {
        use gielinor_world::{
            drop_actions::{Event as DropEvent, Kind},
            simulation::WorldEvent,
        };
        runtime
            .open_interface(actor, Some(parent), 149, true)
            .unwrap();
        runtime
            .set_interface_events(
                actor,
                inventory_component,
                0,
                27,
                gielinor_world::ui::operation_mask(7).unwrap(),
            )
            .unwrap();
        let pause = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 115)
            .unwrap();
        runtime.world.object_types.stackable.insert(8);
        for kind in [Kind::Drop, Kind::Destroy, Kind::Release] {
            let config = runtime.world.dropping.as_mut().unwrap();
            config.rules.get_mut(&8).unwrap().kind = kind;
            let enabled = config.warning_enabled;
            let threshold = config.warning_threshold;
            let data = runtime.world.data_mut(actor).unwrap();
            enabled.set(&mut data.variables, 1).unwrap();
            threshold.set(&mut data.variables, 100).unwrap();
            data.inventory
                .restore_slot(
                    0,
                    Some(gielinor_world::inventory::Item {
                        id: 8,
                        count: 2,
                        vars: 0,
                    }),
                )
                .unwrap();
            let mut request = inventory_component.0.to_be_bytes().to_vec();
            request.extend(0u16.to_be_bytes());
            request.extend(8u16.to_be_bytes());
            request.push(7);
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        button,
                        &request,
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let mut events = pump(&mut runtime);
                events.extend(runtime.tick().unwrap());
                assert!(!events.iter().any(|e| matches!(
                    e,
                    Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
                )));
                if runtime.world.drop_dialog_token(actor).unwrap().is_some() {
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(1));
            }
            let pages: &[u16] = match kind {
                Kind::Drop => &[193, 219],
                Kind::Destroy => &[584],
                Kind::Release => &[219],
            };
            for (page, &interface) in pages.iter().enumerate() {
                let token = runtime.world.drop_dialog_token(actor).unwrap().unwrap();
                let expected_open =
                    gielinor_protocol::interfaces::open_sub((548 << 16) | 23, interface, false);
                loop {
                    let packet = loop {
                        if let Some(packet) = outgoing.next_packet().unwrap() {
                            break packet;
                        }
                        let mut byte = [0];
                        read(&mut runtime, &mut client, &mut byte);
                        outgoing.feed(&byte).unwrap();
                    };
                    if packet == expected_open {
                        break;
                    }
                }
                let [a, b, c, d] = ((interface as u32) << 16).to_be_bytes();
                let payload = [b, a, d, c, 0, 1];
                client
                    .write_all(
                        &gielinor_protocol::framing::encode(
                            pause,
                            &payload,
                            &mut cipher,
                            gielinor_protocol::framing::OpcodeWidth::Byte,
                        )
                        .unwrap(),
                    )
                    .unwrap();
                let deadline = Instant::now() + Duration::from_secs(2);
                loop {
                    let mut events = pump(&mut runtime);
                    events.extend(runtime.tick().unwrap());
                    assert!(!events.iter().any(|e| matches!(
                        e,
                        Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
                    )));
                    if runtime.world.drop_dialog_token(actor).unwrap() != Some(token) {
                        if page + 1 == pages.len() {
                            assert!(events.iter().any(|e| matches!(e, Event::World(WorldEvent::Drop { event: DropEvent::Committed { kind: actual, .. }, .. }) if *actual == kind)));
                        }
                        break;
                    }
                    assert!(Instant::now() < deadline);
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
            assert!(runtime
                .world
                .data(actor)
                .unwrap()
                .inventory
                .get(0)
                .is_none());
            assert!(!runtime.world.protection(actor).unwrap().suspended);
        }
    }
    if with_login {
        use gielinor_world::{inventory::Item, simulation::WorldEvent, ui::Component};
        runtime
            .open_interface(actor, Some(Component::new(548, 22)), 155, true)
            .unwrap();
        let data = runtime.world.data_mut(actor).unwrap();
        data.equipment
            .restore_slot(
                3,
                Some(Item {
                    id: 8,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 8,
                    count: 3,
                    vars: 0,
                }),
            )
            .unwrap();
        data.stats.set_current(6, 99);
        let mut request = ((155u32 << 16) | 6).to_be_bytes().to_vec();
        request.extend([255, 255, 255, 255, 1]);
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    button,
                    &request,
                    &mut cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut events = pump(&mut runtime);
            events.extend(runtime.tick().unwrap());
            if events
                .iter()
                .any(|e| matches!(e, Event::World(WorldEvent::AutocastPickerOpen { .. })))
            {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        let expected =
            gielinor_protocol::interfaces::events(201 << 16, 0, 58, (1 << 1) | (1 << 17)).unwrap();
        loop {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            if packet == expected {
                break;
            }
        }
        // The fixture shares a base varp with the layout field; restore spellbook zero.
        let base = runtime.definitions.varps[&1000];
        runtime
            .world
            .data_mut(actor)
            .unwrap()
            .variables
            .set(base, 0, true);
        for choice in [1u16, 0] {
            if choice == 0 {
                runtime
                    .open_interface(actor, Some(Component::new(548, 22)), 201, true)
                    .unwrap();
                runtime
                    .set_interface_events(actor, Component(201 << 16), 0, 58, (1 << 1) | (1 << 17))
                    .unwrap();
            }
            let mut request = (201u32 << 16).to_be_bytes().to_vec();
            request.extend(choice.to_be_bytes());
            request.extend([255, 255, 1]);
            client
                .write_all(
                    &gielinor_protocol::framing::encode(
                        button,
                        &request,
                        &mut cipher,
                        gielinor_protocol::framing::OpcodeWidth::Byte,
                    )
                    .unwrap(),
                )
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let mut events = pump(&mut runtime);
                events.extend(runtime.tick().unwrap());
                if events.iter().any(|e| {
                    matches!(
                        e,
                        Event::World(WorldEvent::AutocastSelected {
                            restore_tab: true,
                            ..
                        })
                    )
                }) {
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(1));
            }
            let data = runtime.world.data(actor).unwrap();
            let config = &runtime.world.combat_wear.as_ref().unwrap().autocast;
            assert_eq!(config.spell.get(&data.variables).unwrap(), choice as i32);
            assert_eq!(config.enabled.get(&data.variables).unwrap(), choice as i32);
            assert_eq!(
                config.defensive.get(&data.variables).unwrap(),
                choice as i32
            );
            assert_eq!(
                data.inventory.get(0),
                Some(Item {
                    id: 8,
                    count: 3,
                    vars: 0
                })
            );
        }
    }
    if with_login {
        exercise_tower_diary(&mut runtime, actor, &mut client, &mut outgoing);
        exercise_tower_tanner(&mut runtime, actor, &mut client, &mut cipher, &mut outgoing);
    }
    exercise_prayer_tab(&mut runtime, actor, &mut client, &mut cipher, &mut outgoing);
    exercise_creator_controls(&mut runtime, actor, &mut client, &mut cipher, &mut outgoing);
    exercise_tutorial_output(&mut runtime, actor, &mut client, &mut outgoing);
    exercise_tutorial_character(&mut runtime, actor, &mut client, &mut cipher, &mut outgoing);
    // A real encrypted NPC option reaches the banker approach handler and opens both panes.
    let coords = runtime.world.data(actor).unwrap().coords;
    let banker = runtime
        .world
        .spawn_npc(
            gielinor_world::npc::Npc::new(
                1,
                coords.translate(2, 0, 0).unwrap(),
                1,
                gielinor_world::npc::Restrict::NoMove,
                gielinor_world::npc::BlockWalk::None,
            )
            .unwrap(),
        )
        .unwrap();
    runtime.world.npc_options.types.insert(
        1,
        gielinor_world::npc_options::Definition {
            operations: [None, None, Some("Bank".into()), None, None],
            ..Default::default()
        },
    );
    runtime
        .world
        .npc_handlers
        .insert((1, 3), gielinor_world::npc_interaction::Handler::Bank);
    let index = banker.slot.to_be_bytes();
    let payload = [0, 128, index[0], index[1].wrapping_add(128)];
    client
        .write_all(
            &gielinor_protocol::framing::encode(
                *gielinor_protocol::tables::CLIENT
                    .iter()
                    .find(|p| p.opcode == 19)
                    .unwrap(),
                &payload,
                &mut cipher,
                gielinor_protocol::framing::OpcodeWidth::Byte,
            )
            .unwrap(),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let mut events = pump(&mut runtime);
        events.extend(runtime.tick().unwrap());
        if events.iter().any(|e| {
            matches!(
                e,
                Event::World(gielinor_world::simulation::WorldEvent::NpcInteraction {
                    event: gielinor_world::npc_interaction::Event::Bank,
                    ..
                })
            )
        }) {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    let mut main = false;
    let mut side = false;
    let expected_main = gielinor_protocol::interfaces::open_sub(bank_parent.0, 12, false);
    let expected_side = gielinor_protocol::interfaces::open_sub(side_parent.0, 15, false);
    while !main || !side {
        let packet = loop {
            if let Some(p) = outgoing.next_packet().unwrap() {
                break p;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        main |= packet == expected_main;
        side |= packet == expected_side;
    }
    assert_eq!(runtime.world.data(actor).unwrap().coords, coords);
    queues.accept.set(false);
    drop(client);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.connection(actor).is_some() {
        pump(&mut runtime);
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(queues.requests.borrow().is_empty());
    for _ in 0..17 {
        runtime.tick().unwrap();
    }
    assert!(runtime.world.data(actor).is_ok());
    assert!(!runtime.is_drained());
    // Socket loss closes the bank during grace. Its callback schedules
    // compression before the save snapshot and retained account-worker retry.
    assert!(queues.requests.borrow().is_empty());
    queues.accept.set(true);
    pump(&mut runtime);
    let Some(Request::Save { token, .. }) = queues.requests.borrow_mut().pop_front() else {
        panic!("save was lost on queue saturation")
    };
    queues.completed.borrow_mut().push_back(Completion::Saved {
        token,
        result: Err("simulated disk failure".into()),
    });
    assert!(pump(&mut runtime)
        .iter()
        .any(|e| matches!(e, Event::SaveFailed { .. })));
    assert!(runtime.world.data(actor).is_ok());
    assert!(queues.requests.borrow().is_empty());
    for _ in 0..10 {
        runtime.tick().unwrap();
    }
    let Some(Request::Save { token, data }) = queues.requests.borrow_mut().pop_front() else {
        panic!("failed save was not retried")
    };
    accounts.save(&data).unwrap();
    queues.completed.borrow_mut().push_back(Completion::Saved {
        token,
        result: Ok(()),
    });
    pump(&mut runtime);
    runtime.tick().unwrap();
    assert!(runtime.world.data(actor).is_err());
    assert!(runtime.is_drained());
    assert!(accounts
        .load("parity_actor", 1)
        .unwrap()
        .unwrap()
        .last_logout
        .is_some());
    assert_eq!(
        accounts
            .load("parity_actor", 1)
            .unwrap()
            .unwrap()
            .coords
            .x(),
        3203
    );
}
fn exercise_tower_tanner(
    runtime: &mut Runtime<Backend>,
    actor: gielinor_world::simulation::ActorId,
    client: &mut TcpStream,
    cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
    outgoing: &mut TickDecoder,
) {
    use gielinor_protocol::{
        chat,
        client_script::{self, Argument},
        interfaces,
    };
    use gielinor_world::{
        inventory::Item,
        npc::{BlockWalk, Npc, Restrict},
        npc_interaction::Handler,
        tower_diary::Progress,
        tower_tanning::{Config, HandlerConfig},
    };
    let data = runtime.world.data(actor).unwrap();
    let coords = data.coords;
    let saved_diary = data.tower_diary;
    let saved_arrival = data.tower_arrival;
    let saved_inventory = data.inventory.slots().to_vec();
    let levels = [7, 13, 3].map(|id| data.stats.base(id) as i32);
    let staff_taken = data.tower_arrival.staff_taken;
    let data = runtime.world.data_mut(actor).unwrap();
    for slot in 0..saved_inventory.len() {
        data.inventory.restore_slot(slot, None).unwrap();
    }
    data.inventory
        .restore_slot(
            0,
            Some(Item {
                id: 8,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    data.tower_diary = Some(Progress {
        counts: [4, 0, 0, 0, 0, 0],
        schema11: true,
        ..Default::default()
    });
    runtime.world.tower_tanning = Some(HandlerConfig {
        origin: coords.translate(-50, -50, 0).unwrap(),
        inventory: Config {
            hides: [8].into(),
            leather: 7,
        },
        skills: [7, 13, 3],
        bones: 526,
    });
    let npc = runtime
        .world
        .spawn_npc(
            Npc::new(
                2,
                coords.translate(1, 0, 0).unwrap(),
                1,
                Restrict::NoMove,
                BlockWalk::None,
            )
            .unwrap(),
        )
        .unwrap();
    runtime.world.npc_options.types.insert(
        2,
        gielinor_world::npc_options::Definition {
            operations: [None, None, Some("Tan".into()), None, None],
            ..Default::default()
        },
    );
    runtime
        .world
        .npc_handlers
        .insert((2, 3), Handler::TowerTanner);
    let index = npc.slot.to_be_bytes();
    let packet = *gielinor_protocol::tables::CLIENT
        .iter()
        .find(|p| p.opcode == 19)
        .unwrap();
    client
        .write_all(
            &gielinor_protocol::framing::encode(
                packet,
                &[0, 128, index[0], index[1].wrapping_add(128)],
                cipher,
                gielinor_protocol::framing::OpcodeWidth::Byte,
            )
            .unwrap(),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        pump(runtime);
        let events = runtime.tick().unwrap();
        if events.iter().any(|e| {
            matches!(
                e,
                Event::World(gielinor_world::simulation::WorldEvent::TowerDiary { .. })
            )
        }) {
            break;
        }
        assert!(Instant::now() < deadline, "Tanner action did not execute");
        std::thread::sleep(Duration::from_millis(1));
    }
    let data = runtime.world.data(actor).unwrap();
    assert_eq!(data.inventory.get(0).unwrap().id, 7);
    let marks = data.tower_diary.unwrap().marks(levels, staff_taken);
    assert_eq!((marks.counts[0], marks.bounty), (5, 1));
    let header = (153 << 16) | 1;
    let mut expected = [2, 3, 4, 5]
        .map(|id| interfaces::hide((153 << 16) | id, true))
        .to_vec();
    expected.push(client_script::run(600, &[0, 0, 10, header as i32].map(Argument::Int)).unwrap());
    expected.push(interfaces::text(header, &gielinor_world::tower_diary_ui::body(marks)).unwrap());
    expected.push(chat::game(0, None, "The chest holds a bronze axe.").unwrap());
    expected.push(
        chat::game(
            0,
            None,
            "The tanner tans 1 cowhide(s) into leather. Hides 5/5.",
        )
        .unwrap(),
    );
    let mut found = Vec::new();
    while found.last() != expected.last() {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(runtime, client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        if expected.contains(&packet) {
            found.push(packet);
        }
    }
    assert_eq!(found, expected, "Encrypted tanner task/chat ordering");
    // The resulting bounty is claimed through the encrypted location encoder.
    let position = runtime.world.data(actor).unwrap().coords;
    let chest = gielinor_world::locations::Location::new(
        position.translate(-1, 0, 0).unwrap(),
        60000,
        10,
        0,
    )
    .unwrap();
    let saved_chest_type = runtime.world.location_types.insert(
        chest.id,
        gielinor_world::loc_collision::LocCollision {
            width: 1,
            length: 1,
            block_walk: 0,
            block_range: false,
            break_route_finding: false,
        },
    );
    runtime.world.locations.add(chest).unwrap();
    runtime.world.tower_chest = Some(gielinor_world::tower_chest::HandlerConfig {
        location: chest.id,
        origin: position.translate(-50, -50, 0).unwrap(),
        skills: [7, 13, 3],
        bones: 526,
        inventory: gielinor_world::tower_chest::Config {
            ids: [8; 12],
            merge_aliases: Default::default(),
        },
    });
    let z = (chest.coords.z() as u16).to_be_bytes();
    let x = (chest.coords.x() as u16).to_le_bytes();
    let id = (chest.id as u16).to_be_bytes();
    let payload = [
        z[0],
        z[1].wrapping_add(128),
        x[0],
        x[1],
        128,
        128,
        id[0],
        id[1],
    ];
    let op = *gielinor_protocol::tables::CLIENT
        .iter()
        .find(|p| p.opcode == 73)
        .unwrap();
    client
        .write_all(
            &gielinor_protocol::framing::encode(
                op,
                &payload,
                cipher,
                gielinor_protocol::framing::OpcodeWidth::Byte,
            )
            .unwrap(),
        )
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        pump(runtime);
        let events = runtime.tick().unwrap();
        if events.iter().any(|e|matches!(e,Event::World(gielinor_world::simulation::WorldEvent::TowerDiary{marks,..}) if marks.done[0])) {break;}
        assert!(Instant::now() < deadline, "Chest click did not execute");
        std::thread::sleep(Duration::from_millis(1));
    }
    let progress = runtime.world.data(actor).unwrap().tower_diary.unwrap();
    assert!(progress.done[0]);
    assert_eq!(progress.bounty, 0);
    assert_eq!(
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 8)
            .map(|i| i.count)
            .sum::<i32>(),
        1
    );
    let mut expected = vec![chat::game(0, None, "The chest yields Bronze axe.").unwrap()];
    expected.extend([2, 3, 4, 5].map(|id| interfaces::hide((153 << 16) | id, true)));
    expected.push(client_script::run(600, &[0, 0, 10, header as i32].map(Argument::Int)).unwrap());
    expected.push(
        interfaces::text(
            header,
            &gielinor_world::tower_diary_ui::body(progress.marks(levels, staff_taken)),
        )
        .unwrap(),
    );
    let mut found = Vec::new();
    while found.last() != expected.last() {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(runtime, client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        if expected.contains(&packet) {
            found.push(packet);
        }
    }
    assert_eq!(
        found, expected,
        "Encrypted chest grant precedes diary refresh"
    );
    runtime.world.locations.delete(chest);
    if let Some(ty) = saved_chest_type {
        runtime.world.location_types.insert(chest.id, ty);
    } else {
        runtime.world.location_types.remove(&chest.id);
    }
    runtime.world.tower_chest = None;
    runtime.world.despawn_npc(npc).unwrap();
    runtime.world.npc_handlers.remove(&(2, 3));
    runtime.world.npc_options.types.remove(&2);
    runtime.world.tower_tanning = None;
    let data = runtime.world.data_mut(actor).unwrap();
    data.tower_diary = saved_diary;
    data.tower_arrival = saved_arrival;
    for (slot, item) in saved_inventory.into_iter().enumerate() {
        data.inventory.restore_slot(slot, item).unwrap();
    }
}

fn exercise_tower_diary(
    runtime: &mut Runtime<Backend>,
    actor: gielinor_world::simulation::ActorId,
    client: &mut TcpStream,
    outgoing: &mut TickDecoder,
) {
    use gielinor_protocol::{
        client_script::{self, Argument},
        interfaces,
    };
    use gielinor_world::{tower_diary::Progress, ui::Component};
    let before = runtime.world.data(actor).unwrap().tower_diary;
    // Replace the fixture's known new-account marks with explicit saved progress;
    // login assertions above cover the initial empty journal separately.
    let progress = Progress {
        counts: [3, 0, 0, 0, 0, 0],
        schema11: true,
        ..Default::default()
    };
    runtime.world.data_mut(actor).unwrap().tower_diary = Some(progress);
    runtime
        .open_interface(actor, Some(Component::new(548, 22)), 153, true)
        .unwrap();
    let header = (153 << 16) | 1;
    let expected = [2, 3, 4, 5]
        .map(|id| interfaces::hide((153 << 16) | id, true))
        .into_iter()
        .chain(std::iter::once(
            client_script::run(600, &[0, 0, 10, header as i32].map(Argument::Int)).unwrap(),
        ))
        .chain(std::iter::once(
            interfaces::text(
                header,
                &gielinor_world::tower_diary_ui::body(progress.marks([1, 1, 10], false)),
            )
            .unwrap(),
        ))
        .collect::<Vec<_>>();
    let mut found = Vec::new();
    loop {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(runtime, client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        if expected.contains(&packet) {
            found.push(packet);
        }
        if found.last() == expected.last() {
            break;
        }
    }
    assert_eq!(
        found, expected,
        "Tower journal sequence on encrypted transport"
    );
    assert_eq!(
        runtime.world.data(actor).unwrap().tower_diary,
        Some(progress)
    );
    runtime.world.data_mut(actor).unwrap().tower_diary = before;
}

fn exercise_tutorial_character(
    runtime: &mut Runtime<Backend>,
    actor: gielinor_world::simulation::ActorId,
    client: &mut TcpStream,
    cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
    outgoing: &mut TickDecoder,
) {
    use gielinor_world::ui::Component;
    fn drain(
        runtime: &mut Runtime<Backend>,
        actor: gielinor_world::simulation::ActorId,
        client: &mut TcpStream,
        outgoing: &mut TickDecoder,
    ) -> Vec<gielinor_protocol::framing::Packet> {
        let marker = gielinor_protocol::varp::update(65000, 239929);
        let connection = runtime.connection(actor).unwrap();
        runtime
            .transport
            .send(connection, marker.opcode, &marker.payload)
            .unwrap();
        let mut packets = vec![];
        loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                if packet == marker {
                    return packets;
                }
                packets.push(packet);
            } else {
                let mut byte = [0];
                read(runtime, client, &mut byte);
                outgoing.feed(&byte).unwrap();
            }
        }
    }
    fn button(
        runtime: &mut Runtime<Backend>,
        client: &mut TcpStream,
        cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
        component: u32,
        expected_route: gielinor_world::ui::ButtonRoute,
    ) {
        let def = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == 47)
            .unwrap();
        let mut payload = component.to_be_bytes().to_vec();
        payload.extend(u16::MAX.to_be_bytes());
        payload.extend(u16::MAX.to_be_bytes());
        payload.push(1);
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    def,
                    &payload,
                    cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let events = pump(runtime);
            assert!(!events
                .iter()
                .any(|e| matches!(e, Event::PacketRejected { .. })));
            if events.iter().any(|e|matches!(e,Event::InterfaceButton{button,route,..} if button.component==component && *route==expected_route)) {break;}
            assert!(Instant::now() < deadline, "tutorial button not admitted");
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    for choice in 0..3 {
        runtime.set_varp(actor, 5003, 0).unwrap();
        runtime.set_varp(actor, 5004, 0).unwrap();
        drain(runtime, actor, client, outgoing);
        runtime.begin_tutorial(actor).unwrap();
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .contains_interface(679));
        let packets = drain(runtime, actor, client, outgoing);
        let hides: Vec<_> = packets.iter().filter(|p| p.opcode == 63).cloned().collect();
        assert_eq!(
            hides,
            [false, true, false].map(|v| gielinor_protocol::interfaces::hide((614 << 16) | 1, v))
        );
        assert!(packets.contains(&gielinor_protocol::music::song(62, 0, 0, 0, 0)));
        button(
            runtime,
            client,
            cipher,
            (679 << 16) | 3,
            gielinor_world::ui::ButtonRoute::Modal,
        );
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 1);
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .contains_interface(929));
        assert!(!runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .contains_interface(679));
        drain(runtime, actor, client, outgoing);
        button(
            runtime,
            client,
            cipher,
            (929 << 16) | choice,
            gielinor_world::ui::ButtonRoute::Modal,
        );
        let vars = &runtime.world.data(actor).unwrap().variables;
        assert_eq!(vars.get(5003), 2);
        assert_eq!(vars.get(5005), i32::from(choice == 0));
        assert!(!runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .contains_interface(929));
        let packets = drain(runtime, actor, client, outgoing);
        assert!(
            packets.contains(&gielinor_protocol::interfaces::hide((614 << 16) | 1, true)),
            "world click blocker must be released"
        );
    }
    // Original bootstrap fixtures cover completed accounts, both body types,
    // invalid progress, and both creator stages. Compare the actual TCP stream.
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-bootstrap.expected"
    ))
    .expect("generate the original tutorial bootstrap fixture first");
    let mut compared = 0;
    for row in fixture.lines() {
        let fields: Vec<_> = row.split('|').collect();
        assert_eq!(fields.len(), 7);
        if fields[3] != "10" {
            continue;
        }
        for child in [23, 24, 25] {
            runtime
                .close_interface(actor, Component::new(548, child))
                .unwrap();
        }
        for id in 5000..=5005 {
            runtime.set_varp(actor, id, 0).unwrap();
        }
        runtime
            .set_varp(actor, 5003, fields[0].parse().unwrap())
            .unwrap();
        runtime
            .set_varp(actor, 5004, fields[1].parse().unwrap())
            .unwrap();
        let mut design = runtime.appearance_design(actor).unwrap().clone();
        design.body_type = fields[2].parse().unwrap();
        design.pronoun = design.body_type + 1;
        design.hidden = false;
        runtime.set_appearance(actor, design).unwrap();
        drain(runtime, actor, client, outgoing);
        runtime.begin_tutorial(actor).unwrap();
        let actual = drain(runtime, actor, client, outgoing)
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
        assert_eq!(actual, fields[6], "bootstrap packets: {row}");
        assert_eq!(
            runtime.appearance_design(actor).unwrap().hidden.to_string(),
            fields[4]
        );
        let vars = &runtime.world.data(actor).unwrap().variables;
        let values = (5000..=5005)
            .map(|id| vars.get(id).to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(values, fields[5], "bootstrap variables: {row}");
        compared += 1;
    }
    assert_eq!(compared, 12);
    for child in [23, 24, 25] {
        runtime
            .close_interface(actor, Component::new(548, child))
            .unwrap();
    }
    // Side-panel hooks do not test the completed flag in the original.
    runtime.set_varp(actor, 5004, 1).unwrap();
    runtime.set_varp(actor, 5003, 30).unwrap();
    runtime
        .open_interface(actor, Some(Component::new(548, 25)), 162, true)
        .unwrap();
    drain(runtime, actor, client, outgoing);
    for step in [31, 32] {
        button(
            runtime,
            client,
            cipher,
            162 << 16,
            gielinor_world::ui::ButtonRoute::Overlay,
        );
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), step);
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(5004), 1);
        assert!(!drain(runtime, actor, client, outgoing).is_empty());
    }
    runtime.set_varp(actor, 5003, 3).unwrap();
    drain(runtime, actor, client, outgoing);
    button(
        runtime,
        client,
        cipher,
        614 << 16,
        gielinor_world::ui::ButtonRoute::Overlay,
    );
    let packets = drain(runtime, actor, client, outgoing);
    assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 3);
    assert!(packets.iter().all(|p| p.opcode == 50 || p.opcode == 114));
    assert_eq!(packets.last().unwrap(), &gielinor_protocol::client_script::run(1974, &[gielinor_protocol::client_script::Argument::String("<col=0000ff>Settings menu</col><br>Good. Now speak to the Gielinor Guide again to continue.".into())]).unwrap());
    drain(runtime, actor, client, outgoing);
    runtime.begin_tutorial(actor).unwrap();
    assert!(drain(runtime, actor, client, outgoing).is_empty());
    // Real encrypted NPC op1 and Continue packets traverse the protected
    // dialogue pipeline. Its final focus replaces the chat modal with overlay162.
    runtime.set_varp(actor, 5004, 0).unwrap();
    runtime.set_varp(actor, 5003, 2).unwrap();
    let coords = runtime.world.observe(actor).unwrap().coords;
    let guide = runtime
        .world
        .spawn_npc(
            gielinor_world::npc::Npc::new(
                1,
                coords.translate(1, 0, 0).unwrap(),
                1,
                gielinor_world::npc::Restrict::NoMove,
                gielinor_world::npc::BlockWalk::None,
            )
            .unwrap(),
        )
        .unwrap();
    runtime.world.npc_options.types.insert(
        1,
        gielinor_world::npc_options::Definition {
            operations: [Some("Talk-to".into()), None, None, None, None],
            ..Default::default()
        },
    );
    runtime.world.npc_handlers.insert(
        (1, 1),
        gielinor_world::npc_interaction::Handler::TutorialGuide { step_varp: 5003 },
    );
    drain(runtime, actor, client, outgoing);
    let send = |client: &mut TcpStream,
                cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
                opcode,
                payload: &[u8]| {
        let def = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == opcode)
            .unwrap();
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    def,
                    payload,
                    cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
    };
    let [lo, hi] = guide.slot.to_le_bytes();
    send(client, cipher, 102, &[lo, hi, 0, 128]);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.dialogue_token(actor).unwrap().is_none() {
        let mut events = pump(runtime);
        events.extend(runtime.tick().unwrap());
        assert!(!events.iter().any(|e| matches!(
            e,
            Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
        )));
        assert!(
            Instant::now() < deadline,
            "native guide operation did not start dialogue"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    drain(runtime, actor, client, outgoing);
    for page in 0..4 {
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 2);
        assert!(runtime
            .interface_state(actor)
            .unwrap()
            .modals
            .contains_interface(231));
        let token = runtime.world.dialogue_token(actor).unwrap();
        let [a, b, c, d] = ((231u32 << 16) | 5).to_be_bytes();
        send(client, cipher, 115, &[b, a, d, c, 255, 255]);
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.dialogue_token(actor).unwrap() == token {
            let mut events = pump(runtime);
            events.extend(runtime.tick().unwrap());
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
            )));
            assert!(
                Instant::now() < deadline,
                "guide Continue did not advance page{page}"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let packets = drain(runtime, actor, client, outgoing);
        if page == 3 {
            assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 3);
            assert!(packets.contains(
                &gielinor_protocol::client_script::run(
                    915,
                    &[gielinor_protocol::client_script::Argument::Int(11)]
                )
                .unwrap()
            ));
            assert!(runtime
                .interface_state(actor)
                .unwrap()
                .overlays
                .contains_interface(162));
            assert!(!runtime
                .interface_state(actor)
                .unwrap()
                .modals
                .contains_interface(231));
            assert!(!runtime.world.protection(actor).unwrap().suspended);
        }
    }
    // Real encrypted location operations cross the tutorial doorway in both
    // directions. The second op refreshes progress without rewinding it.
    let source = runtime.world.data(actor).unwrap().coords;
    let loc = gielinor_world::locations::Location::new(source, 60000, 0, 0).unwrap();
    runtime.world.location_types.insert(
        60000,
        gielinor_world::loc_collision::LocCollision {
            width: 1,
            length: 1,
            block_walk: 0,
            block_range: false,
            break_route_finding: false,
        },
    );
    // Earlier door coverage leaves a dynamic replacement on this tile. Install
    // through the live registry so it cannot shadow a static test location.
    runtime.world.change_location(loc, true).unwrap();
    assert_eq!(runtime.world.locations.find_type(source, 60000), Some(loc));
    runtime
        .set_tutorial_step(
            actor,
            gielinor_world::tutorial_steps::Step::GielinorDoor,
            None,
        )
        .unwrap();
    drain(runtime, actor, client, outgoing);
    for target in [source.translate(-1, 0, 0).unwrap(), source] {
        let [zh, zl] = (source.z() as u16).to_be_bytes();
        let [xl, xh] = (source.x() as u16).to_le_bytes();
        let [idh, idl] = 60000u16.to_be_bytes();
        send(
            client,
            cipher,
            73,
            &[zh, zl.wrapping_add(128), xl, xh, 128, 128, idh, idl],
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.data(actor).unwrap().coords != target {
            let mut events = pump(runtime);
            events.extend(runtime.tick().unwrap());
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
            )));
            assert!(
                Instant::now() < deadline,
                "tutorial gate operation did not cross doorway: target={target:?}, current={:?}",
                runtime.world.data(actor).unwrap().coords
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 5);
        let packets = drain(runtime, actor, client, outgoing);
        let hint = gielinor_protocol::hint_arrow::Arrow::Npc(guide.slot).encode();
        assert!(packets.contains(&hint));
        assert!(
            packets.iter().position(|p| *p == hint).unwrap()
                > packets.iter().rposition(|p| p.opcode == 28).unwrap()
        );
    }
    runtime.world.change_location(loc, false).unwrap();
    runtime.world.npc_handlers.insert(
        (1, 1),
        gielinor_world::npc_interaction::Handler::SurvivalTutor(
            gielinor_world::tutorial_survival::Config {
                step_varp: 5003,
                net: 8,
                axe: 8,
                tinderbox: 8,
            },
        ),
    );
    let item_count = |runtime: &Runtime<Backend>| {
        runtime
            .world
            .data(actor)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|item| item.id == 8)
            .map(|item| item.count)
            .sum::<i32>()
    };
    let before = item_count(runtime);
    drain(runtime, actor, client, outgoing);
    let [lo, hi] = guide.slot.to_le_bytes();
    send(client, cipher, 102, &[lo, hi, 0, 128]);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.dialogue_token(actor).unwrap().is_none() {
        pump(runtime);
        runtime.tick().unwrap();
        assert!(
            Instant::now() < deadline,
            "Survival Tutor dialogue did not begin"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    drain(runtime, actor, client, outgoing);
    for page in 0..2 {
        assert_eq!(item_count(runtime), before);
        let token = runtime.world.dialogue_token(actor).unwrap();
        let [a, b, c, d] = ((231u32 << 16) | 5).to_be_bytes();
        send(client, cipher, 115, &[b, a, d, c, 255, 255]);
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.dialogue_token(actor).unwrap() == token {
            pump(runtime);
            runtime.tick().unwrap();
            assert!(Instant::now() < deadline, "Survival Tutor Continue stalled");
            std::thread::sleep(Duration::from_millis(1));
        }
        let packets = drain(runtime, actor, client, outgoing);
        if page == 1 {
            assert_eq!(item_count(runtime), before + 1);
            assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 6);
            assert!(packets.contains(
                &gielinor_protocol::client_script::run(
                    915,
                    &[gielinor_protocol::client_script::Argument::Int(3)]
                )
                .unwrap()
            ));
            assert!(runtime
                .interface_state(actor)
                .unwrap()
                .overlays
                .contains_interface(162));
        }
    }
    use gielinor_world::npc_interaction::Handler;
    let tutor_cases: Vec<(i32, Handler, &[u32], i32, Option<i32>)> = vec![
        (
            15,
            Handler::MasterChef(gielinor_world::tutorial_chef::Config {
                step_varp: 5003,
                flour: 1933,
                water: 1929,
            }),
            &[231, 217, 231],
            16,
            None,
        ),
        (
            20,
            Handler::QuestTutor { step_varp: 5003 },
            &[231],
            21,
            Some(2),
        ),
        (
            21,
            Handler::QuestTutor { step_varp: 5003 },
            &[231, 231, 231, 231, 231],
            22,
            None,
        ),
        (
            23,
            Handler::MiningTutor(gielinor_world::tutorial_mining_tutor::Config {
                step_varp: 5003,
                pickaxe: 1265,
                hammer: 2347,
            }),
            &[231, 217, 231, 231],
            24,
            None,
        ),
        (
            26,
            Handler::MiningTutor(gielinor_world::tutorial_mining_tutor::Config {
                step_varp: 5003,
                pickaxe: 1265,
                hammer: 2347,
            }),
            &[217, 231],
            27,
            None,
        ),
    ];
    for (initial, handler, interfaces, expected, tab) in tutor_cases {
        runtime.world.npc_handlers.insert((1, 1), handler);
        runtime
            .set_tutorial_step(
                actor,
                gielinor_world::tutorial_steps::Step::from_var(initial),
                None,
            )
            .unwrap();
        drain(runtime, actor, client, outgoing);
        send(client, cipher, 102, &[lo, hi, 0, 128]);
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.dialogue_token(actor).unwrap().is_none() {
            pump(runtime);
            runtime.tick().unwrap();
            assert!(
                Instant::now() < deadline,
                "Tutor{initial} dialogue did not begin"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let opening_packets = drain(runtime, actor, client, outgoing);
        if initial == 26 {
            assert!(
                opening_packets.contains(&gielinor_protocol::interfaces::animation(
                    (217 << 16) | 2,
                    Some(6000)
                ))
            );
        }
        for (index, &interface) in interfaces.iter().enumerate() {
            assert_eq!(
                runtime.world.data(actor).unwrap().variables.get(5003),
                initial
            );
            assert!(runtime
                .interface_state(actor)
                .unwrap()
                .modals
                .contains_interface(interface as u16));
            let token = runtime.world.dialogue_token(actor).unwrap();
            let [a, b, c, d] = ((interface << 16) | 5).to_be_bytes();
            send(client, cipher, 115, &[b, a, d, c, 255, 255]);
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime.world.dialogue_token(actor).unwrap() == token {
                pump(runtime);
                runtime.tick().unwrap();
                assert!(
                    Instant::now() < deadline,
                    "Tutor{initial} Continue stalled on page{index}"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
            let packets = drain(runtime, actor, client, outgoing);
            if initial == 23 && index == 0 {
                let name = &runtime.world.data(actor).unwrap().display_name;
                assert!(packets.contains(
                    &gielinor_protocol::interfaces::text(
                        (217 << 16) | 6,
                        &format!("You can call me {name}.")
                    )
                    .unwrap()
                ));
            }
            if index + 1 == interfaces.len() {
                assert_eq!(
                    runtime.world.data(actor).unwrap().variables.get(5003),
                    expected
                );
                assert!(runtime
                    .interface_state(actor)
                    .unwrap()
                    .overlays
                    .contains_interface(162));
                if let Some(tab) = tab {
                    assert!(packets.contains(
                        &gielinor_protocol::client_script::run(
                            915,
                            &[gielinor_protocol::client_script::Argument::Int(tab)]
                        )
                        .unwrap()
                    ));
                }
            }
        }
        if initial == 23 || initial == 26 {
            let item = if initial == 23 { 1265 } else { 2347 };
            assert_eq!(
                runtime
                    .world
                    .data(actor)
                    .unwrap()
                    .inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|entry| entry.id == item)
                    .map(|entry| entry.count)
                    .sum::<i32>(),
                1
            );
        }
        if initial == 15 {
            for id in [1933, 1929] {
                assert_eq!(
                    runtime
                        .world
                        .data(actor)
                        .unwrap()
                        .inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == id)
                        .map(|i| i.count)
                        .sum::<i32>(),
                    1
                );
            }
            // Exercise IF_BUTTONT through encryption, permissions, protected
            // cancellation, authoritative slots, recipe and tutorial focus.
            let inv_component = Component::new(149, 0);
            runtime
                .open_interface(actor, Some(Component::new(548, 20)), 149, true)
                .unwrap();
            runtime
                .set_interface_events(actor, inv_component, 0, 27, (1 << 16) | (1 << 21))
                .unwrap();
            let inv = &runtime.world.data(actor).unwrap().inventory;
            let slot = |id| {
                inv.slots()
                    .iter()
                    .position(|item| item.is_some_and(|i| i.id == id))
                    .unwrap() as u16
            };
            let [sh, sl] = slot(1933).to_be_bytes();
            let [tl, th] = slot(1929).to_le_bytes();
            let [oh, ol] = 1933u16.to_be_bytes();
            let [dl, dh] = 1929u16.to_le_bytes();
            let [a, b, c, d] = inv_component.0.to_be_bytes();
            let mut request = inv_component.0.to_be_bytes().to_vec();
            request.extend([
                sh,
                sl.wrapping_add(128),
                dl.wrapping_add(128),
                dh,
                tl,
                th,
                oh,
                ol.wrapping_add(128),
                c,
                d,
                a,
                b,
            ]);
            drain(runtime, actor, client, outgoing);
            send(client, cipher, 27, &request);
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime.world.data(actor).unwrap().variables.get(5003) == 16 {
                let mut events = pump(runtime);
                events.extend(runtime.tick().unwrap());
                assert!(
                    !events.iter().any(|e| matches!(
                        e,
                        Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
                    )),
                    "Dough operation was rejected or synchronization failed"
                );
                assert!(Instant::now() < deadline, "Dough target operation stalled");
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 17);
            let inv = &runtime.world.data(actor).unwrap().inventory;
            for id in [1931, 1925, 2307] {
                assert!(
                    inv.slots().iter().flatten().any(|i| i.id == id),
                    "Recipe item {id} missing"
                );
            }
            assert!(!inv
                .slots()
                .iter()
                .flatten()
                .any(|i| matches!(i.id, 1933 | 1929)));
            let packets = drain(runtime, actor, client, outgoing);
            assert!(packets.contains(
                &gielinor_protocol::chat::game(0, None, "You make some dough.").unwrap()
            ));
            let range_coords = runtime
                .world
                .data(actor)
                .unwrap()
                .coords
                .translate(1, 0, 0)
                .unwrap();
            let range =
                gielinor_world::locations::Location::new(range_coords, 60001, 10, 0).unwrap();
            runtime.world.location_types.insert(
                60001,
                gielinor_world::loc_collision::LocCollision {
                    width: 1,
                    length: 1,
                    block_walk: 2,
                    block_range: false,
                    break_route_finding: false,
                },
            );
            runtime.world.change_location(range, true).unwrap();
            let before_xp = runtime.world.data(actor).unwrap().stats.xp(7);
            let [zh, zl] = (range_coords.z() as u16).to_be_bytes();
            let [xl, xh] = (range_coords.x() as u16).to_le_bytes();
            let [idh, idl] = 60001u16.to_be_bytes();
            send(
                client,
                cipher,
                73,
                &[zh, zl.wrapping_add(128), xl, xh, 128, 128, idh, idl],
            );
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime.world.data(actor).unwrap().variables.get(5003) == 17 {
                let mut events = pump(runtime);
                events.extend(runtime.tick().unwrap());
                assert!(
                    !events.iter().any(|e| matches!(
                        e,
                        Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
                    )),
                    "Range operation was rejected or synchronization failed"
                );
                for event in &events {
                    if let Event::World(gielinor_world::simulation::WorldEvent::ActionRejected {
                        error,
                        ..
                    }) = event
                    {
                        panic!("Range world action rejected: {error:?}");
                    }
                }
                assert!(
                    Instant::now() < deadline,
                    "Tutorial range operation stalled"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 18);
            assert_eq!(
                runtime.world.data(actor).unwrap().stats.xp(7),
                before_xp + 40
            );
            let inv = &runtime.world.data(actor).unwrap().inventory;
            assert!(inv.slots().iter().flatten().any(|i| i.id == 2309));
            assert!(!inv.slots().iter().flatten().any(|i| i.id == 2307));
            let packets = drain(runtime, actor, client, outgoing);
            assert!(packets.contains(
                &gielinor_protocol::chat::game(0, None, "You manage to bake some bread.").unwrap()
            ));
            // Use the same range through OPLOCT, retaining authoritative slot
            // checks and distinguishing TgtLoc from IF_BUTTONT permissions.
            let inv = &mut runtime.world.data_mut(actor).unwrap().inventory;
            let dough_slot = inv.slots().iter().position(Option::is_none).unwrap();
            inv.restore_slot(
                dough_slot,
                Some(gielinor_world::inventory::Item {
                    id: 2307,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let [sh, sl] = (dough_slot as u16).to_be_bytes();
            let mut use_range = vec![0, sh, sl.wrapping_add(128), idl, idh];
            use_range.extend(inv_component.0.to_le_bytes());
            use_range.extend([zl.wrapping_add(128), zh, xl, xh]);
            use_range.extend(2307u16.to_le_bytes());
            send(client, cipher, 62, &use_range);
            let deadline = Instant::now() + Duration::from_secs(2);
            loop {
                let mut events = pump(runtime);
                events.extend(runtime.tick().unwrap());
                if events.iter().any(|e| matches!(e, Event::World(gielinor_world::simulation::WorldEvent::ClearMapFlag(id)) if *id == actor)) { break; }
                assert!(
                    Instant::now() < deadline,
                    "Denied location target not processed"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(
                runtime.world.data(actor).unwrap().stats.xp(7),
                before_xp + 40
            );
            assert_eq!(
                runtime
                    .world
                    .data(actor)
                    .unwrap()
                    .inventory
                    .get(dough_slot)
                    .unwrap()
                    .id,
                2307
            );
            drain(runtime, actor, client, outgoing);
            runtime
                .set_interface_events(
                    actor,
                    inv_component,
                    0,
                    27,
                    (1 << 13) | (1 << 16) | (1 << 21),
                )
                .unwrap();
            send(client, cipher, 62, &use_range);
            let deadline = Instant::now() + Duration::from_secs(2);
            while runtime.world.data(actor).unwrap().stats.xp(7) == before_xp + 40 {
                let mut events = pump(runtime);
                events.extend(runtime.tick().unwrap());
                assert!(
                    !events.iter().any(|e| matches!(
                        e,
                        Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
                    )),
                    "Item-on-range operation rejected"
                );
                assert!(Instant::now() < deadline, "Item-on-range operation stalled");
                std::thread::sleep(Duration::from_millis(1));
            }
            assert_eq!(
                runtime.world.data(actor).unwrap().stats.xp(7),
                before_xp + 80
            );
            assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 18);
            let packets = drain(runtime, actor, client, outgoing);
            assert!(packets.contains(
                &gielinor_protocol::chat::game(0, None, "You manage to bake some bread.").unwrap()
            ));
            runtime.world.change_location(range, false).unwrap();
        }
    }
    // The original Magic Instructor uses a custom choice title, not the default.
    runtime.world.npc_handlers.insert(
        (1, 1),
        gielinor_world::npc_interaction::Handler::MagicTutor(
            gielinor_world::tutorial_magic_tutor::Config {
                step_varp: 5003,
                air: 8,
                mind: 8,
            },
        ),
    );
    runtime.set_varp(actor, 5003, 51).unwrap();
    runtime.set_varp(actor, 5004, 0).unwrap();
    drain(runtime, actor, client, outgoing);
    let [lo, hi] = guide.slot.to_le_bytes();
    send(client, cipher, 102, &[lo, hi, 0, 128]);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.dialogue_token(actor).unwrap().is_none() {
        pump(runtime);
        runtime.tick().unwrap();
        assert!(Instant::now() < deadline, "Magic Instructor did not start");
        std::thread::sleep(Duration::from_millis(1));
    }
    drain(runtime, actor, client, outgoing);
    for (component, selection) in [((231u32 << 16) | 5, u16::MAX), ((219u32 << 16) | 1, 2)] {
        let token = runtime.world.dialogue_token(actor).unwrap();
        let [a, b, c, d] = component.to_be_bytes();
        let [high, low] = selection.to_be_bytes();
        send(client, cipher, 115, &[b, a, d, c, high, low]);
        let deadline = Instant::now() + Duration::from_secs(2);
        while runtime.world.dialogue_token(actor).unwrap() == token {
            let mut events = pump(runtime);
            events.extend(runtime.tick().unwrap());
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
            )));
            assert!(
                Instant::now() < deadline,
                "Magic Instructor resume did not advance"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let packets = drain(runtime, actor, client, outgoing);
        if selection == u16::MAX {
            use gielinor_protocol::client_script::Argument;
            assert!(packets.contains(
                &gielinor_protocol::client_script::run(
                    58,
                    &[
                        Argument::String(gielinor_world::tutorial_magic_tutor::GO_TITLE.into()),
                        Argument::String("Yes.|No.".into()),
                    ]
                )
                .unwrap()
            ));
        }
    }
    assert_eq!(runtime.world.data(actor).unwrap().variables.get(5003), 51);
    assert!(runtime.world.dialogue_token(actor).unwrap().is_none());
    // Original Wind Strike selection changes focus but never fabricates a cast.
    let wind = (679 << 16) | 3;
    runtime
        .open_interface(actor, Some(Component::new(548, 23)), 679, true)
        .unwrap();
    runtime
        .set_interface_events(
            actor,
            Component(wind),
            -1,
            -1,
            gielinor_world::ui::operation_mask(1).unwrap(),
        )
        .unwrap();
    for (raw, done, eligible) in [
        (49, 0, true),
        (50, 0, true),
        (48, 0, false),
        (49, 1, false),
        (49, 2, true),
    ] {
        runtime.set_varp(actor, 5003, raw).unwrap();
        runtime.set_varp(actor, 5004, done).unwrap();
        drain(runtime, actor, client, outgoing);
        button(
            runtime,
            client,
            cipher,
            wind,
            gielinor_world::ui::ButtonRoute::Overlay,
        );
        let packets = drain(runtime, actor, client, outgoing);
        assert_eq!(
            runtime.world.data(actor).unwrap().variables.get(5003),
            if eligible { 50 } else { raw }
        );
        let message =
            gielinor_protocol::chat::game(0, None, gielinor_world::tutorial_magic_ui::WIND_MESSAGE)
                .unwrap();
        assert_eq!(
            packets.iter().filter(|packet| **packet == message).count(),
            usize::from(eligible)
        );
        if eligible {
            assert!(packets.contains(
                &gielinor_protocol::client_script::run(
                    2081,
                    &[gielinor_protocol::client_script::Argument::Int(wind as i32)]
                )
                .unwrap()
            ));
        } else {
            assert!(packets.is_empty());
        }
    }
    runtime
        .close_interface(actor, Component::new(548, 23))
        .unwrap();
    // Reach both booths through encrypted OPLOC1, including denial, the
    // completed-flag bypass, and the distinction between gifting and topping up.
    let saved_bank = runtime.world.data(actor).unwrap().bank.clone();
    for (id, raw, done, coins, expected_step, allowed) in [
        (61000u16, 40, 0, 0, 40, false),
        (61000, 41, 0, 0, 42, true),
        (61000, 41, 0, 1, 42, true),
        (61000, 0, 1, 24, 42, true),
        (61001, 41, 0, 0, 41, false),
        (61001, 42, 0, 0, 43, true),
        (61001, 0, 1, 0, 43, true),
    ] {
        use gielinor_world::{
            inventory::{Insert, Inventory, StackType},
            tutorial_bank,
        };
        runtime.set_varp(actor, 5003, raw).unwrap();
        runtime.set_varp(actor, 5004, done).unwrap();
        let mut bank = Inventory::new(28, StackType::Always, false);
        if coins > 0 {
            bank.insert(&runtime.world.object_types, Insert::new(8, coins))
                .unwrap();
        }
        runtime.world.data_mut(actor).unwrap().bank = bank;
        let coords = runtime.world.data(actor).unwrap().coords;
        let booth = gielinor_world::locations::Location::new(coords, id as u32, 10, 0).unwrap();
        runtime.world.change_location(booth, true).unwrap();
        drain(runtime, actor, client, outgoing);
        let [zh, zl] = (coords.z() as u16).to_be_bytes();
        let [xl, xh] = (coords.x() as u16).to_le_bytes();
        let [idh, idl] = id.to_be_bytes();
        send(
            client,
            cipher,
            73,
            &[zh, zl.wrapping_add(128), xl, xh, 128, 128, idh, idl],
        );
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut events = pump(runtime);
            events.extend(runtime.tick().unwrap());
            assert!(!events.iter().any(|e| matches!(
                e,
                Event::PacketRejected { .. } | Event::SynchronizationFailed { .. }
            )));
            if events.iter().any(|e| {
                matches!(
                    e,
                    Event::World(
                        gielinor_world::simulation::WorldEvent::TutorialBank { .. }
                            | gielinor_world::simulation::WorldEvent::LocationMessage { .. }
                    )
                )
            }) {
                break;
            }
            assert!(Instant::now() < deadline, "tutorial booth did not resolve");
            std::thread::sleep(Duration::from_millis(1));
        }
        let packets = drain(runtime, actor, client, outgoing);
        assert_eq!(
            runtime.world.data(actor).unwrap().variables.get(5003),
            expected_step
        );
        let messages: &[&str] = match (id, allowed) {
            (61000, true) => &[tutorial_bank::BANK_MESSAGE],
            (61000, false) => &[tutorial_bank::BANK_DENY],
            (_, true) => &tutorial_bank::POLL_MESSAGES,
            (_, false) => &[tutorial_bank::POLL_DENY],
        };
        for message in messages {
            let packet = gielinor_protocol::chat::game(0, None, message).unwrap();
            assert_eq!(packets.iter().filter(|p| **p == packet).count(), 1);
        }
        assert_eq!(
            runtime
                .world
                .data(actor)
                .unwrap()
                .bank
                .get(0)
                .map_or(0, |item| item.count),
            if id == 61000 && allowed && coins == 0 {
                25
            } else {
                coins
            }
        );
        assert_eq!(
            runtime
                .interface_state(actor)
                .unwrap()
                .modals
                .contains_interface(12),
            id == 61000 && allowed
        );
        if id == 61000 && allowed {
            runtime
                .close_interface(actor, Component::new(548, 21))
                .unwrap();
            runtime
                .close_interface(actor, Component::new(548, 22))
                .unwrap();
        }
        runtime.world.change_location(booth, false).unwrap();
        drain(runtime, actor, client, outgoing);
    }
    runtime.world.data_mut(actor).unwrap().bank = saved_bank;
    runtime.world.despawn_npc(guide).unwrap();
    runtime.set_varp(actor, 5004, 1).unwrap();
    drain(runtime, actor, client, outgoing);
    runtime
        .close_interface(actor, Component::new(548, 24))
        .unwrap();
    runtime
        .close_interface(actor, Component::new(548, 25))
        .unwrap();
    drain(runtime, actor, client, outgoing);
}

fn exercise_tutorial_output(
    runtime: &mut Runtime<Backend>,
    actor: gielinor_world::simulation::ActorId,
    client: &mut TcpStream,
    outgoing: &mut TickDecoder,
) {
    use gielinor_server::tutorial_ui::Effect;
    use gielinor_world::{
        ui::Component,
        variables::{Transmit, Varbit, Varp},
    };
    let parent = Component::new(548, 23);
    let connection = runtime.connection(actor).unwrap();
    let marker = gielinor_protocol::varp::update(65000, 239614);
    runtime
        .transport
        .send(connection, marker.opcode, &marker.payload)
        .unwrap();
    loop {
        if let Some(packet) = outgoing.next_packet().unwrap() {
            if packet == marker {
                break;
            }
        } else {
            let mut byte = [0];
            read(runtime, client, &mut byte);
            outgoing.feed(&byte).unwrap();
        }
    }
    runtime.definitions.varps.insert(
        5002,
        Varp {
            id: 5002,
            permanent: false,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
    );
    runtime.definitions.varbits.insert(
        5002,
        Varbit {
            base: 5002,
            lsb: 4,
            msb: 7,
        },
    );
    let arrow = gielinor_protocol::hint_arrow::Arrow::Npc(37).encode();
    let hide = gielinor_protocol::interfaces::hide(679 << 16, true);
    runtime
        .apply_tutorial_effects(
            actor,
            [
                Effect::Panel {
                    parent,
                    interface: 679,
                    open: true,
                },
                Effect::Panel {
                    parent,
                    interface: 679,
                    open: true,
                },
                Effect::Packet(arrow.clone()),
                Effect::Packet(hide.clone()),
                Effect::VariableBit {
                    bit: 5002,
                    value: 12,
                },
                Effect::VariableBit {
                    bit: 5002,
                    value: 0,
                },
                Effect::Panel {
                    parent,
                    interface: 679,
                    open: false,
                },
                Effect::Panel {
                    parent,
                    interface: 679,
                    open: false,
                },
            ],
        )
        .unwrap();
    assert!(!runtime
        .interface_state(actor)
        .unwrap()
        .overlays
        .contains_interface(679));
    assert_eq!(runtime.world.data(actor).unwrap().variables.get(5002), 0);
    runtime
        .transport
        .send(connection, marker.opcode, &marker.payload)
        .unwrap();
    let mut actual = vec![];
    loop {
        if let Some(packet) = outgoing.next_packet().unwrap() {
            if packet == marker {
                break;
            }
            actual.push(packet);
        } else {
            let mut byte = [0];
            read(runtime, client, &mut byte);
            outgoing.feed(&byte).unwrap();
        }
    }
    assert_eq!(
        actual,
        vec![
            gielinor_protocol::interfaces::open_sub(
                runtime.interface_state(actor).unwrap().translate(parent).0,
                679,
                true
            ),
            arrow,
            hide,
            gielinor_protocol::varp::update(5002, 192),
            gielinor_protocol::varp::update(5002, 0),
            gielinor_protocol::interfaces::close_sub(
                runtime.interface_state(actor).unwrap().translate(parent).0
            ),
        ]
    );
}

fn exercise_creator_controls(
    runtime: &mut Runtime<Backend>,
    actor: gielinor_world::simulation::ActorId,
    client: &mut TcpStream,
    cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
    outgoing: &mut TickDecoder,
) {
    use gielinor_world::ui::{ButtonRoute, Component};
    let parent = Component::new(548, 23);
    runtime
        .open_interface(actor, Some(parent), 679, false)
        .unwrap();
    runtime
        .set_interface_events(
            actor,
            Component::new(679, 2),
            1,
            3,
            gielinor_world::ui::operation_mask(1).unwrap(),
        )
        .unwrap();
    let button = *gielinor_protocol::tables::CLIENT
        .iter()
        .find(|d| d.opcode == 47)
        .unwrap();
    for (component, child, expected) in [
        (679u32 << 16, -1, vec![(5000, 1), (5001, 1)]),
        (679u32 << 16, -1, vec![(5000, 1)]), // same body still resynchronizes UI
        ((679u32 << 16) | 2, 2, vec![(5001, 1)]), // repeated pronoun sets rebuild
        ((679u32 << 16) | 2, 3, vec![(5001, 2)]),
        ((679u32 << 16) | 1, -1, vec![(5000, 0), (5001, 0)]),
    ] {
        let mut payload = component.to_be_bytes().to_vec();
        payload.extend((child as u16).to_be_bytes());
        payload.extend(u16::MAX.to_be_bytes());
        payload.push(1);
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    button,
                    &payload,
                    cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let events = pump(runtime);
            assert!(!events
                .iter()
                .any(|e| matches!(e, Event::PacketRejected { .. })));
            if events.iter().any(|e| matches!(e, Event::InterfaceButton {button, route: ButtonRoute::Modal, ..} if button.component == component)) { break; }
            assert!(
                Instant::now() < deadline,
                "creator modal input not admitted"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        for &(id, value) in &expected {
            assert_eq!(runtime.world.data(actor).unwrap().variables.get(id), value);
        }
        // A diagnostic packet delimits writes from this admitted input; pending
        // earlier tick output is consumed but never mistaken for creator writes.
        let marker = gielinor_protocol::varp::update(65000, 239679);
        runtime
            .transport
            .send(
                runtime.connection(actor).unwrap(),
                marker.opcode,
                &marker.payload,
            )
            .unwrap();
        let mut actual = vec![];
        loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                if packet == marker {
                    break;
                }
                if expected
                    .iter()
                    .any(|&(id, value)| packet == gielinor_protocol::varp::update(id, value))
                {
                    actual.push(packet);
                }
            } else {
                let mut byte = [0];
                read(runtime, client, &mut byte);
                outgoing.feed(&byte).unwrap();
            }
        }
        assert_eq!(
            actual,
            expected
                .into_iter()
                .map(|(id, value)| gielinor_protocol::varp::update(id, value))
                .collect::<Vec<_>>()
        );
    }
    runtime.close_interface(actor, parent).unwrap();
}
fn exercise_prayer_tab(
    runtime: &mut Runtime<Backend>,
    actor: gielinor_world::simulation::ActorId,
    client: &mut TcpStream,
    cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
    outgoing: &mut TickDecoder,
) {
    use gielinor_world::{
        bank_actions::Binding,
        player_regen,
        prayer_book::{self, Prayer},
        prayer_drain, prayer_runtime,
        ui::Component,
        variables::{Transmit, Varbit, Varp},
    };
    let bit = |id, lsb, msb| Binding {
        base: Varp {
            id,
            permanent: false,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit { base: id, lsb, msb },
    };
    runtime.world.prayer_book = Some(prayer_runtime::Config {
        book: prayer_book::Config {
            prayer_skill: 5,
            defence_skill: 1,
            all_active: bit(4000, 0, 0),
            quick_active: bit(4001, 0, 0),
            drain_counter: bit(4002, 0, 30),
            disable_sound: 9,
            prayers: [(
                0,
                Prayer {
                    id: 0,
                    name: "Test prayer".into(),
                    level: 1,
                    sound: 10,
                    enabled: bit(4000, 0, 0),
                    overhead: Some(2),
                    unlock: None,
                    unlock_state: 1,
                    defence: None,
                    locked_message: None,
                    regen: None,
                },
            )]
            .into(),
            collisions: BTreeMap::new(),
            components: [(600 << 16, 0)].into(),
        },
        drain: prayer_drain::Config {
            enabled: bit(4000, 0, 0),
            counter: bit(4002, 0, 30),
            resistance: bit(4003, 0, 30),
            skill: 5,
            effects: vec![(0, 1)],
        },
        regeneration: player_regen::Config {
            skills: vec![3, 5],
            hitpoints: 3,
            prayer: 5,
            bracelet: -1,
            drained_timer: 2,
            boosted_timer: 3,
            health_timer: 4,
            rapid_timer: 5,
        },
        drain_timer: 7,
        depletion_sound: 11,
    });
    runtime.message_box = Some(gielinor_server::message_box::Presentation {
        parent: Component::new(548, 23),
        interface: 229,
        text: Component::new(229, 3),
        button: Component::new(229, 4),
    });
    runtime
        .world
        .data_mut(actor)
        .unwrap()
        .stats
        .set_current(5, 1);
    runtime
        .open_interface(actor, Some(Component::new(548, 22)), 600, true)
        .unwrap();
    let send = |client: &mut TcpStream,
                cipher: &mut Option<gielinor_protocol::isaac::Isaac>,
                opcode,
                payload: &[u8]| {
        let def = *gielinor_protocol::tables::CLIENT
            .iter()
            .find(|d| d.opcode == opcode)
            .unwrap();
        client
            .write_all(
                &gielinor_protocol::framing::encode(
                    def,
                    payload,
                    cipher,
                    gielinor_protocol::framing::OpcodeWidth::Byte,
                )
                .unwrap(),
            )
            .unwrap();
    };
    let mut button = (600u32 << 16).to_be_bytes().to_vec();
    button.extend(u16::MAX.to_be_bytes());
    button.extend(u16::MAX.to_be_bytes());
    button.push(1);
    for (enabled, sound) in [(1, 10), (0, 9)] {
        send(client, cipher, 47, &button);
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            pump(runtime);
            if runtime.world.data(actor).unwrap().variables.get(4000) == enabled {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "Prayer button did not reach world"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        let expected = gielinor_protocol::game::synth_sound(sound, 1, 0);
        let mut saw_variable = false;
        loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                saw_variable |= packet == gielinor_protocol::varp::update(4000, enabled);
                if packet == expected {
                    break;
                }
            } else {
                let mut byte = [0];
                read(runtime, client, &mut byte);
                outgoing.feed(&byte).unwrap();
            }
        }
        assert!(saw_variable, "Prayer sound preceded variable transmission");
    }
    runtime
        .world
        .prayer_book
        .as_mut()
        .unwrap()
        .book
        .prayers
        .get_mut(&0)
        .unwrap()
        .level = 99;
    send(client, cipher, 47, &button);
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        pump(runtime);
        if runtime.world.prayer_notice_token(actor).unwrap().is_some() {
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime
        .interface_state(actor)
        .unwrap()
        .modals
        .contains_interface(229));
    let end = gielinor_protocol::client_script::run(
        1508,
        &[gielinor_protocol::client_script::Argument::String(
            "0".into(),
        )],
    )
    .unwrap();
    let mut saw_text = false;
    loop {
        if let Some(packet) = outgoing.next_packet().unwrap() {
            saw_text |=
                packet.opcode == 80 && packet.payload.windows(13).any(|s| s == b"level of 99 t");
            if packet == end {
                break;
            }
        } else {
            let mut byte = [0];
            read(runtime, client, &mut byte);
            outgoing.feed(&byte).unwrap();
        }
    }
    assert!(saw_text, "Requirement notice text missing");
    let [a, b, c, d] = ((229u32 << 16) | 4).to_be_bytes();
    send(client, cipher, 115, &[b, a, d, c, 255, 255]);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.prayer_notice_token(actor).unwrap().is_some() {
        pump(runtime);
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(runtime
        .interface_state(actor)
        .unwrap()
        .modals
        .contains_interface(229));
    runtime.tick().unwrap();
    assert!(!runtime
        .interface_state(actor)
        .unwrap()
        .modals
        .contains_interface(229));
    assert!(!runtime.world.protection(actor).unwrap().suspended);
    // A delayed Continue loses protected access but still closes on the main pass.
    send(client, cipher, 47, &button);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.prayer_notice_token(actor).unwrap().is_none() {
        pump(runtime);
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    runtime.world.delay_player(actor, 2).unwrap();
    send(client, cipher, 115, &[b, a, d, c, 255, 255]);
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.world.prayer_notice_token(actor).unwrap().is_some() {
        pump(runtime);
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!runtime.world.protection(actor).unwrap().suspended);
    runtime.tick().unwrap();
    assert!(!runtime
        .interface_state(actor)
        .unwrap()
        .modals
        .contains_interface(229));
    runtime.tick().unwrap();
    runtime
        .close_interface(actor, Component::new(548, 22))
        .unwrap();
    // The processed clock trails the next input clock. Let the delay expire in
    // PlayerMain before the following banker input, which is otherwise rejected.
    runtime.tick().unwrap();
    assert!(!runtime.world.protection(actor).unwrap().delayed());
}
fn configure_autocast(runtime: &mut Runtime<Backend>) {
    use gielinor_world::{
        autocast, autocast_select, bank_actions::Binding, combat_wear, magic, runes, ui::Component,
        variables::Varbit,
    };
    let base = runtime.definitions.varps[&1000];
    let bit = |lsb, msb| Binding {
        base,
        bit: Varbit {
            base: 1000,
            lsb,
            msb,
        },
    };
    let spell = autocast::Spell {
        item: 7,
        requirements: vec![runes::Requirement {
            item: 8,
            count: 3,
            worn_slot: None,
        }],
    };
    let config = autocast::Config {
        enabled: bit(0, 0),
        spell: bit(1, 6),
        defensive: bit(7, 7),
        staff: autocast::Saved {
            spell: bit(8, 13),
            defensive: bit(14, 14),
        },
        bladed: autocast::Saved {
            spell: bit(15, 20),
            defensive: bit(21, 21),
        },
        weapons: [(
            8,
            autocast::Weapon {
                category: 18,
                additional: Default::default(),
            },
        )]
        .into(),
        spells: [(1, spell)].into(),
        autocast_items: [(1, 7)].into(),
        restricted: Default::default(),
    };
    runtime
        .configure_combat_wear(combat_wear::Config {
            autocast: config,
            runes: magic::Config {
                catalog: runes::Catalog::default(),
                fake: [bit(30, 30); 2],
                fountain: bit(30, 30),
                blighted: bit(30, 30),
                regular_pouches: Default::default(),
                divine_pouches: Default::default(),
                pouch_types: [bit(30, 30); 4],
                pouch_counts: [(2000, bit(30, 30)); 4],
                lms_pouch: 999,
                packs: BTreeMap::new(),
                names: magic::Names::default(),
            },
            level: bit(0, 7),
            decimal: bit(8, 15),
            combat_stats: [0, 1, 2, 3, 4, 6, 5],
        })
        .unwrap();
    runtime
        .configure_autocast_picker(
            gielinor_server::autocast_picker::Presentation {
                parent: Component::new(548, 22),
                interface: 201,
                combat_interface: 155,
                spells: Component(201 << 16),
                normal: Component::new(155, 5),
                defensive: Component::new(155, 6),
                layout: 1000,
            },
            autocast_select::Config {
                book: bit(22, 23),
                magic_stat: 6,
                spells: [(
                    1,
                    autocast_select::Access {
                        book: Some(0),
                        level: 50,
                    },
                )]
                .into(),
            },
        )
        .unwrap();
}
