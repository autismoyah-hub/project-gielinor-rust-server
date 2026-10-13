//! Authoritative world state. Network clients and future testing actors use the same actions.
use crate::npc::{Mode, Npc, RandomSource, SeededRandom};
use crate::{
    collision::{CollisionMap, Strategy},
    entities::EntityList,
    inventory::{Inventory, StackType},
    movement::{update_energy, Movement, RunModifiers, Speed},
    routing::coordinate_route,
    stats::Stats,
    tick::{ClockExhausted, MapClock, Phase, PHASES},
    CoordGrid,
};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
mod autocast_actions;
mod combat_actions;
mod dialogue_actions;
mod door_actions;
mod drop_action;
mod ground_actions;
#[cfg(test)]
mod logout_tests;
mod melee_actions;
mod npc_actions;
mod npc_combat_actions;
mod npc_death_actions;
mod prayer_actions;
mod prayer_book_actions;
mod tower_actions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActorId {
    pub slot: u16,
    pub generation: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NpcId {
    pub slot: u16,
    pub generation: u64,
}
struct NpcEntry {
    id: NpcId,
    npc: Npc,
    hits: crate::hits::Queue,
    death_queued: bool,
    death_state: Option<crate::npc_death::State>,
    allocated_cycle: i32,
    allocated_pending: bool,
}
#[derive(Debug, Clone, Copy)]
pub struct NpcSnapshot {
    pub actor: NpcId,
    pub type_id: u16,
    pub movement: MovementSnapshot,
    pub direction: u8,
    pub face_player: Option<ActorId>,
    pub face_changed: bool,
    pub allocated_cycle: i32,
    pub allocated_this_tick: bool,
}
#[derive(Debug, Clone)]
pub struct NpcObservation {
    pub version: u16,
    pub tick: i32,
    pub actor: NpcId,
    pub type_id: u16,
    pub coords: CoordGrid,
    pub stats: [i32; 6],
}
#[derive(Clone, Debug)]
pub struct PlayerData {
    pub account_id: u64,
    /// Persistent character identity, matching Kotlin Player.uuid.
    pub character_id: u64,
    pub display_name: String,
    pub coords: CoordGrid,
    pub stats: Stats,
    pub inventory: Inventory,
    pub equipment: Inventory,
    pub bank: Inventory,
    pub other_inventories: BTreeMap<i32, Inventory>,
    pub variables: crate::variables::Variables,
    pub xp_rate_hundreds: i32,
    pub energy: i32,
    pub tower_arrival: crate::tower_arrival::State,
}
#[derive(Clone, Copy, Debug)]
pub enum WeightSource {
    Inventory,
    Equipment,
    Bank,
    Other(i32),
}
impl WeightSource {
    fn inventory(self, data: &PlayerData) -> Option<&Inventory> {
        match self {
            Self::Inventory => Some(&data.inventory),
            Self::Equipment => Some(&data.equipment),
            Self::Bank => Some(&data.bank),
            Self::Other(id) => data.other_inventories.get(&id),
        }
    }
}
impl PlayerData {
    /// Account creation policy must supply the configured spawn and initial stats.
    pub fn new(account_id: u64, display_name: String, coords: CoordGrid, stats: Stats) -> Self {
        Self {
            account_id,
            character_id: account_id,
            display_name,
            coords,
            stats,
            inventory: Inventory::new(28, StackType::Normal, false),
            equipment: Inventory::new(15, StackType::Normal, false),
            bank: Inventory::new(1220, StackType::Always, true),
            other_inventories: BTreeMap::new(),
            variables: crate::variables::Variables::default(),
            xp_rate_hundreds: 100,
            energy: 10000,
            tower_arrival: Default::default(),
        }
    }
}
#[derive(Clone)]
enum Deferred {
    BankCompression,
    GameframeMove(crate::gameframe::Movement),
    CombatStance(u8),
    BulwarkDelay,
    AutocastOpen(bool),
    PrayerToggle(u8),
    Preserve,
}
fn perform_stance(
    config: &crate::combat_stance::Config,
    data: &mut PlayerData,
    queues: &mut crate::queues::Scheduler<Deferred>,
    stance: u8,
    logged_in: bool,
) -> Result<Vec<crate::variables::Update>, ActionError> {
    let mut vars = data.variables.clone();
    let result = config
        .set(
            &mut vars,
            data.equipment.get(3).map(|v| v.id),
            stance,
            logged_in,
        )
        .map_err(|_| ActionError::InvalidItem)?;
    if result.bulwark_delay {
        queues.remove_all(3);
        queues
            .add(
                3,
                crate::queues::Category::LongDiscard,
                8,
                Deferred::BulwarkDelay,
            )
            .map_err(|_| ActionError::QueueFull)?;
    }
    data.variables = vars;
    Ok(result.variables)
}
struct Player {
    areas: crate::area_tracking::State,
    pending_areas: VecDeque<crate::area_tracking::Event>,
    music_resizable: bool,
    login_cycle: i32,
    aggressive_npc: Option<NpcId>,
    last_npc_combat: i32,
    last_pvp_combat: i32,
    retaliate_npcs: VecDeque<(i32, NpcId)>,
    id: ActorId,
    combat_decimal: i32,
    action_delay: i32,
    data: PlayerData,
    movement: Movement,
    active_speed: Speed,
    saving: bool,
    pending_logout: bool,
    logout_modal_close_pending: bool,
    disconnect_grace: crate::disconnect::Grace,
    run: RunModifiers,
    hits: crate::hits::Queue,
    death_queued: bool,
    logout_blocked_until: i32,
    protection: crate::protected::State,
    food_delay: bool,
    location_interaction: Option<(crate::locations::Location, u8)>,
    pending_door: Option<crate::doors::Pending>,
    observer_id: Option<i64>,
    autocast_picker_defensive: Option<bool>,
    loot_drop_duration: Option<i32>,
    object_interaction: Option<crate::ground::Handle>,
    npc_interaction: Option<crate::npc_interaction::Pending>,
    pending_pickup: Option<crate::ground_transfer::Pickup>,
    pending_drop: Option<(u64, crate::drop_actions::Pending)>,
    prayer: Option<crate::prayer::State>,
    prayer_notice: Option<u64>,
    overhead: Option<u8>,
    dialogue: Option<crate::dialogue::Pending>,
    bank_count: Option<(u64, crate::bank_actions::CountPurpose)>,
    dialog_generation: u64,
    has_modal: bool,
    queues: crate::queues::Scheduler<Deferred>,
    soft_timers: crate::player_timers::Timers,
    weight_sources: Option<Vec<WeightSource>>,
}
impl Player {
    fn begin_held_action(&mut self) {
        // The current source's normal launch clears interactions; launchLenient
        // only replaces the suspended coroutine, even if objectVerify rejects it.
        if !self.protection.access_protected(self.has_modal) {
            self.location_interaction = None;
            self.object_interaction = None;
            self.npc_interaction = None;
            self.movement.interacting = false;
        }
        self.protection.suspended = false;
        self.food_delay = false;
        self.pending_door = None;
        self.pending_pickup = None;
        self.pending_drop = None;
        self.prayer = None;
        self.dialogue = None;
    }
    fn set_active_speed(&mut self, speed: Speed) {
        self.active_speed = speed;
        self.movement.cached_speed = speed;
        self.movement.speed = speed;
        self.movement.temporary_speed = Some(speed);
    }
}
#[derive(Debug, Clone)]
pub struct Observation {
    pub version: u16,
    pub tick: i32,
    pub actor: ActorId,
    pub coords: CoordGrid,
    pub energy: i32,
    pub inventory: Vec<Option<crate::inventory::Item>>,
}
/// Immutable protocol input captured before pending movement is cleared or energy is updated.
#[derive(Debug, Clone, Copy)]
pub struct MovementSnapshot {
    pub coords: CoordGrid,
    /// Tile before the final step; preserves a bend in a two-step NPC run.
    pub before_last_step: CoordGrid,
    pub cached_speed: Speed,
    pub speed: Speed,
    pub steps: u8,
    pub teleport: bool,
    pub telejump: bool,
    pub hidden: bool,
}
impl From<&Movement> for MovementSnapshot {
    fn from(movement: &Movement) -> Self {
        Self {
            coords: movement.coords,
            before_last_step: movement.last_processed,
            cached_speed: movement.cached_speed,
            speed: movement.speed,
            steps: movement.pending_steps,
            teleport: movement.pending_teleport,
            telejump: movement.pending_telejump,
            hidden: movement.hidden,
        }
    }
}
impl MovementSnapshot {
    pub fn resolved_speed(self) -> Speed {
        if self.telejump {
            Speed::Stationary
        } else if self.teleport || self.steps == 1 {
            Speed::Walk
        } else if self.steps == 2 {
            Speed::Run
        } else {
            self.speed
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Action {
    BurySlot {
        slot: usize,
    },
    ResumeBury {
        token: u64,
        choice: i32,
    },
    ResumeDialogue {
        token: u64,
        choice: i32,
    },
    OperateNpc {
        target: NpcId,
        operation: u8,
        control: bool,
    },
    DropSlot {
        slot: usize,
    },
    ResumeDrop {
        token: u64,
        choice: i32,
    },
    TakeObject {
        coords: CoordGrid,
        id: u16,
        control: bool,
    },
    OperateDoor {
        coords: CoordGrid,
        id: u32,
        control: bool,
    },
    OperateLocation {
        coords: CoordGrid,
        id: u32,
        operation: u8,
        control: bool,
    },
    ResumeBankCount {
        token: u64,
        count: i32,
    },
    Bank(crate::bank_actions::Request),
    EquipSlot {
        slot: usize,
    },
    UnequipSlot {
        slot: usize,
    },
    MoveInventorySlot {
        from: usize,
        into: usize,
    },
    /// Held-op 1 reads the authoritative slot; it never trusts a client object ID.
    EatSlot {
        slot: usize,
    },
    DrinkSlot {
        slot: usize,
        item: i32,
        empty: bool,
    },
    Walk {
        destination: CoordGrid,
        run: bool,
    },
    /// Client route request; control-click inverts the persistent run preference.
    MoveClick {
        destination: CoordGrid,
        control: bool,
    },
    SetRunning(bool),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionError {
    StaleActor,
    LoggingOut,
    WrongPlane,
    OutOfRange,
    Rooted,
    QueueFull,
    InsufficientEnergy,
    Delayed,
    InvalidItem,
    InvalidStat,
    Protected,
    InvalidBank,
    InvalidLocation,
    UnsupportedCombat,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    Duplicate,
    Full,
    QueueFull,
}
#[derive(Debug)]
pub enum WorldEvent {
    TowerArrival {
        actor: ActorId,
        arrival: crate::tower_arrival::Arrival,
        hitpoints_message: Option<&'static str>,
    },
    CloseModals {
        actor: ActorId,
    },
    PrayerBook {
        actor: ActorId,
        event: crate::prayer_book::Event,
    },
    PrayerNotice {
        actor: ActorId,
        token: u64,
        text: String,
    },
    Stat {
        actor: ActorId,
        update: crate::stat_ops::Update,
    },
    Prayer {
        actor: ActorId,
        event: crate::prayer::Event,
    },
    Dialogue {
        actor: ActorId,
        event: crate::dialogue::Event,
    },
    CombatSound {
        actor: ActorId,
        sound: u16,
    },
    PlayerCombatAnimation {
        actor: ActorId,
        sequence: u16,
    },
    NpcCombatError {
        actor: NpcId,
        error: ActionError,
    },
    NpcCombatAnimation {
        actor: NpcId,
        sequence: u16,
    },
    CombatAnimation {
        actor: ActorId,
        sequence: u16,
        sound: u16,
    },
    CombatFeedback {
        actor: ActorId,
        events: Vec<crate::autocast::Event>,
    },
    CombatXp {
        actor: ActorId,
        awards: Vec<(crate::combat_xp::Skill, crate::skills::Advancement)>,
    },
    NpcInteraction {
        actor: ActorId,
        event: crate::npc_interaction::Event,
    },
    AutocastPickerOpen {
        actor: ActorId,
        weapon: i32,
    },
    AutocastSelected {
        actor: ActorId,
        events: Vec<crate::autocast::Event>,
        restore_tab: bool,
    },
    Drop {
        actor: ActorId,
        event: crate::drop_actions::Event,
    },
    GroundAction {
        actor: ActorId,
        effect: crate::ground_transfer::Effect,
    },
    DoorSound {
        actor: ActorId,
        sound: u16,
    },
    ClearMapFlag(ActorId),
    LocationMessage {
        actor: ActorId,
        message: &'static str,
    },
    CannotReach(ActorId),
    FaceLocation {
        actor: ActorId,
        coords: CoordGrid,
        width: u8,
        length: u8,
    },
    WornEffects {
        actor: ActorId,
        result: Result<Vec<crate::autocast::Event>, String>,
    },
    CombatVariables {
        actor: ActorId,
        updates: Vec<crate::variables::Update>,
    },
    SessionVariables {
        actor: ActorId,
        updates: Vec<crate::variables::Update>,
    },
    Area {
        actor: ActorId,
        event: crate::area_tracking::Event,
    },
    Music {
        actor: ActorId,
        events: Vec<crate::music::Event>,
    },
    MusicError {
        actor: ActorId,
        reason: String,
    },
    GameframeMove {
        actor: ActorId,
        movement: crate::gameframe::Movement,
    },
    Bank {
        actor: ActorId,
        result: Result<Vec<crate::bank_actions::Event>, crate::bank_actions::Error>,
    },
    EquipmentResult {
        actor: ActorId,
        item: i32,
        unequipping: bool,
        result: Result<Vec<crate::equipment::Event>, crate::equipment::Error>,
    },
    Food {
        actor: ActorId,
        event: crate::food::Event,
    },
    Potion {
        actor: ActorId,
        event: crate::potions::Event,
    },
    Phase(Phase, i32),
    Initialized {
        connection: u64,
        actor: ActorId,
    },
    LoginRejected {
        connection: u64,
        reason: AdmissionError,
    },
    Save {
        actor: ActorId,
        data: PlayerData,
    },
    Deleted(ActorId),
    ActionRejected {
        actor: ActorId,
        error: ActionError,
    },
    Observe(Observation),
    SynchronizePlayers {
        tick: i32,
        players: Vec<(ActorId, MovementSnapshot)>,
    },
    SynchronizeNpcs {
        tick: i32,
        npcs: Vec<NpcSnapshot>,
    },
    Ground(crate::ground::Event),
    Location(crate::locations::Event),
    LocationError(&'static str),
    GroundError(crate::ground::Error),
    Hit {
        target: crate::hits::Target,
        impact: crate::hits::Impact,
    },
    DeathRequested(crate::hits::Target),
}
pub struct World {
    pub music: Option<crate::music_player::Config>,
    pub area_sources: crate::area_tracking::Sources,
    pub session_vars: Option<crate::session_vars::Config>,
    pub login_resets: Option<crate::login_resets::Config>,
    /// Host-supplied wall time, kept separate from the deterministic world clock.
    pub unix_millis: u64,
    pub tower_arrival: Option<crate::tower_arrival::Config>,
    pub combat_stance: Option<crate::combat_stance::Config>,
    pub combat_wear: Option<crate::combat_wear::Config>,
    pub autocast_selection: Option<crate::autocast_select::Config>,
    pub banking: Option<crate::bank_actions::Config>,
    pub wear_types: crate::equipment::WearCatalog,
    pub foods: BTreeMap<i32, crate::food::Edible>,
    pub potions: BTreeMap<i32, crate::potions::Potion>,
    pub prayer: Option<crate::prayer::Config>,
    pub regeneration: Option<crate::player_regen::Config>,
    pub prayer_book: Option<crate::prayer_runtime::Config>,
    pub eating_animation: u16,
    pub item_weights: BTreeMap<i32, i32>,
    pub clock: MapClock,
    pub collision: CollisionMap,
    pub areas: crate::areas::Areas,
    pub multiway_area: Option<u16>,
    pub objects: crate::ground::Objects,
    pub object_types: crate::inventory::Catalog,
    pub object_respawns: BTreeMap<u16, i32>,
    pub pickup_effects: Option<crate::ground_transfer::PickupEffects>,
    pub dropping: Option<crate::drop_actions::Config>,
    pub person_flyer: Option<i32>,
    pub locations: crate::locations::Locations,
    pub location_types: BTreeMap<u32, crate::loc_collision::LocCollision>,
    pub location_approach: BTreeMap<u32, u8>,
    pub doors: crate::doors::Catalog,
    pub location_handlers: BTreeMap<(u32, u8), crate::location_content::Handler>,
    pub npc_options: crate::npc_options::Catalog,
    pub attack_profiles: crate::attack_profile::Catalog,
    pub melee: Option<crate::melee::Config>,
    pub npc_melee: Option<crate::npc_melee::Config>,
    pub npc_death: Option<crate::npc_death::Config>,
    pub npc_bindings: BTreeMap<u16, crate::npc_bind::Binding>,
    pub combat_state: Option<crate::combat_state::Config>,
    pub npc_combat: BTreeMap<u16, crate::npc_combat::Definition>,
    pub npc_handlers: BTreeMap<(u16, u8), crate::npc_interaction::Handler>,
    location_timers: crate::loc_timers::Timers,
    players: EntityList<Player>,
    generation: u64,
    accounts: HashMap<u64, ActorId>,
    reserved: HashSet<u64>,
    logins: VecDeque<(u64, PlayerData)>,
    saved: VecDeque<ActorId>,
    actions: VecDeque<(ActorId, Action)>,
    events: Vec<WorldEvent>,
    order: Vec<usize>,
    npcs: EntityList<NpcEntry>,
    random: SeededRandom,
}
impl Default for World {
    fn default() -> Self {
        Self {
            areas: Default::default(),
            music: None,
            area_sources: Default::default(),
            tower_arrival: None,
            multiway_area: None,
            combat_stance: None,
            combat_wear: None,
            autocast_selection: None,
            banking: None,
            wear_types: crate::equipment::WearCatalog::default(),
            foods: BTreeMap::new(),
            potions: BTreeMap::new(),
            prayer: None,
            regeneration: None,
            session_vars: None,
            login_resets: None,
            unix_millis: 0,
            prayer_book: None,
            eating_animation: 829,
            item_weights: BTreeMap::new(),
            clock: MapClock::default(),
            collision: CollisionMap::default(),
            objects: crate::ground::Objects::default(),
            object_types: crate::inventory::Catalog::default(),
            object_respawns: BTreeMap::new(),
            pickup_effects: None,
            dropping: None,
            person_flyer: None,
            locations: Default::default(),
            location_types: BTreeMap::new(),
            location_approach: BTreeMap::new(),
            doors: Default::default(),
            location_handlers: Default::default(),
            npc_options: Default::default(),
            attack_profiles: Default::default(),
            melee: None,
            npc_melee: None,
            npc_death: None,
            npc_bindings: BTreeMap::new(),
            combat_state: None,
            npc_combat: BTreeMap::new(),
            npc_handlers: Default::default(),
            location_timers: Default::default(),
            players: EntityList::players(),
            generation: 0,
            accounts: HashMap::new(),
            reserved: HashSet::new(),
            logins: VecDeque::new(),
            saved: VecDeque::new(),
            actions: VecDeque::new(),
            events: vec![],
            order: vec![],
            npcs: EntityList::new(32767, 0),
            random: SeededRandom(2391),
        }
    }
}
impl World {
    pub fn change_location_for(
        &mut self,
        loc: crate::locations::Location,
        add: bool,
        duration: i32,
    ) -> Result<crate::locations::Changed, &'static str> {
        let changed = self.change_location(loc, add)?;
        self.location_timers
            .record(loc, add, changed, self.clock, duration);
        Ok(changed)
    }
    pub fn change_location(
        &mut self,
        loc: crate::locations::Location,
        add: bool,
    ) -> Result<crate::locations::Changed, &'static str> {
        let collision = &mut self.collision;
        let types = &self.location_types;
        let (changed, mutation) = self.locations.change(loc, add, |mutation| {
            // Validate every affected footprint before modifying live collision.
            let mut scratch = CollisionMap::default();
            for &(loc, add) in &mutation.collision {
                if let Some(&ty) = types.get(&loc.id) {
                    crate::loc_collision::toggle(
                        &mut scratch,
                        loc.coords,
                        loc.shape,
                        loc.angle,
                        ty,
                        add,
                    )?;
                }
            }
            for &(loc, add) in &mutation.collision {
                if let Some(&ty) = types.get(&loc.id) {
                    crate::loc_collision::toggle(
                        collision, loc.coords, loc.shape, loc.angle, ty, add,
                    )?;
                }
            }
            Ok(())
        })?;
        // Flush earlier object mutations before this location mutation so the
        // shared zone stream preserves cross-registry script ordering.
        self.events.extend(
            self.objects
                .drain_events()
                .into_iter()
                .map(WorldEvent::Ground),
        );
        self.events
            .extend(mutation.updates.into_iter().map(WorldEvent::Location));
        Ok(changed)
    }
    pub fn animate_location(&mut self, location: crate::locations::Location, sequence: u16) {
        self.events.extend(
            self.objects
                .drain_events()
                .into_iter()
                .map(WorldEvent::Ground),
        );
        self.events
            .push(WorldEvent::Location(crate::locations::Event::Animate {
                location,
                sequence,
            }));
    }
    /// Network visibility feeds the reference NPC wandering optimization. Headless
    /// actors retain their independent simulation override.
    pub fn set_viewed_npcs(&mut self, actors: impl IntoIterator<Item = NpcId>) {
        for (_, entry) in self.npcs.iter_mut() {
            entry.npc.viewed = false;
        }
        for id in actors {
            if let Some(npc) = self.npc_mut(id) {
                npc.viewed = true;
            }
        }
    }
    pub fn configure_npc_combat(&mut self, catalog: BTreeMap<u16, crate::npc_combat::Definition>) {
        for (_, entry) in self.npcs.iter_mut() {
            entry.npc.combat_xp_multiplier = self
                .npc_bindings
                .get(&entry.npc.type_id)
                .map(|b| b.xp_multiplier)
                .unwrap_or_else(|| {
                    catalog
                        .get(&entry.npc.type_id)
                        .map_or(0, |d| d.xp_multiplier)
                });
        }
        self.npc_combat = catalog;
    }
    pub fn spawn_npc(&mut self, mut npc: Npc) -> Result<NpcId, Npc> {
        let Some(slot) = self.npcs.next_free() else {
            return Err(npc);
        };
        self.generation = self
            .generation
            .checked_add(1)
            .expect("actor generation exhausted");
        let id = NpcId {
            slot: slot as u16,
            generation: self.generation,
        };
        if let Some(config) = self.npc_combat.get(&npc.type_id) {
            npc.combat_xp_multiplier = config.xp_multiplier;
        }
        if let Some(binding) = self.npc_bindings.get(&npc.type_id) {
            binding.apply(&mut npc);
            if let Some(sequence) = binding.spawn_animation {
                self.events.push(WorldEvent::NpcCombatAnimation {
                    actor: id,
                    sequence,
                });
            }
        }
        npc.movement.occupancy(&mut self.collision, true);
        let result = self.npcs.insert(NpcEntry {
            id,
            npc,
            hits: crate::hits::Queue::default(),
            death_queued: false,
            death_state: None,
            allocated_cycle: self.clock.0,
            allocated_pending: true,
        });
        assert!(result.is_ok());
        Ok(id)
    }
    pub fn npc(&self, id: NpcId) -> Option<&Npc> {
        self.npcs
            .get(id.slot as usize)
            .filter(|entry| entry.id == id)
            .map(|entry| &entry.npc)
    }
    pub fn configure_npc_bindings(&mut self, bindings: BTreeMap<u16, crate::npc_bind::Binding>) {
        for (_, entry) in self.npcs.iter_mut() {
            if let Some(binding) = bindings.get(&entry.npc.type_id) {
                binding.apply(&mut entry.npc);
                if let Some(sequence) = binding.spawn_animation {
                    self.events.push(WorldEvent::NpcCombatAnimation {
                        actor: entry.id,
                        sequence,
                    });
                }
            }
        }
        self.npc_bindings = bindings;
    }
    /// Server combat/content entry point; damage is never an externally submitted actor action.
    pub fn queue_hit(
        &mut self,
        target: crate::hits::Target,
        hit: crate::hits::Hit,
        delay: i32,
    ) -> Result<(), crate::hits::Error> {
        use crate::hits::{Error, Target};
        match target {
            Target::Npc(id) => self
                .npcs
                .get_mut(id.slot as usize)
                .filter(|e| e.id == id)
                .ok_or(Error::InvalidTarget)?
                .hits
                .push(hit, delay),
            Target::Player(id) => self
                .players
                .get_mut(id.slot as usize)
                .filter(|p| p.id == id && !p.saving)
                .ok_or(Error::InvalidTarget)?
                .hits
                .push(hit, delay),
        }
    }
    pub fn npc_mut(&mut self, id: NpcId) -> Option<&mut Npc> {
        self.npcs
            .get_mut(id.slot as usize)
            .filter(|entry| entry.id == id)
            .map(|entry| &mut entry.npc)
    }
    pub fn despawn_npc(&mut self, id: NpcId) -> Option<Npc> {
        self.npc(id)?;
        let entry = self.npcs.remove(id.slot as usize)?;
        entry.npc.movement.occupancy(&mut self.collision, false);
        Some(entry.npc)
    }
    pub fn observe_npc(&self, id: NpcId) -> Option<NpcObservation> {
        let npc = self.npc(id)?;
        Some(NpcObservation {
            version: 1,
            tick: self.clock.0,
            actor: id,
            type_id: npc.type_id,
            coords: npc.movement.coords,
            stats: npc.stats,
        })
    }
    /// Cancel admission when transport disconnects before the login phase.
    pub fn cancel_login(&mut self, connection: u64) -> Option<PlayerData> {
        let index = self.logins.iter().position(|(id, _)| *id == connection)?;
        let (_, data) = self.logins.remove(index)?;
        self.reserved.remove(&data.account_id);
        Some(data)
    }
    pub fn queue_login(&mut self, connection: u64, data: PlayerData) -> Result<(), AdmissionError> {
        if self.accounts.contains_key(&data.account_id) || self.reserved.contains(&data.account_id)
        {
            return Err(AdmissionError::Duplicate);
        }
        if self.logins.len() >= 1000 {
            return Err(AdmissionError::QueueFull);
        }
        self.reserved.insert(data.account_id);
        self.logins.push_back((connection, data));
        Ok(())
    }
    fn player(&self, id: ActorId) -> Result<&Player, ActionError> {
        self.players
            .get(id.slot as usize)
            .filter(|p| p.id == id)
            .ok_or(ActionError::StaleActor)
    }
    pub fn observe(&self, id: ActorId) -> Result<Observation, ActionError> {
        let p = self.player(id)?;
        Ok(Observation {
            version: 1,
            tick: self.clock.0,
            actor: id,
            coords: p.movement.coords,
            energy: p.data.energy,
            inventory: p.data.inventory.slots().to_vec(),
        })
    }
    pub fn data(&self, id: ActorId) -> Result<&PlayerData, ActionError> {
        Ok(&self.player(id)?.data)
    }
    /// Character identity, distinct from account ID, controls private ground-item visibility.
    pub fn set_observer_id(&mut self, id: ActorId, observer: i64) -> Result<(), ActionError> {
        self.player(id)?;
        self.players.get_mut(id.slot as usize).unwrap().observer_id = Some(observer);
        Ok(())
    }
    /// Trusted server content only. External actors submit validated Actions.
    /// Movement must still go through the movement system, not data.coords.
    pub fn data_mut(&mut self, id: ActorId) -> Result<&mut PlayerData, ActionError> {
        let player = self
            .players
            .get_mut(id.slot as usize)
            .filter(|player| player.id == id)
            .ok_or(ActionError::StaleActor)?;
        if player.saving {
            return Err(ActionError::LoggingOut);
        }
        Ok(&mut player.data)
    }
    pub fn actors(&self) -> impl Iterator<Item = ActorId> + '_ {
        self.players.iter().map(|(_, player)| player.id)
    }
    pub fn protection(&self, id: ActorId) -> Result<crate::protected::State, ActionError> {
        Ok(self.player(id)?.protection)
    }
    pub fn count_dialog_token(&self, id: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self.player(id)?.bank_count.map(|pending| pending.0))
    }
    pub fn set_modal_open(&mut self, id: ActorId, open: bool) -> Result<(), ActionError> {
        self.player(id)?;
        self.players.get_mut(id.slot as usize).unwrap().has_modal = open;
        Ok(())
    }
    /// Bank close queues a normal one-cycle callback, retained while access is protected.
    pub fn queue_bank_compression(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        p.queues
            .add(
                0,
                crate::queues::Category::Normal,
                1,
                Deferred::BankCompression,
            )
            .map_err(|_| ActionError::QueueFull)?;
        Ok(())
    }
    pub fn queue_gameframe_move(
        &mut self,
        id: ActorId,
        delay: i32,
        movement: crate::gameframe::Movement,
    ) -> Result<(), ActionError> {
        self.player(id)?;
        if delay <= 0 {
            return Err(ActionError::OutOfRange);
        }
        self.players
            .get_mut(id.slot as usize)
            .unwrap()
            .queues
            .add(
                1,
                crate::queues::Category::Soft,
                delay,
                Deferred::GameframeMove(movement),
            )
            .map_err(|_| ActionError::QueueFull)
    }
    pub fn select_combat_stance(
        &mut self,
        id: ActorId,
        stance: u8,
    ) -> Result<Option<Vec<crate::variables::Update>>, ActionError> {
        self.player(id)?;
        if stance > 3 {
            return Err(ActionError::OutOfRange);
        }
        let config = self
            .combat_stance
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let player = self.players.get_mut(id.slot as usize).unwrap();
        if player.protection.access_protected(player.has_modal) {
            player.queues.remove_all(2);
            player
                .queues
                .add(
                    2,
                    crate::queues::Category::Normal,
                    1,
                    Deferred::CombatStance(stance),
                )
                .map_err(|_| ActionError::QueueFull)?;
            return Ok(None);
        }
        perform_stance(
            config,
            &mut player.data,
            &mut player.queues,
            stance,
            player.protection.processed_clock > 0,
        )
        .map(Some)
    }
    pub fn toggle_auto_retaliate(
        &mut self,
        id: ActorId,
    ) -> Result<Option<crate::variables::Update>, ActionError> {
        self.player(id)?;
        let config = self
            .combat_stance
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let player = self.players.get_mut(id.slot as usize).unwrap();
        if player.protection.access_protected(player.has_modal) {
            return Ok(None);
        }
        Ok(config.toggle_retaliate(
            &mut player.data.variables,
            player.protection.processed_clock > 0,
        ))
    }
    pub fn bulwark_combat_delayed(&self, id: ActorId) -> Result<bool, ActionError> {
        Ok(self.player(id)?.queues.contains(3))
    }
    pub fn cancel_count_dialog(&mut self, id: ActorId) -> Result<bool, ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        let canceled = p.bank_count.take().is_some();
        if canceled {
            p.protection.suspended = false;
        }
        Ok(canceled)
    }
    /// Player.ifClose cancels dialog suspensions and weak queues before closing modals.
    /// Return whether the client needs TriggerOnDialogAbort; ordinary delays survive.
    pub fn close_dialogs(&mut self, id: ActorId) -> Result<bool, ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        let count = p.bank_count.take().is_some();
        let pause = p.pending_drop.take().is_some();
        let prayer = matches!(p.prayer, Some(crate::prayer::State::Confirm { .. }));
        let notice = p.prayer_notice.take().is_some();
        if prayer {
            p.prayer = None;
        }
        p.queues.clear_weak();
        if count || pause || prayer || notice {
            p.protection.suspended = false;
        }
        Ok(count || pause || prayer || notice)
    }
    /// Trusted content delay; network input cannot choose its own protection state.
    pub fn delay_player(&mut self, id: ActorId, cycles: i32) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        p.protection.current_clock = self.clock.0;
        p.protection.delay(cycles);
        Ok(())
    }
    pub fn set_run_weight(&mut self, id: ActorId, grams: i32) -> Result<(), ActionError> {
        let player = self
            .players
            .get_mut(id.slot as usize)
            .filter(|player| player.id == id)
            .ok_or(ActionError::StaleActor)?;
        if player.saving {
            return Err(ActionError::LoggingOut);
        }
        player.run.weight_grams = grams;
        Ok(())
    }
    /// Transmission subscriptions decide which inventories contribute to weight.
    pub fn set_weight_sources(
        &mut self,
        id: ActorId,
        sources: Vec<WeightSource>,
    ) -> Result<(), ActionError> {
        self.player(id)?;
        self.players
            .get_mut(id.slot as usize)
            .unwrap()
            .weight_sources = Some(sources);
        Ok(())
    }
    pub fn submit(&mut self, id: ActorId, action: Action) -> Result<(), ActionError> {
        let p = self.player(id)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if self.actions.len() >= 4096 {
            return Err(ActionError::QueueFull);
        }
        self.actions.push_back((id, action));
        Ok(())
    }
    /// Preserve the actor for Kotlin's reconnect grace after a transport loss.
    pub fn client_disconnected(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        p.disconnect_grace.disconnect();
        p.movement.abort();
        Ok(())
    }
    pub fn set_music_resizable(&mut self, id: ActorId, resizable: bool) -> Result<(), ActionError> {
        self.player(id)?;
        self.players
            .get_mut(id.slot as usize)
            .unwrap()
            .music_resizable = resizable;
        Ok(())
    }
    /// Shutdown bypasses access protection as well as disconnect grace, while
    /// retaining required queues and area exits before the durable save.
    pub fn request_shutdown(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        self.players
            .get_mut(id.slot as usize)
            .unwrap()
            .protection
            .pending_shutdown = true;
        self.request_logout(id)
    }
    /// Host acknowledgement after UI close hooks have queued their gameplay work.
    pub fn logout_modals_closed(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        self.players
            .get_mut(id.slot as usize)
            .unwrap()
            .logout_modal_close_pending = false;
        Ok(())
    }
    /// Explicit logout bypasses socket grace but still drains Kotlin's save barriers.
    pub fn request_logout(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        if p.saving {
            return Ok(());
        }
        p.pending_logout = true;
        p.movement.abort();
        self.process_pending_logout(id)
    }
    fn process_pending_logout(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        if !p.pending_logout || p.saving {
            return Ok(());
        }
        let closing_modal = p.has_modal;
        if closing_modal {
            p.has_modal = false;
            p.logout_modal_close_pending = true;
            self.events.push(WorldEvent::CloseModals { actor: id });
        }
        p.pending_areas.extend(p.areas.process(
            p.movement.coords,
            p.movement.coords,
            true,
            &self.areas,
        ));
        // The host handles CloseModals (including bank close queues) before the
        // next tick. Never take the save snapshot ahead of those callbacks.
        if p.logout_modal_close_pending
            || p.protection.access_protected(false)
            || !p.pending_areas.is_empty()
            || p.queues.has_non_discardable()
        {
            return Ok(());
        }
        p.pending_logout = false;
        p.saving = true;
        p.movement.abort();
        p.data.coords = p.movement.coords;
        self.events.push(WorldEvent::Save {
            actor: id,
            data: p.data.clone(),
        });
        Ok(())
    }
    /// Called after durable save succeeds. Keep the account reserved on save failure.
    pub fn saved(&mut self, id: ActorId) -> Result<(), ActionError> {
        if !self.player(id)?.saving {
            return Err(ActionError::LoggingOut);
        }
        if !self.saved.contains(&id) {
            self.saved.push_back(id);
        }
        Ok(())
    }
    pub fn drain_events(&mut self) -> Vec<WorldEvent> {
        std::mem::take(&mut self.events)
    }
    /// Inject an externally controlled permutation to reproduce Kotlin shuffle fixtures.
    pub fn set_player_order(&mut self, slots: Vec<usize>) -> Result<(), ActionError> {
        let expected: HashSet<_> = self.players.iter().map(|(s, _)| s).collect();
        let actual: HashSet<_> = slots.iter().copied().collect();
        if expected != actual || actual.len() != slots.len() {
            return Err(ActionError::StaleActor);
        }
        self.order = slots;
        Ok(())
    }
    fn worn_changed(&mut self, id: ActorId, slot: usize) {
        self.update_prayer_resistance(id);
        if let (Some(config), Some(stance)) = (&self.combat_wear, &self.combat_stance) {
            let player = self.players.get_mut(id.slot as usize).unwrap();
            let result = config.changed(
                &mut player.data,
                stance,
                slot,
                player.combat_decimal,
                player.protection.processed_clock > 0,
            );
            self.events
                .push(WorldEvent::WornEffects { actor: id, result });
        }
    }
    fn action(
        &mut self,
        id: ActorId,
        action: Action,
        random: &mut impl RandomSource,
    ) -> Result<(), ActionError> {
        let p = self.player(id)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        match action {
            Action::BurySlot { slot } => return self.start_bury(id, slot),
            Action::ResumeBury { token, choice } => return self.resume_bury(id, token, choice),
            Action::ResumeDialogue { token, choice } => {
                return self.resume_dialogue(id, token, choice, random)
            }
            Action::OperateNpc {
                target,
                operation,
                control,
            } => return self.request_npc(id, target, operation, control),
            Action::DropSlot { slot } => return self.start_drop(id, slot),
            Action::ResumeDrop { token, choice } => return self.resume_drop(id, token, choice),
            Action::TakeObject {
                coords,
                id: object_id,
                control,
            } => {
                return self.request_pickup(id, coords, object_id, control);
            }
            Action::OperateDoor {
                coords,
                id: location_id,
                control,
            } => {
                return self.request_door(id, coords, location_id, control);
            }
            Action::OperateLocation {
                coords,
                id: location_id,
                operation,
                control,
            } => {
                return self.request_location(id, coords, location_id, operation, control);
            }
            Action::ResumeBankCount { token, count } => {
                if p.bank_count.is_none_or(|pending| pending.0 != token) {
                    return Ok(());
                }
                let config = self.banking.as_ref().ok_or(ActionError::InvalidBank)?;
                let p = self.players.get_mut(id.slot as usize).unwrap();
                let (_, purpose) = p.bank_count.take().unwrap();
                p.protection.suspended = false;
                if p.protection.delayed() {
                    return Err(ActionError::Delayed);
                }
                let result = config.resume_count(&mut p.data, &self.object_types, purpose, count);
                if let Ok(events) = &result {
                    for event in events {
                        if let crate::bank_actions::Event::WornDeposited { slot, .. } = event {
                            self.worn_changed(id, *slot);
                        }
                    }
                }
                self.events.push(WorldEvent::Bank { actor: id, result });
            }
            Action::Bank(request) => {
                if p.protection.modal_button_protected() {
                    return Err(ActionError::Protected);
                }
                let config = self.banking.as_ref().ok_or(ActionError::InvalidBank)?;
                let p = self.players.get_mut(id.slot as usize).unwrap();
                let result = config.apply(&mut p.data, &self.object_types, request);
                if let Ok(events) = &result {
                    for event in events {
                        if let crate::bank_actions::Event::QuantityInput(purpose) = event {
                            p.dialog_generation = p.dialog_generation.wrapping_add(1);
                            p.bank_count = Some((p.dialog_generation, *purpose));
                            p.protection.suspended = true;
                        }
                    }
                }
                if let Ok(events) = &result {
                    for event in events {
                        if let crate::bank_actions::Event::WornDeposited { slot, .. } = event {
                            self.worn_changed(id, *slot);
                        }
                    }
                }
                self.events.push(WorldEvent::Bank { actor: id, result });
            }
            Action::EquipSlot { slot } | Action::UnequipSlot { slot } => {
                let unequipping = matches!(action, Action::UnequipSlot { .. });
                let p = self.players.get_mut(id.slot as usize).unwrap();
                if !unequipping {
                    p.begin_held_action();
                }
                let blocked = if unequipping {
                    p.protection.access_protected(false)
                } else {
                    p.protection.delayed()
                };
                let inventory = if unequipping {
                    &mut p.data.equipment
                } else {
                    &mut p.data.inventory
                };
                let item = inventory.get(slot);
                if blocked || item.is_none() {
                    if !inventory.slots().is_empty() {
                        let first = inventory.get(0);
                        let _ = inventory.restore_slot(0, first);
                    }
                    return Err(if blocked {
                        ActionError::Protected
                    } else {
                        ActionError::InvalidItem
                    });
                }
                let item = item.unwrap().id;
                let result = if unequipping {
                    crate::equipment::unequip(
                        &mut p.data.equipment,
                        &mut p.data.inventory,
                        slot,
                        &self.object_types,
                    )
                } else {
                    crate::equipment::equip(
                        &mut p.data.inventory,
                        &mut p.data.equipment,
                        slot,
                        &self.object_types,
                        &self.wear_types,
                        &p.data.stats,
                    )
                };
                if let Ok(events) = &result {
                    for event in events {
                        if let crate::equipment::Event::Changed { slot, .. } = event {
                            self.worn_changed(id, *slot);
                        }
                    }
                }
                self.events.push(WorldEvent::EquipmentResult {
                    actor: id,
                    item,
                    unequipping,
                    result,
                });
            }
            Action::MoveInventorySlot { from, into } => {
                let p = self.players.get_mut(id.slot as usize).unwrap();
                if p.protection.access_protected(false) {
                    if !p.data.inventory.slots().is_empty() {
                        let first = p.data.inventory.get(0);
                        let _ = p.data.inventory.restore_slot(0, first);
                    }
                    return Err(ActionError::Protected);
                }
                p.data
                    .inventory
                    .move_slot(&self.object_types, from, into)
                    .map_err(|_| ActionError::InvalidItem)?;
            }
            Action::DrinkSlot {
                slot,
                item: expected,
                empty,
            } => {
                let p = self.players.get_mut(id.slot as usize).unwrap();
                p.begin_held_action();
                let item = p.data.inventory.get(slot);
                let error = if p.protection.delayed() {
                    Some(ActionError::Delayed)
                } else if item
                    .is_none_or(|item| item.id != expected || !self.potions.contains_key(&item.id))
                {
                    Some(ActionError::InvalidItem)
                } else {
                    None
                };
                if let Some(error) = error {
                    if !p.data.inventory.slots().is_empty() {
                        let first = p.data.inventory.get(0);
                        let _ = p.data.inventory.restore_slot(0, first);
                    }
                    return Err(error);
                }
                let potion = &self.potions[&expected];
                let mut emitted = Vec::new();
                crate::potions::consume(
                    potion,
                    empty,
                    &mut p.data.inventory,
                    &mut p.data.stats,
                    &mut p.data.energy,
                    &self.object_types,
                    self.eating_animation,
                    |e| emitted.push(e),
                )
                .map_err(|_| ActionError::InvalidStat)?;
                for event in emitted {
                    if let crate::potions::Event::Delay(cycles) = event {
                        p.protection.delay(cycles);
                        p.protection.suspended = true;
                        p.food_delay = true;
                    }
                    self.events.push(WorldEvent::Potion { actor: id, event });
                }
            }
            Action::EatSlot { slot } => {
                let p = self.players.get_mut(id.slot as usize).unwrap();
                // HeldOpScript.launchLenient cancels the previous coroutine even
                // when objectVerify subsequently rejects the delayed operation.
                p.begin_held_action();
                let item = p.data.inventory.get(slot);
                let error = if p.protection.delayed() {
                    Some(ActionError::Delayed)
                } else if item.is_none_or(|item| !self.foods.contains_key(&item.id)) {
                    Some(ActionError::InvalidItem)
                } else {
                    None
                };
                if let Some(error) = error {
                    if !p.data.inventory.slots().is_empty() {
                        let first = p.data.inventory.get(0);
                        let _ = p.data.inventory.restore_slot(0, first);
                    }
                    return Err(error);
                }
                let food = &self.foods[&item.unwrap().id];
                let mut emitted = Vec::new();
                crate::food::eat(
                    food,
                    &mut p.data.inventory,
                    &mut p.data.stats,
                    &self.object_types,
                    3,
                    self.eating_animation,
                    |event| emitted.push(event),
                )
                .map_err(|_| ActionError::InvalidStat)?;
                for event in emitted {
                    if let crate::food::Event::Delay(cycles) = event {
                        p.protection.delay(cycles);
                        p.protection.suspended = true;
                        p.food_delay = true;
                    }
                    self.events.push(WorldEvent::Food { actor: id, event });
                }
            }
            Action::SetRunning(run) => {
                let p = self.players.get_mut(id.slot as usize).unwrap();
                if p.data.energy < 100 {
                    return Err(ActionError::InsufficientEnergy);
                }
                let speed = if run { Speed::Run } else { Speed::Walk };
                p.set_active_speed(speed);
            }
            Action::Walk { destination, .. } | Action::MoveClick { destination, .. } => {
                if p.protection.delayed() {
                    return Err(ActionError::Delayed);
                }
                if p.movement.rooted {
                    return Err(ActionError::Rooted);
                }
                if destination.level() != p.movement.coords.level() {
                    return Err(ActionError::WrongPlane);
                }
                if (destination.x() - p.movement.coords.x()).abs() > 64
                    || (destination.z() - p.movement.coords.z()).abs() > 64
                {
                    return Err(ActionError::OutOfRange);
                }
                let route = coordinate_route(
                    &self.collision,
                    p.movement.coords,
                    destination,
                    p.movement.size,
                    true,
                    25,
                    Strategy::Normal,
                );
                let p = self.players.get_mut(id.slot as usize).unwrap();
                let temporary = match action {
                    Action::Walk { run, .. } => Some(if run && p.data.energy >= 100 {
                        Speed::Run
                    } else {
                        Speed::Walk
                    }),
                    Action::MoveClick { control: true, .. } => {
                        Some(if p.active_speed == Speed::Run || p.data.energy < 100 {
                            Speed::Walk
                        } else {
                            Speed::Run
                        })
                    }
                    _ => None,
                };
                p.movement.temporary_speed = temporary;
                p.movement.interacting = false;
                p.location_interaction = None;
                p.object_interaction = None;
                p.npc_interaction = None;
                if p.prayer.take().is_some() {
                    p.protection.suspended = false;
                    self.events.push(WorldEvent::Prayer {
                        actor: id,
                        event: crate::prayer::Event::Closed,
                    });
                }
                if p.pending_drop.take().is_some() {
                    p.protection.suspended = false;
                    self.events.push(WorldEvent::Drop {
                        actor: id,
                        event: crate::drop_actions::Event::Closed,
                    });
                }
                if p.dialogue.take().is_some() {
                    p.protection.suspended = false;
                    self.events.push(WorldEvent::Dialogue {
                        actor: id,
                        event: crate::dialogue::Event::Closed,
                    });
                }
                p.movement.route(route.waypoints, p.active_speed);
            }
        }
        Ok(())
    }
    pub fn tick(&mut self) -> Result<(), ClockExhausted> {
        let mut random = self.random;
        let result = self.tick_with_random(&mut random);
        self.random = random;
        result
    }
    pub fn tick_with_random(
        &mut self,
        random: &mut impl RandomSource,
    ) -> Result<(), ClockExhausted> {
        if self.clock.0 == i32::MAX {
            return Err(ClockExhausted);
        }
        for phase in PHASES {
            if phase == Phase::Late {
                self.clock.tick()?;
            }
            self.events.push(WorldEvent::Phase(phase, self.clock.0));
            match phase {
                Phase::WorldMain => {
                    if let Err(error) = self.objects.process_delayed(self.clock, &self.object_types)
                    {
                        self.events.push(WorldEvent::GroundError(error));
                    }
                }
                Phase::NpcMain => {
                    let mut failed = vec![];
                    let ids: Vec<_> = self.npcs.iter().map(|(_, entry)| entry.id).collect();
                    for id in ids {
                        if self.process_npc_death(id) {
                            continue;
                        }
                        let entry = self.npcs.get_mut(id.slot as usize).unwrap();
                        let npc = &mut entry.npc;
                        if !npc.visible || npc.delayed {
                            continue;
                        }
                        npc.regenerate();
                        for hit in entry.hits.process(self.clock.0, false) {
                            let impact = hit.impact(npc.stats[3], true);
                            npc.stats[3] = impact.health;
                            let target = crate::hits::Target::Npc(entry.id);
                            self.events.push(WorldEvent::Hit { target, impact });
                            if let Some(crate::hits::Target::Player(actor)) = hit.source {
                                if self
                                    .players
                                    .get(actor.slot as usize)
                                    .is_some_and(|p| p.id == actor)
                                {
                                    if let Some(sound) = self
                                        .melee
                                        .as_ref()
                                        .and_then(|c| c.npc_defend_sounds.get(&npc.type_id))
                                        .copied()
                                    {
                                        self.events.push(WorldEvent::CombatSound { actor, sound });
                                    }
                                }
                            }
                            if impact.death && !entry.death_queued {
                                entry.death_queued = true;
                                self.events.push(WorldEvent::DeathRequested(target));
                            }
                        }
                        if self.process_npc_death(id) {
                            continue;
                        }
                        if self.process_npc_combat(id, random) {
                            continue;
                        }
                        if self.process_dialogue_face(id) {
                            self.npcs
                                .get_mut(id.slot as usize)
                                .unwrap()
                                .npc
                                .process_movement(&mut self.collision, self.clock.0);
                            continue;
                        }
                        let entry = self.npcs.get_mut(id.slot as usize).unwrap();
                        let npc = &mut entry.npc;
                        let mode_result = match npc.mode {
                            Mode::None => Ok(()),
                            Mode::Wander => npc.wander(&mut self.collision, self.clock.0, random),
                            Mode::Patrol => npc.patrol(&mut self.collision, self.clock.0),
                        };
                        if mode_result.is_err() {
                            failed.push(entry.id);
                            continue;
                        }
                        npc.process_movement(&mut self.collision, self.clock.0);
                    }
                    for id in failed {
                        self.despawn_npc(id);
                    }
                }
                Phase::WorldPostTick => {
                    for (loc, add) in self.location_timers.due(self.clock) {
                        if let Err(error) = self.change_location(loc, add) {
                            self.events.push(WorldEvent::LocationError(error));
                        }
                    }
                    self.events.push(WorldEvent::SynchronizeNpcs {
                        tick: self.clock.0,
                        npcs: self
                            .npcs
                            .iter()
                            .filter(|(_, e)| e.npc.visible)
                            .map(|(_, e)| NpcSnapshot {
                                actor: e.id,
                                type_id: e.npc.type_id,
                                movement: MovementSnapshot::from(&e.npc.movement),
                                direction: e.npc.respawn_direction,
                                face_player: e.npc.face_player,
                                face_changed: e.npc.face_changed,
                                allocated_cycle: e.allocated_cycle,
                                allocated_this_tick: e.allocated_pending,
                            })
                            .collect(),
                    });
                    for (_, entry) in self.npcs.iter_mut() {
                        entry.npc.movement.finish_tick();
                        entry.npc.face_changed = false;
                        entry.allocated_pending = false;
                    }
                    if let Err(error) = self
                        .objects
                        .process_durations(self.clock, &self.object_types)
                    {
                        self.events.push(WorldEvent::GroundError(error));
                    }
                    self.events.extend(
                        self.objects
                            .drain_events()
                            .into_iter()
                            .map(WorldEvent::Ground),
                    );
                }
                Phase::PlayerInput => {
                    for (_, player) in self.players.iter_mut() {
                        if !player.saving {
                            player.protection.current_clock = self.clock.0;
                        }
                    }
                    while let Some((actor, action)) = self.actions.pop_front() {
                        if let Err(error) = self.action(actor, action, random) {
                            self.events
                                .push(WorldEvent::ActionRejected { actor, error });
                        }
                    }
                }
                Phase::PlayerMain => {
                    let alive_npcs: HashSet<_> = self
                        .npcs
                        .iter()
                        .filter(|(_, e)| e.npc.stats[3] > 0)
                        .map(|(_, e)| e.id)
                        .collect();
                    for slot in self.order.clone() {
                        self.process_tower_arrival(slot);
                        self.process_prayer(slot);
                        self.resume_door(slot);
                        self.resume_pickup(slot);
                        self.validate_door(slot);
                        self.validate_pickup(slot);
                        let Some(p) = self.players.get_mut(slot) else {
                            continue;
                        };
                        if !p.saving {
                            p.protection.processed_clock = self.clock.0;
                            if p.food_delay && !p.protection.delayed() {
                                p.food_delay = false;
                                p.protection.suspended = false;
                            }
                            let mut access = crate::queues::Access {
                                protected: p.protection.access_protected(p.has_modal),
                                pending_logout: p.pending_logout,
                            };
                            let mut pending_queues = std::mem::take(&mut p.queues);
                            if pending_queues
                                .process(
                                    self.clock.0,
                                    &mut access,
                                    1024,
                                    |queues, access, event| {
                                        if matches!(&event, crate::queues::Event::CloseModals) {
                                            if p.has_modal {
                                                p.logout_modal_close_pending = true;
                                            }
                                            let count = p.bank_count.take().is_some();
                                            let drop = p.pending_drop.take().is_some();
                                            let notice = p.prayer_notice.take().is_some();
                                            let dialogue = p.dialogue.take().is_some();
                                            let prayer = matches!(
                                                p.prayer,
                                                Some(crate::prayer::State::Confirm { .. })
                                            );
                                            if prayer {
                                                p.prayer = None;
                                            }
                                            if count || drop || notice || dialogue || prayer {
                                                p.protection.suspended = false;
                                            }
                                            p.has_modal = false;
                                            queues.clear_weak();
                                            self.events
                                                .push(WorldEvent::CloseModals { actor: p.id });
                                        }
                                        if let crate::queues::Event::Launch(queue) = event {
                                            match queue.args {
                                                Deferred::PrayerToggle(id) => {
                                                    if let Some(config) = &self.prayer_book {
                                                        if let Err(error) =
                                                            prayer_book_actions::toggle(
                                                                config,
                                                                p,
                                                                queues,
                                                                id,
                                                                self.clock.0,
                                                                &mut self.events,
                                                            )
                                                        {
                                                            self.events.push(
                                                                WorldEvent::ActionRejected {
                                                                    actor: p.id,
                                                                    error,
                                                                },
                                                            );
                                                        }
                                                    }
                                                }
                                                Deferred::Preserve => {
                                                    if let Some(config) = &self.regeneration {
                                                        config.activate_preserve(
                                                            &mut p.soft_timers,
                                                            self.clock.0,
                                                        );
                                                    }
                                                }
                                                Deferred::GameframeMove(movement) => {
                                                    self.events.push(WorldEvent::GameframeMove {
                                                        actor: p.id,
                                                        movement,
                                                    })
                                                }
                                                Deferred::CombatStance(stance) => {
                                                    if let Some(config) = &self.combat_stance {
                                                        match perform_stance(
                                                            config,
                                                            &mut p.data,
                                                            queues,
                                                            stance,
                                                            p.protection.processed_clock > 0,
                                                        ) {
                                                            Ok(updates) => self.events.push(
                                                                WorldEvent::CombatVariables {
                                                                    actor: p.id,
                                                                    updates,
                                                                },
                                                            ),
                                                            Err(error) => self.events.push(
                                                                WorldEvent::ActionRejected {
                                                                    actor: p.id,
                                                                    error,
                                                                },
                                                            ),
                                                        }
                                                    }
                                                }
                                                Deferred::BulwarkDelay => {}
                                                Deferred::AutocastOpen(defensive) => {
                                                    if let Some(config) = &self.combat_wear {
                                                        // Normal protected queue launch clears the old interaction.
                                                        p.location_interaction = None;
                                                        p.object_interaction = None;
                                                        p.npc_interaction = None;
                                                        p.movement.interacting = false;
                                                        p.protection.suspended = false;
                                                        p.food_delay = false;
                                                        p.pending_door = None;
                                                        p.prayer = None;
                                                        p.pending_pickup = None;
                                                        p.pending_drop = None;
                                                        p.dialogue = None;
                                                        if let Some(weapon) =
                                                            autocast_actions::open_picker(
                                                                &p.data,
                                                                &mut p.autocast_picker_defensive,
                                                                config,
                                                                defensive,
                                                            )
                                                        {
                                                            self.events.push(
                                                                WorldEvent::AutocastPickerOpen {
                                                                    actor: p.id,
                                                                    weapon,
                                                                },
                                                            );
                                                        }
                                                    }
                                                }
                                                Deferred::BankCompression => {
                                                    if let Some(config) = &self.banking {
                                                        let result = config.apply(
                                                            &mut p.data,
                                                            &self.object_types,
                                                            crate::bank_actions::Request::Compress,
                                                        );
                                                        self.events.push(WorldEvent::Bank {
                                                            actor: p.id,
                                                            result,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                        access.protected =
                                            p.protection.access_protected(p.has_modal);
                                    },
                                )
                                .is_err()
                            {
                                self.events.push(WorldEvent::ActionRejected {
                                    actor: p.id,
                                    error: ActionError::QueueFull,
                                });
                            }
                            p.queues = pending_queues;
                            let mut main_failed = false;
                            let mut soft_timers = std::mem::take(&mut p.soft_timers);
                            soft_timers.process(self.clock.0, |timers, timer| {
                                if let Some(config) = &self.music {
                                    if timer == config.timer {
                                        let context = crate::music_player::Context {
                                            logged_in: p.protection.processed_clock > 0,
                                            resizable: p.music_resizable,
                                        };
                                        match config.tick(&mut p.data.variables, context, random) {
                                            Ok(events) => self.events.push(WorldEvent::Music {
                                                actor: p.id,
                                                events,
                                            }),
                                            Err(failure) => {
                                                self.events.push(WorldEvent::Music {
                                                    actor: p.id,
                                                    events: failure.events,
                                                });
                                                self.events.push(WorldEvent::MusicError {
                                                    actor: p.id,
                                                    reason: failure.reason,
                                                });
                                                main_failed = true;
                                                return false;
                                            }
                                        }
                                    }
                                }
                                if let Some(config) = &self.session_vars {
                                    match config.tick(
                                        timer,
                                        &mut p.data.variables,
                                        self.clock.0,
                                        self.unix_millis,
                                    ) {
                                        Ok(updates) if !updates.is_empty() => {
                                            self.events.push(WorldEvent::SessionVariables {
                                                actor: p.id,
                                                updates,
                                            })
                                        }
                                        Ok(_) => {}
                                        Err(_) => {
                                            timers.remove(timer);
                                            self.events.push(WorldEvent::ActionRejected {
                                                actor: p.id,
                                                error: ActionError::InvalidStat,
                                            });
                                        }
                                    }
                                }
                                if let Some(config) = &self.prayer_book {
                                    if timer == config.drain_timer {
                                        let result = config.tick(
                                            &mut p.data.variables,
                                            &mut p.data.stats,
                                            p.overhead,
                                            timers,
                                            p.protection.processed_clock > 0,
                                        );
                                        match result {
                                            Ok(output) => {
                                                let mut queues = std::mem::take(&mut p.queues);
                                                if let Err(error) = prayer_book_actions::publish(
                                                    p,
                                                    &mut queues,
                                                    output,
                                                    &mut self.events,
                                                ) {
                                                    self.events.push(WorldEvent::ActionRejected {
                                                        actor: p.id,
                                                        error,
                                                    });
                                                }
                                                p.queues = queues;
                                            }
                                            Err(_) => {
                                                timers.remove(timer);
                                                self.events.push(WorldEvent::ActionRejected {
                                                    actor: p.id,
                                                    error: ActionError::InvalidStat,
                                                });
                                            }
                                        }
                                    }
                                }
                                if let Some(config) = &self.regeneration {
                                    for update in config.tick(
                                        timer,
                                        &mut p.data.stats,
                                        p.data.equipment.get(9).map(|item| item.id),
                                    ) {
                                        self.events.push(WorldEvent::Stat {
                                            actor: p.id,
                                            update,
                                        });
                                    }
                                }
                                true
                            });
                            p.soft_timers = soft_timers;
                            if main_failed {
                                // PlayerMainProcess.tryOrDisconnect stops the main
                                // pass; forced logout bypasses socket grace, but
                                // still waits for protected work and close hooks.
                                p.pending_logout = true;
                                p.movement.abort();
                                continue;
                            }
                            let normalized = self.area_sources.normalize(p.movement.coords);
                            p.pending_areas.extend(p.areas.process(
                                p.movement.coords,
                                normalized,
                                p.pending_logout || p.protection.pending_shutdown,
                                &self.areas,
                            ));
                            if !p.protection.access_protected(p.has_modal) {
                                while let Some(event) = p.pending_areas.pop_front() {
                                    self.events.push(WorldEvent::Area { actor: p.id, event });
                                    if let (
                                        Some(config),
                                        crate::area_tracking::Event::Enter(area),
                                    ) = (&self.music, event)
                                    {
                                        if config.catalog.modern.contains_key(&area)
                                            || config.catalog.classic.contains_key(&area)
                                        {
                                            let context = crate::music_player::Context {
                                                logged_in: p.protection.processed_clock > 0,
                                                resizable: p.music_resizable,
                                            };
                                            match config.enter(
                                                &mut p.data.variables,
                                                context,
                                                area,
                                                random,
                                            ) {
                                                Ok(events) => self.events.push(WorldEvent::Music {
                                                    actor: p.id,
                                                    events,
                                                }),
                                                Err(failure) => {
                                                    self.events.push(WorldEvent::Music {
                                                        actor: p.id,
                                                        events: failure.events,
                                                    });
                                                    self.events.push(WorldEvent::MusicError {
                                                        actor: p.id,
                                                        reason: failure.reason,
                                                    });
                                                    main_failed = true;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if main_failed {
                                p.pending_logout = true;
                                p.movement.abort();
                                continue;
                            }
                            for hit in p.hits.process(self.clock.0, false) {
                                if !hit.valid_player_source(|id| alive_npcs.contains(&id)) {
                                    continue;
                                }
                                let impact =
                                    hit.impact(p.data.stats.current(3) as u8 as i32, false);
                                p.data.stats.set_current(3, impact.health as i8);
                                p.logout_blocked_until = self.clock.deadline(16);
                                let target = crate::hits::Target::Player(p.id);
                                self.events.push(WorldEvent::Hit { target, impact });
                                if impact.death && !p.death_queued {
                                    p.death_queued = true;
                                    self.events.push(WorldEvent::DeathRequested(target));
                                }
                            }
                            let actor = p.id;
                            if p.pending_logout {
                                // PlayerMainProcess clears interactions while fake-logged.
                                p.npc_interaction = None;
                                p.location_interaction = None;
                                p.object_interaction = None;
                                p.movement.abort();
                                // Its final ifClose disconnect phase cancels dialog
                                // suspensions, but ordinary delays continue to expire.
                                let dialogue = p.dialogue.take().is_some();
                                let count = p.bank_count.take().is_some();
                                let drop = p.pending_drop.take().is_some();
                                let notice = p.prayer_notice.take().is_some();
                                let prayer =
                                    matches!(p.prayer, Some(crate::prayer::State::Confirm { .. }));
                                if prayer {
                                    p.prayer = None;
                                }
                                if dialogue || count || drop || notice || prayer {
                                    p.protection.suspended = false;
                                }
                                if p.has_modal {
                                    p.logout_modal_close_pending = true;
                                    self.events.push(WorldEvent::CloseModals { actor: p.id });
                                }
                                p.has_modal = false;
                                p.queues.clear_weak();
                                continue;
                            }
                            self.process_player_retaliation(actor);
                            let interacted = self.process_npc_interaction(slot, true, random);
                            let p = self.players.get_mut(slot).unwrap();
                            if !interacted {
                                p.movement.process(&mut self.collision, self.clock.0);
                            }
                            p.data.coords = p.movement.coords;
                            if !interacted {
                                self.process_npc_interaction(slot, false, random);
                            }
                        }
                        self.process_door(slot);
                        self.process_pickup(slot);
                        // PlayerMainProcess.ifClose runs throughout socket grace,
                        // before PlayerLogoutProcess decides whether to queue a save.
                        let close = self
                            .players
                            .get(slot)
                            .filter(|p| !p.saving && p.disconnect_grace.disconnected())
                            .map(|p| p.id);
                        if let Some(actor) = close {
                            self.close_dialogs(actor).expect("current player");
                            self.cancel_dialogue(actor).expect("current player");
                            let p = self.players.get_mut(slot).unwrap();
                            if p.has_modal {
                                p.has_modal = false;
                                p.logout_modal_close_pending = true;
                                self.events.push(WorldEvent::CloseModals { actor });
                            }
                        }
                    }
                }
                Phase::PlayerLogout => {
                    let due: Vec<_> = self
                        .players
                        .iter_mut()
                        .filter_map(|(_, p)| {
                            (!p.saving
                                && !p.pending_logout
                                && p.disconnect_grace
                                    .tick(self.clock.0, p.logout_blocked_until))
                            .then_some(p.id)
                        })
                        .collect();
                    for actor in due {
                        self.request_logout(actor).expect("current player");
                    }
                    let pending: Vec<_> = self
                        .players
                        .iter()
                        .filter_map(|(_, p)| p.pending_logout.then_some(p.id))
                        .collect();
                    for actor in pending {
                        self.process_pending_logout(actor).expect("current player");
                    }
                    for _ in 0..200 {
                        let Some(id) = self.saved.pop_front() else {
                            break;
                        };
                        if self.player(id).is_err() {
                            continue;
                        }
                        let p = self.players.remove(id.slot as usize).unwrap();
                        p.movement.occupancy(&mut self.collision, false);
                        self.accounts.remove(&p.data.account_id);
                        self.order.retain(|s| *s != id.slot as usize);
                        self.events.push(WorldEvent::Deleted(id));
                    }
                }
                Phase::PlayerLogin => {
                    for _ in 0..100 {
                        let Some((connection, mut data)) = self.logins.pop_front() else {
                            break;
                        };
                        self.reserved.remove(&data.account_id);
                        let Some(slot) = self.players.next_free() else {
                            self.events.push(WorldEvent::LoginRejected {
                                connection,
                                reason: AdmissionError::Full,
                            });
                            continue;
                        };
                        self.generation = self
                            .generation
                            .checked_add(1)
                            .expect("actor generation exhausted");
                        let id = ActorId {
                            slot: slot as u16,
                            generation: self.generation,
                        };
                        let movement = Movement::new(data.coords);
                        movement.occupancy(&mut self.collision, true);
                        let account = data.account_id;
                        let run = RunModifiers {
                            agility: data.stats.current(16) as u8 as i32,
                            ..Default::default()
                        };
                        let mut soft_timers = crate::player_timers::Timers::default();
                        // Kotlin Player.softTimer uses currentMapClock, still -1 at
                        // login. Input processing assigns the world clock next cycle.
                        let protection = crate::protected::State::default();
                        if let Some(config) = &self.login_resets {
                            config
                                .apply(&mut data.variables)
                                .expect("validated login reset varbit configuration");
                        }
                        if let Some(config) = &self.session_vars {
                            config
                                .login(
                                    &mut data.variables,
                                    &mut soft_timers,
                                    protection.current_clock,
                                )
                                .expect("validated session varbit configuration");
                        }
                        if let Some(config) = &self.regeneration {
                            config.login(&mut soft_timers, protection.current_clock);
                        }
                        if let Some(config) = &self.music {
                            soft_timers
                                .schedule(config.timer, protection.current_clock, 1)
                                .expect("positive interval");
                        }
                        let player = Player {
                            areas: Default::default(),
                            pending_areas: Default::default(),
                            music_resizable: false,
                            login_cycle: self.clock.0,
                            aggressive_npc: None,
                            last_npc_combat: -1000,
                            last_pvp_combat: -1000,
                            retaliate_npcs: Default::default(),
                            combat_decimal: 0,
                            action_delay: 0,
                            id,
                            data,
                            movement,
                            active_speed: Speed::Walk,
                            saving: false,
                            pending_logout: false,
                            logout_modal_close_pending: false,
                            disconnect_grace: Default::default(),
                            run,
                            hits: crate::hits::Queue::default(),
                            death_queued: false,
                            logout_blocked_until: 0,
                            protection,
                            food_delay: false,
                            location_interaction: None,
                            pending_door: None,
                            observer_id: None,
                            autocast_picker_defensive: None,
                            loot_drop_duration: None,
                            object_interaction: None,
                            npc_interaction: None,
                            pending_pickup: None,
                            pending_drop: None,
                            prayer: None,
                            prayer_notice: None,
                            overhead: None,
                            dialogue: None,
                            bank_count: None,
                            dialog_generation: 0,
                            has_modal: false,
                            queues: crate::queues::Scheduler::default(),
                            soft_timers,
                            weight_sources: None,
                        };
                        assert!(self.players.insert(player).is_ok());
                        self.order.push(slot);
                        self.accounts.insert(account, id);
                        self.update_prayer_resistance(id);
                        self.events.push(WorldEvent::Initialized {
                            connection,
                            actor: id,
                        });
                        self.process_tower_arrival(slot);
                    }
                }
                Phase::PlayerPostTick => {
                    self.events.push(WorldEvent::SynchronizePlayers {
                        tick: self.clock.0,
                        players: self
                            .players
                            .iter()
                            .filter(|(_, p)| !p.saving)
                            .map(|(_, p)| (p.id, MovementSnapshot::from(&p.movement)))
                            .collect(),
                    });
                    for slot in &self.order {
                        let p = self.players.get_mut(*slot).unwrap();
                        if !p.saving {
                            if let Some(sources) = &p.weight_sources {
                                let mut grams = 0i32;
                                for source in sources {
                                    if let Some(inv) = source.inventory(&p.data) {
                                        for item in inv.slots().iter().flatten() {
                                            grams = grams.wrapping_add(
                                                self.item_weights
                                                    .get(&item.id)
                                                    .copied()
                                                    .unwrap_or(0),
                                            );
                                        }
                                    }
                                }
                                p.run.weight_grams = grams;
                            }
                            p.run.agility = p.data.stats.current(16) as u8 as i32;
                            p.data.energy =
                                update_energy(p.data.energy, p.movement.pending_steps, p.run);
                            if p.data.energy == 0 {
                                p.set_active_speed(Speed::Walk);
                            }
                        }
                        p.movement.finish_tick();
                    }
                    let ids: Vec<_> = self.players.iter().map(|(_, p)| p.id).collect();
                    for id in ids {
                        self.events
                            .push(WorldEvent::Observe(self.observe(id).unwrap()));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn admitted_logout(world: &mut World) -> ActorId {
        world.queue_login(1, account()).unwrap();
        world.tick().unwrap();
        world
            .drain_events()
            .into_iter()
            .find_map(|e| match e {
                WorldEvent::Initialized { actor, .. } => Some(actor),
                _ => None,
            })
            .unwrap()
    }
    #[test]
    fn logout_drains_area_exits_before_saving_and_keeps_account_reserved() {
        let mut world = World::default();
        world
            .areas
            .register_coord(account().coords, [7, 9, 7])
            .unwrap();
        let actor = admitted_logout(&mut world);
        world.tick().unwrap();
        world.drain_events();
        world.request_logout(actor).unwrap();
        world.request_logout(actor).unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        assert_eq!(
            world.submit(actor, Action::SetRunning(true)),
            Err(ActionError::LoggingOut)
        );
        assert_eq!(
            world.queue_login(2, account()),
            Err(AdmissionError::Duplicate)
        );
        world.tick().unwrap();
        let events = world.drain_events();
        let exits = events
            .iter()
            .filter_map(|e| match e {
                WorldEvent::Area {
                    event: crate::area_tracking::Event::Exit(id),
                    ..
                } => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(exits, [7, 9]);
        let save = events
            .iter()
            .position(|e| matches!(e, WorldEvent::Save { .. }))
            .unwrap();
        assert!(events[..save]
            .iter()
            .any(|e| matches!(e, WorldEvent::Area { .. })));
        world.tick().unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.saved(actor).unwrap();
        world.tick().unwrap();
        world.queue_login(3, account()).unwrap();
    }
    #[test]
    fn pending_logout_waits_for_normal_queue_and_delay_but_accelerates_long_queue() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        p.queues
            .add(
                99,
                crate::queues::Category::LongAccelerate,
                100,
                Deferred::BankCompression,
            )
            .unwrap();
        p.queues
            .add(
                98,
                crate::queues::Category::Normal,
                2,
                Deferred::BankCompression,
            )
            .unwrap();
        world.delay_player(actor, 3).unwrap();
        world.request_logout(actor).unwrap();
        for _ in 0..3 {
            assert!(!world
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::Save { .. })));
            world.tick().unwrap();
        }
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn shutdown_bypasses_delay_but_still_processes_required_queue() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        world.delay_player(actor, 1000).unwrap();
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        p.queues
            .add(
                99,
                crate::queues::Category::LongAccelerate,
                1000,
                Deferred::BankCompression,
            )
            .unwrap();
        world.request_shutdown(actor).unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn weak_and_long_discard_queues_do_not_block_logout() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        for category in [
            crate::queues::Category::Weak,
            crate::queues::Category::LongDiscard,
        ] {
            p.queues
                .add(99, category, 1000, Deferred::BankCompression)
                .unwrap();
        }
        world.request_logout(actor).unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn modal_opened_by_pending_work_is_closed_before_save() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        world.delay_player(actor, 2).unwrap();
        world.request_logout(actor).unwrap();
        world.set_modal_open(actor, true).unwrap();
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::CloseModals { .. })));
        for _ in 0..3 {
            world.tick().unwrap();
            assert!(!world
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::Save { .. })));
        }
        world.logout_modals_closed(actor).unwrap();
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn queue_initiated_modal_close_must_be_acknowledged_before_same_cycle_logout() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        world.set_modal_open(actor, true).unwrap();
        world
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .queues
            .add(
                99,
                crate::queues::Category::Strong,
                1,
                Deferred::BankCompression,
            )
            .unwrap();
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::CloseModals { .. })));
        world.request_logout(actor).unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.queue_bank_compression(actor).unwrap();
        world.logout_modals_closed(actor).unwrap();
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn socket_grace_closes_modal_before_logout_without_skipping_required_callbacks() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        world.set_modal_open(actor, true).unwrap();
        world.client_disconnected(actor).unwrap();
        world.tick().unwrap();
        let events = world.drain_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, WorldEvent::CloseModals { .. })));
        assert!(!events.iter().any(|e| matches!(e, WorldEvent::Save { .. })));
        world.queue_bank_compression(actor).unwrap();
        world.logout_modals_closed(actor).unwrap();
        for _ in 0..15 {
            world.tick().unwrap();
            assert!(!world
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::Save { .. })));
        }
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn logout_waits_for_host_modal_close_callbacks_and_their_queue() {
        let mut world = World::default();
        let actor = admitted_logout(&mut world);
        world.set_modal_open(actor, true).unwrap();
        world.request_logout(actor).unwrap();
        world.request_logout(actor).unwrap();
        let events = world.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::CloseModals { .. }))
                .count(),
            1
        );
        assert!(!events.iter().any(|e| matches!(e, WorldEvent::Save { .. })));
        world.tick().unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        // Runtime close_interface queues BankInvScript compression before ack.
        world.queue_bank_compression(actor).unwrap();
        world.logout_modals_closed(actor).unwrap();
        world.request_logout(actor).unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
    }
    #[test]
    fn music_exception_preserves_effects_stops_timer_pass_and_forces_logout() {
        let (mut world, actor) = admitted();
        let config = crate::music_player::tests::config();
        let clock = world.clock.0;
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        for (varp, value) in [
            (config.mode, 1),
            (config.volume, 255),
            (config.playlist, 65281),
            (config.area.base, 10),
        ] {
            p.data.variables.set(varp, value, false);
        }
        p.soft_timers.schedule(config.timer, clock - 1, 1).unwrap();
        p.soft_timers.schedule(99, clock - 1, 1).unwrap();
        world.music = Some(config);
        world.drain_events();
        world.tick().unwrap();
        let events = world.drain_events();
        let music = events
            .iter()
            .position(|e| {
                matches!(e, WorldEvent::Music { events, .. }
            if events.iter().any(|e| matches!(e, crate::music::Event::Midi { .. })))
            })
            .unwrap();
        let error = events
            .iter()
            .position(|e| matches!(e, WorldEvent::MusicError { .. }))
            .unwrap();
        let save = events
            .iter()
            .position(|e| {
                matches!(e, WorldEvent::Save { data, .. }
            if data.variables.get(42) == 2 && data.variables.get(31) == 65281)
            })
            .unwrap();
        assert!(music < error && error < save);
        let p = world.players.get(actor.slot as usize).unwrap();
        assert!(p.saving);
        // A thrown callback leaves its own timer and later callbacks untouched.
        assert_eq!(p.soft_timers.get(3).unwrap().expiry, clock);
        assert_eq!(p.soft_timers.get(99).unwrap().expiry, clock);
        assert!(world.accounts.contains_key(&p.data.account_id));
    }
    #[test]
    fn music_area_hooks_wait_for_protected_access_and_persist_unlocks() {
        let mut world = World::default();
        let config = crate::music_player::tests::config();
        let mode = config.mode;
        world.music = Some(config);
        let data = account();
        world.areas.register_coord(data.coords, [9]).unwrap();
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world
            .drain_events()
            .into_iter()
            .find_map(|e| match e {
                WorldEvent::Initialized { actor, .. } => Some(actor),
                _ => None,
            })
            .unwrap();
        world.data_mut(actor).unwrap().variables.set(mode, 1, false);
        world.set_modal_open(actor, true).unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().variables.get(20), 0);
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Area { .. })));
        world.set_modal_open(actor, false).unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().variables.get(20), 6);
        let events = world.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(
                    e,
                    WorldEvent::Area {
                        event: crate::area_tracking::Event::Enter(9),
                        ..
                    }
                ))
                .count(),
            1
        );
        assert!(events.iter().any(|e| matches!(e, WorldEvent::Music { events, .. } if events.iter().any(|e| matches!(e, crate::music::Event::Midi {..})))));
        world.tick().unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Area { .. })));
        world.request_logout(actor).unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { data, .. } if data.variables.get(20) == 6)));
    }
    #[test]
    fn equipment_callbacks_capture_each_action_before_the_next_weapon_swap() {
        use crate::{
            autocast,
            bank_actions::Binding,
            combat_stance, combat_wear, magic, runes,
            variables::{Transmit, Varbit, Varp},
        };
        let (mut world, actor) = admitted();
        world.tick().unwrap();
        world.drain_events();
        let binding = |id| Binding {
            base: Varp {
                id,
                transmit: Transmit::Always,
                permanent: true,
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: id,
                lsb: 0,
                msb: 15,
            },
        };
        world.combat_stance = Some(combat_stance::Config {
            stance: binding(0).base,
            melee: binding(1).base,
            retaliate: binding(2).base,
            saved: BTreeMap::from([(18, binding(3)), (0, binding(4))]),
            styles: BTreeMap::from([(18, 0x4321), (0, 0x4321)]),
            weapons: BTreeMap::from([
                (
                    1,
                    combat_stance::Weapon {
                        category: 18,
                        bulwark: false,
                    },
                ),
                (
                    2,
                    combat_stance::Weapon {
                        category: 0,
                        bulwark: false,
                    },
                ),
            ]),
        });
        world.combat_wear = Some(combat_wear::Config {
            autocast: autocast::Config {
                enabled: binding(5),
                spell: binding(6),
                defensive: binding(7),
                staff: autocast::Saved {
                    spell: binding(8),
                    defensive: binding(9),
                },
                bladed: autocast::Saved {
                    spell: binding(10),
                    defensive: binding(11),
                },
                weapons: BTreeMap::from([
                    (
                        1,
                        autocast::Weapon {
                            category: 18,
                            additional: Default::default(),
                        },
                    ),
                    (
                        2,
                        autocast::Weapon {
                            category: 0,
                            additional: Default::default(),
                        },
                    ),
                ]),
                spells: BTreeMap::new(),
                autocast_items: BTreeMap::new(),
                restricted: Default::default(),
            },
            runes: magic::Config {
                catalog: runes::Catalog::default(),
                fake: [binding(14); 2],
                fountain: binding(14),
                blighted: binding(14),
                regular_pouches: Default::default(),
                divine_pouches: Default::default(),
                pouch_types: [binding(14); 4],
                pouch_counts: [(14, binding(14)); 4],
                lms_pouch: 999,
                packs: BTreeMap::new(),
                names: Default::default(),
            },
            level: binding(12),
            decimal: binding(13),
            combat_stats: [0, 1, 2, 3, 4, 6, 5],
        });
        for id in [1, 2] {
            world.wear_types.types.insert(
                id,
                crate::equipment::WearType {
                    primary: Some(3),
                    secondary: [None; 2],
                    requirements: vec![],
                },
            );
        }
        let data = world.data_mut(actor).unwrap();
        data.variables.set(binding(3).base, 2, false);
        data.variables.set(binding(4).base, 1, false);
        for (slot, id) in [(0, 1), (1, 2)] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(crate::inventory::Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        world.submit(actor, Action::EquipSlot { slot: 0 }).unwrap();
        world.submit(actor, Action::EquipSlot { slot: 1 }).unwrap();
        world.tick().unwrap();
        let updates: Vec<_> = world
            .drain_events()
            .into_iter()
            .filter_map(|event| match event {
                WorldEvent::WornEffects { result, .. } => Some(result.unwrap()),
                _ => None,
            })
            .collect();
        assert_eq!(updates.len(), 2);
        for (index, stance) in [2, 1].into_iter().enumerate() {
            let values: Vec<_> = updates[index]
                .iter()
                .map(|event| match event {
                    autocast::Event::Variable(u) => (u.id, u.value),
                    _ => panic!("unexpected error message"),
                })
                .collect();
            assert_eq!(
                &values[..6],
                &[
                    (0, stance),
                    (5, 0),
                    (6, 0),
                    (7, 0),
                    (0, stance),
                    (1, stance)
                ]
            );
            assert_eq!(values[6].0, 12);
            assert_eq!(values[7], (13, 0));
        }
        assert_eq!(world.data(actor).unwrap().variables.get(0), 1);
        assert_eq!(world.data(actor).unwrap().equipment.get(3).unwrap().id, 2);
    }
    #[test]
    fn latest_protected_stance_wins_retaliate_discards_and_bulwark_delay_expires() {
        use crate::{
            combat_stance::{Config, Weapon},
            variables::{Transmit, Varp},
        };
        let (mut world, actor) = admitted();
        world.tick().unwrap();
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        world.combat_stance = Some(Config {
            stance: varp(0),
            melee: varp(1),
            retaliate: varp(2),
            saved: BTreeMap::new(),
            styles: BTreeMap::from([(28, 0x4321)]),
            weapons: BTreeMap::from([(
                1,
                Weapon {
                    category: 28,
                    bulwark: true,
                },
            )]),
        });
        let data = world.data_mut(actor).unwrap();
        data.variables.set(varp(0), 3, false);
        data.equipment
            .restore_slot(
                3,
                Some(crate::inventory::Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        world.delay_player(actor, 3).unwrap();
        assert!(world.select_combat_stance(actor, 0).unwrap().is_none());
        assert!(world.select_combat_stance(actor, 2).unwrap().is_none());
        assert!(world.toggle_auto_retaliate(actor).unwrap().is_none());
        assert_eq!(world.data(actor).unwrap().variables.get(2), 0);
        assert_eq!(
            world
                .player(actor)
                .unwrap()
                .queues
                .snapshot()
                .iter()
                .filter(|q| q.id == 2)
                .count(),
            1
        );
        let mut fired = false;
        for _ in 0..8 {
            world.tick().unwrap();
            if world
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::CombatVariables { .. }))
            {
                fired = true;
                break;
            }
        }
        assert!(fired);
        assert_eq!(world.data(actor).unwrap().variables.get(0), 2);
        assert_eq!(world.data(actor).unwrap().variables.get(1), 2);
        assert!(world.bulwark_combat_delayed(actor).unwrap());
        for _ in 0..7 {
            world.tick().unwrap();
            assert!(world.bulwark_combat_delayed(actor).unwrap());
        }
        world.tick().unwrap();
        assert!(!world.bulwark_combat_delayed(actor).unwrap());
        assert_eq!(
            world.toggle_auto_retaliate(actor).unwrap().unwrap().value,
            1
        );
    }
    fn account() -> PlayerData {
        PlayerData::new(
            1,
            "parity_actor".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        )
    }
    fn admitted() -> (World, ActorId) {
        let mut world = World::default();
        world.collision.allocate(3200, 3200, 0);
        world.queue_login(1, account()).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        (world, actor)
    }
    #[test]
    fn bank_close_compression_waits_for_modal_and_delay_then_runs_once() {
        let (mut world, actor) = admitted();
        let config = crate::bank_actions::tests::config();
        let data = world.data_mut(actor).unwrap();
        config.capacity.set(&mut data.variables, 10).unwrap();
        config.tabs[9].set(&mut data.variables, 3).unwrap();
        data.bank
            .restore_slot(
                2,
                Some(crate::inventory::Item {
                    id: 995,
                    count: 7,
                    vars: 0,
                }),
            )
            .unwrap();
        world.banking = Some(config.clone());
        world.set_modal_open(actor, true).unwrap();
        world.queue_bank_compression(actor).unwrap();
        world.tick().unwrap();
        assert!(world.data(actor).unwrap().bank.get(0).is_none());
        world.set_modal_open(actor, false).unwrap();
        world.delay_player(actor, 2).unwrap();
        world.tick().unwrap();
        assert!(world.data(actor).unwrap().bank.get(0).is_none());
        world.tick().unwrap();
        assert!(world.data(actor).unwrap().bank.get(0).is_none());
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().bank.get(0).unwrap().count, 7);
        assert_eq!(
            config.tabs[9].get(&world.data(actor).unwrap().variables),
            Ok(1)
        );
        world.drain_events();
        world.tick().unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|event| matches!(event, WorldEvent::Bank { .. })));
    }
    #[test]
    fn bank_count_resume_is_single_use_and_canceled_tokens_cannot_resume_new_prompts() {
        let (mut world, actor) = admitted();
        let config = crate::bank_actions::tests::config();
        let data = world.data_mut(actor).unwrap();
        config.capacity.set(&mut data.variables, 10).unwrap();
        data.inventory
            .restore_slot(
                0,
                Some(crate::inventory::Item {
                    id: 995,
                    count: 20,
                    vars: 0,
                }),
            )
            .unwrap();
        world.object_types.stackable.insert(995);
        world.banking = Some(config);
        let prompt = Action::Bank(crate::bank_actions::Request::Button {
            deposit: true,
            slot: 0,
            op: 7,
        });
        world.submit(actor, prompt).unwrap();
        world.tick().unwrap();
        let old = world.count_dialog_token(actor).unwrap().unwrap();
        assert!(world.protection(actor).unwrap().suspended);
        assert!(world.close_dialogs(actor).unwrap());
        assert!(!world.protection(actor).unwrap().suspended);
        world.submit(actor, prompt).unwrap();
        world
            .submit(
                actor,
                Action::ResumeBankCount {
                    token: old,
                    count: 19,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert!(world.data(actor).unwrap().bank.get(0).is_none());
        let token = world.count_dialog_token(actor).unwrap().unwrap();
        assert_ne!(token, old);
        for _ in 0..2 {
            world
                .submit(actor, Action::ResumeBankCount { token, count: -5 })
                .unwrap();
        }
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().bank.get(0).unwrap().count, 5);
        assert_eq!(
            world.data(actor).unwrap().inventory.get(0).unwrap().count,
            15
        );
        assert_eq!(
            world
                .banking
                .as_ref()
                .unwrap()
                .last_quantity
                .get(&world.data(actor).unwrap().variables),
            Ok(5)
        );
        assert!(!world.protection(actor).unwrap().suspended);
        world.submit(actor, prompt).unwrap();
        world.tick().unwrap();
        let token = world.count_dialog_token(actor).unwrap().unwrap();
        world
            .submit(
                actor,
                Action::ResumeBankCount {
                    token,
                    count: i32::MIN,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().bank.get(0).unwrap().count, 5);
        assert!(world.drain_events().iter().any(|event| matches!(
            event,
            WorldEvent::Bank {
                result: Err(crate::bank_actions::Error::Variable(
                    crate::variables::Error::ValueOutOfRange
                )),
                ..
            }
        )));
    }
    #[test]
    fn queued_potions_reject_stale_items_share_delays_and_empty_without_delay() {
        use crate::{
            inventory::Item,
            potions::{Event, Potion, ResolvedEffect},
        };
        let (mut world, actor) = admitted();
        let potion = |id, next| Potion {
            id,
            next,
            vial: 9,
            sip: "sip".into(),
            remaining: "left".into(),
            effects: vec![ResolvedEffect::Boost {
                skill: 0,
                constant: 3,
                percent: 10,
            }],
        };
        world.potions.insert(7, potion(7, 8));
        world.potions.insert(8, potion(8, 9));
        let data = world.data_mut(actor).unwrap();
        data.stats.set_base(0, 50);
        data.stats.set_current(0, 45);
        data.inventory
            .restore_slot(
                2,
                Some(Item {
                    id: 7,
                    count: 1,
                    vars: 10,
                }),
            )
            .unwrap();
        world
            .submit(
                actor,
                Action::DrinkSlot {
                    slot: 2,
                    item: 8,
                    empty: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().inventory.get(2).unwrap().id, 7);
        assert!(!world.protection(actor).unwrap().delayed());
        world.drain_events();
        world
            .submit(
                actor,
                Action::DrinkSlot {
                    slot: 2,
                    item: 7,
                    empty: false,
                },
            )
            .unwrap();
        world
            .submit(
                actor,
                Action::DrinkSlot {
                    slot: 0,
                    item: 8,
                    empty: true,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(0), 53);
        assert_eq!(
            world.data(actor).unwrap().inventory.get(0),
            Some(Item {
                id: 8,
                count: 1,
                vars: 0
            })
        );
        assert!(world.data(actor).unwrap().inventory.get(2).is_none());
        assert!(world.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::ActionRejected {
                error: ActionError::Delayed,
                ..
            }
        )));
        for _ in 0..4 {
            world.tick().unwrap();
        }
        world.drain_events();
        world
            .submit(
                actor,
                Action::DrinkSlot {
                    slot: 0,
                    item: 8,
                    empty: true,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().inventory.get(0).unwrap().id, 9);
        assert_eq!(world.data(actor).unwrap().stats.current(0), 53);
        assert!(!world.protection(actor).unwrap().delayed());
        let events = world.drain_events();
        assert!(events.iter().any(|e| matches!(
            e,
            WorldEvent::Potion {
                event: Event::Message(_),
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(
            e,
            WorldEvent::Potion {
                event: Event::Animation(_) | Event::Delay(_) | Event::Stat(_),
                ..
            }
        )));
    }
    #[test]
    fn queued_food_uses_current_slot_and_blocks_repeated_eating_until_processed_expiry() {
        let (mut world, actor) = admitted();
        world.foods.insert(
            7,
            crate::food::Edible {
                id: 7,
                heal: 3,
                delay: 1,
                replacement: None,
                message: "You eat the shrimp.".into(),
            },
        );
        let data = world.data_mut(actor).unwrap();
        data.stats.set_base(3, 10);
        data.stats.set_current(3, 2);
        data.inventory
            .restore_slot(
                2,
                Some(crate::inventory::Item {
                    id: 7,
                    count: 3,
                    vars: 0,
                }),
            )
            .unwrap();
        world.submit(actor, Action::EatSlot { slot: 2 }).unwrap();
        world.submit(actor, Action::EatSlot { slot: 2 }).unwrap();
        world
            .submit(actor, Action::MoveInventorySlot { from: 2, into: 4 })
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 5);
        assert_eq!(
            world.data(actor).unwrap().inventory.get(2).unwrap().count,
            2
        );
        assert!(world.protection(actor).unwrap().delayed());
        assert!(!world.protection(actor).unwrap().suspended);
        assert!(world.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::ActionRejected {
                error: ActionError::Delayed,
                ..
            }
        )));
        world.submit(actor, Action::EatSlot { slot: 2 }).unwrap();
        world.tick().unwrap();
        assert!(!world.protection(actor).unwrap().suspended);
        assert_eq!(world.data(actor).unwrap().stats.current(3), 5);
        world.submit(actor, Action::EatSlot { slot: 2 }).unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 8);
        assert_eq!(
            world.data(actor).unwrap().inventory.get(2).unwrap().count,
            1
        );
    }
    #[test]
    fn delay_expiry_is_processed_after_input_and_does_not_stop_existing_routes() {
        let (mut world, actor) = admitted();
        let destination = CoordGrid::new(3206, 3200, 0).unwrap();
        world
            .submit(
                actor,
                Action::Walk {
                    destination,
                    run: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3201);
        world.delay_player(actor, 1).unwrap();
        for expected_x in [3202, 3203] {
            world.drain_events();
            world
                .submit(
                    actor,
                    Action::MoveClick {
                        destination: CoordGrid::new(3199, 3200, 0).unwrap(),
                        control: false,
                    },
                )
                .unwrap();
            world.tick().unwrap();
            assert_eq!(world.observe(actor).unwrap().coords.x(), expected_x);
            assert!(world.drain_events().iter().any(|event| matches!(
                event,
                WorldEvent::ActionRejected {
                    error: ActionError::Delayed,
                    ..
                }
            )));
        }
        world
            .submit(
                actor,
                Action::MoveClick {
                    destination: CoordGrid::new(3199, 3200, 0).unwrap(),
                    control: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3202);
    }
    #[test]
    fn exhaustion_changes_the_remaining_route_to_walk() {
        let (mut world, actor) = admitted();
        world.submit(actor, Action::SetRunning(true)).unwrap();
        world
            .submit(
                actor,
                Action::MoveClick {
                    destination: CoordGrid::new(3207, 3200, 0).unwrap(),
                    control: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3202);
        world
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .energy = 1;
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3204);
        assert_eq!(world.observe(actor).unwrap().energy, 0);
        let snapshot = world
            .drain_events()
            .into_iter()
            .filter_map(|event| match event {
                WorldEvent::SynchronizePlayers { players, .. } => players
                    .into_iter()
                    .find(|(id, _)| *id == actor)
                    .map(|(_, s)| s),
                _ => None,
            })
            .last()
            .unwrap();
        assert_eq!(snapshot.steps, 2);
        assert_eq!(snapshot.resolved_speed(), Speed::Run);
        assert_eq!(world.player(actor).unwrap().movement.pending_steps, 0);
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3205);
        assert!(world.observe(actor).unwrap().energy > 0);
        let p = world.player(actor).unwrap();
        assert_eq!(p.active_speed, Speed::Walk);
        assert_eq!(p.movement.cached_speed, Speed::Walk);
    }
    #[test]
    fn toggle_updates_current_route_and_control_click_is_temporary() {
        let (mut world, actor) = admitted();
        let destination = CoordGrid::new(3207, 3200, 0).unwrap();
        world
            .submit(
                actor,
                Action::MoveClick {
                    destination,
                    control: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3201);
        world.submit(actor, Action::SetRunning(true)).unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3203);
        world
            .submit(
                actor,
                Action::MoveClick {
                    destination,
                    control: true,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3204);
        assert_eq!(world.player(actor).unwrap().active_speed, Speed::Run);
        world
            .submit(
                actor,
                Action::MoveClick {
                    destination,
                    control: false,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3206);
        world
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .energy = 99;
        assert_eq!(
            world.action(actor, Action::SetRunning(false), &mut SeededRandom(1)),
            Err(ActionError::InsufficientEnergy)
        );
    }
    #[test]
    fn boosted_health_above_signed_byte_range_survives_damage() {
        use crate::hits::{Hit, Style, Target};
        let (mut world, actor) = admitted();
        world
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .stats
            .set_current(3, 200u8 as i8);
        world
            .queue_hit(
                Target::Player(actor),
                Hit {
                    display: None,
                    source: None,
                    style: Style::Typeless,
                    damage: 20,
                },
                1,
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3) as u8, 180);
        assert!(!world
            .drain_events()
            .iter()
            .any(|event| matches!(event, WorldEvent::DeathRequested(_))));
    }
    #[test]
    fn close_dialogs_clears_only_weak_queues_and_preserves_non_dialog_delay() {
        use crate::queues::Category;
        let (mut world, actor) = admitted();
        world.delay_player(actor, 5).unwrap();
        let player = world.players.get_mut(actor.slot as usize).unwrap();
        player
            .queues
            .add(100, Category::Weak, 1, Deferred::BankCompression)
            .unwrap();
        player
            .queues
            .add(101, Category::Normal, 1, Deferred::BankCompression)
            .unwrap();
        assert!(!world.close_dialogs(actor).unwrap());
        let player = world.players.get(actor.slot as usize).unwrap();
        assert!(!player.queues.contains(100));
        assert!(player.queues.contains(101));
        assert!(player.protection.delayed());
    }
    #[test]
    fn queued_hits_apply_in_entity_main_and_request_death_once() {
        use crate::hits::{Hit, Style, Target};
        let (mut world, actor) = admitted();
        world
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .stats
            .set_current(3, 10);
        let target = Target::Player(actor);
        let hit = Hit {
            display: None,
            source: None,
            style: Style::Typeless,
            damage: 20,
        };
        world.queue_hit(target, hit, 2).unwrap();
        world.queue_hit(target, hit, 2).unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 10);
        world.drain_events();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 0);
        let events = world.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e,WorldEvent::DeathRequested(t)if *t==target))
                .count(),
            1
        );
        let hits: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                WorldEvent::Hit { impact, .. } => Some(*impact),
                _ => None,
            })
            .collect();
        assert_eq!(hits.len(), 2);
        assert_eq!((hits[0].applied, hits[0].displayed), (10, 20));
        assert_eq!(hits[1].applied, 0);
    }
    #[test]
    fn socket_loss_waits_for_grace_and_keeps_account_reserved_until_saved() {
        let mut w = World::default();
        w.queue_login(1, account()).unwrap();
        w.tick().unwrap();
        let id = w
            .drain_events()
            .into_iter()
            .find_map(|e| match e {
                WorldEvent::Initialized { actor, .. } => Some(actor),
                _ => None,
            })
            .unwrap();
        w.client_disconnected(id).unwrap();
        for _ in 0..16 {
            w.tick().unwrap();
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::Save { .. })));
            assert!(w.observe(id).is_ok());
        }
        assert_eq!(w.queue_login(2, account()), Err(AdmissionError::Duplicate));
        w.tick().unwrap();
        assert_eq!(
            w.drain_events()
                .iter()
                .filter(|e| matches!(e, WorldEvent::Save { .. }))
                .count(),
            1
        );
        assert!(w.observe(id).is_ok());
        w.saved(id).unwrap();
        w.tick().unwrap();
        assert!(w.observe(id).is_err());
    }
    #[test]
    fn duplicate_admission_movement_and_save_before_removal() {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.queue_login(1, account()).unwrap();
        assert_eq!(w.queue_login(2, account()), Err(AdmissionError::Duplicate));
        w.tick().unwrap();
        let id = w
            .drain_events()
            .iter()
            .find_map(|e| {
                if let WorldEvent::Initialized { actor, .. } = e {
                    Some(*actor)
                } else {
                    None
                }
            })
            .unwrap();
        w.submit(
            id,
            Action::Walk {
                destination: CoordGrid::new(3203, 3200, 0).unwrap(),
                run: true,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(w.observe(id).unwrap().coords.x(), 3202);
        w.request_logout(id).unwrap();
        assert_eq!(
            w.submit(id, Action::SetRunning(true)),
            Err(ActionError::LoggingOut)
        );
        w.tick().unwrap();
        assert!(w.observe(id).is_ok());
        w.saved(id).unwrap();
        w.tick().unwrap();
        assert_eq!(w.observe(id).err(), Some(ActionError::StaleActor));
        w.queue_login(3, account()).unwrap();
    }
}
