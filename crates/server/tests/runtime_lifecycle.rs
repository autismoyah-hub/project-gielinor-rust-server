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
    fn next_packet(&mut self) -> Result<Option<gielinor_protocol::framing::Packet>, gielinor_protocol::framing::FrameError> {
        while let Some(packet) = self.inner.next_packet()? {
            if packet.opcode == 83 {
                assert!(packet.payload.is_empty());
                assert!(self.player_info, "Duplicate or premature server tick delimiter");
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
        assert!(Instant::now() < deadline);
        if offset < bytes.len() {
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    client.set_nonblocking(false).unwrap();
}
#[test]
fn tcp_login_world_admission_and_save_backpressure_keep_account_reserved() {
    exercise_lifecycle(false);
}
#[test]
fn configured_login_initializes_vars_camera_gameframe_menus_and_minimap_over_tcp() {
    exercise_lifecycle(true);
}
fn exercise_lifecycle(with_login: bool) {
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
    ]);
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
    runtime.configure_interfaces(components).unwrap();
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
    let account = accounts.load("parity_actor", 1).unwrap().unwrap();
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
    let mut accepted = [0; 36];
    read(&mut runtime, &mut client, &mut accepted);
    assert_eq!(&accepted[..2], &[2, 37]);
    let mut outgoing = TickDecoder { inner: gielinor_protocol::framing::Decoder::new(
        gielinor_protocol::tables::SERVER,
        Some(gielinor_protocol::isaac::Isaac::new(&[0x80000032, 289, 51, 49]).unwrap()),
        gielinor_protocol::framing::OpcodeWidth::Smart,
        65535,
        131072,
    ), tick_ends: 0, player_info: false };
    let mut inventory_ids = vec![];
    let initial_opcodes = if with_login {
        vec![
            49, 124, 75, 21, 73, 44, 97, 97, 97, 97, 97, 3, 114, 114, 96, 7, 108, 7, 114, 114, 7,
            108, 7, 114, 114, 114, 114, 114, 6, 6, 6, 6, 6, 114, 46, 31, 64, 92, 43, 22, 22, 47,
            116, 122, 28, 85, 47,
        ]
    } else {
        vec![49, 22, 22, 46, 31, 64, 47, 116, 122, 28, 85, 47]
    };
    for expected in initial_opcodes {
        let packet = loop {
            if let Some(packet) = outgoing.next_packet().unwrap() {
                break packet;
            }
            let mut byte = [0];
            read(&mut runtime, &mut client, &mut byte);
            outgoing.feed(&byte).unwrap();
        };
        assert_eq!(packet.opcode, expected);
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
        } else if expected == 22 {
            let id = u16::from_be_bytes([packet.payload[4], packet.payload[5]]);
            inventory_ids.push(id);
            assert_eq!(&packet.payload[..4], &(-(1234 + id as i32)).to_be_bytes());
            assert_eq!(&packet.payload[6..], &[0, 0]);
        } else if expected == 46 {
            assert_eq!(&packet.payload[..3], &[10, 10, 131]);
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
    // The initial viewport resets all 49 neighbouring zones in x-major order.
    for x in (24..=72).step_by(8) {
        for z in (24..=72).step_by(8) {
            let packet = loop {
                if let Some(packet) = outgoing.next_packet().unwrap() {
                    break packet;
                }
                let mut byte = [0];
                read(&mut runtime, &mut client, &mut byte);
                outgoing.feed(&byte).unwrap();
            };
            assert_eq!(packet, gielinor_protocol::zone::reset(x, z, 0));
        }
    }
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
            if cycle == 0 {
                assert!(!opcodes.contains(&96));
                assert_eq!(runtime.interface_state(actor).unwrap().top_level, Some(548));
            } else {
                assert_eq!(
                    &opcodes[..17],
                    &[97, 97, 97, 96, 123, 114, 114, 123, 7, 96, 123, 114, 114, 123, 7, 7, 43]
                );
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
    for expected in [18, 47, 116, 122, 28, 85, 47] {
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
        for &opcode in expected {
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
    for opcode in [18, 47, 116, 122, 28, 47] {
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
        for opcode in [18, 18, 47, 116, 122, 28, 47] {
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
    let mut bank_open_opcodes = vec![97, 97, 7, 7];
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
        if (4..23).contains(&index) {
            assert_eq!(packet, expected_bonus[index - 4]);
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
        for &opcode in expected {
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
        for opcode in [97, 47, 116, 122, 28, 47] {
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
        for opcode in [114, 47, 116, 122, 28, 47] {
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
        for opcode in [97, 97, 18, 18, 47, 116, 122, 28, 47] {
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
    for opcode in [114, 47, 116, 122, 28, 47] {
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
        if cycle == 0 {
            assert_eq!(
                packets[..3].iter().map(|p| p.opcode).collect::<Vec<_>>(),
                [64, 74, 74]
            );
            assert_eq!(
                packets[0].payload,
                gielinor_protocol::game::run_energy(10000).unwrap()
            );
            assert_eq!(
                packets[1],
                gielinor_protocol::chat::game(0, None, "You drink some of your super energy.")
                    .unwrap()
            );
            assert_eq!(
                packets[2],
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
    // Encrypted OpLocV2 must reach the authoritative door action and observer zone stream.
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
    for _ in 0..2 {
        runtime.tick().unwrap();
    }
    assert!(runtime.world.data(actor).is_ok());
    assert!(!runtime.is_drained());
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
