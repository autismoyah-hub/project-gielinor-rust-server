//! Authoritative world state. Network clients and future testing actors use the same actions.
mod cinderworks_checkpoint;
mod cinderworks_actions;
mod cinderworks_entry;
mod cinderworks_encounter_actions;
mod cinderworks_recovery;
mod cinderworks_fishing;
mod cinderworks_services;
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
#[cfg(test)]
mod consumable_tests;
#[cfg(test)]
mod food_life_tests;
mod dialogue_actions;
mod signpost_actions;
mod fence_actions;
mod location_choice_actions;
mod stair_actions;
mod location_climb_actions;
mod location_swap_actions;
mod spade_actions;
mod bookcase_actions;
mod pickable_actions;
mod door_actions;
mod guild_passage_actions;
mod drop_action;
mod chemistry_actions;
mod ground_actions;
mod held_use_actions;
mod location_use_actions;
#[cfg(test)]
mod logout_tests;
mod melee_actions;
mod movement_actions;
pub use movement_actions::MovementInput;
mod elemental_spell_actions;
mod deposit_box_actions;
mod banknote_actions;
mod bank_access_actions;
mod worn_item_swap_actions;
mod worn_stat_boost_actions;
mod skillcape_vendor_actions;
mod container_emptying_actions;
mod item_pack_actions;
mod milking_actions;
mod water_source_actions;
mod cooking_actions;
mod grinding_actions;
mod swamp_tar_actions;
mod dough_actions;
mod finished_potion_actions;
mod potion_upgrade_actions;
mod combat_mixing_actions;
mod unfinished_potion_actions;
mod herb_cleaning_actions;
mod mining_actions;
mod npc_actions;
mod thieving_actions;
mod tool_crafting_actions;
mod counted_assembly_actions;
mod assembly_production_actions;
mod facility_crafting_actions;
mod runecrafting_actions;
mod npc_combat_actions;
mod npc_death_actions;
mod npc_search;
mod prayer_actions;
mod prayer_book_actions;
mod quantity_dialog;
mod ranged_actions;
mod ranged_ammo_actions;
mod shop_actions;
mod smelting_actions;
mod smithing_actions;
mod soft_timer_actions;
mod wine_actions;
mod spell_actions;
mod stat_event_actions;
mod tower_actions;
mod tower_chest_actions;
mod tower_combat_actions;
mod tower_login_actions;
mod tower_offering_actions;
mod tower_staff_actions;
mod tanning_actions;
mod tower_tanning_actions;
mod tutorial_bread_actions;
mod tutorial_fire_actions;
mod tutorial_fishing_actions;
mod tutorial_gate_actions;
mod tutorial_leave_actions;
mod tutorial_mining_actions;
mod tutorial_shrimp_actions;
mod tutorial_woodcutting_actions;
mod woodcutting_actions;

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
    hunt_zone: [i32; 3],
    hunt_order: u64,
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
    /// None means the legacy arrival projection cannot establish diary marks.
    pub tower_diary: Option<crate::tower_diary::Progress>,
    pub chemistry: crate::chemistry::State,
    pub tower_run: crate::tower_run::Run,
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
            tower_diary: None,
            chemistry: Default::default(),
            tower_run: Default::default(),
        }
    }
}
#[derive(Clone)]
enum Deferred {
    Smithing(crate::smithing_make::Job),
    Assembly(crate::assembly::Pending),
    AssemblyBatch(crate::assembly::BatchPending),
    Spade(spade_actions::Pending),
    Bookcase(bookcase_actions::Pending),
    Milking(crate::milking::Job),
    WaterFilling(crate::water_sources::Job),
    Cooking(crate::cooking::Job),
    CinderworksFishing(cinderworks_fishing::Job),
    Pickpocket(thieving_actions::Job),
    ToolCrafting(crate::tool_crafting::Pending),
    AssemblyProduction(crate::assembly_production::Pending),
    FacilityCrafting(crate::facility_crafting::Pending),
    Runecrafting(crate::runecrafting::Pending),
    Grinding(crate::grinding::Pending),
    SwampTar(crate::swamp_tar::Pending),
    Dough(crate::dough::Pending),
    FinishedPotion(crate::finished_potions::Pending),
    PotionUpgrade(crate::potion_upgrades::Pending),
    CombatMixing(crate::combat_mixing::Pending),
    UnfinishedPotion(crate::unfinished_potions::Pending),
    ItemPack {
        pack: i32,
        rule: crate::item_packs::Rule,
    },
    StandardHit(crate::hits::Hit),
    PlayerDeath,
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
#[derive(Clone, Copy)]
enum LocationAction {
    Operation(u8),
    HeldUse { slot: i32, object: i32 },
}
struct Player {
    cinderworks_checkpoint: Option<cinderworks_checkpoint::Pending>,
    cinderworks_recovery_at:Option<i32>,
    dough_request: Option<crate::dough::Request>,
    tool_crafting_request:Option<crate::tool_crafting::Request>,
    counted_assembly_request:Option<crate::counted_assembly::Request>,
    assembly_production_request:Option<crate::assembly_production::Request>,
    finished_potion_request: Option<crate::finished_potions::Request>,
    potion_upgrade_request: Option<crate::potion_upgrades::Request>,
    combat_mixing_request: Option<crate::combat_mixing::Request>,
    unfinished_potion_request: Option<crate::unfinished_potions::Request>,
    member: bool,
    stat_events: crate::stat_events::Events,
    areas: crate::area_tracking::State,
    pending_areas: VecDeque<crate::area_tracking::Event>,
    music_resizable: bool,
    login_cycle: i32,
    tower_diary_login: Option<(crate::tower_diary::Sync, CoordGrid)>,
    tower_restore_pending: bool,
    tower_tastes: crate::tower_tastes::State,
    aggressive_npc: Option<NpcId>,
    last_npc_combat: i32,
    last_pvp_combat: i32,
    retaliate_npcs: VecDeque<(i32, NpcId)>,
    id: ActorId,
    combat_decimal: i32,
    action_delay: i32,
    consumables: crate::consumable_clocks::State,
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
    pickpocket_stun_until:i32,
    tutorial_fire: Option<crate::tutorial_fire::Pending>,
    main_modal: Option<u16>,
    location_interaction: Option<(crate::locations::Location, LocationAction)>,
    location_revision: Option<u64>,
    woodcutting_animation_at: i32,
    pending_location_action: Option<location_swap_actions::Pending>,
    pending_door: Option<crate::doors::Pending>,
    observer_id: Option<i64>,
    autocast_picker_defensive: Option<bool>,
    loot_drop_duration: Option<i32>,
    object_interaction: Option<crate::ground::Handle>,
    npc_interaction: Option<crate::npc_interaction::Pending>,
    pending_pickup: Option<crate::ground_transfer::Pickup>,
    pending_drop: Option<(u64, crate::drop_actions::Pending)>,
    pending_chemistry: Option<(u64,usize,i32,bool,bool)>,
    prayer: Option<crate::prayer::State>,
    prayer_notice: Option<u64>,
    overhead: Option<u8>,
    dialogue: Option<crate::dialogue::Pending>,
    bank_count: Option<(u64, crate::bank_actions::CountPurpose)>,
    shop_session: Option<crate::shop::Session>,
    quantity_dialog: Option<u64>,
    quantity_input: bool,
    dialog_generation: u64,
    has_modal: bool,
    queues: crate::queues::Scheduler<Deferred>,
    soft_timers: crate::player_timers::Timers,
    weight_sources: Option<Vec<WeightSource>>,
}
impl Player {
    fn record_stat_update(
        &mut self,
        tutorial: bool,
        update: crate::stat_ops::Update,
    ) -> Result<(), ActionError> {
        if tutorial && update.changed {
            self.stat_events
                .changed(&crate::tutorial_rules::bindings(), update.skill)
                .map_err(|_| ActionError::QueueFull)?;
        }
        Ok(())
    }
    fn pickpocket_stunned(&self)->bool {self.pickpocket_stun_until>self.protection.processed_clock}
    fn begin_held_action(&mut self) {
        self.queues.remove_all(spade_actions::QUEUE);
        self.queues.remove_all(bookcase_actions::QUEUE);
        self.queues.remove_all(crate::thieving::QUEUE);
        self.queues.remove_all(crate::tool_crafting::QUEUE);
        self.queues.remove_all(crate::assembly::QUEUE);
        self.queues.remove_all(crate::milking::QUEUE);
        self.queues.remove_all(crate::water_sources::QUEUE);
        self.queues.remove_all(crate::cooking::QUEUE);
        self.queues.remove_all(crate::grinding::QUEUE);
        self.queues.remove_all(crate::swamp_tar::QUEUE);
        self.queues.remove_all(crate::dough::QUEUE);
        self.dough_request = None;
        self.tool_crafting_request=None;
        self.counted_assembly_request=None;
        self.assembly_production_request=None;
        self.queues.remove_all(crate::assembly_production::QUEUE);
        self.queues.remove_all(crate::facility_crafting::QUEUE);
        self.finished_potion_request = None;
        self.queues.remove_all(crate::finished_potions::QUEUE);
        self.potion_upgrade_request = None;
        self.combat_mixing_request = None;
        self.queues.remove_all(crate::combat_mixing::QUEUE);
        self.queues.remove_all(crate::potion_upgrades::QUEUE);
        self.unfinished_potion_request = None;
        self.queues.remove_all(crate::unfinished_potions::QUEUE);
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
        self.tutorial_fire = None;
        self.quantity_dialog = None;
        self.quantity_input = false;
        self.pending_location_action=None;
        self.pending_door = None;
        self.pending_pickup = None;
        self.pending_drop = None;
        self.pending_chemistry = None;
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
    /// Original type-search reference action; network gameplay uses BuryItemSlot.
    BurySlot { slot: usize },
    DigItemSlot { slot: usize, item: i32 },
    BuryItemSlot { slot: usize, item: i32 },
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
    CastNpc {
        target: NpcId,
        spell: i32,
        control: bool,
    },
    ChemistryItem {slot:usize,item:i32,operation:u8,worn:bool},
    ResumeChemistry {token:u64,choice:i32},
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
    UseLocation {
        coords: CoordGrid,
        id: u32,
        slot: i32,
        object: i32,
        control: bool,
    },
    ResumeBankCount {
        token: u64,
        count: i32,
    },
    Bank(crate::bank_actions::Request),
    Shop {
        session: u64,
        request: crate::shop::Request,
    },
    EquipSlot {
        slot: usize,
    },
    UnequipSlot {
        slot: usize,
    },
    UnequipItemSlot {slot:usize,item:i32},
    MoveInventorySlot {
        from: usize,
        into: usize,
    },
    /// Held-op 1 reads the authoritative slot; it never trusts a client object ID.
    EatSlot {
        slot: usize,
    },
    /// Selected Wiki food profiles bind the claimed item as well as its slot.
    /// Execution rechecks the claim after earlier queued inventory actions.
    EatItemSlot {
        slot: usize,
        item: i32,
    },
    CleanHerb {
        slot: usize,
        item: i32,
    },
    OpenItemPack { slot: usize, item: i32 },
    DepositBoxAll { worn:bool },
    DepositBoxItem { slot:usize,item:i32,operation:u8 },
    DepositBoxMode { mode:i32 },
    SwapWornItem { slot: usize, item: i32 },
    BoostWornItem { slot: usize, item: i32 },
    ConvertHeldItem { slot: usize, item: i32 },
    EmptyContainer { slot: usize, item: i32 },
    OpenCoinPouch { slot: usize, item: i32, all: bool },
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
    UnknownTowerDiary,
    UnknownTowerBones,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionError {
    Duplicate,
    Full,
    QueueFull,
}
#[derive(Debug)]
pub enum WorldEvent {
    TileGraphic(crate::zones::Graphic),
    CinderworksCheckpoint { actor:ActorId, data:PlayerData },
    CinderworksProgress { actor:ActorId, run:crate::tower_run::Run },
    CountedAssemblyMade {actor: ActorId, first: i32, second: i32, output: i32, count: i32},
    ChemistryDialog {actor:ActorId,token:Option<u64>,remaining:u8,breaking:bool},
    LocationRestoration(crate::loc_timers::Restoration),
    ShopValue {
        actor: ActorId,
        item: i32,
        player_buys: bool,
        result: Result<i32, crate::shop::Error>,
    },
    Shop {
        actor: ActorId,
        shop: u32,
        result: Result<crate::shop::Exchange, crate::shop::Error>,
    },
    Woodcutting {
        actor: ActorId,
        trace: crate::woodcutting::Trace,
    },
    Mining {
        actor: ActorId,
        trace: crate::gathering::Trace,
    },
    GameMessage {
        actor: ActorId,
        text: String,
    },
    TowerDiary {
        actor: ActorId,
        marks: crate::tower_diary::Marks,
    },
    LocationBankRequested {actor:ActorId,request:crate::bank_access::Request},
    TutorialBank {
        actor: ActorId,
        booth: crate::tutorial_bank::Booth,
        next: Option<crate::tutorial_steps::Step>,
    },
    TutorialLeaveProgress {
        actor: ActorId,
        effects: Vec<crate::tutorial_progression::Effect>,
    },
    AnvilRequested {
        actor: ActorId,
        preferred: Option<i32>,
    },
    SmeltingRequested {
        actor: ActorId,
    },
    FacilityCraftingRequested { actor:ActorId, request:crate::facility_crafting::Request },
    SmithingProductMade {
        actor: ActorId,
        product: i32,
        count: i32,
    },
    SkillAdvanced {
        actor: ActorId,
        skill: u8,
        advancement: crate::skills::Advancement,
    },
    ResetFaceEntity {
        actor: ActorId,
    },
    TutorialCombatProgress {
        actor: ActorId,
        effects: Vec<crate::tutorial_progression::Effect>,
    },
    TutorialInventoryProgress {
        actor: ActorId,
        effects: Vec<crate::tutorial_progression::Effect>,
    },
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
    Milking { actor:ActorId, trace:crate::milking::Transition },
    WaterFilled { actor: ActorId, trace: crate::water_sources::Transition },
    CookingMade { actor: ActorId, trace: crate::cooking::Transition },
    Grinding { actor: ActorId, trace: crate::grinding::Transition },
    SwampTarMade { actor: ActorId, trace: crate::swamp_tar::Transition },
    DoughMade { actor: ActorId, trace: crate::dough::Transition },
    FinishedPotionMade { actor: ActorId, trace: crate::finished_potions::Transition },
    PotionUpgradeMade { actor: ActorId, trace: crate::potion_upgrades::Transition },
    CombatMixingMade { actor: ActorId, trace: crate::combat_mixing::Transition },
    UnfinishedPotionMade { actor: ActorId, trace: crate::unfinished_potions::Transition },
    AssemblyBatchMade {actor:ActorId,trace:crate::assembly::BatchTransition},
    AssemblyMade { actor: ActorId, trace: crate::assembly::Transition },
    HerbCleaned { actor: ActorId, trace: crate::herb_cleaning::Transition },
    CombatSound {
        actor: ActorId,
        sound: u16,
    },
    SignpostRead {actor:ActorId,location:crate::locations::Location,origin:CoordGrid},
    PlayerExactMove {actor:ActorId,from:CoordGrid,to:CoordGrid,offsets:[i8;4],delays:[u16;2],direction:u16,sequence:u16,sequence_delay:u8},
    PlayerAnimation {actor:ActorId,sequence:u16},
    PlayerSound {actor:ActorId,sound:u16},
    PickableCollected {actor:ActorId,location:crate::locations::Location,item:i32},
    PickableDepletion {actor:ActorId,trace:crate::pickables::ChanceTransition},
    PickableAdvanced {actor:ActorId,trace:crate::pickables::Transition},
    PlayerCombatAnimation {
        actor: ActorId,
        sequence: u16,
    },
    PlayerSpotAnimation {
        actor: ActorId,
        spot: u16,
        delay: u16,
        height: u16,
        slot: u8,
    },
    NpcCombatError {
        actor: NpcId,
        error: ActionError,
    },
    NpcDrop(crate::npc_death::DropTrace),
    NpcImpact(crate::hits::NpcImpactTrace),
    NpcMeleeXp(crate::combat_xp::MeleeTrace),
    NpcCombatAnimation {
        actor: NpcId,
        sequence: u16,
    },
    NpcDelayedAnimation {
        actor: NpcId,
        sequence: u16,
        delay: u8,
    },
    NpcSpotAnimation {
        actor: NpcId,
        spot: u16,
        delay: u16,
        height: u16,
        slot: u8,
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
    LocationClimbed {actor:ActorId,trace:crate::location_content::ClimbTransition},
    LocationSwitched {actor:ActorId,trace:crate::location_content::SwapTransition},
    LocationSpam {actor:ActorId,message:&'static str},
    LocationMessage {
        actor: ActorId,
        message: &'static str,
    },
    EngineMessage {
        actor: ActorId,
        message: &'static str,
    },
    TutorialGateFocus {
        actor: ActorId,
        step: crate::tutorial_steps::Step,
        /// Resolved at the departure position, before the gate moves the actor.
        hint: Option<NpcId>,
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
    WearposEffects {
        actor: ActorId,
        result: Result<Vec<crate::wearpos::Event>, String>,
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
    BankCount(crate::bank_actions::CountTrace),
    EquipmentResult {
        actor: ActorId,
        item: i32,
        unequipping: bool,
        result: Result<Vec<crate::equipment::Event>, crate::equipment::Error>,
    },
    FoodTransition {
        actor: ActorId,
        trace: crate::food::Transition,
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
    Projectile(crate::projectile::Flight),
    AreaSound(crate::zones::Sound),
    AmmoConsumed {
        actor: ActorId,
        result: crate::ranged_ammo::Consumption,
    },
    LocationError(&'static str),
    GroundError(crate::ground::Error),
    Hit {
        target: crate::hits::Target,
        impact: crate::hits::Impact,
    },
    DeathRequested(crate::hits::Target),
}
pub struct World {
    pub furnaces: BTreeMap<u32, crate::smelting::Furnace>,
    pub smelting_ores: HashSet<i32>,
    pub anvils: HashSet<u32>,
    pub smithing_bars: HashSet<i32>,
    pub smithing: Option<crate::smithing_make::Config>,
    pub tutorial_woodcutting: Option<crate::tutorial_woodcutting::Config>,
    pub tutorial_fishing: Option<crate::tutorial_fishing::Config>,
    pub tutorial_bread: Option<crate::tutorial_bread::Config>,
    pub tutorial_shrimp: Option<crate::tutorial_shrimp::Config>,
    pub tutorial_mining: Option<crate::tutorial_mining::Config>,
    pub tutorial_combat: Option<crate::tutorial_combat::Config>,
    pub tutorial_leave: Option<crate::tutorial_leave::Config>,
    pub tutorial_bank: Option<crate::tutorial_bank::Config>,
    pub held_use_types: BTreeMap<i32, crate::held_use::Type>,
    pub held_use_handlers: crate::held_use::Registry<crate::held_use::Handler>,
    /// Opt-in until scheduler/native regression coverage is complete.
    pub player_hit_queues: Option<crate::hits::PlayerQueues>,
    pub tutorial_rules_completed_varp: Option<u16>,
    pub tutorial_gates: Option<crate::tutorial_gates::Config>,
    pub music: Option<crate::music_player::Config>,
    pub area_sources: crate::area_tracking::Sources,
    pub session_vars: Option<crate::session_vars::Config>,
    pub login_resets: Option<crate::login_resets::Config>,
    /// Host-supplied wall time, kept separate from the deterministic world clock.
    pub unix_millis: u64,
    pub tower_arrival: Option<crate::tower_arrival::Config>,
    pub cinderworks:Option<crate::tower_cinderworks::Config>,
    pub cinderworks_encounters:[Option<crate::cinderworks_encounter::Encounter>;2],
    pub tanning: BTreeMap<u16, Vec<crate::tanning::Recipe>>,
    pub tower_tanning: Option<crate::tower_tanning::HandlerConfig>,
    pub tower_chest: Option<crate::tower_chest::HandlerConfig>,
    pub tower_staff: Option<crate::tower_staff::HandlerConfig>,
    pub tower_offering: Option<crate::tower_offering::HandlerConfig>,
    pub tower_restore: Option<crate::tower_restore::HandlerConfig>,
    pub tower_combat: Option<crate::tower_combat::Config>,
    pub woodcutting: Option<crate::woodcutting::Config>,
    pub mining: Option<crate::mining::Config>,
    pub gathering_outfits: BTreeMap<u8, crate::gathering_outfits::Outfit>,
    /// Explicit stock partitions; empty until a verified content profile registers one.
    pub shops: BTreeMap<u32, crate::shop::Shop>,
    active_trees: crate::woodcutting::ActiveTrees,
    pub combat_stance: Option<crate::combat_stance::Config>,
    pub combat_wear: Option<crate::combat_wear::Config>,
    pub wearpos: Option<crate::wearpos::Config>,
    pub bulwark_wear: Option<crate::bulwark_wear::Config>,
    pub autocast_selection: Option<crate::autocast_select::Config>,
    pub banking: Option<crate::bank_actions::Config>,
    pub wear_types: crate::equipment::WearCatalog,
    pub spade: Option<crate::spade::Rule>,
    spade_hooks: Vec<crate::spade::Hook>,
    pub foods: BTreeMap<i32, crate::food::Edible>,
    pub consumable_rules: Option<crate::consumable_clocks::Rules>,
    pub member_foods: std::collections::BTreeSet<i32>,
    pub food_boosts: BTreeMap<i32,Vec<crate::food::StatBoost>>,
    pub food_subtractions: BTreeMap<i32,Vec<crate::food::StatSubtraction>>,
    pub food_stat_healing: BTreeMap<i32,Vec<crate::food::StatHealing>>,
    pub food_formulas: BTreeMap<i32,crate::food::StatEffects>,
    pub food_energy: BTreeMap<i32,i32>,
    pub food_healing: BTreeMap<i32,crate::food::Healing>,
    pub food_random_healing: BTreeMap<i32,crate::food::RandomHealing>,
    pub slot_preserving_foods: std::collections::BTreeSet<i32>,
    pub potions: BTreeMap<i32, crate::potions::Potion>,
    pub milking: Option<crate::milking::Config>,
    pub water_sources: Option<crate::water_sources::Config>,
    pub cooking: Option<crate::cooking::Config>,
    pub dough: Option<crate::dough::Config>,
    pub finished_potions: Option<crate::finished_potions::Config>,
    pub potion_upgrades: Option<crate::potion_upgrades::Config>,
    pub combat_mixing: Option<crate::combat_mixing::Config>,
    pub unfinished_potions: Option<crate::unfinished_potions::Config>,
    pub swamp_tar: Option<crate::swamp_tar::Config>,
    pub herb_cleaning: Option<crate::herb_cleaning::Config>,
    pub item_packs: BTreeMap<i32, crate::item_packs::Rule>,
    pub member_item_packs: std::collections::BTreeSet<i32>,
    /// Legacy field name; this single registry also owns ordinary light extinguishing.
    pub worn_item_swaps: BTreeMap<i32,crate::worn_item_swap::Rule>,
    pub worn_stat_boosts: BTreeMap<i32,crate::worn_stat_boost::Rule>,
    pub empty_containers: BTreeMap<i32,crate::container_emptying::Rule>,
    pub coin_pouches: BTreeMap<i32,crate::item_packs::PouchRule>,
    pub prayer: Option<crate::prayer::Config>,
    pub wine: Option<crate::wine::Config>,
    pub zamorak_wine: Option<crate::wine::Config>,
    pub prayer_preservation: Option<crate::prayer_preservation::Rule>,
    pub prayer_member_bones: std::collections::BTreeSet<i32>,
    pub prayer_ashes: BTreeMap<i32,u16>,
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
    pub pickup_policy: crate::ground_transfer::PickupPolicy,
    pub dropping: Option<crate::drop_actions::Config>,
    pub person_flyer: Option<i32>,
    pub locations: crate::locations::Locations,
    pub location_types: BTreeMap<u32, crate::loc_collision::LocCollision>,
    pub location_approach: BTreeMap<u32, u8>,
    pub doors: crate::doors::Catalog,
    pub guild_passages: BTreeMap<u32, crate::guild_passages::Gate>,
    pub deposit_box: Option<crate::deposit_box::Config>,
    pub location_banks: BTreeMap<u32,crate::bank_access::Gate>,
    pub ordinary_bank_booths: std::collections::BTreeSet<u32>,
    pub runecrafting:Option<crate::runecrafting::Config>,
    pub location_handlers: BTreeMap<(u32, u8), crate::location_content::Handler>,
    pub npc_options: crate::npc_options::Catalog,
    pub attack_profiles: crate::attack_profile::Catalog,
    pub melee: Option<crate::melee::Config>,
    pub ranged: Option<crate::ranged::Config>,
    pub ranged_weapons: Option<crate::ranged_weapon::Catalog>,
    pub spell_combat: Option<crate::spell_combat::Config>,
    pub elemental_spells: Option<crate::elemental_spell::Catalog>,
    pub npc_melee: Option<crate::npc_melee::Config>,
    pub npc_death: Option<crate::npc_death::Config>,
    pub npc_bindings: BTreeMap<u16, crate::npc_bind::Binding>,
    pub combat_state: Option<crate::combat_state::Config>,
    pub npc_combat: BTreeMap<u16, crate::npc_combat::Definition>,
    pub pickpocket:Option<crate::thieving::Config>,
    pub facility_crafting:BTreeMap<u32,crate::facility_crafting::Config>,
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
    npc_hunt_order: u64,
    random: SeededRandom,
}
impl Default for World {
    fn default() -> Self {
        Self {
            furnaces: BTreeMap::new(),
            smelting_ores: HashSet::new(),
            anvils: HashSet::new(),
            smithing_bars: HashSet::new(),
            smithing: None,
            tutorial_woodcutting: None,
            tutorial_fishing: None,
            tutorial_bread: None,
            tutorial_shrimp: None,
            tutorial_mining: None,
            tutorial_combat: None,
            tutorial_leave: None,
            tutorial_bank: None,
            held_use_types: Default::default(),
            held_use_handlers: Default::default(),
            tutorial_gates: None,
            player_hit_queues: None,
            tutorial_rules_completed_varp: None,
            areas: Default::default(),
            music: None,
            area_sources: Default::default(),
            tower_arrival: None,
            cinderworks:None,
            cinderworks_encounters:[None,None],
            tanning: Default::default(),
            tower_tanning: None,
            tower_chest: None,
            tower_staff: None,
            tower_offering: None,
            tower_restore: None,
            tower_combat: None,
            woodcutting: None,
            mining: None,
            gathering_outfits: BTreeMap::new(),
            shops: BTreeMap::new(),
            active_trees: Default::default(),
            multiway_area: None,
            combat_stance: None,
            combat_wear: None,
            wearpos: None,
            bulwark_wear: None,
            autocast_selection: None,
            banking: None,
            wear_types: crate::equipment::WearCatalog::default(),
            spade: None,
            spade_hooks: Vec::new(),
            foods: BTreeMap::new(),
            consumable_rules: None,
            member_foods: Default::default(),
            food_boosts: BTreeMap::new(),
            food_subtractions: BTreeMap::new(),
            food_stat_healing: BTreeMap::new(),
            food_formulas: BTreeMap::new(),
            food_energy: BTreeMap::new(),
            food_healing: BTreeMap::new(),
            food_random_healing: BTreeMap::new(),
            slot_preserving_foods: Default::default(),
            potions: BTreeMap::new(),
            item_packs: BTreeMap::new(),
            member_item_packs: Default::default(),
            worn_item_swaps: BTreeMap::new(),
            worn_stat_boosts: BTreeMap::new(),
            empty_containers: BTreeMap::new(),
            coin_pouches: BTreeMap::new(),
            milking: None,
            water_sources: None,
            cooking: None,
            dough: None,
            herb_cleaning: None,
            unfinished_potions: None,
            swamp_tar: None,
            finished_potions: None,
            potion_upgrades: None,
            combat_mixing: None,
            prayer: None,
            wine: None,
            zamorak_wine: None,
            prayer_preservation: None,
            prayer_member_bones: Default::default(),
            prayer_ashes: BTreeMap::new(),
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
            pickup_policy: Default::default(),
            dropping: None,
            person_flyer: None,
            locations: Default::default(),
            location_types: BTreeMap::new(),
            location_approach: BTreeMap::new(),
            doors: Default::default(),
            guild_passages: Default::default(),
            deposit_box: None,
            location_banks: BTreeMap::new(),
            ordinary_bank_booths: Default::default(),
            location_handlers: Default::default(),
            runecrafting:None,
            npc_options: Default::default(),
            attack_profiles: Default::default(),
            melee: None,
            ranged: None,
            ranged_weapons: None,
            spell_combat: None,
            elemental_spells: None,
            npc_melee: None,
            npc_death: None,
            npc_bindings: BTreeMap::new(),
            combat_state: None,
            npc_combat: BTreeMap::new(),
            npc_handlers: Default::default(),
            pickpocket:None,
            facility_crafting:BTreeMap::new(),
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
            npc_hunt_order: 0,
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
    /// Transient zone effect: sent to current observers once, never persisted.
    pub fn emit_projectile(
        &mut self,
        flight: crate::projectile::Flight,
    ) -> Result<(), ActionError> {
        if flight.start == CoordGrid::NULL || flight.end == CoordGrid::NULL {
            return Err(ActionError::InvalidLocation);
        }
        self.events.push(WorldEvent::Projectile(flight));
        Ok(())
    }
    pub fn emit_area_sound(&mut self, sound: crate::zones::Sound) -> Result<(), ActionError> {
        if sound.coords == CoordGrid::NULL {
            return Err(ActionError::InvalidLocation);
        }
        self.events.push(WorldEvent::AreaSound(sound));
        Ok(())
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
        let hunt_zone = npc_search::zone(npc.movement.coords);
        self.npc_hunt_order = self
            .npc_hunt_order
            .checked_add(1)
            .expect("NPC zone order exhausted");
        let result = self.npcs.insert(NpcEntry {
            id,
            npc,
            hunt_zone,
            hunt_order: self.npc_hunt_order,
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
            Target::Player(id) => {
                hit.validate()?;
                let p = self
                    .players
                    .get_mut(id.slot as usize)
                    .filter(|p| p.id == id && !p.saving)
                    .ok_or(Error::InvalidTarget)?;
                if let Some(config) = self.player_hit_queues {
                    p.queues
                        .add(
                            config.standard,
                            crate::queues::Category::Strong,
                            delay,
                            Deferred::StandardHit(hit),
                        )
                        .map_err(|_| Error::QueueFull)
                } else {
                    p.hits.push(hit, delay)
                }
            }
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
    /// Network movement must reject these states before running ifClose callbacks.
    pub fn movement_input_allowed(&self, id: ActorId) -> Result<bool, ActionError> {
        let player = self.player(id)?;
        Ok(!player.protection.delayed() && !player.movement.rooted && !player.pickpocket_stunned() && player.cinderworks_checkpoint.is_none())
    }
    pub fn count_dialog_token(&self, id: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self.player(id)?.bank_count.map(|pending| pending.0))
    }
    pub fn set_modal_open(&mut self, id: ActorId, open: bool) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        p.has_modal = open;
        if open {
            p.queues.remove_all(crate::thieving::QUEUE);
            p.queues.remove_all(crate::tool_crafting::QUEUE);
            p.queues.remove_all(crate::assembly_production::QUEUE);
            p.queues.remove_all(crate::facility_crafting::QUEUE);
            if matches!(p.prayer,Some(crate::prayer::State::Bury{slot:Some(_),..})) {
                p.prayer=None;p.protection.suspended=false;
            }
            p.queues.remove_all(crate::assembly::QUEUE);
            p.queues.remove_all(crate::milking::QUEUE);
            p.queues.remove_all(crate::water_sources::QUEUE);
            p.queues.remove_all(crate::cooking::QUEUE);
            p.queues.remove_all(crate::grinding::QUEUE);
            p.queues.remove_all(crate::dough::QUEUE);
        }
        if !open {
            p.shop_session = None;
        }
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
        let bank = p.bank_count.take().is_some();
        let number = p.quantity_input && p.quantity_dialog.take().is_some();
        if p.quantity_input {
            p.quantity_input = false;
        }
        let canceled = bank || number;
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
        p.shop_session = None;
        let count = p.bank_count.take().is_some();
        let quantity = p.quantity_dialog.take().is_some();
        p.quantity_input = false;
        let pause = p.pending_drop.take().is_some() | p.pending_chemistry.take().is_some();
        let prayer = matches!(p.prayer, Some(crate::prayer::State::Confirm { .. }));
        let notice = p.prayer_notice.take().is_some();
        if prayer {
            p.prayer = None;
        }
        p.queues.clear_weak();
        if count || quantity || pause || prayer || notice {
            p.protection.suspended = false;
        }
        Ok(count || quantity || pause || prayer || notice)
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
        if p.saving || p.pending_logout || p.cinderworks_checkpoint.is_some() {
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
        if p.pending_location_action.take().is_some(){p.protection.suspended=false;}
        p.movement.abort();
        self.process_pending_logout(id)
    }
    fn process_pending_logout(&mut self, id: ActorId) -> Result<(), ActionError> {
        self.player(id)?;
        let p = self.players.get_mut(id.slot as usize).unwrap();
        if !p.pending_logout || p.saving || p.cinderworks_checkpoint.is_some() {
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
    fn worn_changed(&mut self, id: ActorId, slot: usize, changed_item: i32) {
        self.update_prayer_resistance(id);
        if let Some(config) = &self.bulwark_wear {
            let player = self.players.get_mut(id.slot as usize).unwrap();
            if let Some(change) = config.changed(
                &mut player.data.variables,
                slot,
                changed_item,
                player.data.equipment.get(3).map(|i| i.id),
                self.clock.0,
                player.protection.processed_clock > 0,
            ) {
                self.events.push(WorldEvent::PlayerSpotAnimation {
                    actor: id,
                    spot: change.glow,
                    delay: 0,
                    height: 0,
                    slot: 0,
                });
                self.events.push(WorldEvent::CombatVariables {
                    actor: id,
                    updates: change.variable.into_iter().collect(),
                });
            }
        }
        if let Some(config) = &self.wearpos {
            let player = self.players.get_mut(id.slot as usize).unwrap();
            let result = config.changed(
                &mut player.data,
                slot,
                changed_item,
                player.combat_decimal,
                player.protection.processed_clock > 0,
            );
            self.events
                .push(WorldEvent::WearposEffects { actor: id, result });
        }
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
        if p.saving || p.pending_logout || p.cinderworks_checkpoint.is_some() {
            return Err(ActionError::LoggingOut);
        }
        match action {
            Action::Shop { session, request } => {
                return self.process_shop_action(id, session, request)
            }
            Action::BurySlot { slot } => return self.start_bury(id, slot, None),
            Action::DigItemSlot { slot, item } => return self.start_spade(id,slot,item),
            Action::BuryItemSlot { slot, item } => return self.start_bury(id, slot, Some(item)),
            Action::ResumeBury { token, choice } => return self.resume_bury(id, token, choice),
            Action::ResumeDialogue { token, choice } => {
                return self.resume_dialogue(id, token, choice, random)
            }
            Action::OperateNpc {
                target,
                operation,
                control,
            } => return self.request_npc(id, target, operation, control),
            Action::CastNpc {
                target,
                spell,
                control,
            } => {
                return self.request_npc_spell(id, target, spell, control);
            }
            Action::BoostWornItem {slot,item} => return self.boost_worn_item(id,slot,item),
            Action::ChemistryItem {slot,item,operation,worn} => return self.chemistry_item(id,slot,item,operation,worn),
            Action::ResumeChemistry {token,choice} => return self.resume_chemistry(id,token,choice),
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
            Action::UseLocation {
                coords,
                id: location_id,
                slot,
                object,
                control,
            } => {
                return self.request_location_use(id, coords, location_id, slot, object, control);
            }
            Action::ResumeBankCount { token, count } => {
                let remembered = self
                    .banking
                    .as_ref()
                    .and_then(|config| config.last_quantity.get(&p.data.variables).ok());
                let mut trace = crate::bank_actions::CountTrace {
                    tick: self.clock.0,
                    actor: id,
                    token,
                    pending: p.bank_count,
                    input: count,
                    remembered_before: remembered,
                    remembered_after: remembered,
                    outcome: crate::bank_actions::CountOutcome::Stale,
                };
                if p.bank_count.is_none_or(|pending| pending.0 != token) {
                    self.events.push(WorldEvent::BankCount(trace));
                    return Ok(());
                }
                let config = self.banking.as_ref().ok_or(ActionError::InvalidBank)?;
                let p = self.players.get_mut(id.slot as usize).unwrap();
                let (_, purpose) = p.bank_count.take().unwrap();
                p.protection.suspended = false;
                if p.protection.delayed() {
                    trace.outcome = crate::bank_actions::CountOutcome::Delayed;
                    self.events.push(WorldEvent::BankCount(trace));
                    return Err(ActionError::Delayed);
                }
                let result = config.resume_count(&mut p.data, &self.object_types, purpose, count);
                trace.remembered_after = config.last_quantity.get(&p.data.variables).ok();
                trace.outcome = if result.is_ok() {
                    crate::bank_actions::CountOutcome::Resolved
                } else {
                    crate::bank_actions::CountOutcome::Error
                };
                if let Ok(events) = &result {
                    for event in events {
                        if let crate::bank_actions::Event::WornDeposited { slot, item } = event {
                            self.worn_changed(id, *slot, *item);
                        }
                    }
                }
                self.events.push(WorldEvent::Bank { actor: id, result });
                self.events.push(WorldEvent::BankCount(trace));
            }
            Action::DepositBoxAll{worn} => self.deposit_box_all(id,worn)?,
            Action::DepositBoxItem{slot,item,operation} => self.deposit_box_item(id,slot,item,operation)?,
            Action::DepositBoxMode{mode} => self.deposit_box_mode(id,mode)?,
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
                        if let crate::bank_actions::Event::WornDeposited { slot, item } = event {
                            self.worn_changed(id, *slot, *item);
                        }
                    }
                }
                self.events.push(WorldEvent::Bank { actor: id, result });
            }
            Action::EquipSlot { slot } | Action::UnequipSlot { slot } | Action::UnequipItemSlot {slot,..} => {
                let unequipping = matches!(action, Action::UnequipSlot { .. }|Action::UnequipItemSlot{..});
                if let Action::UnequipItemSlot{item,..}=action {
                    if !p.data.equipment.get(slot).is_some_and(|current|current.id==item){return Err(ActionError::InvalidItem);}
                }
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
                        if let crate::equipment::Event::Changed {
                            slot,
                            id: changed_item,
                        } = event
                        {
                            self.worn_changed(id, *slot, *changed_item);
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
            Action::OpenItemPack { slot, item } => self.open_item_pack(id, slot, item)?,
            Action::SwapWornItem { slot, item } => self.swap_worn_item(id,slot,item)?,
            Action::ConvertHeldItem { slot, item } => self.convert_held_item(id,slot,item)?,
            Action::EmptyContainer { slot, item } => self.empty_container(id,slot,item)?,
            Action::OpenCoinPouch { slot, item, all } => self.open_coin_pouch(id,slot,item,all)?,
            Action::CleanHerb { slot, item } => self.clean_herb(id, slot, item)?,
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
                let native = self.consumable_rules.is_some();
                let plan = if let Some(rules) = &self.consumable_rules {
                    let policy = *rules.potions.get(&expected).ok_or(ActionError::InvalidItem)?;
                    if policy.kind != crate::consumable_clocks::Kind::Potion
                        || policy.consume_delay != crate::potions::DRINK_DELAY || policy.combat_delay != 0
                        || (policy.members && !p.member) { return Err(ActionError::InvalidItem); }
                    if empty { None } else { Some(p.consumables.plan(self.clock.0,policy,p.action_delay)
                        .map_err(|e| if e == crate::consumable_clocks::Error::Delayed {ActionError::Delayed} else {ActionError::InvalidItem})?) }
                } else { None };
                let mut emitted = Vec::new();
                let consumed=crate::potions::consume_slot(
                    potion,
                    slot,
                    empty,
                    &mut p.data.inventory,
                    &mut p.data.stats,
                    &mut p.data.energy,
                    &self.object_types,
                    self.eating_animation,
                    |e| emitted.push(e),
                )
                .map_err(|_| ActionError::InvalidStat)?;
                if !consumed {return Err(ActionError::InvalidItem);}
                if let Some(plan) = plan { p.consumables=plan.state; p.action_delay=plan.action_delay; }
                for event in emitted {
                    if let crate::potions::Event::Stat(update) = &event {
                        p.record_stat_update(
                            self.tutorial_rules_completed_varp.is_some(),
                            *update,
                        )?;
                    }
                    if let crate::potions::Event::Delay(cycles) = event {
                        if !native {
                            p.protection.delay(cycles);
                            p.protection.suspended = true;
                            p.food_delay = true;
                        }
                    }
                    self.events.push(WorldEvent::Potion { actor: id, event });
                }
            }
            Action::EatSlot { slot } | Action::EatItemSlot { slot, .. } => {
                let expected = match action {
                    Action::EatItemSlot { item, .. } => Some(item),
                    _ => None,
                };
                let p = self.players.get_mut(id.slot as usize).unwrap();
                // A visible HP edit does not complete the existing death lifecycle.
                // Reject before cancellation, item/stat publication or native clocks.
                if p.death_queued || p.data.stats.current(3) as u8 == 0 {
                    return Err(ActionError::Protected);
                }
                // HeldOpScript.launchLenient cancels the previous coroutine even
                // when objectVerify subsequently rejects the delayed operation.
                p.begin_held_action();
                let item = p.data.inventory.get(slot);
                let error = if p.protection.delayed() {
                    Some(ActionError::Delayed)
                } else if item.is_none_or(|item| {
                    !self.foods.contains_key(&item.id)
                        || expected.is_some_and(|expected| item.id != expected)
                }) {
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
                if self.member_foods.contains(&item.unwrap().id) && !p.member {
                    self.events.push(WorldEvent::LocationMessage{actor:id,message:"You need to be a member to eat that."});
                    return Err(ActionError::InvalidItem);
                }
                let food = &self.foods[&item.unwrap().id];
                let adjusted=match self.food_healing.get(&food.id){Some(rule)=>Some(crate::food::Edible{heal:rule.amount(p.data.stats.base(3) as u8).map_err(|_|ActionError::InvalidStat)?,..food.clone()}),None=>None};
                let food=adjusted.as_ref().unwrap_or(food);
                let plan = if let Some(rules) = &self.consumable_rules {
                    let policy = *rules.foods.get(&food.id).ok_or(ActionError::InvalidItem)?;
                    if policy.kind == crate::consumable_clocks::Kind::Potion || policy.consume_delay != food.delay
                        || (policy.members && !p.member) { return Err(ActionError::InvalidItem); }
                    Some(p.consumables.plan(self.clock.0,policy,p.action_delay)
                        .map_err(|e| if e == crate::consumable_clocks::Error::Delayed {ActionError::Delayed} else {ActionError::InvalidItem})?)
                } else { None };

                let mut emitted = Vec::new();
                let selected = self.slot_preserving_foods.contains(&food.id);
                let hp_before = p.data.stats.current(3) as u8;
                let fine_xp_before = p.data.stats.fine_xp(3);
                let energy_percent=self.food_energy.get(&food.id).copied().unwrap_or(0);
                if !(0..=100).contains(&energy_percent){return Err(ActionError::InvalidItem);}
                // Reuse fixed-point potion arithmetic; donor food restore never
                // lowers a pre-existing value above the ordinary energy cap.
                let energy_after=crate::potions::restored_energy(p.data.energy,energy_percent,10000).max(p.data.energy);
                let mut boosts=self.food_boosts.get(&food.id).cloned().unwrap_or_default();
                let mut subtractions=self.food_subtractions.get(&food.id).cloned().unwrap_or_default();
                if let Some(rule)=self.food_formulas.get(&food.id){let(b,s)=rule.resolve(&p.data.stats,3).map_err(|_|ActionError::InvalidStat)?;boosts.extend(b);subtractions.extend(s);}
                let boosts=boosts.as_slice();let subtractions=subtractions.as_slice();
                let restores=self.food_stat_healing.get(&food.id).map(Vec::as_slice).unwrap_or(&[]);
                let eaten = if !restores.is_empty() {
                    crate::food::eat_slot_with_stat_healing(food,slot,&mut p.data.inventory,&mut p.data.stats,
                        &self.object_types,3,self.eating_animation,boosts,subtractions,restores,self.food_random_healing.get(&food.id).copied(),random,|event|emitted.push(event))
                } else if !subtractions.is_empty() {
                    crate::food::eat_slot_with_secondary_effects(food,slot,&mut p.data.inventory,&mut p.data.stats,
                        &self.object_types,3,self.eating_animation,boosts,subtractions,self.food_random_healing.get(&food.id).copied(),random,|event|emitted.push(event))
                } else if let Some(&healing)=self.food_random_healing.get(&food.id) {
                    crate::food::eat_slot_with_random_healing(food,slot,&mut p.data.inventory,&mut p.data.stats,
                        &self.object_types,3,self.eating_animation,boosts,healing,random,|event|emitted.push(event))
                } else {
                    crate::food::eat_slot_with_boosts(food,slot,&mut p.data.inventory,&mut p.data.stats,
                        &self.object_types,3,self.eating_animation,boosts,|event|emitted.push(event))
                }.map_err(|_| ActionError::InvalidStat)?;
                if !eaten {return Err(ActionError::InvalidItem);}
                if let Some(plan) = plan { p.consumables=plan.state; p.action_delay=plan.action_delay; }
                if energy_after!=p.data.energy {p.data.energy=energy_after;emitted.push(crate::food::Event::Energy(energy_after));}
                if eaten && (selected || plan.is_some()) {
                    self.events.push(WorldEvent::FoodTransition {
                        actor: id,
                        trace: crate::food::Transition {
                            tick: self.clock.0,
                            slot,
                            before: item.unwrap(),
                            after: p.data.inventory.get(slot),
                            hp_before,
                            hp_after: p.data.stats.current(3) as u8,
                            fine_xp_before,
                            fine_xp_after: p.data.stats.fine_xp(3),
                            configured_delay: food.delay,
                            attack_delay: plan.map(|_| p.consumables.remaining_attack_delay(self.clock.0,p.action_delay)),
                        },
                    });
                }
                for event in emitted {
                    if let crate::food::Event::Stat(update) = &event {
                        p.record_stat_update(
                            self.tutorial_rules_completed_varp.is_some(),
                            *update,
                        )?;
                    }
                    if let crate::food::Event::Delay(cycles) = event {
                        if plan.is_none() {
                            p.protection.delay(cycles);
                            p.protection.suspended = true;
                            p.food_delay = true;
                        }
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
                if p.movement.rooted || p.pickpocket_stunned() {
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
                if p.pending_location_action.take().is_some(){p.protection.suspended=false;}
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
                if p.pending_chemistry.take().is_some() {
                    p.protection.suspended=false;
                    self.events.push(WorldEvent::ChemistryDialog{actor:id,token:None,remaining:p.data.chemistry.remaining(),breaking:false});
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
                // clearPendingAction cancels weak work and the active coroutine;
                // ordinary/strong queues and the existing delay clock survive.
                p.queues.clear_weak();
                p.bank_count = None;
                p.prayer_notice = None;
                p.begin_held_action();
                self.events.push(WorldEvent::ResetFaceEntity { actor: id });
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
                for shop in self.shops.values_mut() {
                    shop.restock(self.clock.0);
                }
            }
            self.events.push(WorldEvent::Phase(phase, self.clock.0));
            match phase {
                Phase::ControllerMain => {
                    self.process_cinderworks_encounters();
                    self.process_cinderworks_recovery();
                    self.active_trees.tick(self.clock.0, &self.locations);
                    // Gathering deadlines use the action clock. Processing before
                    // PlayerMain keeps N ticks of visible stump lifetime; the
                    // post-tick clock has already advanced and would expire early.
                    for pending in self
                        .location_timers
                        .replacements_due(self.clock, &self.locations)
                    {
                        let result = if !pending.current {
                            "stale"
                        } else if let Err(error) = self.change_location(pending.original, true) {
                            self.events.push(WorldEvent::LocationError(error));
                            "rejected"
                        } else {
                            "restored"
                        };
                        self.events.push(WorldEvent::LocationRestoration(
                            crate::loc_timers::Restoration {
                                tick: self.clock.0,
                                pending,
                                result,
                            },
                        ));
                    }
                }
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
                        if self.process_npc_death(id, random) {
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
                            if self
                                .npc_death
                                .as_ref()
                                .is_some_and(|config| config.chickens.contains_key(&npc.type_id))
                            {
                                self.events.push(WorldEvent::NpcImpact(
                                    crate::hits::NpcImpactTrace {
                                        tick: self.clock.0,
                                        npc: entry.id,
                                        npc_type: npc.type_id,
                                        hp_before: npc.stats[3],
                                        impact,
                                    },
                                ));
                            }
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
                        if self.process_npc_death(id, random) {
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
                    // NpcPostTickProcess updates zone registry membership before
                    // publishing this cycle's protocol information.
                    let moved: Vec<_> = self
                        .npcs
                        .iter()
                        .filter(|(_, e)| {
                            let m = &e.npc.movement;
                            m.last_movement >= self.clock.0 - 1
                                || m.pending_teleport
                                || m.pending_telejump
                        })
                        .map(|(_, e)| e.id)
                        .collect();
                    for id in moved {
                        self.update_npc_hunt_zone(id, false);
                    }
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
                    let mut spade_completions=Vec::new();
                    let mut chemistry_worn_changes=Vec::new();
                    let alive_npcs: HashSet<_> = self
                        .npcs
                        .iter()
                        .filter(|(_, e)| e.npc.stats[3] > 0)
                        .map(|(_, e)| e.id)
                        .collect();
                    for slot in self.order.clone() {
                        if self.players.get(slot).is_some_and(|p|p.cinderworks_checkpoint.is_some()) { continue; }
                        self.process_tower_arrival(slot);
                        self.process_prayer(slot,random);
                        self.resume_tutorial_fire(slot);
                        self.resume_location_action(slot);
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
                                            let drop = p.pending_drop.take().is_some() | p.pending_chemistry.take().is_some();
                                            let notice = p.prayer_notice.take().is_some();
                                            let dialogue = p.dialogue.take().is_some();
                                            let quantity = p.quantity_dialog.take().is_some();
                                            p.quantity_input = false;
                                            let prayer = matches!(
                                                p.prayer,
                                                Some(crate::prayer::State::Confirm { .. })
                                            );
                                            if prayer {
                                                p.prayer = None;
                                            }
                                            if count
                                                || drop
                                                || notice
                                                || dialogue
                                                || prayer
                                                || quantity
                                            {
                                                p.protection.suspended = false;
                                            }
                                            p.has_modal = false;
                                            queues.clear_weak();
                                            self.events
                                                .push(WorldEvent::CloseModals { actor: p.id });
                                        }
                                        if let crate::queues::Event::Launch(queue) = event {
                                            match queue.args {
                                                Deferred::ItemPack { pack, rule } => {
                                                    item_pack_actions::complete(
                                                        p,
                                                        queues,
                                                        &self.object_types,
                                                        pack,
                                                        rule,
                                                        self.member_item_packs.contains(&pack),
                                                        &mut self.events,
                                                    );
                                                }
                                                Deferred::UnfinishedPotion(job) => {
                                                    unfinished_potion_actions::complete(p, queues, job, &self.object_types, self.clock.0, &mut self.events);
                                                }
                                                Deferred::FinishedPotion(job) => {
                                                    finished_potion_actions::complete(p, queues, job, &self.object_types, self.clock.0, &mut self.events, random, &mut chemistry_worn_changes);
                                                }
                                                Deferred::PotionUpgrade(job) => {
                                                    potion_upgrade_actions::complete(p, queues, job, &self.object_types, self.clock.0, &mut self.events, random, &mut chemistry_worn_changes);
                                                }
                                                Deferred::CombatMixing(job) => {
                                                    combat_mixing_actions::complete(p, queues, job, &self.object_types, self.clock.0, &mut self.events, random);
                                                }
                                                Deferred::Dough(job) => {
                                                    dough_actions::complete(p, queues, job, &self.object_types, self.clock.0, &mut self.events);
                                                }
                                                Deferred::SwampTar(job) => {
                                                    swamp_tar_actions::complete(p,queues,job,&self.object_types,self.clock.0,&mut self.events);
                                                }
                                                Deferred::Grinding(job) => {
                                                    grinding_actions::complete(p, queues, job, &self.object_types, self.clock.0, &mut self.events);
                                                }
                                                Deferred::ToolCrafting(pending) => {tool_crafting_actions::complete(p,queues,pending,&self.object_types,self.tutorial_rules_completed_varp.is_some(),&mut self.events,random);}
                                                Deferred::Pickpocket(job) => {
                                                    if let Some(config)=&self.pickpocket {
                                                        let target=job.target;
                                                        let caught=thieving_actions::complete(p,queues,job,config,&self.object_types,self.npcs.get(target.slot as usize),self.player_hit_queues,self.clock.0,self.tutorial_rules_completed_varp.is_some(),&mut self.events,random);
                                                        if caught {if let Some(n)=self.npcs.get_mut(target.slot as usize).filter(|n|n.id==target) {n.npc.face_player=Some(p.id);n.npc.face_changed=true;}}
                                                    }
                                                }
                                                Deferred::Cooking(job) => {
                                                    let first=self.events.len();
                                                    if let Some(config)=&self.cooking {cooking_actions::complete(p,job,config,&self.object_types,self.locations.revision(job.target),self.clock.0,self.tutorial_rules_completed_varp.is_some(),&mut self.events,random);}
                                                    cinderworks_services::observe(p,self.cinderworks.as_ref(),first,&mut self.events);
                                                }
                                                Deferred::CinderworksFishing(job) => {
                                                    if let Some(config)=&self.cinderworks {
                                                        let valid=self.npcs.get(job.target.slot as usize).is_some_and(|n|n.id==job.target&&n.npc.movement.coords==job.coords&&config.work.is_some_and(|w|w.fishing_spot==n.npc.type_id));
                                                        cinderworks_fishing::complete(p,queues,job,config,valid,&self.object_types,self.tutorial_rules_completed_varp.is_some(),&mut self.events);
                                                    }
                                                }
                                                Deferred::WaterFilling(job) => {
                                                    if let Some(config) = &self.water_sources {
                                                        water_source_actions::complete(p, queues, job, config, &self.object_types, self.locations.revision(job.target), self.clock.0, &mut self.events);
                                                    }
                                                }
                                                Deferred::Spade(job) => {if let Some(done)=spade_actions::complete(p,job,&mut self.events){spade_completions.push(done);}}
                                                Deferred::Bookcase(job) => {bookcase_actions::complete(p,job,self.locations.revision(job.target),&mut self.events,random);}
                                                Deferred::Milking(job) => {
                                                    if let Some(config) = &self.milking {
                                                        milking_actions::complete(
                                                            p, queues, job, config,
                                                            &self.object_types,
                                                            self.locations.revision(job.target),
                                                            self.clock.0,
                                                            &mut self.events,
                                                        );
                                                    }
                                                }
                                                Deferred::AssemblyBatch(pending) => {
                                                    if p.has_modal || p.pending_logout || p.saving {return;}
                                                    let job=pending.job;let skill=job.recipe.skill;
                                                    let (outcome,advancement)=job.complete_awarding(&mut p.data.inventory,&self.object_types,&mut p.data.stats,p.data.xp_rate_hundreds,p.member);
                                                    if let Some(advancement)=advancement {self.events.push(WorldEvent::SkillAdvanced{actor:p.id,skill:job.recipe.skill,advancement});}
                                                    if outcome==crate::assembly::Outcome::Made {self.events.push(WorldEvent::AssemblyBatchMade{actor:p.id,trace:crate::assembly::BatchTransition{job,queued_at:pending.queued_at,committed_at:self.clock.0}});}
                                                    self.events.push(WorldEvent::LocationMessage{actor:p.id,message:crate::assembly::outcome_message_for_skill(outcome,skill)});
                                                }
                                                Deferred::Runecrafting(pending)=>{let revision=self.locations.revision(pending.location);runecrafting_actions::complete(p,pending,self.runecrafting.as_ref(),&self.object_types,revision,self.tutorial_rules_completed_varp.is_some(),random,&mut self.events);}
                                                Deferred::FacilityCrafting(pending) => {let revision=self.locations.revision(pending.request.location);facility_crafting_actions::complete(p,queues,pending,&self.object_types,revision,self.tutorial_rules_completed_varp.is_some(),&mut self.events,random);}
                                                Deferred::AssemblyProduction(pending) => {assembly_production_actions::complete(p,queues,pending,&self.object_types,[self.wine,self.zamorak_wine],self.clock.0,&mut self.events);}
                                                Deferred::Assembly(pending) => {
                                                    held_use_actions::complete_assembly_reward_with_tools(p,pending.job,&self.object_types,pending.queued_at,self.clock.0,pending.xp_tenths,pending.required_tools,&mut self.events);
                                                }
                                                Deferred::Smithing(job) => {
                                                    let first=self.events.len();
                                                    if let Some(config) = self.smithing {
                                                        if let Err(error) =
                                                            smithing_actions::complete(
                                                                p,
                                                                queues,
                                                                config,
                                                                job,
                                                                &self.object_types,
                                                                self.tutorial_rules_completed_varp
                                                                    .is_some(),
                                                                &mut self.events,
                                                                random,
                                                            )
                                                        {
                                                            p.pending_logout = true;
                                                            self.events.push(
                                                                WorldEvent::ActionRejected {
                                                                    actor: p.id,
                                                                    error,
                                                                },
                                                            );
                                                        }
                                                    }
                                                    cinderworks_services::observe(p,self.cinderworks.as_ref(),first,&mut self.events);
                                                }
                                                Deferred::StandardHit(hit) => {
                                                    if let Some(config) = self.player_hit_queues {
                                                        if let Err(error) = stat_event_actions::hit(
                                                            p,
                                                            queues,
                                                            config,
                                                            self.tutorial_rules_completed_varp
                                                                .is_some(),
                                                            hit,
                                                            self.clock,
                                                            &alive_npcs,
                                                            &mut self.events,
                                                        ) {
                                                            p.pending_logout = true;
                                                            self.events.push(
                                                                WorldEvent::ActionRejected {
                                                                    actor: p.id,
                                                                    error,
                                                                },
                                                            );
                                                        }
                                                    }
                                                }
                                                Deferred::PlayerDeath => {
                                                    if !p.death_queued {
                                                        p.death_queued = true;
                                                        p.consumables = Default::default();p.pending_location_action=None;
                                                        self.events.push(
                                                            WorldEvent::DeathRequested(
                                                                crate::hits::Target::Player(p.id),
                                                            ),
                                                        );
                                                    }
                                                }
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
                                                        p.tutorial_fire = None;
                                                        p.pending_location_action=None;
                                                        p.pending_door = None;
                                                        p.prayer = None;
                                                        p.pending_pickup = None;
                                                        p.pending_drop = None; p.pending_chemistry = None;
                                                        p.dialogue = None;
                                                        p.quantity_dialog = None;
                                                        p.quantity_input = false;
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
                                let success =
                                    self.process_player_soft_timer(slot, timers, timer, random);
                                main_failed |= !success;
                                success
                            });
                            let p = self.players.get_mut(slot).unwrap();
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
                            stat_event_actions::publish(
                                p,
                                self.tutorial_rules_completed_varp,
                                self.player_hit_queues,
                                &mut self.events,
                            );
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
                                                        p.consumables = Default::default();p.pending_location_action=None;
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
                                let quantity = p.quantity_dialog.take().is_some();
                                p.quantity_input = false;
                                let count = p.bank_count.take().is_some();
                                let drop = p.pending_drop.take().is_some() | p.pending_chemistry.take().is_some();
                                let notice = p.prayer_notice.take().is_some();
                                let prayer =
                                    matches!(p.prayer, Some(crate::prayer::State::Confirm { .. }));
                                if prayer {
                                    p.prayer = None;
                                }
                                if dialogue || count || drop || notice || prayer || quantity {
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
                            if !interacted && !p.pickpocket_stunned() {
                                p.movement.process(&mut self.collision, self.clock.0);
                            }
                            p.data.coords = p.movement.coords;
                            if !interacted {
                                self.process_npc_interaction(slot, false, random);
                            }
                        }
                        self.process_door(slot, random);
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
                    for (actor,job) in spade_completions {self.complete_spade_hook(actor,job);}
                    for (actor,slot,item) in chemistry_worn_changes { self.worn_changed(actor,slot,item); }
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
                        for config in [self.wine,self.zamorak_wine].into_iter().flatten() {
                            if config.pending(&data) && protection.current_clock.checked_add(crate::wine::INTERVAL).is_some() {
                                soft_timers.schedule(config.timer,protection.current_clock,crate::wine::INTERVAL).expect("positive wine interval");
                            }
                        }
                        if let Some(config) = &self.music {
                            soft_timers
                                .schedule(config.timer, protection.current_clock, 1)
                                .expect("positive interval");
                        }
                        // FloorOneScript restores/syncs marks before relocating the
                        // player. Keep effect coordinates and render after admission.
                        let tower_diary_login = self.tower_tanning.as_ref().and_then(|config| {
                            data.tower_diary.map(|progress| {
                                let sync = progress
                                    .marks(
                                        config.skills.map(|id| data.stats.base(id) as i32),
                                        data.tower_arrival.staff_taken,
                                    )
                                    .session_sync();
                                let progress: crate::tower_diary::Progress = sync.marks.into();
                                data.tower_diary = Some(progress);
                                data.tower_arrival.diary_progress |= progress.has_progress();
                                (sync, data.coords)
                            })
                        });
                        if let Some(config) = &self.tower_restore {
                            soft_timers
                                .schedule(config.timer, protection.current_clock, 8)
                                .expect("positive interval");
                        }
                        let player = Player {
                            cinderworks_checkpoint: None,
                            cinderworks_recovery_at:None,
                            member: false,
                            dough_request: None,
                            tool_crafting_request:None,
                            counted_assembly_request:None,
                            assembly_production_request:None,
                            finished_potion_request: None,
                            potion_upgrade_request: None,
                            combat_mixing_request: None,
                            unfinished_potion_request: None,
                            tower_restore_pending: false,
                            tower_tastes: Default::default(),
                            tower_diary_login,
                            stat_events: Default::default(),
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
                            consumables: Default::default(),
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
                            pickpocket_stun_until:i32::MIN,
                            tutorial_fire: None,
                            main_modal: None,
                            location_interaction: None,
                            location_revision: None,
                            woodcutting_animation_at: 0,
                            pending_location_action: None,
                            pending_door: None,
                            observer_id: None,
                            autocast_picker_defensive: None,
                            loot_drop_duration: None,
                            object_interaction: None,
                            npc_interaction: None,
                            pending_pickup: None,
                            pending_drop: None,
                            pending_chemistry: None,
                            prayer: None,
                            prayer_notice: None,
                            overhead: None,
                            dialogue: None,
                            bank_count: None,
                            shop_session: None,
                            quantity_dialog: None,
                            quantity_input: false,
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
                        if let Some(config) = &self.tower_restore {
                            let p = self.players.get_mut(slot).unwrap();
                            p.tower_restore_pending =
                                crate::tower_layout::contains(config.origin, p.movement.coords);
                        }
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
        world.wearpos = Some(crate::wearpos::Config {
            special: binding(15).base,
            stance: binding(0).base,
            category: binding(16),
            level: binding(12),
            decimal: binding(13),
            combat_stats: [0, 1, 2, 3, 4, 6, 5],
            items: BTreeMap::from([
                (
                    1,
                    crate::wearpos::Item {
                        category: 18,
                        sound: Some(10),
                        player_op: Some("Pelt".into()),
                        ..Default::default()
                    },
                ),
                (
                    2,
                    crate::wearpos::Item {
                        category: 0,
                        sound: Some(20),
                        ..Default::default()
                    },
                ),
            ]),
        });
        world.bulwark_wear = Some(crate::bulwark_wear::Config {
            items: [1].into(),
            passive_delay: binding(17).base,
            glow: 1336,
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
        let equipped_clock = world.clock.0;
        world.tick().unwrap();
        let events = world.drain_events();
        assert_eq!(
            world.data(actor).unwrap().variables.get(17),
            equipped_clock + 8
        );
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    WorldEvent::PlayerSpotAnimation {
                        spot: 1336,
                        slot: 0,
                        delay: 0,
                        height: 0,
                        ..
                    }
                ))
                .count(),
            1
        );
        let wearpos: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                WorldEvent::WearposEffects { result, .. } => Some(result.as_ref().unwrap()),
                _ => None,
            })
            .collect();
        assert_eq!(wearpos.len(), 2);
        for (index, (sound, category, op)) in [(10, 18, Some("Pelt".to_owned())), (20, 0, None)]
            .into_iter()
            .enumerate()
        {
            use crate::wearpos::Event as WearEvent;
            assert_eq!(wearpos[index][0], WearEvent::Sound(sound));
            assert_eq!(wearpos[index][1], WearEvent::PlayerOp(op));
            assert!(
                wearpos[index].contains(&WearEvent::Variable(crate::variables::Update {
                    id: 16,
                    value: category
                }))
            );
        }
        assert!(world.data(actor).unwrap().variables.contains(15));
        let updates: Vec<_> = events
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
        let events = world.drain_events();
        let traces = events
            .iter()
            .filter_map(|event| match event {
                WorldEvent::BankCount(trace) => Some(trace),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            traces.iter().map(|trace| trace.outcome).collect::<Vec<_>>(),
            [
                crate::bank_actions::CountOutcome::Stale,
                crate::bank_actions::CountOutcome::Resolved,
                crate::bank_actions::CountOutcome::Stale,
                crate::bank_actions::CountOutcome::Error,
            ]
        );
        assert_eq!(traces[0].token, old);
        assert_ne!(traces[0].pending.unwrap().0, old);
        assert!(traces[2].pending.is_none());
        assert_eq!(traces[3].input, i32::MIN);
        assert_eq!(
            (traces[3].remembered_before, traces[3].remembered_after),
            (Some(5), Some(5))
        );
        assert!(events.iter().any(|event| matches!(
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
    fn bank_delayed_count_consumes_prompt_without_transfer_or_remembered_quantity() {
        let (mut world, actor) = admitted();
        let config = crate::bank_actions::tests::config();
        let data = world.data_mut(actor).unwrap();
        config.capacity.set(&mut data.variables, 10).unwrap();
        config.last_quantity.set(&mut data.variables, 3).unwrap();
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
        world.banking = Some(config.clone());
        world
            .submit(
                actor,
                Action::Bank(crate::bank_actions::Request::Button {
                    deposit: true,
                    slot: 0,
                    op: 7,
                }),
            )
            .unwrap();
        world.tick().unwrap();
        let token = world.count_dialog_token(actor).unwrap().unwrap();
        world.drain_events();
        world.delay_player(actor, 2).unwrap();
        world
            .submit(actor, Action::ResumeBankCount { token, count: 5 })
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.count_dialog_token(actor).unwrap(), None);
        assert!(!world.protection(actor).unwrap().suspended);
        let events = world.drain_events();
        let trace = events
            .iter()
            .find_map(|event| match event {
                WorldEvent::BankCount(trace) => Some(trace),
                _ => None,
            })
            .unwrap();
        assert_eq!(trace.outcome, crate::bank_actions::CountOutcome::Delayed);
        assert_eq!(trace.pending.map(|p| p.0), Some(token));
        assert_eq!(
            (trace.remembered_before, trace.remembered_after),
            (Some(3), Some(3))
        );
        assert!(trace
            .json()
            .contains("\"outcome\":\"delayed\",\"transfer_accepted\":null"));
        assert!(events.iter().any(|event| matches!(
            event,
            WorldEvent::ActionRejected {
                error: ActionError::Delayed,
                ..
            }
        )));
        assert!(!events
            .iter()
            .any(|event| matches!(event, WorldEvent::Bank { .. })));
        for _ in 0..3 {
            world.tick().unwrap();
        }
        // A duplicate server token cannot resurrect the consumed coroutine.
        world
            .submit(actor, Action::ResumeBankCount { token, count: 5 })
            .unwrap();
        world.tick().unwrap();
        let data = world.data(actor).unwrap();
        assert_eq!(data.inventory.get(0).unwrap().count, 20);
        assert!(data.bank.get(0).is_none());
        assert_eq!(config.last_quantity.get(&data.variables), Ok(3));
        assert!(!world
            .drain_events()
            .iter()
            .any(|event| matches!(event, WorldEvent::Bank { .. })));
    }
    #[test]
    fn bank_count_resolves_current_slot_and_remembers_quantity_when_slot_is_empty() {
        // Characterization of the existing slot-bound purpose, not native wire
        // generation protection or a compiled Kotlin parity claim.
        for replacement in [
            None,
            Some(crate::inventory::Item {
                id: 314,
                count: 9,
                vars: 0,
            }),
        ] {
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
            world.object_types.stackable.extend([995, 314]);
            world.banking = Some(config.clone());
            world
                .submit(
                    actor,
                    Action::Bank(crate::bank_actions::Request::Button {
                        deposit: true,
                        slot: 0,
                        op: 7,
                    }),
                )
                .unwrap();
            world.tick().unwrap();
            let token = world.count_dialog_token(actor).unwrap().unwrap();
            // Trusted fixture mutation while suspended: original coins remain
            // in another slot; the count purpose captures a slot, not item ID.
            let data = world.data_mut(actor).unwrap();
            let original = data.inventory.get(0);
            data.inventory.restore_slot(1, original).unwrap();
            data.inventory.restore_slot(0, replacement).unwrap();
            world
                .submit(actor, Action::ResumeBankCount { token, count: -5 })
                .unwrap();
            world.tick().unwrap();
            let data = world.data(actor).unwrap();
            assert_eq!(data.inventory.get(1).unwrap().count, 20);
            assert_eq!(config.last_quantity.get(&data.variables), Ok(5));
            if replacement.is_some() {
                assert_eq!(data.inventory.get(0).unwrap().count, 4);
                assert_eq!(data.bank.get(0).unwrap().id, 314);
                assert_eq!(data.bank.get(0).unwrap().count, 5);
            } else {
                assert!(data.inventory.get(0).is_none());
                assert!(data.bank.get(0).is_none());
            }
            assert_eq!(world.count_dialog_token(actor).unwrap(), None);
            assert!(!world.protection(actor).unwrap().suspended);
            let traces = world
                .drain_events()
                .into_iter()
                .filter_map(|event| match event {
                    WorldEvent::BankCount(trace) => Some(trace),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(traces.len(), 1);
            assert_eq!(
                traces[0].outcome,
                crate::bank_actions::CountOutcome::Resolved
            );
            assert_eq!(traces[0].input, -5);
            assert_eq!(traces[0].remembered_after, Some(5));
            assert_eq!(
                traces[0].pending,
                Some((
                    token,
                    crate::bank_actions::CountPurpose::Transfer {
                        deposit: true,
                        slot: 0,
                    }
                ))
            );
            // Both a completed transfer and an empty-slot no-op resolve the
            // coroutine; neither is converted to an invented transfer verdict.
            assert!(traces[0].json().contains("\"transfer_accepted\":null"));
        }
    }
    #[test]
    fn queued_potion_consumption_uses_claimed_slot_and_refuses_failed_replacement() {
        use crate::{inventory::Item,potions::{Potion,Event}};
        for reject in [false,true] {
            let (mut w,a)=admitted();w.potions.insert(1,Potion{id:1,next:2,vial:3,sip:"sip".into(),remaining:"left".into(),effects:Vec::new()});
            if reject {w.object_types.dummy.insert(2);}
            for slot in [0,2] {w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:1,count:1,vars:0})).unwrap();}
            let before=w.data(a).unwrap().inventory.clone();w.submit(a,Action::DrinkSlot{slot:2,item:1,empty:false}).unwrap();w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().inventory.get(0),before.get(0));let events=w.drain_events();
            if reject {assert_eq!(w.data(a).unwrap().inventory,before);assert!(!w.protection(a).unwrap().delayed());assert!(!events.iter().any(|e|matches!(e,WorldEvent::Potion{..})));}
            else {assert_eq!(w.data(a).unwrap().inventory.get(2).unwrap().id,2);assert!(events.iter().any(|e|matches!(e,WorldEvent::Potion{event:Event::Animation(_),..})));}
        }
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
                    slot: 2,
                    item: 8,
                    empty: true,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(0), 53);
        assert_eq!(
            world.data(actor).unwrap().inventory.get(2),
            Some(Item {
                id: 8,
                count: 1,
                vars: 0
            })
        );
        assert!(world.data(actor).unwrap().inventory.get(0).is_none());
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
                    slot: 2,
                    item: 8,
                    empty: true,
                },
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().inventory.get(2).unwrap().id, 9);
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
    fn dynamic_food_uses_live_base_hp_and_invalid_rule_preserves_item(){
        for (base,current,expected,valid) in [(20,1,3,true),(99,98,99,true),(20,25,25,true),(20,1,1,false)]{let (mut w,actor)=admitted();w.foods.insert(991,crate::food::Edible{id:991,heal:0,delay:3,replacement:Some(992),message:"Melon".into()});w.food_healing.insert(991,crate::food::Healing{percent:if valid{5}else{101},constant:1,round_up:false});let p=w.players.get_mut(actor.slot as usize).unwrap();p.data.stats.set_base(3,base);p.data.stats.set_current(3,current);p.data.inventory.restore_slot(1,Some(crate::inventory::Item{id:991,count:1,vars:0})).unwrap();w.submit(actor,Action::EatItemSlot{slot:1,item:991}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.stats.current(3),expected);assert_eq!(p.stats.fine_xp(3),0);assert_eq!(p.inventory.get(1).unwrap().id,if valid{992}else{991});}
    }
    #[test]
    fn energy_food_eat_events_restore_exact_percent_and_failed_effects_never_award(){
        for (energy,expected) in [(0,Some(1000)),(9500,Some(10000)),(10000,None)]{
            let (mut w,actor)=admitted();w.foods.insert(991,crate::food::Edible{id:991,heal:11,delay:1,replacement:Some(992),message:"Summer pie".into()});w.food_boosts.insert(991,vec![crate::food::StatBoost{skill:16,amount:5}]);w.food_energy.insert(991,10);w.member_foods.insert(991);let p=w.players.get_mut(actor.slot as usize).unwrap();p.member=true;p.data.energy=energy;p.data.stats.set_base(3,99);p.data.stats.set_current(3,50);p.data.stats.set_base(16,99);p.data.stats.set_current(16,99);p.data.inventory.restore_slot(1,Some(crate::inventory::Item{id:991,count:1,vars:0})).unwrap();w.drain_events();w.submit(actor,Action::EatItemSlot{slot:1,item:991}).unwrap();w.tick().unwrap();let updates:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::Food{event:crate::food::Event::Energy(n),..}=e{Some(n)}else{None}).collect();assert_eq!(updates,expected.into_iter().collect::<Vec<_>>());let p=w.data(actor).unwrap();assert_eq!((p.stats.current(3),p.stats.current(16)),(61,104));assert_eq!(p.inventory.get(1).unwrap().id,992);assert_eq!(p.stats.fine_xp(16),0);if let Some(expected)=expected{assert!((expected..=(expected+25).min(10000)).contains(&p.energy));}
        }
        for bad in 0..3{let (mut w,actor)=admitted();w.foods.insert(991,crate::food::Edible{id:991,heal:11,delay:1,replacement:Some(992),message:"Summer pie".into()});w.food_boosts.insert(991,vec![crate::food::StatBoost{skill:16,amount:if bad==1{-1}else{5}}]);w.food_energy.insert(991,if bad==2{101}else{10});w.member_foods.insert(991);let p=w.players.get_mut(actor.slot as usize).unwrap();p.member=bad!=0;p.data.energy=500;p.data.stats.set_base(3,99);p.data.stats.set_current(3,50);p.data.stats.set_base(16,99);p.data.stats.set_current(16,99);p.data.inventory.restore_slot(1,Some(crate::inventory::Item{id:991,count:1,vars:0})).unwrap();w.drain_events();w.submit(actor,Action::EatItemSlot{slot:1,item:991}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.inventory.get(1).unwrap().id,991);assert_eq!((p.stats.current(3),p.stats.current(16)),(50,99));assert!((500..=524).contains(&p.energy));assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{event:crate::food::Event::Energy(_),..})));}
    }
    #[test]
    fn queued_random_food_respects_live_membership_selected_identity_and_remainder_stack() {
        struct Draw(u32);
        impl crate::npc::RandomSource for Draw {fn below(&mut self,n:u32)->u32{assert_eq!(n,4);self.0}}
        for (member,claimed,draw,expected,consumed) in [(true,991,0,8,true),(true,991,3,11,true),(false,991,0,1,false),(true,999,0,1,false)] {
            let (mut w,actor)=admitted();w.foods.insert(991,crate::food::Edible{id:991,heal:7,delay:3,replacement:Some(992),message:"Spider".into()});w.member_foods.insert(991);w.slot_preserving_foods.insert(991);w.food_random_healing.insert(991,crate::food::RandomHealing{minimum:7,maximum:10});w.object_types.stackable.insert(992);
            let p=w.players.get_mut(actor.slot as usize).unwrap();p.member=member;p.data.stats.set_base(3,99);p.data.stats.set_current(3,1);p.data.inventory.restore_slot(0,Some(crate::inventory::Item{id:992,count:3,vars:0})).unwrap();p.data.inventory.restore_slot(1,Some(crate::inventory::Item{id:991,count:1,vars:0})).unwrap();
            w.drain_events();w.submit(actor,Action::EatItemSlot{slot:1,item:claimed}).unwrap();w.tick_with_random(&mut Draw(draw)).unwrap();
            let p=w.data(actor).unwrap();assert_eq!(p.stats.current(3),expected);assert_eq!(p.stats.fine_xp(3),0);assert_eq!(p.inventory.get(1).is_none(),consumed);assert_eq!(p.inventory.get(0).unwrap().count,if consumed{4}else{3});
            assert_eq!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::FoodTransition{..})),consumed);
        }
    }




    #[test]
    fn queued_jangerberries_share_member_selected_hp_boost_prayer_heal_and_defence_transaction(){
        let(mut w,actor)=admitted();w.foods.insert(247,crate::food::Edible{id:247,heal:2,delay:3,replacement:None,message:"Berries".into()});w.food_boosts.insert(247,vec![crate::food::StatBoost{skill:0,amount:2},crate::food::StatBoost{skill:2,amount:1}]);w.food_subtractions.insert(247,vec![crate::food::StatSubtraction{skill:1,amount:1}]);w.food_stat_healing.insert(247,vec![crate::food::StatHealing{skill:5,amount:1}]);w.member_foods.insert(247);w.slot_preserving_foods.insert(247);let p=w.players.get_mut(actor.slot as usize).unwrap();for skill in[0,1,2,3,5]{p.data.stats.set_base(skill,99);p.data.stats.set_current(skill,99);}p.data.stats.set_current(3,50);p.data.stats.set_current(5,10);for slot in[0,2]{p.data.inventory.restore_slot(slot,Some(crate::inventory::Item{id:247,count:1,vars:0})).unwrap();}let before=p.data.stats.clone();w.submit(actor,Action::EatItemSlot{slot:2,item:247}).unwrap();w.tick().unwrap();assert_eq!(w.data(actor).unwrap().stats,before);assert_eq!(w.data(actor).unwrap().inventory.get(2).unwrap().id,247);w.players.get_mut(actor.slot as usize).unwrap().member=true;w.submit(actor,Action::EatItemSlot{slot:2,item:247}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert!(p.inventory.get(2).is_none());assert_eq!(p.inventory.get(0).unwrap().id,247);assert_eq!((p.stats.current(0),p.stats.current(1),p.stats.current(2),p.stats.current(3),p.stats.current(5)),(101,98,100,52,11));for skill in[0,1,2,3,5]{assert_eq!(p.stats.fine_xp(skill),0);}assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{event:crate::food::Event::Stat(u),..}if u.skill==5&&u.old==10&&u.level==11)));
    }
    #[test]
    fn queued_late_invalid_food_stat_healing_cannot_publish_earlier_hp_or_fixed_effects(){
        let(mut w,actor)=admitted();w.foods.insert(247,crate::food::Edible{id:247,heal:2,delay:3,replacement:None,message:"Berries".into()});w.food_boosts.insert(247,vec![crate::food::StatBoost{skill:0,amount:2}]);w.food_subtractions.insert(247,vec![crate::food::StatSubtraction{skill:1,amount:1}]);w.food_stat_healing.insert(247,vec![crate::food::StatHealing{skill:5,amount:1},crate::food::StatHealing{skill:24,amount:1}]);let p=w.players.get_mut(actor.slot as usize).unwrap();p.member=true;p.data.stats.set_base(3,99);p.data.stats.set_current(3,50);p.data.stats.set_current(5,0);p.data.inventory.restore_slot(2,Some(crate::inventory::Item{id:247,count:1,vars:0})).unwrap();let before=(p.data.inventory.slots().to_vec(),p.data.stats.clone());w.submit(actor,Action::EatItemSlot{slot:2,item:247}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.inventory.slots(),before.0.as_slice());assert_eq!(p.stats,before.1);assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{..}|WorldEvent::FoodTransition{..})));
    }
    #[test]
    fn queued_ale_formulas_use_live_base_boost_and_current_drain_without_boost_stacking(){
        let(mut w,actor)=admitted();w.foods.insert(1917,crate::food::Edible{id:1917,heal:1,delay:3,replacement:Some(1919),message:"Ale".into()});w.food_formulas.insert(1917,crate::food::StatEffects{boosts:vec![crate::food::StatFormula{skill:2,percent:2,constant:1}],subtractions:vec![crate::food::StatFormula{skill:0,percent:6,constant:1}]});let p=w.players.get_mut(actor.slot as usize).unwrap();for slot in[0,2]{p.data.inventory.restore_slot(slot,Some(crate::inventory::Item{id:1917,count:1,vars:0})).unwrap();}p.data.stats.set_base(3,99);p.data.stats.set_current(3,50);w.submit(actor,Action::EatItemSlot{slot:2,item:1917}).unwrap();let p=w.players.get_mut(actor.slot as usize).unwrap();for skill in[0,2]{p.data.stats.set_base(skill,99);p.data.stats.set_current(skill,99);}w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!((p.stats.current(0),p.stats.current(2),p.stats.current(3)),(93,101,51));assert_eq!(p.inventory.get(2).unwrap().id,1919);
        for _ in 0..4{w.tick().unwrap();}w.players.get_mut(actor.slot as usize).unwrap().data.stats.set_current(0,10);w.submit(actor,Action::EatItemSlot{slot:0,item:1917}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!((p.stats.current(0),p.stats.current(2),p.stats.current(3)),(9,101,52));assert_eq!(p.inventory.get(0).unwrap().id,1919);for skill in[0,2,3]{assert_eq!(p.stats.fine_xp(skill),0);}
    }
    #[test]
    fn queued_ale_invalid_formula_and_live_member_gate_cannot_partially_consume_or_apply_fixed_effects(){
        for mode in 0..3{let(mut w,actor)=admitted();w.foods.insert(5755,crate::food::Edible{id:5755,heal:1,delay:3,replacement:Some(1919),message:"Ale".into()});w.food_formulas.insert(5755,crate::food::StatEffects{boosts:vec![crate::food::StatFormula{skill:7,percent:5,constant:1}],subtractions:vec![crate::food::StatFormula{skill:if mode==0{24}else{0},percent:if mode==1{101}else{5},constant:2}]});w.food_boosts.insert(5755,vec![crate::food::StatBoost{skill:18,amount:2}]);w.member_foods.insert(5755);let p=w.players.get_mut(actor.slot as usize).unwrap();p.member=mode!=2;p.data.stats.set_base(3,99);p.data.stats.set_current(3,50);p.data.inventory.restore_slot(2,Some(crate::inventory::Item{id:5755,count:1,vars:0})).unwrap();let before=(p.data.inventory.slots().to_vec(),p.data.stats.clone());w.submit(actor,Action::EatItemSlot{slot:2,item:5755}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.inventory.slots(),before.0.as_slice());assert_eq!(p.stats,before.1);assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{..}|WorldEvent::FoodTransition{..})));}
    }
    #[test]
    fn queued_wine_drink_uses_shared_delay_and_selected_jug_flat_attack_drain(){
        let(mut w,actor)=admitted();w.foods.insert(1993,crate::food::Edible{id:1993,heal:11,delay:3,replacement:Some(1935),message:"You drink the wine.".into()});w.food_subtractions.insert(1993,vec![crate::food::StatSubtraction{skill:0,amount:2}]);w.slot_preserving_foods.insert(1993);
        let p=w.players.get_mut(actor.slot as usize).unwrap();p.data.stats.set_base(0,99);p.data.stats.set_current(0,5);p.data.stats.set_base(3,99);p.data.stats.set_current(3,50);p.data.stats.set_fine_xp(0,12345).unwrap();for slot in 0..p.data.inventory.slots().len(){p.data.inventory.restore_slot(slot,Some(crate::inventory::Item{id:1993,count:1,vars:0})).unwrap();}
        w.submit(actor,Action::EatItemSlot{slot:2,item:1993}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.inventory.get(2).unwrap().id,1935);assert_eq!(p.inventory.get(0).unwrap().id,1993);assert_eq!((p.stats.current(3),p.stats.current(0)),(61,3));assert_eq!(p.stats.fine_xp(0),12345);assert_eq!(p.stats.fine_xp(3),0);assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{event:crate::food::Event::Stat(u),..}if u.skill==0 && u.old==5 && u.level==3)));
        w.submit(actor,Action::EatItemSlot{slot:0,item:1993}).unwrap();w.tick().unwrap();assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id,1993);for _ in 0..3{w.tick().unwrap();}
        w.submit(actor,Action::EatItemSlot{slot:0,item:1993}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.inventory.get(0).unwrap().id,1935);assert_eq!((p.stats.current(3),p.stats.current(0)),(72,1));
    }
    #[test]
    fn queued_wine_drink_stale_slot_and_invalid_secondary_cannot_consume_or_heal(){
        for mode in 0..3{let(mut w,actor)=admitted();w.foods.insert(1993,crate::food::Edible{id:1993,heal:11,delay:3,replacement:Some(1935),message:"Wine".into()});w.food_subtractions.insert(1993,vec![crate::food::StatSubtraction{skill:0,amount:if mode==1{-2}else{2}}]);let p=w.players.get_mut(actor.slot as usize).unwrap();p.data.stats.set_current(3,1);p.data.stats.set_current(0,5);p.data.inventory.restore_slot(2,Some(crate::inventory::Item{id:1993,count:1,vars:if mode==2{1}else{0}})).unwrap();let before=(p.data.inventory.clone(),p.data.stats.clone());w.submit(actor,Action::EatItemSlot{slot:2,item:if mode==0{1935}else{1993}}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!(p.inventory.slots(),before.0.slots());assert_eq!(p.stats,before.1);assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::Food{..}|WorldEvent::FoodTransition{..})));}
    }
    #[test]
    fn queued_food_secondary_boosts_follow_actual_eat_dispatch(){
        let (mut w,actor)=admitted();w.foods.insert(991,crate::food::Edible{id:991,heal:11,delay:1,replacement:Some(992),message:"Wild pie".into()});w.food_boosts.insert(991,vec![crate::food::StatBoost{skill:18,amount:5},crate::food::StatBoost{skill:4,amount:4}]);let p=w.players.get_mut(actor.slot as usize).unwrap();for skill in [3,4,18]{p.data.stats.set_base(skill,99);p.data.stats.set_current(skill,99);}p.data.inventory.restore_slot(2,Some(crate::inventory::Item{id:991,count:1,vars:0})).unwrap();w.member_foods.insert(991);w.submit(actor,Action::EatItemSlot{slot:2,item:991}).unwrap();w.tick().unwrap();assert_eq!(w.data(actor).unwrap().inventory.get(2).unwrap().id,991);assert_eq!(w.data(actor).unwrap().stats.current(18),99);w.players.get_mut(actor.slot as usize).unwrap().member=true;w.submit(actor,Action::EatItemSlot{slot:2,item:991}).unwrap();w.tick().unwrap();let p=w.data(actor).unwrap();assert_eq!((p.stats.current(3),p.stats.current(18),p.stats.current(4)),(99,104,103));assert_eq!(p.inventory.get(2).unwrap().id,992);assert_eq!(p.stats.fine_xp(18),0);
    }
    #[test]
    fn queued_ordinary_food_uses_selected_duplicate_and_can_eat_after_brew_overheal() {
        let (mut w,actor)=admitted();let food=crate::food::Edible{id:991,heal:20,delay:3,replacement:None,message:"Shark".into()};w.foods.insert(food.id,food);
        let p=w.players.get_mut(actor.slot as usize).unwrap();p.data.stats.set_base(3,99);p.data.stats.set_current(3,115);for slot in [0,2]{p.data.inventory.restore_slot(slot,Some(crate::inventory::Item{id:991,count:1,vars:0})).unwrap();}
        w.submit(actor,Action::EatItemSlot{slot:2,item:991}).unwrap();w.tick().unwrap();let p=w.players.get(actor.slot as usize).unwrap();assert_eq!(p.data.stats.current(3),115);assert!(p.data.inventory.get(2).is_none());assert_eq!(p.data.inventory.get(0).unwrap().id,991);assert!(p.protection.delayed());
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
        // Ordinary food uses one item per slot; malformed nonstackable stacks
        // are rejected by the gameplay adapter rather than defining queue timing.
        for slot in [2,3,5] {data.inventory.restore_slot(slot,Some(crate::inventory::Item{id:7,count:1,vars:0})).unwrap();}
        world.submit(actor, Action::EatSlot { slot: 2 }).unwrap();
        world.submit(actor, Action::EatSlot { slot: 2 }).unwrap();
        world
            .submit(actor, Action::MoveInventorySlot { from: 2, into: 4 })
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 5);
        assert!(world.data(actor).unwrap().inventory.get(2).is_none());
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
        world.submit(actor, Action::EatSlot { slot: 3 }).unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 8);
        assert!(world.data(actor).unwrap().inventory.get(3).is_none());
        assert_eq!(world.data(actor).unwrap().inventory.get(5).unwrap().count,1);
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
    fn accepted_movement_cancels_weak_work_but_retains_normal_queues() {
        let (mut world, actor) = admitted();
        let p = world.players.get_mut(actor.slot as usize).unwrap();
        p.queues
            .add(900, crate::queues::Category::Weak, 5, Deferred::Preserve)
            .unwrap();
        p.queues
            .add(901, crate::queues::Category::Normal, 5, Deferred::Preserve)
            .unwrap();
        p.protection.suspended = true;
        p.food_delay = true;
        let delay_until = p.protection.delay_until;
        assert!(world.movement_input_allowed(actor).unwrap());
        world
            .action(
                actor,
                Action::MoveClick {
                    destination: CoordGrid::new(3202, 3200, 0).unwrap(),
                    control: false,
                },
                &mut SeededRandom(1),
            )
            .unwrap();
        let p = world.player(actor).unwrap();
        assert!(!p.queues.contains(900));
        assert!(p.queues.contains(901));
        assert!(!p.protection.suspended);
        assert!(!p.food_delay);
        assert_eq!(p.protection.delay_until, delay_until);
        assert!(world.drain_events().iter().any(|event| matches!(event,
            WorldEvent::ResetFaceEntity { actor: id } if *id == actor)));
    }
    #[test]
    fn delayed_or_rooted_movement_preserves_weak_work_and_suspension() {
        for rooted in [false, true] {
            let (mut world, actor) = admitted();
            let p = world.players.get_mut(actor.slot as usize).unwrap();
            p.queues
                .add(900, crate::queues::Category::Weak, 5, Deferred::Preserve)
                .unwrap();
            p.protection.suspended = true;
            p.food_delay = true;
            p.movement.rooted = rooted;
            if !rooted {
                p.protection.delay_until = 100;
            }
            assert!(!world.movement_input_allowed(actor).unwrap());
            assert_eq!(
                world.action(
                    actor,
                    Action::MoveClick {
                        destination: CoordGrid::new(3202, 3200, 0).unwrap(),
                        control: false,
                    },
                    &mut SeededRandom(1)
                ),
                Err(if rooted {
                    ActionError::Rooted
                } else {
                    ActionError::Delayed
                })
            );
            let p = world.player(actor).unwrap();
            assert!(p.queues.contains(900));
            assert!(p.protection.suspended);
            assert!(p.food_delay);
        }
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
    fn tutorial_hit_queue_closes_modal_waits_for_delay_and_cancels_pending_death() {
        let (mut world, actor) = admitted();
        world.player_hit_queues = Some(crate::hits::PlayerQueues {
            standard: 800,
            death: 801,
        });
        world.tutorial_rules_completed_varp = Some(900);
        world.data_mut(actor).unwrap().stats.set_current(3, 10);
        world.set_modal_open(actor, true).unwrap();
        world.delay_player(actor, 3).unwrap();
        world
            .queue_hit(
                crate::hits::Target::Player(actor),
                crate::hits::Hit {
                    display: None,
                    source: None,
                    style: crate::hits::Style::Typeless,
                    damage: 20,
                },
                1,
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 10);
        assert!(!world.players.get(actor.slot as usize).unwrap().has_modal);
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::CloseModals { .. })));
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 10);
        world.tick().unwrap();
        // Late advances the clock after PlayerMain: the third pass still has
        // processed_clock==3; a deadline of4 expires on the following pass.
        assert_eq!(world.data(actor).unwrap().stats.current(3), 10);
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 1);
        let events = world.drain_events();
        assert!(events.iter().any(|e| matches!(e, WorldEvent::LocationMessage { message, .. } if *message == crate::tutorial_rules::FLOOR_MESSAGE)));
        assert!(!events
            .iter()
            .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
        assert!(!world
            .players
            .get(actor.slot as usize)
            .unwrap()
            .queues
            .contains(801));
        world.tick().unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
    }
    #[test]
    fn completed_tutorial_hit_retains_normal_death_queue_delay() {
        let (mut world, actor) = admitted();
        world.player_hit_queues = Some(crate::hits::PlayerQueues {
            standard: 800,
            death: 801,
        });
        world.tutorial_rules_completed_varp = Some(900);
        let data = world.data_mut(actor).unwrap();
        data.variables = crate::variables::Variables::restore([(900, 1)].into());
        data.stats.set_current(3, 10);
        world
            .queue_hit(
                crate::hits::Target::Player(actor),
                crate::hits::Hit {
                    display: None,
                    source: None,
                    style: crate::hits::Style::Typeless,
                    damage: 20,
                },
                1,
            )
            .unwrap();
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.current(3), 0);
        assert!(world
            .players
            .get(actor.slot as usize)
            .unwrap()
            .queues
            .contains(801));
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::DeathRequested(_))));
        world.tick().unwrap();
        assert_eq!(
            world
                .drain_events()
                .iter()
                .filter(|e| matches!(e, WorldEvent::DeathRequested(_)))
                .count(),
            1
        );
    }
    #[test]
    fn tutorial_stat_delivery_waits_for_modal_and_reads_current_completion() {
        for completed in [0, 1] {
            let (mut world, actor) = admitted();
            world.tutorial_rules_completed_varp = Some(900);
            let p = world.players.get_mut(actor.slot as usize).unwrap();
            p.data.stats.set_current(0, 20);
            let change = crate::skills::advance(&mut p.data.stats, 0, 500.0, 1.0, 99).unwrap();
            p.stat_events
                .advance(&crate::tutorial_rules::bindings(), 0, change)
                .unwrap();
            p.has_modal = true;
            world.tick().unwrap();
            assert_eq!(world.data(actor).unwrap().stats.current(0), 20);
            world.data_mut(actor).unwrap().variables =
                crate::variables::Variables::restore([(900, completed)].into());
            world.set_modal_open(actor, false).unwrap();
            world.tick().unwrap();
            let stats = &world.data(actor).unwrap().stats;
            assert_eq!(stats.current(0), if completed == 1 { 20 } else { 3 });
            assert_eq!(stats.xp(0), if completed == 1 { 500 } else { 174 });
        }
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
