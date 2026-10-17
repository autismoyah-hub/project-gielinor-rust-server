//! Bounded network/account/world lifecycle. Gameplay dispatch and synchronization consume events.
use crate::{
    account_worker::{Completion, Request, Worker},
    characters::{Character, Policy},
    transport::{NetworkEvent, Transport},
};
use gielinor_cache::game::Definitions;
use gielinor_protocol::framing::Packet;
use gielinor_storage::accounts::Save;
use gielinor_world::simulation::{ActorId, AdmissionError, PlayerData, World, WorldEvent};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    sync::mpsc::{TryRecvError, TrySendError},
};
mod anvil;
mod autocast_picker;
mod dialogue;
mod drop_dialog;
mod location_target;
mod mining;
mod movement;
mod npc_interaction;
mod prayer;
mod prayer_tab;
mod shop;
mod skill_multi;
mod dough;
mod finished_potions;
mod unfinished_potions;
mod herb_cleaning;
mod smelting;
mod tower_combat;
mod tower_restore;
mod tutorial;
mod woodcutting;

fn inventory<'a>(
    data: &'a PlayerData,
    policy: &Policy,
    id: u16,
) -> Option<&'a gielinor_world::inventory::Inventory> {
    let id = id as u32;
    if id == policy.inventory {
        Some(&data.inventory)
    } else if id == policy.equipment {
        Some(&data.equipment)
    } else if id == policy.bank {
        Some(&data.bank)
    } else {
        data.other_inventories.get(&(id as i32))
    }
}
fn inventory_mut<'a>(
    data: &'a mut PlayerData,
    policy: &Policy,
    id: u16,
) -> Option<&'a mut gielinor_world::inventory::Inventory> {
    let id = id as u32;
    if id == policy.inventory {
        Some(&mut data.inventory)
    } else if id == policy.equipment {
        Some(&mut data.equipment)
    } else if id == policy.bank {
        Some(&mut data.bank)
    } else {
        data.other_inventories.get_mut(&(id as i32))
    }
}
fn inventory_weight(
    tx: &crate::inventory_sync::Transmissions,
    data: &PlayerData,
    policy: &Policy,
    defs: &Definitions,
) -> io::Result<i32> {
    tx.weight(
        |id| {
            Some((
                inventory(data, policy, id)?,
                defs.inventories.get(&(id as u32))?.flags & 8 != 0,
            ))
        },
        |id| defs.items.get(&(id as u32)).map(|ty| ty.weight as i32),
    )
    .map_err(|e| io::Error::other(format!("{e:?}")))
}
pub trait AccountBackend {
    fn submit(&self, request: Request) -> Result<(), TrySendError<Request>>;
    fn poll(&self) -> Result<Completion, TryRecvError>;
}
impl AccountBackend for Worker {
    fn submit(&self, request: Request) -> Result<(), TrySendError<Request>> {
        Worker::submit(self, request)
    }
    fn poll(&self) -> Result<Completion, TryRecvError> {
        Worker::poll(self)
    }
}
pub enum Event {
    SkillMultiSelected {
        actor: ActorId,
        selection: Option<crate::skill_multi::Selection>,
    },
    CountDialogInput {
        actor: ActorId,
        count: i32,
        accepted: bool,
    },
    InterfaceButton {
        actor: ActorId,
        button: gielinor_protocol::buttons::Button,
        route: gielinor_world::ui::ButtonRoute,
    },
    InterfaceDrag {
        actor: ActorId,
        drag: gielinor_protocol::buttons::Drag,
        route: gielinor_world::ui::ButtonRoute,
    },
    InterfaceTarget {
        actor: ActorId,
        target: gielinor_protocol::buttons::Target,
        route: gielinor_world::ui::ButtonRoute,
    },
    InterfaceClosed {
        actor: ActorId,
        parent: gielinor_world::ui::Component,
        interface: u16,
    },
    InterfaceOpened {
        actor: ActorId,
        parent: Option<gielinor_world::ui::Component>,
        interface: u16,
        overlay: bool,
    },
    PlayerReady {
        connection: u64,
        actor: ActorId,
        new_account: bool,
    },
    Packet {
        connection: u64,
        actor: ActorId,
        packet: Packet,
    },
    World(WorldEvent),
    SaveFailed {
        actor: ActorId,
        reason: String,
    },
    /// Worker success acknowledgement. The tick records observation in the
    /// runtime, not the wall-clock instant of the SQLite commit.
    SaveCompleted {
        actor: ActorId,
        generation: u64,
        observed_tick: i32,
    },
    PacketRejected {
        connection: u64,
        actor: ActorId,
        opcode: u16,
        reason: String,
    },
    SynchronizationFailed {
        actor: ActorId,
        reason: String,
    },
}
/// Root-world player/NPC synchronization. Zone/UI and dynamic worlds require their
/// own presentation state and are not synthesized by this adapter.
pub struct Presentation {
    pub animations: crate::appearance::AnimationPolicy,
    /// Attack, defence, strength, hitpoints, ranged, magic, prayer.
    pub combat_stats: [u8; 7],
    pub radius: i32,
}
#[derive(Clone)]
pub struct BankPresentation {
    pub main_interface: u16,
    pub side_interface: u16,
    pub main_components: Vec<u32>,
    pub side_component: u32,
    pub default_capacity: i32,
}
impl Presentation {
    fn combat_level(&self, data: &PlayerData) -> i32 {
        let [a, d, s, h, r, m, p] = self.combat_stats.map(|id| data.stats.base(id) as u8 as i32);
        gielinor_world::skills::combat_level(a, d, s, h, r, m, p)
    }
    fn appearance(
        &self,
        design: &crate::appearance::Design,
        data: &PlayerData,
        defs: &Definitions,
    ) -> Result<Vec<u8>, String> {
        let [a, d, s, h, r, m, p] = self.combat_stats.map(|id| data.stats.base(id) as u8 as i32);
        let combat = u8::try_from(gielinor_world::skills::combat_level(a, d, s, h, r, m, p))
            .map_err(|_| "Combat level exceeds protocol range")?;
        crate::appearance::build(
            design,
            &data.display_name,
            combat,
            &data.equipment,
            defs,
            &self.animations,
        )
        .map_err(str::to_owned)?
        .encode()
        .map_err(|e| format!("{e:?}"))
    }
}
enum ConnectionState {
    Authenticating { resized: bool },
    Queued(Character, bool),
    Playing(ActorId),
}
struct Active {
    login_messages: Vec<&'static str>,
    face_npc: Option<gielinor_world::simulation::NpcId>,
    drop_dialog: Option<(u64, gielinor_world::ui::Component)>,
    prayer_dialog: Option<(u64, gielinor_world::ui::Component)>,
    prayer_notice: Option<crate::message_box::Active>,
    dialogue: Option<crate::dialogue::Active>,
    skill_multi: Option<crate::skill_multi::Active>,
    anvil_count: Option<crate::anvil::CountDialog>,
    pending_face: Option<(gielinor_world::CoordGrid, u8, u8)>,
    face_angle: Option<u16>,
    face_changed: bool,
    bank_events_due: Option<i32>,
    sequence: Option<(u16, u8)>,
    spot_animations: Vec<gielinor_protocol::spot_animation::Animation>,
    ui: gielinor_world::ui::State,
    character: Character,
    connection: Option<u64>,
    design: crate::appearance::Design,
    appearance_rebuild: bool,
    observer: Option<crate::synchronization::Observer>,
    npc_observer: crate::npc_synchronization::Observer,
    zone_observer: gielinor_world::zones::Observer,
    tower_scene: crate::tower_scene::Observer,
    inventories: crate::inventory_sync::Transmissions,
    shop_inventory: Option<shop::Transmission>,
    stats: crate::stat_sync::Observer,
    invisible_levels: BTreeMap<u8, i32>,
    run: Option<(i32, i32)>,
}
struct PendingSave {
    actor: ActorId,
    data: Save,
    inflight: bool,
    retry_at: i32,
    checkpoint: bool,
}
pub struct Runtime<B: AccountBackend> {
    pub constructed: crate::constructed::Regions,
    pub floor_progression: Option<gielinor_world::floor_progression::Policy>,
    pub combat_display: Option<crate::combat_display::Config>,
    combat_updates: HashMap<gielinor_world::hits::Target, crate::combat_display::Display>,
    drop_presentation: Option<crate::drop_dialog::Presentation>,
    dialogue_presentation: Option<crate::dialogue::Presentation>,
    smelting: Option<crate::smelting::Config>,
    dough_presentation: Option<crate::skill_multi::Presentation>,
    anvil: Option<crate::anvil::Config>,
    pub message_box: Option<crate::message_box::Presentation>,
    zones_enabled: bool,
    ground_updates: Vec<gielinor_world::zones::Change>,
    bank_insert: Option<gielinor_world::bank_actions::Binding>,
    bank_controls: BTreeMap<u32, gielinor_world::bank_actions::Control>,
    bank_presentation: Option<BankPresentation>,
    pub music_component: Option<u32>,
    npc_bank_parents: Option<(gielinor_world::ui::Component, gielinor_world::ui::Component)>,
    npc_spell_buttons: BTreeMap<u32, i32>,
    login_presentation: Option<crate::login_init::Presentation>,
    interface_hooks: crate::interface_hooks::Hooks,
    journal: Option<crate::journal::Journal>,
    gameframe_moves: Option<crate::gameframe_move::Config>,
    combat_tab: Option<crate::combat_tab::Presentation>,
    combat_buttons: BTreeMap<u32, Option<u8>>,
    player_design: Option<crate::player_design::Bindings>,
    tutorial: Option<std::sync::Arc<crate::tutorial_character::Config>>,
    tutorial_leave: Option<gielinor_world::tutorial_leave::Config>,
    autocast_picker: Option<crate::autocast_picker::Presentation>,
    interface_controls: BTreeMap<u32, crate::interface_controls::Control>,
    bonuses: Option<(crate::bonuses::Config, crate::bonuses::Styles)>,
    bank_bonus_display: Option<crate::bonuses::BankDisplay>,
    worn_components: BTreeMap<u32, usize>,
    equipment_message_params: Option<[u32; 2]>,
    held_component: Option<u32>,
    components: BTreeMap<u32, gielinor_cache::components::ComponentType>,
    pub world: World,
    pub transport: Transport,
    pub definitions: Definitions,
    pub policy: Policy,
    backend: B,
    connections: BTreeMap<u64, ConnectionState>,
    active: HashMap<ActorId, Active>,
    saves: BTreeMap<u64, PendingSave>,
    next_save_token: u64,
    events: Vec<Event>,
    shutting_down: bool,
    presentation: Option<Presentation>,
    npc_snapshots: Vec<gielinor_world::simulation::NpcSnapshot>,
}
impl<B: AccountBackend> Runtime<B> {
    pub fn new(
        mut world: World,
        transport: Transport,
        definitions: Definitions,
        policy: Policy,
        backend: B,
    ) -> Self {
        world.object_types = definitions.inventory_catalog();
        world.item_weights = definitions
            .items
            .iter()
            .map(|(&id, ty)| (id as i32, ty.weight as i32))
            .collect();
        Self {
            combat_display: None,
            constructed: Default::default(),
            combat_updates: HashMap::new(),
            zones_enabled: false,
            drop_presentation: None,
            dialogue_presentation: None,
            smelting: None,
            dough_presentation: None,
            anvil: None,
            message_box: None,
            ground_updates: Vec::new(),
            bank_insert: None,
            bank_controls: BTreeMap::new(),
            bank_presentation: None,
            music_component: None,
            npc_bank_parents: None,
            npc_spell_buttons: BTreeMap::new(),
            login_presentation: None,
            floor_progression: None,
            interface_hooks: crate::interface_hooks::Hooks::default(),
            journal: None,
            gameframe_moves: None,
            combat_tab: None,
            combat_buttons: BTreeMap::new(),
            player_design: None,
            tutorial: None,
            tutorial_leave: None,
            autocast_picker: None,
            interface_controls: BTreeMap::new(),
            bonuses: None,
            bank_bonus_display: None,
            worn_components: BTreeMap::new(),
            equipment_message_params: None,
            held_component: None,
            components: BTreeMap::new(),
            world,
            transport,
            definitions,
            policy,
            backend,
            connections: BTreeMap::new(),
            active: HashMap::new(),
            saves: BTreeMap::new(),
            next_save_token: 0,
            events: vec![],
            shutting_down: false,
            presentation: None,
            npc_snapshots: vec![],
        }
    }
    /// Attach the account worker after startup validation, before any admission.
    pub fn configure_backend(&mut self, backend: B) -> io::Result<()> {
        if !self.connections.is_empty()
            || !self.active.is_empty()
            || !self.saves.is_empty()
            || self.transport.connection_count() != 0
        {
            return Err(io::Error::other(
                "Configure account backend before admission",
            ));
        }
        self.backend = backend;
        Ok(())
    }
    pub fn enable_player_synchronization(&mut self, presentation: Presentation) -> io::Result<()> {
        if !self.connections.is_empty()
            || !self.active.is_empty()
            || self.world.actors().next().is_some()
        {
            return Err(io::Error::other(
                "Configure synchronization before admission",
            ));
        }
        if !(1..=127).contains(&presentation.radius) {
            return Err(io::Error::other("Invalid synchronization radius"));
        }
        if [
            self.policy.inventory,
            self.policy.equipment,
            self.policy.bank,
        ]
        .into_iter()
        .any(|id| id > u16::MAX as u32)
        {
            return Err(io::Error::other("Inventory type exceeds protocol range"));
        }
        self.presentation = Some(presentation);
        Ok(())
    }
    pub fn enable_ground_synchronization(&mut self) -> io::Result<()> {
        if self.presentation.is_none() || !self.connections.is_empty() || !self.active.is_empty() {
            return Err(io::Error::other(
                "Configure ground synchronization after presentation and before admission",
            ));
        }
        self.zones_enabled = true;
        Ok(())
    }
    pub fn configure_foods(
        &mut self,
        component: u32,
        foods: BTreeMap<i32, gielinor_world::food::Edible>,
        animation: u16,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || !self.active.is_empty() {
            return Err(io::Error::other("Configure food handlers before admission"));
        }
        if !self.components.contains_key(&component) {
            return Err(io::Error::other("Unknown held-inventory component"));
        }
        for (&id, food) in &foods {
            if food.id != id
                || food.heal < 0
                || food.delay <= 0
                || self
                    .definitions
                    .items
                    .get(&(id as u32))
                    .is_none_or(|ty| ty.inventory_ops[0].is_none())
                || food
                    .replacement
                    .is_some_and(|id| !self.definitions.items.contains_key(&(id as u32)))
            {
                return Err(io::Error::other(
                    "Invalid food definition or missing held op 1",
                ));
            }
            gielinor_protocol::chat::game(0, None, &food.message)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        self.held_component = Some(component);
        self.world.foods = foods;
        self.world.slot_preserving_foods.clear();
        self.world.eating_animation = animation;
        Ok(())
    }
    pub fn configure_potions(
        &mut self,
        objects: &gielinor_cache::symbols::Symbols,
        stats: &gielinor_cache::symbols::Symbols,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || !self.active.is_empty() {
            return Err(io::Error::other("Configure potions before admission"));
        }
        let potions =
            crate::potions::resolve(objects, stats, &self.definitions).map_err(io::Error::other)?;
        for &id in potions.keys() {
            // Kotlin registers both held handlers even when a cache item lacks
            // an operation label. Labels control the visible client menu.
            if self.world.foods.contains_key(&id) {
                return Err(io::Error::other("Potion has conflicting held handlers"));
            }
        }
        self.world.potions = potions;
        Ok(())
    }
    pub fn configure_equipment(
        &mut self,
        held_component: u32,
        worn_components: BTreeMap<u32, usize>,
        param_symbols: &gielinor_cache::symbols::Symbols,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || !self.active.is_empty() {
            return Err(io::Error::other("Configure equipment before admission"));
        }
        if !self.components.contains_key(&held_component)
            || self.held_component.is_some_and(|id| id != held_component)
            || worn_components
                .iter()
                .any(|(id, slot)| !self.components.contains_key(id) || *slot >= 14)
        {
            return Err(io::Error::other("Invalid equipment interface mapping"));
        }
        let messages = ["statreq_failmessage1", "statreq_failmessage2"].map(|name| {
            param_symbols
                .get(name)
                .copied()
                .ok_or_else(|| io::Error::other(format!("Missing {name}")))
        });
        let [first, second] = messages;
        let params = [first? as u32, second? as u32];
        let catalog = crate::content::equipment(param_symbols, &self.definitions)
            .map_err(io::Error::other)?;
        self.world.wear_types = catalog;
        self.held_component = Some(held_component);
        self.worn_components = worn_components;
        self.equipment_message_params = Some(params);
        Ok(())
    }
    pub fn configure_interface_hooks(
        &mut self,
        hooks: crate::interface_hooks::Hooks,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure interface hooks before admission",
            ));
        }
        for (&interface, actions) in &hooks.opens {
            for action in actions {
                if let crate::interface_hooks::Action::Events { component, .. } = action {
                    if component.interface() != interface
                        || !self.components.contains_key(&component.0)
                    {
                        return Err(io::Error::other(
                            "Unknown listener component or mismatched interface",
                        ));
                    }
                }
            }
            hooks
                .opened(interface, &mut gielinor_world::ui::State::default())
                .map_err(io::Error::other)?;
        }
        self.interface_hooks = hooks;
        Ok(())
    }
    pub fn configure_interface_controls(
        &mut self,
        controls: BTreeMap<u32, crate::interface_controls::Control>,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure interface controls before admission",
            ));
        }
        if controls.keys().any(|id| !self.components.contains_key(id)) {
            return Err(io::Error::other("Unknown control component"));
        }
        self.interface_controls = controls;
        Ok(())
    }
    /// Content is configured after the generic startup UI. Enable its journal
    /// in either configuration order, before any player can be admitted.
    pub fn configure_cinderworks(&mut self, config: gielinor_world::tower_cinderworks::Config) -> io::Result<()> {
        if !self.connections.is_empty() { return Err(io::Error::other("Configure Cinderworks before admission")); }
        self.world.cinderworks = Some(config);
        if let Some(journal) = &mut self.journal { journal.cinderworks = true; }
        Ok(())
    }
    pub fn configure_journal(&mut self, mut journal: crate::journal::Journal) -> io::Result<()> {
        if !self.connections.is_empty() || self.presentation.is_none() {
            return Err(io::Error::other(
                "Configure journal after presentation and before admission",
            ));
        }
        if [journal.parent, journal.contents, journal.click_layer]
            .iter()
            .any(|c| !self.components.contains_key(&c.0))
            || journal
                .tabs
                .iter()
                .chain(std::iter::once(&journal.interface))
                .any(|id| !self.components.keys().any(|c| *c >> 16 == *id as u32))
        {
            return Err(io::Error::other("Unknown journal interface or component"));
        }
        if journal.switching.as_ref().is_some_and(|config| {
            config
                .buttons
                .iter()
                .any(|(c, tab)| !self.components.contains_key(c) || *tab >= 3)
        }) {
            return Err(io::Error::other("Invalid journal control"));
        }
        if journal.diary.as_ref().is_some_and(|diary| {
            diary
                .hidden
                .iter()
                .chain(std::iter::once(&diary.header))
                .any(|c| !self.components.contains_key(c) || *c >> 16 != journal.tabs[2] as u32)
                || diary
                    .skills
                    .iter()
                    .any(|s| !self.definitions.stats.contains_key(&(*s as u32)))
        }) {
            return Err(io::Error::other("Invalid Tower diary configuration"));
        }
        journal.cinderworks = self.world.cinderworks.is_some();
        self.journal = Some(journal);
        Ok(())
    }
    pub fn configure_combat_tab(&mut self, config: crate::combat_tab::Config) -> io::Result<()> {
        if !self.connections.is_empty() || self.presentation.is_none() {
            return Err(io::Error::other(
                "Configure combat after presentation and before admission",
            ));
        }
        if !self
            .components
            .keys()
            .any(|id| *id >> 16 == config.presentation.interface as u32)
            || config.buttons.iter().any(|(id, stance)| {
                !self.components.contains_key(id) || stance.is_some_and(|v| v > 3)
            })
        {
            return Err(io::Error::other("Invalid combat tab components"));
        }
        self.world.combat_stance = Some(config.stance);
        self.combat_buttons = config.buttons;
        self.combat_tab = Some(config.presentation);
        Ok(())
    }
    pub fn configure_combat_wear(
        &mut self,
        config: gielinor_world::combat_wear::Config,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || self.world.combat_stance.is_none() {
            return Err(io::Error::other(
                "Configure equipment listeners after combat and before admission",
            ));
        }
        self.world.combat_wear = Some(config);
        Ok(())
    }
    pub fn configure_bulwark_wear(
        &mut self,
        config: gielinor_world::bulwark_wear::Config,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure bulwark listener before admission",
            ));
        }
        self.world.bulwark_wear = Some(config);
        Ok(())
    }
    pub fn configure_wearpos(&mut self, config: gielinor_world::wearpos::Config) -> io::Result<()> {
        if !self.connections.is_empty() || self.world.combat_stance.is_none() {
            return Err(io::Error::other(
                "Configure wear-position listeners after combat and before admission",
            ));
        }
        self.world.wearpos = Some(config);
        Ok(())
    }
    pub fn configure_gameframe_moves(
        &mut self,
        config: crate::gameframe_move::Config,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure gameframe moves before admission",
            ));
        }
        let login = self
            .login_presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure login before gameframe moves"))?;
        if !self.components.contains_key(&config.button)
            || config
                .events
                .values()
                .any(|c| !self.components.contains_key(&c.0))
            || config
                .choices
                .iter()
                .any(|top| !login.frames.ordered.iter().any(|f| f.top == *top))
            || login
                .frames
                .ordered
                .iter()
                .any(|f| config.targets.iter().any(|c| !f.mappings.contains_key(c)))
        {
            return Err(io::Error::other(
                "Invalid gameframe move mappings or controls",
            ));
        }
        self.gameframe_moves = Some(config);
        Ok(())
    }
    pub fn move_gameframe(&mut self, actor: ActorId, top: u16) -> io::Result<()> {
        if self.gameframe_moves.is_none() {
            return Err(io::Error::other("Gameframe moves are not configured"));
        }
        let login = self
            .login_presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Login is not configured"))?;
        let active = self
            .active
            .get(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        let connection = active
            .connection
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        let mut ui = active.ui.clone();
        let mut vars = self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables
            .clone();
        let logged_in = self
            .world
            .protection(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .processed_clock
            > 0;
        let request = login
            .frames
            .prepare_move(top, &mut vars, &mut ui, logged_in)
            .map_err(io::Error::other)?;
        let mut packets = Vec::new();
        if let Some(mode) = request.client_mode {
            packets.push(
                gielinor_protocol::client_script::run(
                    3998,
                    &[gielinor_protocol::client_script::Argument::Int(mode)],
                )
                .map_err(|e| io::Error::other(format!("{e:?}")))?,
            );
        }
        packets.extend(
            request
                .variables
                .into_iter()
                .map(|v| gielinor_protocol::varp::update(v.id, v.value)),
        );
        self.world
            .queue_gameframe_move(actor, request.delay, request.movement)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        self.world.data_mut(actor).unwrap().variables = vars;
        self.active.get_mut(&actor).unwrap().ui = ui;
        for packet in packets {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub fn configure_login(
        &mut self,
        presentation: crate::login_init::Presentation,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure login before admission"));
        }
        presentation.frames.validate().map_err(io::Error::other)?;
        for frame in &presentation.frames.ordered {
            if frame
                .overlays
                .iter()
                .any(|(parent, _)| !self.components.contains_key(&parent.0))
                || frame
                    .mappings
                    .values()
                    .any(|parent| !self.components.contains_key(&parent.0))
            {
                return Err(io::Error::other("Unknown gameframe component"));
            }
        }
        self.login_presentation = Some(presentation);
        Ok(())
    }
    pub fn configure_realm_messages(
        &mut self,
        messages: gielinor_storage::accounts::LoginMessages,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure realm messages before admission",
            ));
        }
        let login = self
            .login_presentation
            .as_mut()
            .ok_or_else(|| io::Error::other("Login presentation is not configured"))?;
        // Validate the actual outbound encoding before accepting any sessions.
        for (kind, message) in [(108, &messages.welcome), (14, &messages.broadcast)] {
            if let Some(message) = message {
                gielinor_protocol::chat::game(kind, None, message)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
            }
        }
        login.welcome = messages.welcome;
        login.broadcast = messages.broadcast;
        Ok(())
    }
    pub fn configure_bonuses(
        &mut self,
        config: crate::bonuses::Config,
        styles: crate::bonuses::Styles,
        bank_display: Option<crate::bonuses::BankDisplay>,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure bonuses before admission"));
        }
        if let Some(display) = &bank_display {
            if display
                .components
                .iter()
                .chain(std::iter::once(&display.tooltip))
                .any(|id| !self.components.contains_key(id))
            {
                return Err(io::Error::other("Unknown bank bonus component"));
            }
        }
        self.bonuses = Some((config, styles));
        self.bank_bonus_display = bank_display;
        Ok(())
    }
    pub fn equipment_bonuses(
        &self,
        actor: ActorId,
    ) -> io::Result<gielinor_world::bonuses::Bonuses> {
        let (config, _) = self
            .bonuses
            .as_ref()
            .ok_or_else(|| io::Error::other("Bonuses not configured"))?;
        let data = self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        Ok(config.calculate(data))
    }
    pub fn refresh_bank_bonuses(&mut self, actor: ActorId) -> io::Result<()> {
        let (Some((config, styles)), Some(display), Some(connection)) = (
            &self.bonuses,
            &self.bank_bonus_display,
            self.connection(actor),
        ) else {
            return Ok(());
        };
        let data = self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let base = config
            .base_speed(&data.equipment)
            .map_err(io::Error::other)?;
        let category = data
            .equipment
            .get(3)
            .and_then(|item| self.definitions.items.get(&(item.id as u32)))
            .map(|ty| ty.weapon_category);
        let style = styles
            .get(category, data.variables.get(config.attack_style_varp))
            .map_err(io::Error::other)?;
        let actual = gielinor_world::bonuses::attack_speed(base, style == Some(6));
        let packets = display
            .packets(&config.calculate(data), base, actual)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        for packet in packets {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub fn configure_bank_controls(
        &mut self,
        component_symbols: &gielinor_cache::symbols::Symbols,
        varbit_symbols: &gielinor_cache::symbols::Symbols,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure bank controls before admission"));
        }
        let bank = self
            .bank_presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure banking first"))?;
        let controls =
            crate::content::bank_controls(component_symbols, varbit_symbols, &self.definitions)
                .map_err(io::Error::other)?;
        if controls
            .keys()
            .any(|id| *id >> 16 != bank.main_interface as u32 || !self.components.contains_key(id))
        {
            return Err(io::Error::other("Invalid bank control component"));
        }
        self.bank_controls = controls;
        let insert = *component_symbols.get("bankmain:insert").unwrap() as u32;
        if let gielinor_world::bank_actions::Control::Set { binding, .. } =
            self.bank_controls[&insert]
        {
            self.bank_insert = Some(binding);
        }
        Ok(())
    }
    pub fn configure_bank_sidebar(
        &mut self,
        param_symbols: &gielinor_cache::symbols::Symbols,
        varp_symbols: &gielinor_cache::symbols::Symbols,
        content_symbols: &gielinor_cache::symbols::Symbols,
        parameter_types: &BTreeMap<u32, gielinor_cache::parameters::ParameterType>,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure bank sidebar before admission"));
        }
        let sidebar = crate::content::bank_sidebar(
            param_symbols,
            varp_symbols,
            content_symbols,
            parameter_types,
            &self.definitions,
        )
        .map_err(io::Error::other)?;
        self.world
            .banking
            .as_mut()
            .ok_or_else(|| io::Error::other("Configure banking first"))?
            .sidebar = Some(sidebar);
        Ok(())
    }
    pub fn configure_banking(
        &mut self,
        presentation: BankPresentation,
        varbit_symbols: &gielinor_cache::symbols::Symbols,
        param_symbols: &gielinor_cache::symbols::Symbols,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || !self.active.is_empty() {
            return Err(io::Error::other("Configure banking before admission"));
        }
        if presentation.main_components.is_empty()
            || presentation.default_capacity <= 0
            || self
                .definitions
                .inventories
                .get(&self.policy.bank)
                .is_none_or(|ty| presentation.default_capacity as usize > ty.size)
            || presentation.main_components.iter().any(|id| {
                !self.components.contains_key(id) || id >> 16 != presentation.main_interface as u32
            })
            || !self.components.contains_key(&presentation.side_component)
            || presentation.side_component >> 16 != presentation.side_interface as u32
        {
            return Err(io::Error::other("Invalid bank interface/capacity mapping"));
        }
        let config = crate::content::banking(varbit_symbols, param_symbols, &self.definitions)
            .map_err(io::Error::other)?;
        self.world.banking = Some(config);
        self.bank_presentation = Some(presentation);
        Ok(())
    }
    pub fn open_bank(
        &mut self,
        actor: ActorId,
        main_parent: gielinor_world::ui::Component,
        side_parent: gielinor_world::ui::Component,
    ) -> io::Result<()> {
        let bank = self
            .bank_presentation
            .clone()
            .ok_or_else(|| io::Error::other("Banking not configured"))?;
        let config = self.world.banking.as_ref().unwrap();
        let capacity = config.capacity;
        let notes = config.notes;
        let sidebar = config.sidebar.clone();
        if capacity
            .get(
                &self
                    .world
                    .data(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                    .variables,
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            == 0
        {
            let vars = &mut self
                .world
                .data_mut(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .variables;
            if let Some(update) = capacity
                .set(vars, bank.default_capacity)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
            {
                if let Some(connection) = self.connection(actor) {
                    let packet = gielinor_protocol::varp::update(update.id, update.value);
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
        }
        let value = notes
            .bit
            .replace(
                self.world.data(actor).unwrap().variables.get(notes.base.id),
                0,
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        self.set_varp(actor, notes.base.id as u32, value)?;
        if let Some(sidebar) = sidebar {
            let data = self
                .world
                .data_mut(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            let updates = sidebar
                .refresh(&data.inventory, &mut data.variables)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            if let Some(connection) = self.connection(actor) {
                for update in updates {
                    let packet = gielinor_protocol::varp::update(update.id, update.value);
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
        }
        self.open_interface(actor, Some(main_parent), bank.main_interface, false)?;
        self.open_interface(actor, Some(side_parent), bank.side_interface, false)?;
        self.refresh_bank_bonuses(actor)?;
        self.start_inventory_transmission(actor, self.policy.bank as u16)?;
        self.bank_item_events(actor)?;
        self.active.get_mut(&actor).unwrap().bank_events_due = Some(self.world.clock.deadline(2));
        Ok(())
    }
    fn bank_item_events(&mut self, actor: ActorId) -> io::Result<()> {
        let bank = self
            .bank_presentation
            .clone()
            .ok_or_else(|| io::Error::other("Banking not configured"))?;
        let data = self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let bank_last = data.bank.slots().len().saturating_sub(1) as i32;
        let last = bank_last.max(1409);
        let inv_last = data.inventory.slots().len().saturating_sub(1) as i32;
        let main_mask = ((1u64 << 11) - 1) << 32 | 2046 | (2 << 17) | (1 << 20);
        if let Some(&component) = bank.main_components.first() {
            self.set_interface_events(
                actor,
                gielinor_world::ui::Component(component),
                bank_last + 10,
                bank_last + 18,
                1 << 32,
            )?;
            self.set_interface_events(
                actor,
                gielinor_world::ui::Component(component),
                bank_last + 19,
                bank_last + 28,
                1 << 20,
            )?;
        }
        for component in bank.main_components {
            self.set_interface_events(
                actor,
                gielinor_world::ui::Component(component),
                0,
                last,
                main_mask,
            )?;
        }
        let tabs = self.bank_controls.iter().find_map(|(&id, control)| {
            matches!(control, gielinor_world::bank_actions::Control::Tabs).then_some(id)
        });
        if let Some(tabs) = tabs {
            self.set_interface_events(
                actor,
                gielinor_world::ui::Component(tabs),
                10,
                10,
                (1 << 32) | (1 << 38) | (1 << 20),
            )?;
            self.set_interface_events(
                actor,
                gielinor_world::ui::Component(tabs),
                11,
                19,
                (1 << 32) | (1 << 37) | (1 << 38) | (1 << 17) | (1 << 20),
            )?;
        }
        self.set_interface_events(
            actor,
            gielinor_world::ui::Component(bank.side_component),
            0,
            inv_last,
            ((1u64 << 10) - 1) << 32 | (1 << 17) | (1 << 20),
        )
    }
    pub fn configure_interfaces(
        &mut self,
        components: BTreeMap<u32, gielinor_cache::components::ComponentType>,
    ) -> io::Result<()> {
        if !self.connections.is_empty() || !self.active.is_empty() {
            return Err(io::Error::other("Configure interfaces before admission"));
        }
        self.components = components;
        Ok(())
    }
    pub fn interface_state(&self, actor: ActorId) -> Option<&gielinor_world::ui::State> {
        self.active.get(&actor).map(|active| &active.ui)
    }
    pub fn open_interface(
        &mut self,
        actor: ActorId,
        parent: Option<gielinor_world::ui::Component>,
        interface: u16,
        overlay: bool,
    ) -> io::Result<()> {
        if !self
            .components
            .keys()
            .any(|id| *id >> 16 == interface as u32)
            || parent.is_some_and(|p| !self.components.contains_key(&p.0))
        {
            return Err(io::Error::other("Unknown interface/component"));
        }
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        let connection = active
            .connection
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        let mut ui = active.ui.clone();
        let (closed, packet) = if let Some(parent) = parent {
            let closed = ui
                .open_sub(parent, interface, overlay)
                .map_err(io::Error::other)?;
            (
                closed,
                gielinor_protocol::interfaces::open_sub(ui.translate(parent).0, interface, overlay),
            )
        } else {
            ui.top_level = Some(interface);
            (vec![], gielinor_protocol::interfaces::open_top(interface))
        };
        self.transport
            .send(connection, packet.opcode, &packet.payload)?;
        active.ui = ui;
        if active.anvil_count.is_some_and(|dialog| {
            closed
                .iter()
                .any(|(_, interface)| *interface == dialog.interface)
        }) {
            let dialog = active.anvil_count.take().unwrap();
            if self
                .world
                .quantity_dialog_token(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                == Some(dialog.token)
            {
                self.world
                    .cancel_quantity_dialog(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                active.ui.input_dialog_open = false;
                if let Some(connection) = active.connection {
                    self.transport.send(connection, 117, &[])?;
                }
            }
        }
        if active.skill_multi.as_ref().is_some_and(|dialog| {
            closed
                .iter()
                .any(|(_, interface)| *interface == dialog.presentation.interface)
        }) {
            active.skill_multi = None;
            self.world
                .cancel_quantity_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if active.dialogue.as_ref().is_some_and(|dialogue| {
            closed
                .iter()
                .any(|(_, interface)| *interface == dialogue.button.interface())
        }) {
            active.dialogue = None;
            self.world
                .cancel_dialogue(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if active.drop_dialog.is_some_and(|(_, button)| {
            closed
                .iter()
                .any(|(_, interface)| *interface == button.interface())
        }) {
            active.drop_dialog = None;
            self.world
                .cancel_drop_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if active.prayer_dialog.is_some_and(|(_, button)| {
            closed
                .iter()
                .any(|(_, interface)| *interface == button.interface())
        }) {
            active.prayer_dialog = None;
            self.world
                .cancel_prayer_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        for &(_, interface) in &closed {
            self.world
                .close_shop_interface(actor, interface)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if self.message_box.is_some_and(|p| {
            closed
                .iter()
                .any(|(_, interface)| *interface == p.interface)
        }) {
            if let Some(notice) = active.prayer_notice.take() {
                self.world
                    .close_prayer_notice(actor, notice.token)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
            }
        }
        self.world
            .set_modal_open(actor, active.ui.modals.entries().next().is_some())
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        if let Some(tutorial) = &self.tutorial {
            self.world
                .set_main_modal(actor, active.ui.modals.get(tutorial.mainmodal))
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if self.bank_presentation.as_ref().is_some_and(|bank| {
            closed
                .iter()
                .any(|(_, iface)| *iface == bank.main_interface)
        }) && self
            .world
            .cancel_count_dialog(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
        {
            active.ui.input_dialog_open = false;
            self.transport.send(connection, 117, &[])?;
        }
        if self.bank_presentation.as_ref().is_some_and(|bank| {
            closed
                .iter()
                .any(|(_, iface)| *iface == bank.main_interface)
        }) {
            active.bank_events_due = None;
            let stop = active.inventories.stop(self.policy.bank as u16);
            self.transport
                .send(connection, stop.opcode, &stop.payload)?;
            let search = gielinor_protocol::client_script::run(
                101,
                &[gielinor_protocol::client_script::Argument::Int(11)],
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
            self.transport
                .send(connection, search.opcode, &search.payload)?;
            self.world
                .queue_bank_compression(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        self.events.extend(
            closed
                .into_iter()
                .map(|(parent, interface)| Event::InterfaceClosed {
                    actor,
                    parent,
                    interface,
                }),
        );
        self.events.push(Event::InterfaceOpened {
            actor,
            parent,
            interface,
            overlay,
        });
        for packet in self
            .interface_hooks
            .opened(interface, &mut active.ui)
            .map_err(io::Error::other)?
        {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        if let Some(journal) = &self.journal {
            let data = self
                .world
                .data(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            let combat = self.presentation.as_ref().unwrap().combat_level(data);
            for packet in journal
                .opened_for_run(
                    interface,
                    &mut active.ui,
                    &data.variables,
                    combat,
                    journal.marks(data),
                    &self.interface_hooks,
                    data.tower_run,
                )
                .map_err(io::Error::other)?
            {
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        if let Some(tab) = self
            .combat_tab
            .as_ref()
            .filter(|tab| tab.interface == interface)
        {
            let logged_in = self
                .world
                .protection(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .processed_clock
                > 0;
            let data = self
                .world
                .data_mut(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            let category = data
                .equipment
                .get(3)
                .and_then(|item| self.definitions.items.get(&(item.id as u32)))
                .map(|item| item.weapon_category);
            let combat = self.presentation.as_ref().unwrap().combat_level(data);
            for packet in tab
                .tab(&mut data.variables, category, combat, logged_in)
                .map_err(io::Error::other)?
            {
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        Ok(())
    }
    pub fn close_interface(
        &mut self,
        actor: ActorId,
        parent: gielinor_world::ui::Component,
    ) -> io::Result<()> {
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        let connection = active.connection;
        let mut ui = active.ui.clone();
        let closed = ui.close_sub(parent);
        for (parent, _) in &closed {
            let packet = gielinor_protocol::interfaces::close_sub(ui.translate(*parent).0);
            if let Some(connection) = connection {
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        active.ui = ui;
        if active.anvil_count.is_some_and(|dialog| {
            closed
                .iter()
                .any(|(_, interface)| *interface == dialog.interface)
        }) {
            let dialog = active.anvil_count.take().unwrap();
            if self
                .world
                .quantity_dialog_token(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                == Some(dialog.token)
            {
                self.world
                    .cancel_quantity_dialog(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                active.ui.input_dialog_open = false;
                if let Some(connection) = active.connection {
                    self.transport.send(connection, 117, &[])?;
                }
            }
        }
        if active.skill_multi.as_ref().is_some_and(|dialog| {
            closed
                .iter()
                .any(|(_, interface)| *interface == dialog.presentation.interface)
        }) {
            active.skill_multi = None;
            self.world
                .cancel_quantity_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if active.dialogue.as_ref().is_some_and(|dialogue| {
            closed
                .iter()
                .any(|(_, interface)| *interface == dialogue.button.interface())
        }) {
            active.dialogue = None;
            self.world
                .cancel_dialogue(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if active.drop_dialog.is_some_and(|(_, button)| {
            closed
                .iter()
                .any(|(_, interface)| *interface == button.interface())
        }) {
            active.drop_dialog = None;
            self.world
                .cancel_drop_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if active.prayer_dialog.is_some_and(|(_, button)| {
            closed
                .iter()
                .any(|(_, interface)| *interface == button.interface())
        }) {
            active.prayer_dialog = None;
            self.world
                .cancel_prayer_dialog(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        for &(_, interface) in &closed {
            self.world
                .close_shop_interface(actor, interface)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if self.message_box.is_some_and(|p| {
            closed
                .iter()
                .any(|(_, interface)| *interface == p.interface)
        }) {
            if let Some(notice) = active.prayer_notice.take() {
                self.world
                    .close_prayer_notice(actor, notice.token)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
            }
        }
        self.world
            .set_modal_open(actor, active.ui.modals.entries().next().is_some())
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        if let Some(tutorial) = &self.tutorial {
            self.world
                .set_main_modal(actor, active.ui.modals.get(tutorial.mainmodal))
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        if self.bank_presentation.as_ref().is_some_and(|bank| {
            closed
                .iter()
                .any(|(_, iface)| *iface == bank.main_interface)
        }) && self
            .world
            .cancel_count_dialog(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
        {
            active.ui.input_dialog_open = false;
            if let Some(connection) = connection {
                self.transport.send(connection, 117, &[])?;
            }
        }
        if self.bank_presentation.as_ref().is_some_and(|bank| {
            closed
                .iter()
                .any(|(_, iface)| *iface == bank.main_interface)
        }) {
            active.bank_events_due = None;
            let stop = active.inventories.stop(self.policy.bank as u16);
            if let Some(connection) = connection {
                self.transport
                    .send(connection, stop.opcode, &stop.payload)?;
            }
            let search = gielinor_protocol::client_script::run(
                101,
                &[gielinor_protocol::client_script::Argument::Int(11)],
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
            if let Some(connection) = connection {
                self.transport
                    .send(connection, search.opcode, &search.payload)?;
            }
            self.world
                .queue_bank_compression(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        self.events.extend(
            closed
                .into_iter()
                .map(|(parent, interface)| Event::InterfaceClosed {
                    actor,
                    parent,
                    interface,
                }),
        );
        Ok(())
    }
    pub fn set_interface_events(
        &mut self,
        actor: ActorId,
        component: gielinor_world::ui::Component,
        start: i32,
        end: i32,
        events: u64,
    ) -> io::Result<()> {
        if !self.components.contains_key(&component.0) {
            return Err(io::Error::other("Unknown component"));
        }
        let packet = gielinor_protocol::interfaces::events(component.0, start, end, events)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        let connection = active
            .connection
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        let mut ui = active.ui.clone();
        ui.events
            .add(component, start, end, events)
            .map_err(io::Error::other)?;
        self.transport
            .send(connection, packet.opcode, &packet.payload)?;
        active.ui = ui;
        self.world
            .set_modal_open(actor, active.ui.modals.entries().next().is_some())
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        if let Some(tutorial) = &self.tutorial {
            self.world
                .set_main_modal(actor, active.ui.modals.get(tutorial.mainmodal))
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        }
        Ok(())
    }
    pub fn set_appearance(
        &mut self,
        actor: ActorId,
        design: crate::appearance::Design,
    ) -> io::Result<()> {
        let policy = self
            .presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Synchronization is not configured"))?;
        let data = self
            .world
            .data(actor)
            .map_err(|_| io::Error::other("Invalid actor"))?;
        policy
            .appearance(&design, data, &self.definitions)
            .map_err(io::Error::other)?;
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        active.design = design;
        active.appearance_rebuild = true;
        Ok(())
    }
    pub fn appearance_design(&self, actor: ActorId) -> Option<&crate::appearance::Design> {
        self.active.get(&actor).map(|active| &active.design)
    }
    pub fn configure_player_design(
        &mut self,
        bindings: crate::player_design::Bindings,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other("Configure player design before admission"));
        }
        self.player_design = Some(bindings);
        Ok(())
    }
    fn player_design_button(
        &mut self,
        actor: ActorId,
        component: u32,
        child: i32,
    ) -> io::Result<()> {
        let Some(bindings) = &self.player_design else {
            return Ok(());
        };
        let Some(control) = bindings.control(component, child) else {
            return Ok(());
        };
        let logged = self
            .world
            .protection(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .processed_clock
            > 0;
        let data = self
            .world
            .data_mut(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid creator actor"))?;
        let effects = bindings
            .config
            .apply(&mut active.design, &mut data.variables, control, logged)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        active.appearance_rebuild |= effects.rebuild;
        if let Some(connection) = active.connection {
            for update in effects.variables {
                let packet = gielinor_protocol::varp::update(update.id, update.value);
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        Ok(())
    }
    pub fn start_inventory_transmission(&mut self, actor: ActorId, id: u16) -> io::Result<()> {
        let data = self
            .world
            .data(actor)
            .map_err(|_| io::Error::other("Invalid actor"))?;
        if inventory(data, &self.policy, id).is_none() {
            return Err(io::Error::other("Missing inventory"));
        }
        self.active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?
            .inventories
            .start(id);
        Ok(())
    }
    pub fn run_client_script(
        &mut self,
        actor: ActorId,
        script: i32,
        args: &[gielinor_protocol::client_script::Argument],
    ) -> io::Result<()> {
        let packet = gielinor_protocol::client_script::run(script, args)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let connection = self
            .connection(actor)
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        self.transport
            .send(connection, packet.opcode, &packet.payload)
    }
    pub fn set_varp(&mut self, actor: ActorId, id: u32, value: i32) -> io::Result<()> {
        let ty = *self
            .definitions
            .varps
            .get(&id)
            .ok_or_else(|| io::Error::other("Unknown varp"))?;
        let logged_in = self.active.contains_key(&actor);
        let update = self
            .world
            .data_mut(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables
            .set(ty, value, logged_in);
        if let (Some(update), Some(connection)) = (update, self.connection(actor)) {
            let packet = gielinor_protocol::varp::update(update.id, update.value);
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    pub fn set_varbit(&mut self, actor: ActorId, id: u32, value: i32) -> io::Result<()> {
        let bit = *self
            .definitions
            .varbits
            .get(&id)
            .ok_or_else(|| io::Error::other("Unknown varbit"))?;
        let ty = *self
            .definitions
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| io::Error::other("Unknown base varp"))?;
        let logged_in = self.active.contains_key(&actor);
        let update = self
            .world
            .data_mut(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables
            .set_bit(ty, bit, value, logged_in)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        if let (Some(update), Some(connection)) = (update, self.connection(actor)) {
            let packet = gielinor_protocol::varp::update(update.id, update.value);
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        Ok(())
    }
    /// LoginScript sends VarpReset then stored transmittable keys in numeric order.
    /// Call after applying login defaults; this does not synthesize those defaults.
    pub fn resynchronize_variables(&mut self, actor: ActorId, reset: bool) -> io::Result<()> {
        let connection = self
            .connection(actor)
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        let vars = &self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables;
        if reset {
            self.transport.send(connection, 44, &[])?;
        }
        for ty in self.definitions.varps.values() {
            if ty.transmit != gielinor_world::variables::Transmit::Never && vars.contains(ty.id) {
                let packet = gielinor_protocol::varp::update(ty.id, vars.get(ty.id));
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        Ok(())
    }
    /// Used by content when XP is capped or an invisible boost changes.
    pub fn invalidate_stat(
        &mut self,
        actor: ActorId,
        stat: u8,
        invisible_modifier: i32,
    ) -> io::Result<()> {
        if !self.definitions.stats.contains_key(&(stat as u32)) {
            return Err(io::Error::other("Unknown stat"));
        }
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        active.invisible_levels.insert(stat, invisible_modifier);
        active.stats.invalidate(stat);
        Ok(())
    }
    pub fn stop_inventory_transmission(&mut self, actor: ActorId, id: u16) -> io::Result<()> {
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Invalid session actor"))?;
        let connection = active
            .connection
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        let mut next = active.inventories.clone();
        let packet = next.stop(id);
        self.transport
            .send(connection, packet.opcode, &packet.payload)?;
        active.inventories = next;
        Ok(())
    }
    /// Call between ticks to service nonblocking I/O and worker completions. Time is supplied
    /// explicitly so differential tests can replay both UNIX TOTP time and local calendar time.
    pub fn pump(
        &mut self,
        unix_seconds: u64,
        local_datetime: &str,
        seed: impl FnMut() -> io::Result<u64>,
    ) -> io::Result<Vec<Event>> {
        for event in self.transport.poll(seed)? {
            self.network(event, unix_seconds, local_datetime)?;
        }
        for _ in 0..256 {
            match self.backend.poll() {
                Ok(event) => self.completed(event, local_datetime)?,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(io::Error::other(
                        "Account worker disconnected; world state retained",
                    ))
                }
            }
        }
        self.world_events()?;
        self.submit_saves()?;
        Ok(std::mem::take(&mut self.events))
    }
    pub fn tick(&mut self) -> io::Result<Vec<Event>> {
        self.transport.begin_batch()?;
        let result = self.tick_batched();
        let flushed = self.transport.end_batch();
        let events = result?;
        flushed?;
        Ok(events)
    }
    fn tick_batched(&mut self) -> io::Result<Vec<Event>> {
        // Input can resume a protected notice before the main pass. Reopening its
        // parent cancels this queued close, matching Player.openModal/openOverlay.
        let queued: Vec<_> = self
            .active
            .iter_mut()
            .flat_map(|(&actor, active)| {
                std::mem::take(&mut active.ui.close_queue)
                    .into_iter()
                    .map(move |parent| (actor, parent))
            })
            .collect();
        for (actor, parent) in queued {
            self.close_interface(actor, parent)?;
        }
        // Content mutations between ticks must affect this tick's energy drain.
        // Gameplay handlers that mutate inventories during world processing must
        // refresh this same authoritative weight before the run-update phase.
        if self.presentation.is_some() {
            for (&actor, active) in &self.active {
                if active.connection.is_none() {
                    continue;
                }
                let data = self
                    .world
                    .data(actor)
                    .map_err(|_| io::Error::other("Invalid actor"))?;
                let grams =
                    inventory_weight(&active.inventories, data, &self.policy, &self.definitions)?;
                self.world
                    .set_run_weight(actor, grams)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                use gielinor_world::simulation::WeightSource;
                let sources = active
                    .inventories
                    .ids()
                    .filter(|id| {
                        self.definitions
                            .inventories
                            .get(&(*id as u32))
                            .is_some_and(|ty| ty.flags & 8 != 0)
                    })
                    .map(|id| {
                        if id as u32 == self.policy.inventory {
                            WeightSource::Inventory
                        } else if id as u32 == self.policy.equipment {
                            WeightSource::Equipment
                        } else if id as u32 == self.policy.bank {
                            WeightSource::Bank
                        } else {
                            WeightSource::Other(id as i32)
                        }
                    })
                    .collect();
                self.world
                    .set_weight_sources(actor, sources)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
            }
        }
        self.world
            .tick()
            .map_err(|_| io::Error::other("World clock exhausted"))?;
        self.world_events()?;
        let bank_events: Vec<_> = self
            .active
            .iter()
            .filter_map(|(&actor, active)| {
                active
                    .bank_events_due
                    .filter(|due| *due <= self.world.clock.0)
                    .map(|_| actor)
            })
            .collect();
        for actor in bank_events {
            self.active.get_mut(&actor).unwrap().bank_events_due = None;
            if self.connection(actor).is_some() {
                self.bank_item_events(actor)?;
            }
        }
        // PlayerPostTickProcess.flushClient marks the completed update batch.
        // Keep this after delayed interface output as well as world/stat updates.
        for connection in self.active.values().filter_map(|active| active.connection) {
            self.transport.send(connection, 83, &[])?;
        }
        self.submit_saves()?;
        Ok(std::mem::take(&mut self.events))
    }
    pub fn connection(&self, actor: ActorId) -> Option<u64> {
        self.active.get(&actor).and_then(|a| a.connection)
    }
    pub fn is_drained(&self) -> bool {
        self.transport.connection_count() == 0
            && self.connections.is_empty()
            && self.active.is_empty()
            && self.saves.is_empty()
    }
    /// Logout is asynchronous: keep pumping/ticking until accepted saves complete.
    pub fn begin_shutdown(&mut self) {
        self.shutting_down = true;
        self.transport.stop_accepting();
        let connections = self.transport.connection_ids();
        for connection in connections {
            self.transport.disconnect(connection);
            self.disconnected(connection);
        }
        // Shutdown bypasses reconnect grace, including already disconnected actors.
        for &actor in self.active.keys() {
            let _ = self.world.request_shutdown(actor);
        }
    }
    fn reject(&mut self, connection: u64, status: u8) {
        let _ = self.transport.reject_login(connection, status);
        self.connections.remove(&connection);
    }
    fn disconnected(&mut self, connection: u64) {
        match self.connections.remove(&connection) {
            Some(ConnectionState::Playing(actor)) => {
                if let Some(active) = self.active.get_mut(&actor) {
                    active.connection = None;
                }
                let _ = self.world.client_disconnected(actor);
            }
            Some(ConnectionState::Queued(..)) => {
                self.world.cancel_login(connection);
            }
            _ => (),
        }
    }
    fn network(
        &mut self,
        event: NetworkEvent,
        unix_seconds: u64,
        local_datetime: &str,
    ) -> io::Result<()> {
        match event {
            NetworkEvent::Authenticate { connection, login } => {
                if self.shutting_down {
                    self.transport.disconnect(connection);
                    return Ok(());
                }
                if self.connections.contains_key(&connection) {
                    self.transport.disconnect(connection);
                    self.disconnected(connection);
                    return Ok(());
                }
                let resized = login.resized;
                match self.backend.submit(Request::Authenticate {
                    connection,
                    login,
                    unix_seconds,
                    local_datetime: local_datetime.into(),
                }) {
                    Ok(()) => {
                        self.connections
                            .insert(connection, ConnectionState::Authenticating { resized });
                    }
                    Err(TrySendError::Full(_)) => self.reject(connection, 24),
                    Err(TrySendError::Disconnected(_)) => {
                        self.reject(connection, 24);
                        return Err(io::Error::other("Account worker disconnected"));
                    }
                }
            }
            NetworkEvent::Packet { connection, packet } => {
                if let Some(ConnectionState::Playing(actor)) = self.connections.get(&connection) {
                    let actor = *actor;
                    if self.world.cinderworks_checkpoint_pending(actor) { return Ok(()) }
                    let level = self
                        .world
                        .data(actor)
                        .map_err(|_| io::Error::other("Session actor missing"))?
                        .coords
                        .level();
                    let staff = self
                        .active
                        .get(&actor)
                        .ok_or_else(|| io::Error::other("Session character missing"))?
                        .character
                        .identity
                        .staff_level;
                    match crate::gameplay::decode(&packet, level, staff, false) {
                        Ok(crate::gameplay::Request::Npc(op)) => self.npc_button(actor, op)?,
                        Ok(crate::gameplay::Request::NpcTarget(target)) => {
                            self.npc_spell_target(actor, target)?
                        }
                        Ok(crate::gameplay::Request::GroundObject(op)) => {
                            if op.operation == 3
                                && op.suboperation == 0
                                && self.world.pickup_effects.is_some()
                            {
                                if !self
                                    .world
                                    .protection(actor)
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                    .delayed()
                                {
                                    if let Ok(coords) = gielinor_world::CoordGrid::new(
                                        op.x as i32,
                                        op.z as i32,
                                        level,
                                    ) {
                                        let observer = self.active[&actor].character.character_id;
                                        let visible = self.world.objects.at(coords).any(|h| {
                                            let value = h.snapshot();
                                            value.item == op.id && value.visible_to(Some(observer))
                                        });
                                        if visible {
                                            let parents: Vec<_> = self.active[&actor]
                                                .ui
                                                .modals
                                                .entries()
                                                .map(|(parent, _)| parent)
                                                .collect();
                                            for parent in parents {
                                                self.close_interface(actor, parent)?;
                                            }
                                        }
                                        if let Err(error) = self.world.submit(
                                            actor,
                                            gielinor_world::simulation::Action::TakeObject {
                                                coords,
                                                id: op.id,
                                                control: op.control,
                                            },
                                        ) {
                                            self.events.push(Event::PacketRejected {
                                                connection,
                                                actor,
                                                opcode: packet.opcode,
                                                reason: format!("{error:?}"),
                                            });
                                        }
                                    } else {
                                        self.events.push(Event::PacketRejected {
                                            connection,
                                            actor,
                                            opcode: packet.opcode,
                                            reason: "Ground object outside map bounds".into(),
                                        });
                                    }
                                }
                            } else {
                                self.events.push(Event::Packet {
                                    connection,
                                    actor,
                                    packet,
                                });
                            }
                        }
                        Ok(crate::gameplay::Request::LocationTarget(target)) => {
                            if self.held_component == Some(target.source) {
                                self.location_target(actor, target)?;
                            } else {
                                self.events.push(Event::Packet {
                                    connection,
                                    actor,
                                    packet,
                                });
                            }
                        }
                        Ok(crate::gameplay::Request::Location(op)) => {
                            if op.suboperation == 0
                                && self.world.has_location_handler(op.id as u32, op.operation)
                            {
                                if !self
                                    .world
                                    .protection(actor)
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                    .delayed()
                                {
                                    let coords = gielinor_world::CoordGrid::new(
                                        op.x as i32,
                                        op.z as i32,
                                        level,
                                    );
                                    if let Ok(coords) = coords {
                                        if self
                                            .world
                                            .locations
                                            .find_type(coords, op.id as u32)
                                            .is_some()
                                        {
                                            self.close_dialogs_and_modals(actor)?;
                                        }
                                        if let Err(error) = self.world.submit(
                                            actor,
                                            gielinor_world::simulation::Action::OperateLocation {
                                                coords,
                                                id: op.id as u32,
                                                operation: op.operation,
                                                control: op.control,
                                            },
                                        ) {
                                            self.events.push(Event::PacketRejected {
                                                connection,
                                                actor,
                                                opcode: packet.opcode,
                                                reason: format!("{error:?}"),
                                            });
                                        }
                                    } else {
                                        self.events.push(Event::PacketRejected {
                                            connection,
                                            actor,
                                            opcode: packet.opcode,
                                            reason: "Location outside map bounds".into(),
                                        });
                                    }
                                }
                            } else {
                                self.events.push(Event::Packet {
                                    connection,
                                    actor,
                                    packet,
                                });
                            }
                        }
                        Ok(crate::gameplay::Request::Action(action)) => {
                            if let gielinor_world::simulation::Action::MoveClick {
                                destination,
                                control,
                            } = action
                            {
                                self.move_click(actor, destination, control)?;
                                return Ok(());
                            }
                            if let Err(error) = self.world.submit(actor, action) {
                                self.events.push(Event::PacketRejected {
                                    connection,
                                    actor,
                                    opcode: packet.opcode,
                                    reason: format!("{error:?}"),
                                });
                            }
                        }
                        Ok(crate::gameplay::Request::KeepAlive) => (),
                        Ok(crate::gameplay::Request::WindowStatus(status)) => {
                            self.world
                                .set_music_resizable(actor, status.mode == 2)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                            let ui = &mut self.active.get_mut(&actor).unwrap().ui;
                            ui.frame_resizable = status.mode == 2;
                            ui.frame_width = status.width;
                            ui.frame_height = status.height;
                        }
                        Ok(crate::gameplay::Request::ResumePause(pause)) => {
                            if self.resume_skill_multi(actor, pause)? {
                                return Ok(());
                            }
                            // Prayer notices resume synchronously. Other dialogue
                            // systems currently resume in World::PlayerInput, so
                            // their close ownership remains with those handlers.
                            if self.active[&actor].prayer_notice.is_some() {
                                let Some(component) = self.components.get(&pause.component) else {
                                    return Ok(());
                                };
                                if !self.active.get_mut(&actor).unwrap().ui.queue_pause_close(
                                    gielinor_world::ui::Component(pause.component),
                                    pause.subcomponent,
                                    component.events,
                                ) {
                                    return Ok(());
                                }
                            }
                            self.resume_prayer_notice(actor, pause)?;
                            self.resume_dialogue_pause(actor, pause)?;
                            self.resume_drop_pause(actor, pause)?;
                            self.resume_prayer_pause(actor, pause)?;
                        }
                        Ok(crate::gameplay::Request::ResumeCount(count)) => {
                            if self.resume_anvil_count(actor, count)? {
                                return Ok(());
                            }
                            let token = self
                                .world
                                .count_dialog_token(actor)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                            self.events.push(Event::CountDialogInput {
                                actor,
                                count,
                                accepted: token.is_some(),
                            });
                            if let Some(token) = token {
                                self.active.get_mut(&actor).unwrap().ui.input_dialog_open = false;
                                if let Err(error) = self.world.submit(
                                    actor,
                                    gielinor_world::simulation::Action::ResumeBankCount {
                                        token,
                                        count,
                                    },
                                ) {
                                    self.events.push(Event::PacketRejected {
                                        connection,
                                        actor,
                                        opcode: packet.opcode,
                                        reason: format!("{error:?}"),
                                    });
                                }
                            }
                        }
                        Ok(crate::gameplay::Request::Button(button)) => {
                            if let Some(component) = self.components.get(&button.component) {
                                // Probe permissions and interface membership before aborting input.
                                let mut probe = self.active[&actor].ui.clone();
                                probe.modal_button_protected = false;
                                if probe
                                    .button(
                                        gielinor_world::ui::Component(button.component),
                                        button.slot,
                                        component.events,
                                        button.op,
                                    )
                                    .map_err(io::Error::other)?
                                    == gielinor_world::ui::ButtonRoute::Modal
                                    && self
                                        .world
                                        .cancel_count_dialog(actor)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?
                                {
                                    self.active.get_mut(&actor).unwrap().ui.input_dialog_open =
                                        false;
                                    self.transport.send(connection, 117, &[])?;
                                }
                                let protected = self
                                    .world
                                    .protection(actor)
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                    .modal_button_protected();
                                let active = self.active.get_mut(&actor).unwrap();
                                active.ui.modal_button_protected = protected;
                                let route = active
                                    .ui
                                    .button(
                                        gielinor_world::ui::Component(button.component),
                                        button.slot,
                                        component.events,
                                        button.op,
                                    )
                                    .map_err(io::Error::other)?;
                                if matches!(
                                    route,
                                    gielinor_world::ui::ButtonRoute::Overlay
                                        | gielinor_world::ui::ButtonRoute::Modal
                                ) {
                                    self.events.push(Event::InterfaceButton {
                                        actor,
                                        button,
                                        route,
                                    });
                                    if route == gielinor_world::ui::ButtonRoute::Modal {
                                        if button.op == 1 {
                                            self.anvil_button(actor, button.component)?;
                                            self.tutorial_character_button(
                                                actor,
                                                button.component,
                                            )?;
                                        }
                                        self.player_design_button(
                                            actor,
                                            button.component,
                                            button.slot,
                                        )?;
                                    }
                                    if route == gielinor_world::ui::ButtonRoute::Overlay {
                                        self.tutorial_overlay_button(actor, button.component)?;
                                        let prayer = self
                                            .world
                                            .prayer_book
                                            .as_ref()
                                            .and_then(|c| c.book.components.get(&button.component))
                                            .copied();
                                        if let Some(prayer) = prayer.filter(|_| button.op == 1) {
                                            let parents: Vec<_> = self.active[&actor]
                                                .ui
                                                .modals
                                                .entries()
                                                .map(|(p, _)| p)
                                                .collect();
                                            for parent in parents {
                                                self.close_interface(actor, parent)?;
                                            }
                                            if let Err(error) =
                                                self.world.select_prayer(actor, prayer)
                                            {
                                                self.events.push(Event::PacketRejected {
                                                    connection,
                                                    actor,
                                                    opcode: packet.opcode,
                                                    reason: format!("{error:?}"),
                                                });
                                            }
                                        }
                                        self.autocast_button(actor, button)?;
                                        if let Some(stance) =
                                            self.combat_buttons.get(&button.component).copied()
                                        {
                                            let parents: Vec<_> = self.active[&actor]
                                                .ui
                                                .modals
                                                .entries()
                                                .map(|(p, _)| p)
                                                .collect();
                                            for parent in parents {
                                                self.close_interface(actor, parent)?;
                                            }
                                            let result = if let Some(stance) = stance {
                                                self.world.select_combat_stance(actor, stance)
                                            } else {
                                                self.world
                                                    .toggle_auto_retaliate(actor)
                                                    .map(|update| update.map(|v| vec![v]))
                                            };
                                            match result {
                                                Ok(Some(updates)) => {
                                                    for update in updates {
                                                        let packet =
                                                            gielinor_protocol::varp::update(
                                                                update.id,
                                                                update.value,
                                                            );
                                                        self.transport.send(
                                                            connection,
                                                            packet.opcode,
                                                            &packet.payload,
                                                        )?;
                                                    }
                                                }
                                                Ok(None) => {}
                                                Err(error) => {
                                                    self.events.push(Event::PacketRejected {
                                                        connection,
                                                        actor,
                                                        opcode: packet.opcode,
                                                        reason: format!("{error:?}"),
                                                    })
                                                }
                                            }
                                        }
                                        if let Some(config) = &self.gameframe_moves {
                                            if button.component == config.button {
                                                let top = button
                                                    .slot
                                                    .checked_sub(1)
                                                    .and_then(|i| usize::try_from(i).ok())
                                                    .and_then(|i| config.choices.get(i))
                                                    .copied();
                                                let result = top
                                                    .ok_or_else(|| {
                                                        io::Error::other("Invalid client mode slot")
                                                    })
                                                    .and_then(|top| {
                                                        self.move_gameframe(actor, top)
                                                    });
                                                if let Err(error) = result {
                                                    self.events.push(Event::PacketRejected {
                                                        connection,
                                                        actor,
                                                        opcode: packet.opcode,
                                                        reason: error.to_string(),
                                                    });
                                                }
                                            }
                                        }
                                        if let Some(journal) = &self.journal {
                                            if let Some(tab) = journal
                                                .selected_by_button(button.component, button.slot)
                                            {
                                                let logged_in = self
                                                    .world
                                                    .protection(actor)
                                                    .map_err(|e| {
                                                        io::Error::other(format!("{e:?}"))
                                                    })?
                                                    .processed_clock
                                                    > 0;
                                                let data =
                                                    self.world.data_mut(actor).map_err(|e| {
                                                        io::Error::other(format!("{e:?}"))
                                                    })?;
                                                let combat = self
                                                    .presentation
                                                    .as_ref()
                                                    .unwrap()
                                                    .combat_level(data);
                                                let active = self.active.get_mut(&actor).unwrap();
                                                let marks = journal.marks(data);
                                                match journal.switch_for_run(
                                                    tab,
                                                    &mut active.ui,
                                                    &mut data.variables,
                                                    combat,
                                                    active.character.identity.member,
                                                    logged_in,
                                                    marks,
                                                    &self.interface_hooks,
                                                    data.tower_run,
                                                ) {
                                                    Ok(packets) => {
                                                        for packet in packets {
                                                            self.transport.send(
                                                                connection,
                                                                packet.opcode,
                                                                &packet.payload,
                                                            )?;
                                                        }
                                                    }
                                                    Err(reason) => {
                                                        self.events.push(Event::PacketRejected {
                                                            connection,
                                                            actor,
                                                            opcode: packet.opcode,
                                                            reason,
                                                        })
                                                    }
                                                }
                                            }
                                        }
                                        if let Some(control) =
                                            self.interface_controls.get(&button.component)
                                        {
                                            let logged_in = self
                                                .world
                                                .protection(actor)
                                                .map_err(|e| io::Error::other(format!("{e:?}")))?
                                                .processed_clock
                                                > 0;
                                            let data = self
                                                .world
                                                .data_mut(actor)
                                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                                            match control.apply(
                                                &mut data.variables,
                                                button.slot,
                                                logged_in,
                                            ) {
                                                Ok(Some(update)) => {
                                                    let packet = gielinor_protocol::varp::update(
                                                        update.id,
                                                        update.value,
                                                    );
                                                    self.transport.send(
                                                        connection,
                                                        packet.opcode,
                                                        &packet.payload,
                                                    )?;
                                                }
                                                Ok(None) => {}
                                                Err(reason) => {
                                                    self.events.push(Event::PacketRejected {
                                                        connection,
                                                        actor,
                                                        opcode: packet.opcode,
                                                        reason,
                                                    })
                                                }
                                            }
                                        }
                                    }
                                    use gielinor_world::simulation::Action;
                                    let content_action = if self.held_component
                                        == Some(button.component)
                                    {
                                        usize::try_from(button.slot).ok().and_then(|slot| {
                                            if let Some(item) = self
                                                .world
                                                .data(actor)
                                                .ok()
                                                .and_then(|data| data.inventory.get(slot))
                                            {
                                                if button.op == 2 && self.world.herb_cleaning.as_ref().is_some_and(|c| c.recipes.contains_key(&button.object)) {
                                                    return Some(Action::CleanHerb { slot, item: button.object });
                                                }
                                                if button.op == 2
                                                    && self
                                                        .world
                                                        .item_packs
                                                        .contains_key(&button.object)
                                                {
                                                    return Some(Action::OpenItemPack {
                                                        slot,
                                                        item: button.object,
                                                    });
                                                }
                                                if button.op == 2
                                                    && (self
                                                        .world
                                                        .slot_preserving_foods
                                                        .contains(&item.id)
                                                        || self
                                                            .world
                                                            .slot_preserving_foods
                                                            .contains(&button.object))
                                                {
                                                    return Some(Action::EatItemSlot {
                                                        slot,
                                                        item: button.object,
                                                    });
                                                }
                                                if button.op == 7
                                                    && self.world.dropping.as_ref().is_some_and(
                                                        |config| {
                                                            config.rules.contains_key(&item.id)
                                                        },
                                                    )
                                                {
                                                    return Some(Action::DropSlot { slot });
                                                }
                                                if self.world.potions.contains_key(&item.id)
                                                    && matches!(button.op, 2 | 3)
                                                {
                                                    return Some(Action::DrinkSlot {
                                                        slot,
                                                        item: item.id,
                                                        empty: button.op == 3,
                                                    });
                                                }
                                                if button.op == 2
                                                    && self.world.prayer.as_ref().is_some_and(|c| {
                                                        c.bones.contains_key(&item.id)
                                                    })
                                                {
                                                    return Some(Action::BurySlot { slot });
                                                }
                                            }
                                            match button.op {
                                                2 if !self.world.foods.is_empty() => {
                                                    Some(Action::EatSlot { slot })
                                                }
                                                3 if self.equipment_message_params.is_some() => {
                                                    Some(Action::EquipSlot { slot })
                                                }
                                                _ => None,
                                            }
                                        })
                                    } else if button.op == 1 {
                                        self.worn_components
                                            .get(&button.component)
                                            .map(|&slot| Action::UnequipSlot { slot })
                                    } else {
                                        None
                                    };
                                    if let Some(action) = content_action.filter(|_| {
                                        route == gielinor_world::ui::ButtonRoute::Overlay
                                    }) {
                                        let parents: Vec<_> = self.active[&actor]
                                            .ui
                                            .modals
                                            .entries()
                                            .map(|(p, _)| p)
                                            .collect();
                                        for parent in parents {
                                            self.close_interface(actor, parent)?;
                                        }
                                        if let Err(error) = self.world.submit(actor, action) {
                                            self.events.push(Event::PacketRejected {
                                                connection,
                                                actor,
                                                opcode: packet.opcode,
                                                reason: format!("{error:?}"),
                                            });
                                        }
                                    }
                                    if route == gielinor_world::ui::ButtonRoute::Modal {
                                        // The UI map has already checked the open modal and
                                        // operation mask. Only the current trusted shop's
                                        // components may decode a generation-bound exchange.
                                        let shop_action = self.world.shop_button_action(
                                            actor,
                                            gielinor_world::ui::Component(button.component),
                                            button.slot,
                                            button.op,
                                            button.object,
                                        );
                                        let shop_result = shop_action.and_then(|action| {
                                            action.map_or(Ok(()), |action| {
                                                self.world.submit(actor, action)
                                            })
                                        });
                                        if let Err(error) = shop_result {
                                            self.events.push(Event::PacketRejected {
                                                connection,
                                                actor,
                                                opcode: packet.opcode,
                                                reason: format!("{error:?}"),
                                            });
                                        }
                                        let bank_action = self.bank_presentation.as_ref().and_then(|bank| {
                                            if let Some(&control) = self.bank_controls.get(&button.component) {
                                                return Some(Action::Bank(gielinor_world::bank_actions::Request::Control { control, op: button.op, slot: button.slot }));
                                            }
                                            let deposit = if button.component == bank.side_component { true }
                                                else if bank.main_components.contains(&button.component) { false } else { return None; };
                                            usize::try_from(button.slot).ok().map(|slot| Action::Bank(
                                                gielinor_world::bank_actions::Request::Button { deposit, slot, op: button.op }))
                                        });
                                        if let Some(action) = bank_action {
                                            if let Err(error) = self.world.submit(actor, action) {
                                                self.events.push(Event::PacketRejected {
                                                    connection,
                                                    actor,
                                                    opcode: packet.opcode,
                                                    reason: format!("{error:?}"),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(crate::gameplay::Request::Drag(drag)) => {
                            if let (Some(source), Some(target)) = (
                                self.components.get(&drag.source),
                                self.components.get(&drag.target),
                            ) {
                                let mut probe = self.active[&actor].ui.clone();
                                probe.modal_button_protected = false;
                                if probe.drag(
                                    gielinor_world::ui::Component(drag.source),
                                    drag.source_slot,
                                    source.events,
                                    gielinor_world::ui::Component(drag.target),
                                    drag.target_slot,
                                    target.events,
                                ) == gielinor_world::ui::ButtonRoute::Modal
                                    && self
                                        .world
                                        .cancel_count_dialog(actor)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?
                                {
                                    self.active.get_mut(&actor).unwrap().ui.input_dialog_open =
                                        false;
                                    self.transport.send(connection, 117, &[])?;
                                }
                                let protected = self
                                    .world
                                    .protection(actor)
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                    .modal_button_protected();
                                let active = self.active.get_mut(&actor).unwrap();
                                active.ui.modal_button_protected = protected;
                                let route = active.ui.drag(
                                    gielinor_world::ui::Component(drag.source),
                                    drag.source_slot,
                                    source.events,
                                    gielinor_world::ui::Component(drag.target),
                                    drag.target_slot,
                                    target.events,
                                );
                                if matches!(
                                    route,
                                    gielinor_world::ui::ButtonRoute::Overlay
                                        | gielinor_world::ui::ButtonRoute::Modal
                                ) {
                                    self.events
                                        .push(Event::InterfaceDrag { actor, drag, route });
                                    if route == gielinor_world::ui::ButtonRoute::Modal
                                        && self.bank_presentation.as_ref().is_some_and(|bank| {
                                            bank.main_components.contains(&drag.source)
                                        })
                                    {
                                        if let (Some(insert), Ok(from), Ok(into)) = (
                                            self.bank_insert,
                                            usize::try_from(drag.source_slot),
                                            usize::try_from(drag.target_slot),
                                        ) {
                                            use gielinor_world::bank_actions::{Control, Request};
                                            let request = if matches!(
                                                self.bank_controls.get(&drag.target),
                                                Some(Control::Tabs)
                                            ) {
                                                Request::DragIntoTab {
                                                    from,
                                                    tab: if into == 10 {
                                                        9
                                                    } else if (11..=19).contains(&into) {
                                                        into - 11
                                                    } else {
                                                        usize::MAX
                                                    },
                                                    expected: drag.source_object,
                                                }
                                            } else {
                                                Request::DragMain { from, into, insert }
                                            };
                                            if let Err(error) = self.world.submit(
                                                actor,
                                                gielinor_world::simulation::Action::Bank(request),
                                            ) {
                                                self.events.push(Event::PacketRejected {
                                                    connection,
                                                    actor,
                                                    opcode: packet.opcode,
                                                    reason: format!("{error:?}"),
                                                });
                                            }
                                        }
                                    }
                                    if route == gielinor_world::ui::ButtonRoute::Modal
                                        && matches!(
                                            self.bank_controls.get(&drag.source),
                                            Some(gielinor_world::bank_actions::Control::Tabs)
                                        )
                                    {
                                        if let (Ok(from), Ok(into)) = (
                                            usize::try_from(drag.source_slot - 11),
                                            usize::try_from(drag.target_slot - 11),
                                        ) {
                                            if let Err(error) = self.world.submit(actor, gielinor_world::simulation::Action::Bank(gielinor_world::bank_actions::Request::ReorderTabs { from, into })) {
                                                self.events.push(Event::PacketRejected { connection, actor, opcode: packet.opcode, reason: format!("{error:?}") });
                                            }
                                        }
                                    }
                                    if route == gielinor_world::ui::ButtonRoute::Modal
                                        && self
                                            .bank_presentation
                                            .as_ref()
                                            .is_some_and(|bank| drag.source == bank.side_component)
                                    {
                                        if let (Ok(from), Ok(into)) = (
                                            usize::try_from(drag.source_slot),
                                            usize::try_from(drag.target_slot),
                                        ) {
                                            if let Err(error) = self.world.submit(actor, gielinor_world::simulation::Action::Bank(gielinor_world::bank_actions::Request::DragSide { from, into })) {
                                                self.events.push(Event::PacketRejected { connection, actor, opcode: packet.opcode, reason: format!("{error:?}") });
                                            }
                                        }
                                    }
                                    if route == gielinor_world::ui::ButtonRoute::Overlay
                                        && self.held_component == Some(drag.source)
                                    {
                                        if let (Ok(from), Ok(into)) = (
                                            usize::try_from(drag.source_slot),
                                            usize::try_from(drag.target_slot),
                                        ) {
                                            let parents: Vec<_> = self.active[&actor]
                                                .ui
                                                .modals
                                                .entries()
                                                .map(|(p, _)| p)
                                                .collect();
                                            for parent in parents {
                                                self.close_interface(actor, parent)?;
                                            }
                                            if let Err(error) = self.world.submit(actor, gielinor_world::simulation::Action::MoveInventorySlot { from, into }) {
                                                self.events.push(Event::PacketRejected { connection, actor,
                                                    opcode: packet.opcode, reason: format!("{error:?}") });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(crate::gameplay::Request::Target(target)) => {
                            if let (Some(source_type), Some(target_type)) = (
                                self.components.get(&target.source),
                                self.components.get(&target.target),
                            ) {
                                let source_events = source_type.events;
                                let target_events = target_type.events;
                                let source = gielinor_world::ui::Component(target.source);
                                let destination = gielinor_world::ui::Component(target.target);
                                let mut probe = self.active[&actor].ui.clone();
                                probe.modal_button_protected = false;
                                if probe.target_button(
                                    source,
                                    target.source_slot,
                                    source_events,
                                    destination,
                                    target.target_slot,
                                    target_events,
                                ) == gielinor_world::ui::ButtonRoute::Modal
                                    && self
                                        .world
                                        .cancel_count_dialog(actor)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?
                                {
                                    self.active.get_mut(&actor).unwrap().ui.input_dialog_open =
                                        false;
                                    self.transport.send(connection, 117, &[])?;
                                }
                                let protected = self
                                    .world
                                    .protection(actor)
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                    .modal_button_protected();
                                let active = self.active.get_mut(&actor).unwrap();
                                active.ui.modal_button_protected = protected;
                                let route = active.ui.target_button(
                                    source,
                                    target.source_slot,
                                    source_events,
                                    destination,
                                    target.target_slot,
                                    target_events,
                                );
                                if matches!(
                                    route,
                                    gielinor_world::ui::ButtonRoute::Overlay
                                        | gielinor_world::ui::ButtonRoute::Modal
                                ) {
                                    self.events.push(Event::InterfaceTarget {
                                        actor,
                                        target,
                                        route,
                                    });
                                    if route == gielinor_world::ui::ButtonRoute::Overlay
                                        && self.held_component == Some(target.source)
                                        && self.held_component == Some(target.target)
                                    {
                                        let request = gielinor_world::held_use::Request {
                                            selected_slot: target.source_slot,
                                            selected_id: target.source_object,
                                            target_slot: target.target_slot,
                                            target_id: target.target_object,
                                        };
                                        let prepared = self.world.prepare_held_use(actor, request);
                                        let result = match prepared {
                                            Ok(Some(preparation)) => {
                                                let active = self.active.get_mut(&actor).unwrap();
                                                if preparation.abort_dialog {
                                                    active.ui.input_dialog_open = false;
                                                    self.transport.send(connection, 117, &[])?;
                                                }
                                                let parents: Vec<_> = active
                                                    .ui
                                                    .modals
                                                    .entries()
                                                    .map(|(parent, _)| parent)
                                                    .collect();
                                                for parent in parents {
                                                    self.close_interface(actor, parent)?;
                                                }
                                                self.world.finish_held_use(actor, request)
                                            }
                                            Ok(None) => Ok(()),
                                            Err(error) => Err(error),
                                        };
                                        if result.is_ok() {
                                            if let Some(choice)=self.world.take_unfinished_potion_request(actor).map_err(|e|io::Error::other(format!("{e:?}")))? {
                                                self.open_unfinished_potion(actor,choice)?;
                                            }
                                            if let Some(choice)=self.world.take_finished_potion_request(actor).map_err(|e|io::Error::other(format!("{e:?}")))? {
                                                self.open_finished_potion(actor,choice)?;
                                            }
                                            if let Some(choice)=self.world.take_dough_request(actor).map_err(|e|io::Error::other(format!("{e:?}")))? {
                                                self.open_dough(actor,choice)?;
                                            }
                                        }
                                        if let Err(error) = result {
                                            self.events.push(Event::PacketRejected {
                                                connection,
                                                actor,
                                                opcode: packet.opcode,
                                                reason: format!("{error:?}"),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        Ok(crate::gameplay::Request::PrivilegedTelejump(_)) => {
                            self.events.push(Event::PacketRejected {
                                connection,
                                actor,
                                opcode: packet.opcode,
                                reason: "Protected telejump is not implemented".into(),
                            })
                        }
                        Ok(crate::gameplay::Request::Unhandled) => {
                            self.events.push(Event::Packet {
                                connection,
                                actor,
                                packet,
                            })
                        }
                        Err(error) => {
                            self.events.push(Event::PacketRejected {
                                connection,
                                actor,
                                opcode: packet.opcode,
                                reason: format!("{error:?}"),
                            });
                            self.transport.disconnect(connection);
                            self.disconnected(connection);
                        }
                    }
                } else {
                    self.transport.disconnect(connection);
                    self.disconnected(connection);
                }
            }
            NetworkEvent::Disconnected { connection, .. } => self.disconnected(connection),
        }
        Ok(())
    }
    fn completed(&mut self, completion: Completion, now: &str) -> io::Result<()> {
        match completion {
            Completion::Authenticated { connection, result } => {
                let Some(ConnectionState::Authenticating { resized }) =
                    self.connections.get(&connection)
                else {
                    return Ok(());
                };
                let resized = *resized;
                let (character, mut data) = match result.and_then(|auth| {
                    Character::load(auth, &self.definitions, &self.policy, now.into())
                }) {
                    Ok(value) => value,
                    Err(status) => {
                        self.reject(connection, status);
                        return Ok(());
                    }
                };
                // AccountLoadResponseHook derives this temporary bit from the
                // authenticated load response before any PlayerLogin scripts.
                if let Some(config) = &self.world.session_vars {
                    if config
                        .prepare_account(&mut data.variables, character.new_account)
                        .is_err()
                    {
                        self.reject(connection, 13);
                        return Ok(());
                    }
                }
                if self.presentation.as_ref().is_some_and(|p| {
                    p.appearance(
                        &crate::appearance::Design::default(),
                        &data,
                        &self.definitions,
                    )
                    .is_err()
                }) {
                    self.reject(connection, 13);
                    return Ok(());
                }
                match self.world.queue_login(connection, data) {
                    Ok(()) => {
                        self.connections
                            .insert(connection, ConnectionState::Queued(character, resized));
                    }
                    Err(reason) => self.reject(connection, admission_status(reason)),
                }
            }
            Completion::Saved { token, result } => {
                let Some(pending) = self.saves.get_mut(&token) else {
                    return Ok(());
                };
                match result {
                    Ok(()) => {
                        let actor = pending.actor;
                        if pending.checkpoint {
                            self.world.cinderworks_checkpoint_saved(actor)
                                .map_err(|_| io::Error::other("Checkpoint completed for invalid actor"))?;
                        } else {
                            self.world.saved(actor)
                                .map_err(|_| io::Error::other("Save completed for invalid actor"))?;
                        }
                        self.saves.remove(&token);
                        self.events.push(Event::SaveCompleted {
                            actor,
                            generation: actor.generation,
                            observed_tick: self.world.clock.0,
                        });
                    }
                    Err(reason) => {
                        pending.inflight = false;
                        pending.retry_at = self.world.clock.deadline(10);
                        self.events.push(Event::SaveFailed {
                            actor: pending.actor,
                            reason,
                        });
                    }
                }
            }
        }
        Ok(())
    }
    fn world_events(&mut self) -> io::Result<()> {
        for event in self.world.drain_events() {
            match event {
                WorldEvent::ShopValue {
                    actor,
                    item,
                    player_buys,
                    result,
                } => {
                    self.shop_value(actor, item, player_buys, result)?;
                    self.events.push(Event::World(WorldEvent::ShopValue {
                        actor,
                        item,
                        player_buys,
                        result,
                    }));
                }
                WorldEvent::GameMessage { actor, text } => {
                    if let Some(connection) = self.connection(actor) {
                        let packet = gielinor_protocol::chat::game(0, None, &text)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        self.transport
                            .send(connection, packet.opcode, &packet.payload)?;
                    }
                    self.events
                        .push(Event::World(WorldEvent::GameMessage { actor, text }));
                }
                WorldEvent::TowerDiary { actor, marks } => {
                    if let (Some(connection), Some(diary)) = (
                        self.connection(actor),
                        self.journal.as_ref().and_then(|j| j.diary.as_ref()),
                    ) {
                        let run=self.world.data(actor).map_err(|e|io::Error::other(format!("{e:?}")))?.tower_run;
                        let packets=if run.id>0 {diary.cinderworks_packets(run)} else {diary.packets(marks)};
                        for packet in packets.map_err(io::Error::other)? {
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::TowerDiary { actor, marks }));
                }
                WorldEvent::CinderworksProgress {actor,run} => {
                    if let (Some(connection),Some(diary))=(self.connection(actor),self.journal.as_ref().and_then(|journal|journal.diary.as_ref())) {
                        for packet in diary.cinderworks_packets(run).map_err(io::Error::other)? {self.transport.send(connection,packet.opcode,&packet.payload)?;}
                    }
                    self.events.push(Event::World(WorldEvent::CinderworksProgress {actor,run}));
                }
                WorldEvent::TowerArrival {
                    actor,
                    arrival,
                    hitpoints_message,
                } => {
                    if let Some(connection) = self.connection(actor) {
                        let first = if self.world.cinderworks.is_some() {
                            "The Cinderworks sanctuary awaits. Read the plaque to restore its workshop, kitchen and altar."
                        } else if arrival.first {
                            "The Threshold. Read the plaque. Wings are around you."
                        } else {
                            "You wake at the Threshold door."
                        };
                        let messages = std::iter::once(first).chain(hitpoints_message)
                            .chain((arrival.fists_message && self.world.cinderworks.is_none()).then_some("You arrive with nothing but your fists. The diary grants the next tool."));
                        for message in messages {
                            let packet = gielinor_protocol::chat::game(0, None, message)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events.push(Event::World(WorldEvent::TowerArrival {
                        actor,
                        arrival,
                        hitpoints_message,
                    }));
                }
                WorldEvent::Prayer { actor, event } => {
                    self.prayer_event(actor, &event)?;
                    self.events
                        .push(Event::World(WorldEvent::Prayer { actor, event }));
                }
                WorldEvent::PrayerBook { actor, event } => {
                    self.prayer_tab_event(actor, &event)?;
                    self.events
                        .push(Event::World(WorldEvent::PrayerBook { actor, event }));
                }
                WorldEvent::PrayerNotice { actor, token, text } => {
                    self.prayer_notice(actor, token, &text)?;
                    self.events.push(Event::World(WorldEvent::PrayerNotice {
                        actor,
                        token,
                        text,
                    }));
                }
                WorldEvent::SkillAdvanced {
                    actor,
                    skill,
                    advancement,
                } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        active.stats.invalidate(skill);
                    }
                    self.events.push(Event::World(WorldEvent::SkillAdvanced {
                        actor,
                        skill,
                        advancement,
                    }));
                }
                WorldEvent::Stat { actor, update } => {
                    if update.notify {
                        if let Some(active) = self.active.get_mut(&actor) {
                            active.stats.invalidate(update.skill);
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::Stat { actor, update }));
                }
                WorldEvent::Initialized { connection, actor } => {
                    let Some(ConnectionState::Queued(mut character, resized)) =
                        self.connections.remove(&connection)
                    else {
                        return Err(io::Error::other(
                            "Admitted player has no character metadata",
                        ));
                    };
                    character.identity.slot = actor.slot;
                    self.world.set_membership(actor,character.identity.member)
                        .map_err(|e|io::Error::other(format!("Membership admission: {e:?}")))?;
                    self.world
                        .set_music_resizable(actor, resized)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    self.world
                        .set_observer_id(actor, character.character_id)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    let new_account = character.new_account;
                    // BankInvScript assigns the default on PlayerLogin, even
                    // when the player never opens a bank during this session.
                    if let (Some(bank), Some(config), Some(_)) = (
                        &self.bank_presentation,
                        &self.world.banking,
                        &self.login_presentation,
                    ) {
                        let capacity = config.capacity;
                        let data = self
                            .world
                            .data_mut(actor)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        if capacity
                            .get(&data.variables)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?
                            == 0
                        {
                            data.variables
                                .set_bit(capacity.base, capacity.bit, bank.default_capacity, false)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        }
                    }
                    let login_messages = if let Some(policy) = self.floor_progression {
                        let data = self
                            .world
                            .data_mut(actor)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        policy.login(data)
                    } else {
                        Vec::new()
                    };
                    let accepted = self
                        .transport
                        .accept_login(connection, character.identity)
                        .is_ok();
                    self.active.insert(
                        actor,
                        Active {
                            login_messages,
                            bank_events_due: None,
                            drop_dialog: None,
                            prayer_dialog: None,
                            prayer_notice: None,
                            dialogue: None,
                            skill_multi: None,
                            anvil_count: None,
                            sequence: None,
                            spot_animations: Vec::new(),
                            pending_face: None,
                            face_npc: None,
                            face_angle: None,
                            face_changed: false,
                            ui: gielinor_world::ui::State {
                                frame_resizable: resized,
                                ..Default::default()
                            },
                            character,
                            connection: accepted.then_some(connection),
                            design: crate::appearance::Design::default(),
                            appearance_rebuild: false,
                            observer: None,
                            npc_observer: Default::default(),
                            zone_observer: Default::default(),
                            tower_scene: Default::default(),
                            stats: Default::default(),
                            invisible_levels: BTreeMap::new(),
                            run: None,
                            shop_inventory: None,
                            inventories: {
                                let mut tx = crate::inventory_sync::Transmissions::default();
                                if self.presentation.is_some() {
                                    tx.start(self.policy.inventory as u16);
                                    tx.start(self.policy.equipment as u16);
                                }
                                tx
                            },
                        },
                    );
                    if accepted {
                        self.connections
                            .insert(connection, ConnectionState::Playing(actor));
                        self.events.push(Event::PlayerReady {
                            connection,
                            actor,
                            new_account,
                        });
                    } else {
                        self.transport.disconnect(connection);
                        let _ = self.world.request_logout(actor);
                    }
                }
                WorldEvent::LoginRejected { connection, reason } => {
                    self.reject(connection, admission_status(reason))
                }
                WorldEvent::CloseModals { actor } => {
                    let parents = self
                        .active
                        .get(&actor)
                        .map(|active| {
                            active
                                .ui
                                .modals
                                .entries()
                                .map(|(parent, _)| parent)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    for parent in parents {
                        self.close_interface(actor, parent)?;
                    }
                    self.world
                        .logout_modals_closed(actor)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    self.events
                        .push(Event::World(WorldEvent::CloseModals { actor }));
                }
                WorldEvent::Save { actor, data } => self.queue_save(actor, data, false)?,
                WorldEvent::CinderworksCheckpoint {actor,data} => self.queue_save(actor,data,true)?,
                WorldEvent::WearposEffects { actor, result } => {
                    if let Some(connection) = self.connection(actor) {
                        for event in result.as_ref().map_err(|e| io::Error::other(e.clone()))? {
                            use gielinor_world::wearpos::Event as WearEvent;
                            let packet = match event {
                                WearEvent::Variable(u) => {
                                    gielinor_protocol::varp::update(u.id, u.value)
                                }
                                WearEvent::Sound(sound) => {
                                    gielinor_protocol::game::synth_sound(*sound, 1, 0)
                                }
                                WearEvent::PlayerOp(op) => {
                                    gielinor_protocol::login_init::player_operation(
                                        5,
                                        op.is_some(),
                                        op.as_deref(),
                                    )
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                }
                                WearEvent::CombatRange(level) => {
                                    gielinor_protocol::client_script::run(
                                        5224,
                                        &[gielinor_protocol::client_script::Argument::Int(*level)],
                                    )
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                                }
                            };
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::WearposEffects { actor, result }));
                }
                WorldEvent::WornEffects { actor, result } => {
                    if let Some(connection) = self.connection(actor) {
                        for event in result.as_ref().map_err(|e| io::Error::other(e.clone()))? {
                            let packet = match event {
                                gielinor_world::autocast::Event::Variable(u) => {
                                    gielinor_protocol::varp::update(u.id, u.value)
                                }
                                gielinor_world::autocast::Event::Message(message) => {
                                    gielinor_protocol::chat::game(0, None, message)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?
                                }
                            };
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::WornEffects { actor, result }));
                }
                WorldEvent::AutocastPickerOpen { actor, weapon } => {
                    self.open_autocast_picker(actor, weapon)?;
                    self.events
                        .push(Event::World(WorldEvent::AutocastPickerOpen {
                            actor,
                            weapon,
                        }));
                }
                WorldEvent::AutocastSelected {
                    actor,
                    events,
                    restore_tab,
                } => {
                    self.selected_autocast(actor, &events, restore_tab)?;
                    self.events.push(Event::World(WorldEvent::AutocastSelected {
                        actor,
                        events,
                        restore_tab,
                    }));
                }
                WorldEvent::CombatFeedback { actor, events } => {
                    self.selected_autocast(actor, &events, false)?;
                    self.events
                        .push(Event::World(WorldEvent::CombatFeedback { actor, events }));
                }
                WorldEvent::CombatXp { actor, awards } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        for (skill, _) in &awards {
                            active.stats.invalidate(skill.id());
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::CombatXp { actor, awards }));
                }
                WorldEvent::PlayerSpotAnimation {
                    actor,
                    spot,
                    delay,
                    height,
                    slot,
                } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        active
                            .spot_animations
                            .push(gielinor_protocol::spot_animation::Animation {
                                slot,
                                id: spot,
                                delay,
                                height,
                            });
                    }
                    self.events
                        .push(Event::World(WorldEvent::PlayerSpotAnimation {
                            actor,
                            spot,
                            delay,
                            height,
                            slot,
                        }));
                }
                WorldEvent::PlayerCombatAnimation { actor, sequence } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        active.sequence = Some((sequence, 0));
                    }
                    self.events
                        .push(Event::World(WorldEvent::PlayerCombatAnimation {
                            actor,
                            sequence,
                        }));
                }
                WorldEvent::CombatSound { actor, sound } => {
                    if let Some(connection) = self.active.get(&actor).and_then(|a| a.connection) {
                        let packet = gielinor_protocol::game::synth_sound(sound, 1, 0);
                        self.transport
                            .send(connection, packet.opcode, &packet.payload)?;
                    }
                    self.events
                        .push(Event::World(WorldEvent::CombatSound { actor, sound }));
                }
                WorldEvent::NpcDelayedAnimation {
                    actor,
                    sequence,
                    delay,
                } => {
                    self.combat_updates
                        .entry(gielinor_world::hits::Target::Npc(actor))
                        .or_default()
                        .sequence = Some((sequence, delay));
                    self.events
                        .push(Event::World(WorldEvent::NpcDelayedAnimation {
                            actor,
                            sequence,
                            delay,
                        }));
                }
                WorldEvent::NpcSpotAnimation {
                    actor,
                    spot,
                    delay,
                    height,
                    slot,
                } => {
                    self.combat_updates
                        .entry(gielinor_world::hits::Target::Npc(actor))
                        .or_default()
                        .spot_animations
                        .push(gielinor_protocol::spot_animation::Animation {
                            slot,
                            id: spot,
                            delay,
                            height,
                        });
                    self.events.push(Event::World(WorldEvent::NpcSpotAnimation {
                        actor,
                        spot,
                        delay,
                        height,
                        slot,
                    }));
                }
                WorldEvent::NpcCombatAnimation { actor, sequence } => {
                    self.combat_updates
                        .entry(gielinor_world::hits::Target::Npc(actor))
                        .or_default()
                        .sequence = Some((sequence, 0));
                    self.events
                        .push(Event::World(WorldEvent::NpcCombatAnimation {
                            actor,
                            sequence,
                        }));
                }
                WorldEvent::CombatAnimation {
                    actor,
                    sequence,
                    sound,
                } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        active.sequence = Some((sequence, 0));
                        if let Some(connection) = active.connection {
                            let packet = gielinor_protocol::game::synth_sound(sound, 1, 0);
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events.push(Event::World(WorldEvent::CombatAnimation {
                        actor,
                        sequence,
                        sound,
                    }));
                }
                WorldEvent::SessionVariables { actor, updates } => {
                    if let Some(connection) = self.active.get(&actor).and_then(|a| a.connection) {
                        for update in &updates {
                            let packet = gielinor_protocol::varp::update(update.id, update.value);
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events.push(Event::World(WorldEvent::SessionVariables {
                        actor,
                        updates,
                    }));
                }
                WorldEvent::Music { actor, events } => {
                    if let (Some(component), Some(connection)) = (
                        self.music_component,
                        self.active.get(&actor).and_then(|a| a.connection),
                    ) {
                        for packet in crate::music::packets(&events, component)
                            .map_err(|e| io::Error::other(format!("Music packet: {e:?}")))?
                        {
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::Music { actor, events }));
                }
                WorldEvent::CombatVariables { actor, updates } => {
                    if let Some(connection) = self.active.get(&actor).and_then(|a| a.connection) {
                        for update in &updates {
                            let packet = gielinor_protocol::varp::update(update.id, update.value);
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::CombatVariables { actor, updates }));
                }
                WorldEvent::GameframeMove { actor, movement } => {
                    if let (Some(config), Some(login), Some(active)) = (
                        &self.gameframe_moves,
                        &self.login_presentation,
                        self.active.get_mut(&actor),
                    ) {
                        if let Some(connection) = active.connection {
                            let logged_in = self
                                .world
                                .protection(actor)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?
                                .processed_clock
                                > 0;
                            let data = self
                                .world
                                .data_mut(actor)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                            let packets = config
                                .execute(
                                    &login.frames,
                                    &movement,
                                    &mut active.ui,
                                    &mut data.variables,
                                    logged_in,
                                    &self.interface_hooks,
                                )
                                .map_err(io::Error::other)?;
                            for packet in packets {
                                self.transport
                                    .send(connection, packet.opcode, &packet.payload)?;
                            }
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::GameframeMove { actor, movement }));
                }
                WorldEvent::Bank { actor, result } => {
                    if let (Ok(events), Some(connection)) = (&result, self.connection(actor)) {
                        for event in events {
                            let packet = match event {
                                gielinor_world::bank_actions::Event::CloseSearch => Some(
                                    gielinor_protocol::client_script::run(
                                        101,
                                        &[gielinor_protocol::client_script::Argument::Int(11)],
                                    )
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?,
                                ),
                                gielinor_world::bank_actions::Event::Variable(update) => {
                                    Some(gielinor_protocol::varp::update(update.id, update.value))
                                }
                                gielinor_world::bank_actions::Event::Message(message) => Some(
                                    gielinor_protocol::chat::game(0, None, message)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?,
                                ),
                                gielinor_world::bank_actions::Event::QuantityInput(_) => {
                                    self.active.get_mut(&actor).unwrap().ui.input_dialog_open =
                                        true;
                                    Some(
                                        gielinor_protocol::client_script::run(
                                            108,
                                            &[gielinor_protocol::client_script::Argument::String(
                                                "Enter amount:".into(),
                                            )],
                                        )
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?,
                                    )
                                }
                                _ => None,
                            };
                            if let Some(packet) = packet {
                                self.transport
                                    .send(connection, packet.opcode, &packet.payload)?;
                            }
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::Bank { actor, result }));
                }
                WorldEvent::EquipmentResult {
                    actor,
                    item,
                    unequipping,
                    result,
                } => {
                    if let (Err(error), Some(connection)) = (&result, self.connection(actor)) {
                        use gielinor_world::equipment::Error;
                        let messages = match error {
                            Error::NotEnoughWornSpace => {
                                vec!["You don't have enough free space to do that.".to_owned()]
                            }
                            Error::NotEnoughInventorySpace => vec![if unequipping {
                                "You don't have enough free space to do that.".to_owned()
                            } else {
                                "You don't have enough free inventory space to do that.".to_owned()
                            }],
                            Error::Requirements(missing) => {
                                let params = self.equipment_message_params.ok_or_else(|| {
                                    io::Error::other("Equipment messages not configured")
                                })?;
                                crate::content::equipment_requirement_messages(
                                    &self.definitions,
                                    item,
                                    missing,
                                    params,
                                )
                                .map_err(io::Error::other)?
                            }
                            _ => vec![],
                        };
                        for message in messages {
                            let packet = gielinor_protocol::chat::game(0, None, &message)
                                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events.push(Event::World(WorldEvent::EquipmentResult {
                        actor,
                        item,
                        unequipping,
                        result,
                    }));
                }
                WorldEvent::ResetFaceEntity { actor } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        active.face_npc = None;
                        active.pending_face = None;
                        active.face_changed = true;
                    }
                    self.events
                        .push(Event::World(WorldEvent::ResetFaceEntity { actor }));
                }
                WorldEvent::BankRequested { actor } => {
                    let (main, side) = self.npc_bank_parents
                        .ok_or_else(|| io::Error::other("Missing bank presentation parents"))?;
                    // Normal banking only: no tutorial gifts or progression.
                    self.open_bank(actor, main, side)?;
                    self.events.push(Event::World(WorldEvent::BankRequested { actor }));
                }
                WorldEvent::AnvilRequested { actor, preferred } => {
                    self.open_anvil(actor, preferred)?;
                    self.events.push(Event::World(WorldEvent::AnvilRequested {
                        actor,
                        preferred,
                    }));
                }
                WorldEvent::TutorialBank { actor, booth, next } => {
                    self.tutorial_bank_event(actor, booth, next)?;
                    self.events.push(Event::World(WorldEvent::TutorialBank {
                        actor,
                        booth,
                        next,
                    }));
                }
                WorldEvent::SmeltingRequested { actor } => {
                    self.open_smelting(actor)?;
                    self.events
                        .push(Event::World(WorldEvent::SmeltingRequested { actor }));
                }
                WorldEvent::TutorialLeaveProgress { actor, effects } => {
                    self.tutorial_progress_effects(actor, &effects)?;
                    self.events
                        .push(Event::World(WorldEvent::TutorialLeaveProgress {
                            actor,
                            effects,
                        }));
                }
                WorldEvent::TutorialCombatProgress { actor, effects } => {
                    self.tutorial_progress_effects(actor, &effects)?;
                    self.events
                        .push(Event::World(WorldEvent::TutorialCombatProgress {
                            actor,
                            effects,
                        }));
                }
                WorldEvent::TutorialInventoryProgress { actor, effects } => {
                    self.tutorial_progress_effects(actor, &effects)?;
                    self.events
                        .push(Event::World(WorldEvent::TutorialInventoryProgress {
                            actor,
                            effects,
                        }));
                }
                WorldEvent::TutorialGateFocus { actor, step, hint } => {
                    self.tutorial_gate_focus(actor, step, hint)?;
                    self.events
                        .push(Event::World(WorldEvent::TutorialGateFocus {
                            actor,
                            step,
                            hint,
                        }));
                }
                WorldEvent::EngineMessage { actor, message } => {
                    if let Some(connection) = self.connection(actor) {
                        let packet = gielinor_protocol::chat::game(4, None, message)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        self.transport
                            .send(connection, packet.opcode, &packet.payload)?;
                    }
                    self.events
                        .push(Event::World(WorldEvent::EngineMessage { actor, message }));
                }
                WorldEvent::LocationMessage { actor, message } => {
                    if let Some(connection) = self.connection(actor) {
                        let packet = gielinor_protocol::chat::game(0, None, message)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        self.transport
                            .send(connection, packet.opcode, &packet.payload)?;
                    }
                    self.events
                        .push(Event::World(WorldEvent::LocationMessage { actor, message }));
                }
                WorldEvent::FaceLocation {
                    actor,
                    coords,
                    width,
                    length,
                } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        active.face_npc = None;
                        active.pending_face = Some((coords, width, length));
                    }
                    self.events.push(Event::World(WorldEvent::FaceLocation {
                        actor,
                        coords,
                        width,
                        length,
                    }));
                }
                WorldEvent::Drop { actor, event } => {
                    self.drop_event(actor, &event)?;
                    self.events
                        .push(Event::World(WorldEvent::Drop { actor, event }));
                }
                WorldEvent::NpcInteraction { actor, event } => {
                    self.npc_event(actor, event)?;
                    self.events
                        .push(Event::World(WorldEvent::NpcInteraction { actor, event }));
                }
                WorldEvent::Dialogue { actor, event } => {
                    self.dialogue_event(actor, &event)?;
                    self.events
                        .push(Event::World(WorldEvent::Dialogue { actor, event }));
                }
                WorldEvent::GroundAction { actor, effect } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        use gielinor_world::ground_transfer::Effect;
                        let packet = match effect {
                            Effect::Animation(id) => {
                                active.sequence = Some((id, 0));
                                None
                            }
                            Effect::Sound(id) => {
                                Some(gielinor_protocol::game::synth_sound(id, 1, 0))
                            }
                            Effect::Message(message) => Some(
                                gielinor_protocol::chat::game(0, None, message)
                                    .map_err(|e| io::Error::other(format!("{e:?}")))?,
                            ),
                        };
                        if let (Some(connection), Some(packet)) = (active.connection, packet) {
                            self.transport
                                .send(connection, packet.opcode, &packet.payload)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::GroundAction { actor, effect }));
                }
                event @ (WorldEvent::DoorSound { .. }
                | WorldEvent::ClearMapFlag(_)
                | WorldEvent::CannotReach(_)) => {
                    let (actor, packet) = match &event {
                        WorldEvent::DoorSound { actor, sound } => {
                            (*actor, gielinor_protocol::game::synth_sound(*sound, 1, 0))
                        }
                        WorldEvent::ClearMapFlag(actor) => {
                            (*actor, gielinor_protocol::game::clear_map_flag())
                        }
                        WorldEvent::CannotReach(actor) => (
                            *actor,
                            gielinor_protocol::chat::game(4, None, "I can't reach that!")
                                .map_err(|e| io::Error::other(format!("{e:?}")))?,
                        ),
                        _ => unreachable!(),
                    };
                    if let Some(connection) = self.active.get(&actor).and_then(|a| a.connection) {
                        self.transport
                            .send(connection, packet.opcode, &packet.payload)?;
                    }
                    self.events.push(Event::World(event));
                }
                WorldEvent::Potion { actor, event } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        use gielinor_world::potions::Event as PotionEvent;
                        match &event {
                            PotionEvent::Animation(id) => active.sequence = Some((*id, 0)),
                            PotionEvent::Stat(update) if update.notify => {
                                active.stats.invalidate(update.skill)
                            }
                            PotionEvent::Message(message) => {
                                if let Some(connection) = active.connection {
                                    let packet = gielinor_protocol::chat::game(0, None, message)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                                    self.transport.send(
                                        connection,
                                        packet.opcode,
                                        &packet.payload,
                                    )?;
                                }
                            }
                            PotionEvent::Energy(energy) => {
                                if let Some(connection) = active.connection {
                                    let payload = gielinor_protocol::game::run_energy(*energy)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                                    self.transport.send(connection, 64, &payload)?;
                                }
                                if let Some(run) = &mut active.run {
                                    run.1 = *energy;
                                }
                            }
                            _ => (),
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::Potion { actor, event }));
                }
                WorldEvent::Ground(event) => {
                    if self.zones_enabled {
                        self.ground_updates
                            .push(gielinor_world::zones::Change::Ground(event.clone()));
                    }
                    self.events.push(Event::World(WorldEvent::Ground(event)));
                }
                WorldEvent::Location(event) => {
                    if self.zones_enabled {
                        self.ground_updates
                            .push(gielinor_world::zones::Change::Location(event));
                    }
                    self.events.push(Event::World(WorldEvent::Location(event)));
                }
                WorldEvent::Projectile(flight) => {
                    if self.zones_enabled {
                        self.ground_updates
                            .push(gielinor_world::zones::Change::Projectile(flight));
                    }
                    self.events
                        .push(Event::World(WorldEvent::Projectile(flight)));
                }
                WorldEvent::TileGraphic(graphic) => {
                    if self.zones_enabled { self.ground_updates.push(gielinor_world::zones::Change::Graphic(graphic)); }
                    self.events.push(Event::World(WorldEvent::TileGraphic(graphic)));
                }
                WorldEvent::AreaSound(sound) => {
                    if self.zones_enabled {
                        self.ground_updates
                            .push(gielinor_world::zones::Change::Sound(sound));
                    }
                    self.events.push(Event::World(WorldEvent::AreaSound(sound)));
                }
                WorldEvent::AmmoConsumed { actor, result } => {
                    if result.detracted.is_some_and(|d| d.rebuild_appearance) {
                        if let Some(active) = self.active.get_mut(&actor) {
                            active.appearance_rebuild = true;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::AmmoConsumed { actor, result }));
                }
                WorldEvent::Food { actor, event } => {
                    if let Some(active) = self.active.get_mut(&actor) {
                        match &event {
                            gielinor_world::food::Event::Animation(id) => {
                                active.sequence = Some((*id, 0))
                            }
                            gielinor_world::food::Event::Stat(update) if update.notify => {
                                active.stats.invalidate(update.skill)
                            }
                            gielinor_world::food::Event::Message(message) => {
                                if let Some(connection) = active.connection {
                                    let packet = gielinor_protocol::chat::game(0, None, message)
                                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                                    self.transport.send(
                                        connection,
                                        packet.opcode,
                                        &packet.payload,
                                    )?;
                                }
                            }
                            _ => (),
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::Food { actor, event }));
                }
                WorldEvent::Hit { target, impact } => {
                    if let Some(config) = &self.combat_display {
                        let info = match target {
                            gielinor_world::hits::Target::Player(actor) => {
                                self.world.data(actor).ok().map(|data| {
                                    (false, data.stats.base(3) as u8 as i32, config.player_bar)
                                })
                            }
                            gielinor_world::hits::Target::Npc(id) => {
                                self.world.npc(id).and_then(|npc| {
                                    config
                                        .npc_bars
                                        .get(&npc.type_id)
                                        .map(|bar| (true, npc.base_stats[3], *bar))
                                })
                            }
                        };
                        if let Some((npc, maximum, bar)) = info {
                            config
                                .impact(
                                    npc,
                                    impact,
                                    maximum,
                                    bar,
                                    self.combat_updates.entry(target).or_default(),
                                )
                                .map_err(io::Error::other)?;
                        }
                    }
                    self.events
                        .push(Event::World(WorldEvent::Hit { target, impact }));
                }
                WorldEvent::SynchronizeNpcs { tick, npcs } => {
                    self.npc_snapshots.clone_from(&npcs);
                    self.events
                        .push(Event::World(WorldEvent::SynchronizeNpcs { tick, npcs }));
                }
                WorldEvent::SynchronizePlayers { tick, players } => {
                    self.synchronize_players(&players);
                    self.events
                        .push(Event::World(WorldEvent::SynchronizePlayers {
                            tick,
                            players,
                        }));
                }
                WorldEvent::Deleted(actor) => {
                    if let Some(active) = self.active.remove(&actor) {
                        if let Some(connection) = active.connection {
                            self.connections.remove(&connection);
                            // Normal logout preserves high-priority output and discards this tick's UI.
                            let _ = self.transport.logout(connection);
                        }
                    }
                    self.events.push(Event::World(WorldEvent::Deleted(actor)));
                }
                other => self.events.push(Event::World(other)),
            }
        }
        Ok(())
    }
    /// Establish the gameframe and EngineLoginReady content before appearance
    /// snapshots. The observer is committed only after the complete first cycle.
    fn initialize_player_view(
        &mut self,
        actor: ActorId,
        positions: &[(ActorId, gielinor_world::CoordGrid)],
    ) -> io::Result<crate::synchronization::Observer> {
        let policy = self
            .presentation
            .as_ref()
            .ok_or_else(|| io::Error::other("Missing presentation"))?;
        let active = self
            .active
            .get_mut(&actor)
            .ok_or_else(|| io::Error::other("Missing actor"))?;
        let connection = active
            .connection
            .ok_or_else(|| io::Error::other("Disconnected actor"))?;
        let coords = positions
            .iter()
            .find(|p| p.0 == actor)
            .ok_or_else(|| io::Error::other("Missing position"))?
            .1;
        let (observer, packet) =
            crate::synchronization::Observer::initialize_positions(actor, positions, policy.radius)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
        let mut packets = vec![packet];
        self.constructed
            .login(coords, &mut packets)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        for message in active.login_messages.drain(..) {
            packets.push(
                gielinor_protocol::chat::game(0, None, message)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?,
            );
        }
        if let Some(login) = &self.login_presentation {
            let data = self
                .world
                .data_mut(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
            packets.extend(
                login
                    .variables
                    .high_priority(
                        &mut data.variables,
                        &data.display_name,
                        -1,
                        login.welcome.as_deref(),
                        login.broadcast.as_deref(),
                    )
                    .map_err(io::Error::other)?,
            );
            let selected = login
                .frames
                .login(&mut data.variables, &mut active.ui, false)
                .map_err(io::Error::other)?;
            packets.extend(
                selected
                    .variables
                    .into_iter()
                    .map(|v| gielinor_protocol::varp::update(v.id, v.value)),
            );
            packets.push(gielinor_protocol::interfaces::open_top(selected.frame.top));
            for (parent, interface) in selected.frame.overlays {
                packets.push(gielinor_protocol::interfaces::open_sub(
                    active.ui.translate(parent).0,
                    interface,
                    true,
                ));
                packets.extend(
                    self.interface_hooks
                        .opened(interface, &mut active.ui)
                        .map_err(io::Error::other)?,
                );
                if let Some(journal) = &self.journal {
                    packets.extend(
                        journal
                            .opened_for_run(
                                interface,
                                &mut active.ui,
                                &data.variables,
                                policy.combat_level(data),
                                journal.marks(data),
                                &self.interface_hooks,
                                data.tower_run,
                            )
                            .map_err(io::Error::other)?,
                    );
                }
                if let Some(tab) = self
                    .combat_tab
                    .as_ref()
                    .filter(|tab| tab.interface == interface)
                {
                    let category = data
                        .equipment
                        .get(3)
                        .and_then(|item| self.definitions.items.get(&(item.id as u32)))
                        .map(|item| item.weapon_category);
                    let combat = policy.combat_level(data);
                    packets.extend(
                        tab.tab(&mut data.variables, category, combat, false)
                            .map_err(io::Error::other)?,
                    );
                }
            }
        }
        for packet in packets {
            self.transport
                .send(connection, packet.opcode, &packet.payload)?;
        }
        if self.login_presentation.is_some() {
            if let Some(journal) = &self.journal {
                let data = self
                    .world
                    .data(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                for packet in journal.login_packets(data).map_err(io::Error::other)? {
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
            if let Some(sync) = self
                .world
                .finish_tower_diary_login(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
            {
                if let Some(text) = sync.message {
                    let packet = gielinor_protocol::chat::game(0, None, text)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
            self.begin_tutorial(actor)?;
        }
        Ok(observer)
    }
    fn synchronize_players(
        &mut self,
        players: &[(ActorId, gielinor_world::simulation::MovementSnapshot)],
    ) {
        if self.presentation.is_none() {
            self.combat_updates.clear();
            for active in self.active.values_mut() {
                active.spot_animations.clear();
                active.appearance_rebuild = false;
            }
            return;
        };
        let positions: Vec<_> = players
            .iter()
            .map(|(actor, movement)| (*actor, movement.coords))
            .collect();
        let mut failures = vec![];
        let mut initial_observers = HashMap::new();
        for &(actor, _) in players {
            if self
                .active
                .get(&actor)
                .is_some_and(|a| a.connection.is_some() && a.observer.is_none())
            {
                match self.initialize_player_view(actor, &positions) {
                    Ok(observer) => {
                        initial_observers.insert(actor, observer);
                    }
                    Err(error) => failures.push((actor, error.to_string())),
                }
            }
        }
        let policy = self.presentation.as_ref().unwrap();
        let mut images = vec![];
        for &(actor, movement) in players {
            if failures.iter().any(|(id, _)| *id == actor) {
                continue;
            }
            let design = self
                .active
                .get(&actor)
                .map(|a| a.design.clone())
                .unwrap_or_default();
            match self
                .world
                .data(actor)
                .map_err(|e| format!("{e:?}"))
                .and_then(|data| policy.appearance(&design, data, &self.definitions))
            {
                Ok(bytes) => images.push((actor, movement, bytes)),
                Err(reason) => failures.push((actor, reason)),
            }
        }
        for (actor, movement, _) in &images {
            if movement.steps == 0 && !movement.teleport && !movement.telejump {
                if let Some(active) = self.active.get_mut(actor) {
                    if let Some((target, width, length)) = active.pending_face.take() {
                        if let Some(angle) = gielinor_world::facing::square_angle(
                            movement.coords,
                            1,
                            target,
                            width as i32,
                            length as i32,
                        ) {
                            active.face_angle = Some(angle);
                            active.face_changed = true;
                        }
                    }
                }
            }
        }
        let avatars: Vec<_> = images
            .iter()
            .map(
                |(actor, movement, appearance)| crate::synchronization::Avatar {
                    combat: self
                        .combat_updates
                        .get(&gielinor_world::hits::Target::Player(*actor)),
                    sequence: self.active.get(actor).and_then(|active| active.sequence),
                    spot_animations: self
                        .active
                        .get(actor)
                        .map(|active| active.spot_animations.clone())
                        .unwrap_or_default(),
                    face_angle: self.active.get(actor).and_then(|active| active.face_angle),
                    face_npc: self
                        .active
                        .get(actor)
                        .and_then(|active| active.face_npc)
                        .and_then(|npc| self.world.npc(npc).map(|_| npc.slot)),
                    face_changed: self
                        .active
                        .get(actor)
                        .is_some_and(|active| active.face_changed),
                    actor: *actor,
                    movement: *movement,
                    appearance,
                    appearance_rebuild: self
                        .active
                        .get(actor)
                        .is_some_and(|active| active.appearance_rebuild),
                },
            )
            .collect();
        for &(actor, movement) in players {
            let Some(active) = self.active.get_mut(&actor) else {
                continue;
            };
            let Some(connection) = active.connection else {
                continue;
            };
            if failures.iter().any(|(id, _)| *id == actor) {
                continue;
            }
            let result = (|| -> io::Result<()> {
                let restorer_run=self.world.data(actor).map_err(|e|io::Error::other(format!("{e:?}")))?.tower_run;
                let npc_blocks: Vec<_> = self
                    .npc_snapshots
                    .iter()
                    .map(|n| {
                        let display = self
                            .combat_updates
                            .get(&gielinor_world::hits::Target::Npc(n.actor));
                        let hits = display
                            .filter(|d| !d.hits.is_empty())
                            .map(|d| {
                                gielinor_protocol::combat_updates::hitmarks(
                                    true,
                                    actor.slot,
                                    n.actor.slot,
                                    &d.hits,
                                )
                            })
                            .transpose()?;
                        let bars = display
                            .filter(|d| !d.bars.is_empty())
                            .map(|d| {
                                gielinor_protocol::combat_updates::headbars(
                                    true,
                                    actor.slot,
                                    n.actor.slot,
                                    &d.bars,
                                )
                            })
                            .transpose()?;
                        let facing = gielinor_protocol::appearance::Face {
                            walk_mode: 0,
                            instant: false,
                            target: n.face_player.map_or(
                                gielinor_protocol::appearance::FaceTarget::Reset,
                                |target| gielinor_protocol::appearance::FaceTarget::Entity {
                                    entity_type: 2,
                                    index: Some(target.slot as u32),
                                    fallback_angle: 0,
                                },
                            ),
                        }
                        .encode_npc()?;
                        let spots = display
                            .filter(|d| !d.spot_animations.is_empty())
                            .map(|d| {
                                gielinor_protocol::spot_animation::encode_npc(&d.spot_animations)
                            })
                            .transpose()?;
                        let mut blocks = gielinor_protocol::combat_updates::NpcUpdates {
                            name: None,
                            transform: None,
                            combat: None,
                            hitmarks: hits.as_deref(),
                            headbars: bars.as_deref(),
                            sequence: display.and_then(|d| d.sequence),
                            spot_animations: spots.as_deref(),
                            face: n.face_changed.then_some(facing.as_slice()),
                        };
                        let current = blocks.encode();
                        if self.world.cinderworks.as_ref().is_some_and(|c| n.type_id==c.warden && gielinor_world::tower_layout::contains(c.origin,n.movement.coords)) {
                            blocks.name=Some("Ash Warden");blocks.combat=Some(7);
                            blocks.transform=Some(gielinor_world::tower_cinderworks::WARDEN_VISUAL);
                        }
                        blocks.face = (n.face_player.is_some() || n.face_changed)
                            .then_some(facing.as_slice());
                        Ok::<_, gielinor_protocol::buffer::DecodeError>((current, blocks.encode()))
                    })
                    .collect::<Result<_, _>>()
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                let npc_avatars: Vec<_> = self
                    .npc_snapshots
                    .iter()
                    .zip(&npc_blocks)
                    .filter(|(n,_)| self.world.cinderworks.as_ref().is_none_or(|c| {
                        !gielinor_world::tower_layout::contains(c.origin,n.movement.coords)
                        || gielinor_world::tower_scenery::guardian_visible(restorer_run,n.type_id,c.imp,c.warden,
                            self.world.npc(n.actor).is_some_and(|npc|npc.stats[3]>0))
                    }))
                    .map(|(n, block)| crate::npc_synchronization::Avatar {
                        actor: n.actor,
                        type_id: n.type_id,
                        movement: n.movement,
                        direction: n.direction,
                        allocated_this_tick: n.allocated_this_tick,
                        spawn_cycle: Some(n.allocated_cycle as u32),
                        extended: &block.0,
                        initial_extended: &block.1,
                    })
                    .collect();
                let first = initial_observers.contains_key(&actor);
                let mut observer = active
                    .observer
                    .clone()
                    .or_else(|| initial_observers.remove(&actor));
                let mut packets = vec![];
                let mut player_packets = observer
                    .as_mut()
                    .unwrap()
                    .cycle(&avatars)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                self.constructed
                    .replace_rebuilds(movement.coords, &mut player_packets)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                packets.extend(player_packets);
                let base = observer.as_ref().unwrap().build_area_base();
                let origin = movement.coords;
                let offset = |value: i32| {
                    u8::try_from(value)
                        .map_err(|_| io::Error::other("NPC origin outside build area"))
                };
                let origin_packet = gielinor_protocol::world_context::npc_origin(
                    offset(origin.x() - base.x())?,
                    offset(origin.z() - base.z())?,
                );
                let active_world =
                    gielinor_protocol::world_context::active_root(origin.level() as u8)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                // Rebuild packets precede the context switch; info packets follow it.
                let player_info = packets
                    .pop()
                    .ok_or_else(|| io::Error::other("Missing player info"))?;
                debug_assert_eq!(player_info.opcode, 28);
                let mut inventories = active.inventories.clone();
                let data = self
                    .world
                    .data(actor)
                    .map_err(|_| io::Error::other("Invalid actor"))?;
                let inventory_batch = inventories
                    .cycle(|id| inventory(data, &self.policy, id))
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                let mut shop_inventory = active.shop_inventory.clone();
                let mut shop_packets = vec![];
                if let Some(subscription) = &mut shop_inventory {
                    let current = self
                        .world
                        .shop_session_identity(actor)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    if current != Some(subscription.identity) {
                        shop_packets.push(subscription.updates.stop(subscription.container));
                        shop_inventory = None;
                    } else {
                        let stock = self
                            .world
                            .shops
                            .get(&subscription.identity.0)
                            .ok_or_else(|| io::Error::other("Subscribed shop disappeared"))?;
                        let batch = subscription
                            .updates
                            .cycle(|id| (id == subscription.container).then_some(stock.inventory()))
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        shop_packets.extend(batch.packets);
                    }
                }
                if first && self.login_presentation.is_some() {
                    packets.extend(
                        crate::login_init::low_priority(
                            self.world.clock.0.wrapping_sub(1),
                            &data.display_name,
                        )
                        .map_err(|e| io::Error::other(format!("{e:?}")))?,
                    );
                }
                let mut stats = active.stats.clone();
                let stat_ids = self
                    .definitions
                    .stats
                    .keys()
                    .map(|id| {
                        u8::try_from(*id)
                            .map_err(|_| io::Error::other("Stat id outside protocol range"))
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                packets.extend(
                    stats
                        .cycle(&data.stats, stat_ids, |id| {
                            active.invisible_levels.get(&id).copied().unwrap_or(0)
                        })
                        .map_err(|e| io::Error::other(format!("{e:?}")))?,
                );
                let grams = inventory_weight(&inventories, data, &self.policy, &self.definitions)?;
                let run = (grams / 1000, data.energy);
                if active.run.is_none_or(|old| old.0 != run.0) {
                    packets.push(Packet {
                        opcode: 31,
                        payload: gielinor_protocol::game::run_weight(run.0).to_vec(),
                    });
                }
                if active.run.is_none_or(|old| old.1 != run.1) {
                    packets.push(Packet {
                        opcode: 64,
                        payload: gielinor_protocol::game::run_energy(run.1)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?
                            .to_vec(),
                    });
                }
                if first && self.login_presentation.is_some() {
                    packets.extend(crate::login_init::tail());
                }
                packets.push(active_world.clone());
                packets.push(origin_packet);
                packets.push(gielinor_protocol::world_context::empty_entities());
                packets.push(player_info);
                let mut npc_observer = active.npc_observer.clone();
                let had_npcs = !npc_observer.is_empty();
                let npc_packet = npc_observer
                    .cycle(origin, policy.radius > 31, policy.radius, 255, &npc_avatars)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                // An all-removal byte must still reach a previously nonempty viewport.
                if had_npcs || npc_packet.payload != [0] {
                    packets.push(npc_packet);
                }
                packets.push(active_world);
                let mut zone_observer = active.zone_observer.clone();
                let mut tower_scene = active.tower_scene.clone();
                if self.zones_enabled {
                    let mut emissions = zone_observer
                        .cycle_all(
                            origin,
                            base,
                            Some(active.character.character_id),
                            &self.world.objects,
                            &self.world.locations,
                            &self.ground_updates,
                        )
                        .map_err(io::Error::other)?;
                    if let Some(config) = &self.world.cinderworks {
                        crate::tower_scene::phase_effects(config.origin,data.tower_run,origin,&mut emissions);
                        let scenery=tower_scene.cycle(config.origin,data.tower_run,origin,base,
                            zone_observer.visible(),&emissions,self.world.clock.0).map_err(io::Error::other)?;
                        emissions.extend(scenery);
                    }
                    packets.extend(
                        crate::ground_sync::encode(base, &emissions)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?,
                    );
                }
                // PlayerPostTickProcess: map/client info, zones, then inventory snapshots.
                // LoginScript subscribes inventories; it does not send snapshots itself.
                packets.extend(inventory_batch.packets);
                packets.extend(shop_packets);
                for packet in packets {
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
                active.observer = observer;
                active.npc_observer = npc_observer;
                active.zone_observer = zone_observer;
                active.tower_scene = tower_scene;
                active.inventories = inventories;
                active.shop_inventory = shop_inventory;
                active.stats = stats;
                active.run = Some(run);
                if let Ok(data) = self.world.data_mut(actor) {
                    for id in inventory_batch.processed {
                        if let Some(inv) = inventory_mut(data, &self.policy, id) {
                            inv.clear_modified();
                        }
                    }
                }
                Ok(())
            })();
            if let Err(error) = result {
                failures.push((actor, error.to_string()));
            }
        }
        // Shared shop dirtiness must survive every observer in this cycle. A
        // failed encoder/transport leaves it available for a later retry.
        if failures.is_empty() {
            for shop in self.world.shops.values_mut() {
                shop.clear_transmitted();
            }
        }
        self.ground_updates.clear();
        self.combat_updates.clear();
        for active in self.active.values_mut() {
            active.sequence = None;
            active.spot_animations.clear();
            active.face_changed = false;
            active.appearance_rebuild = false;
        }
        for (actor, reason) in failures {
            self.events
                .push(Event::SynchronizationFailed { actor, reason });
            if let Some(connection) = self.connection(actor) {
                self.transport.disconnect(connection);
                self.disconnected(connection);
            }
        }
        self.world.set_viewed_npcs(
            self.active
                .values()
                .filter(|active| active.connection.is_some())
                .flat_map(|active| active.npc_observer.actors()),
        );
    }
    fn queue_save(&mut self, actor: ActorId, data: PlayerData, checkpoint: bool) -> io::Result<()> {
        let active = self
            .active
            .get(&actor)
            .ok_or_else(|| io::Error::other("Missing character metadata for save"))?;
        let save = active
            .character
            .save(&data, &self.definitions, &self.policy)
            .map_err(io::Error::other)?;
        // One writer per actor. A checkpoint blocks logout snapshots until its
        // acknowledgement; unique tokens also reject late duplicate completions.
        if self.saves.values().any(|p|p.actor==actor) {
            return Err(io::Error::other("Overlapping actor saves"));
        }
        self.next_save_token = self.next_save_token.checked_add(1)
            .ok_or_else(||io::Error::other("Account save token exhausted"))?;
        self.saves.insert(self.next_save_token, PendingSave {
            actor,
            data: save,
            inflight: false,
            retry_at: self.world.clock.0,
            checkpoint,
        });
        Ok(())
    }
    fn submit_saves(&mut self) -> io::Result<()> {
        for (&token, pending) in self
            .saves
            .iter_mut()
            .filter(|(_, p)| !p.inflight && p.retry_at <= self.world.clock.0)
            .take(16)
        {
            match self.backend.submit(Request::Save {
                token,
                data: pending.data.clone(),
            }) {
                Ok(()) => pending.inflight = true,
                Err(TrySendError::Full(_)) => break,
                Err(TrySendError::Disconnected(_)) => {
                    return Err(io::Error::other(
                        "Account worker disconnected; pending saves retained",
                    ))
                }
            }
        }
        Ok(())
    }
}
fn admission_status(reason: AdmissionError) -> u8 {
    match reason {
        AdmissionError::Duplicate => 5,
        AdmissionError::Full => 7,
        AdmissionError::QueueFull => 24,
    }
}
