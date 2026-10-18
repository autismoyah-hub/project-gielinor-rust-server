//! Pure world primitives ported from the pinned Kotlin reference.
//! No network, persistence, clock, or global random state is used here.
pub mod areas;
pub mod attack_profile;
pub mod autocast;
pub mod autocast_attack;
pub mod autocast_select;
pub mod deposit_box;
pub mod deposit_box_items;
pub mod bank_access;
pub mod bank;
pub mod bank_actions;
pub mod bank_sidebar;
pub mod bonuses;
pub mod collision;
pub mod combat;
pub mod combat_stance;
pub mod combat_wear;
pub mod combat_xp;
pub mod doors;
pub mod drop_actions;
pub mod elemental_spell;
pub mod engine_queues;
pub mod entities;
pub mod equipment;
pub mod facing;
pub mod food;
pub mod gameframe;
pub mod ground;
pub mod ground_transfer;
pub mod held_use;
pub mod herb_cleaning;
pub mod hit_display;
pub mod hits;
pub mod inventory;
pub mod loc_collision;
mod loc_map;
pub mod loc_reach;
pub mod loc_timers;
pub mod locations;
pub mod magic;
pub mod melee;
pub mod melee_equipment;
pub mod melee_modifiers;
pub mod melee_roll;
pub mod movement;
pub mod npc;
pub mod npc_combat;
pub mod npc_interaction;
pub mod npc_melee;
pub mod npc_melee_roll;
pub mod npc_options;
pub mod potions;
pub mod projectile;
pub mod protected;
pub mod queues;
pub mod ranged;
pub mod ranged_ammo;
pub mod ranged_equipment;
pub mod ranged_modifiers;
pub mod ranged_roll;
pub mod ranged_weapon;
pub mod raycast;
pub mod reach;
pub mod routing;
pub mod rune_consumption;
pub mod runes;
pub mod simulation;
pub mod skills;
pub mod smelting;
pub mod spell_cast;
pub mod spell_combat;
pub mod spell_equipment;
pub mod spell_modifiers;
pub mod spell_roll;
pub mod stat_events;
pub mod stat_ops;
pub mod stats;
pub mod tick;
pub mod tower_arrival;
pub mod ui;
pub mod unfinished_potions;
pub mod finished_potions;
pub mod variables;
pub mod zones;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordError {
    XOutOfRange,
    ZOutOfRange,
    LevelOutOfRange,
}

/// Kotlin CoordGrid's signed packed value, including the distinct NULL sentinel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CoordGrid(i32);

impl CoordGrid {
    pub const ZERO: Self = Self(0);
    pub const NULL: Self = Self(-1);

    /// Raw packing preserves all 32 bits, as the Kotlin primary constructor does.
    pub const fn from_packed(packed: i32) -> Self {
        Self(packed)
    }
    pub const fn packed(self) -> i32 {
        self.0
    }
    pub fn new(x: i32, z: i32, level: i32) -> Result<Self, CoordError> {
        if !(0..=16383).contains(&x) {
            return Err(CoordError::XOutOfRange);
        }
        if !(0..=16383).contains(&z) {
            return Err(CoordError::ZOutOfRange);
        }
        if !(0..=3).contains(&level) {
            return Err(CoordError::LevelOutOfRange);
        }
        Ok(Self((level << 28) | (x << 14) | z))
    }
    pub const fn x(self) -> i32 {
        (self.0 >> 14) & 16383
    }
    pub const fn z(self) -> i32 {
        self.0 & 16383
    }
    pub const fn level(self) -> i32 {
        (self.0 >> 28) & 3
    }
    pub const fn mx(self) -> i32 {
        self.x() / 64
    }
    pub const fn mz(self) -> i32 {
        self.z() / 64
    }
    pub const fn lx(self) -> i32 {
        self.x() % 64
    }
    pub const fn lz(self) -> i32 {
        self.z() % 64
    }
    pub fn translate(self, dx: i32, dz: i32, dl: i32) -> Result<Self, CoordError> {
        // JVM Int addition wraps before Kotlin's constructor validates the result.
        Self::new(
            self.x().wrapping_add(dx),
            self.z().wrapping_add(dz),
            self.level().wrapping_add(dl),
        )
    }
    pub fn chebyshev_distance(self, other: Self) -> i32 {
        // Reference intentionally does not include level in this distance.
        (self.x() - other.x())
            .abs()
            .max((self.z() - other.z()).abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn null_is_distinct_from_largest_valid_coordinate() {
        let max = CoordGrid::new(16383, 16383, 3).unwrap();
        assert_ne!(max, CoordGrid::NULL);
        assert_eq!(CoordGrid::NULL.x(), max.x());
        assert_eq!(CoordGrid::NULL.z(), max.z());
        assert_eq!(CoordGrid::NULL.level(), max.level());
    }
    #[test]
    fn translation_rejects_world_edges() {
        assert_eq!(
            CoordGrid::ZERO.translate(-1, 0, 0),
            Err(CoordError::XOutOfRange)
        );
        assert_eq!(
            CoordGrid::new(16383, 0, 0).unwrap().translate(1, 0, 0),
            Err(CoordError::XOutOfRange)
        );
        assert_eq!(
            CoordGrid::new(0, 16383, 0).unwrap().translate(0, 1, 0),
            Err(CoordError::ZOutOfRange)
        );
        assert_eq!(
            CoordGrid::new(0, 0, 3).unwrap().translate(0, 0, 1),
            Err(CoordError::LevelOutOfRange)
        );
    }
    #[test]
    fn distance_ignores_height_like_kotlin() {
        assert_eq!(
            CoordGrid::new(3200, 3200, 0)
                .unwrap()
                .chebyshev_distance(CoordGrid::new(3205, 3197, 3).unwrap()),
            5
        );
    }
}
pub mod area_tracking;
pub mod bulwark_wear;
pub mod combat_state;
pub mod dialogue;
pub mod dialogue_text;
pub mod disconnect;
pub mod floor_progression;
pub mod hero_points;
pub mod location_content;
pub mod login_resets;
pub mod music;
pub mod music_player;
pub mod music_playlist;
pub mod npc_bind;
pub mod npc_death;
pub mod npc_route;
pub mod player_regen;
pub mod player_timers;
pub mod prayer;
pub mod prayer_book;
pub mod prayer_drain;
pub mod prayer_runtime;
pub mod region_rotation;
pub mod session_vars;
pub mod smithing_make;
pub mod tower_diary;
pub mod tower_diary_ui;
pub mod tower_furnishings;
pub mod tower_hazards;
pub mod tower_layout;
pub mod tanning;
pub mod tower_tanning;
pub mod tutorial_bread;
pub mod tutorial_chef;
pub mod tutorial_dough;
pub mod tutorial_fire;
pub mod tutorial_fishing;
pub mod tutorial_focus;
pub mod tutorial_gates;
pub mod tutorial_mining;
pub mod tutorial_panels;
pub mod tutorial_progression;
pub mod tutorial_quest;
pub mod tutorial_rules;
pub mod tutorial_shrimp;
pub mod tutorial_steps;
pub mod tutorial_survival;
mod tutorial_survival_text;
pub mod tutorial_woodcutting;
pub mod worn_item_swap;
pub mod wearpos;

pub mod anvil;

pub mod anvil_products;

pub mod tutorial_mining_tutor;

pub mod tutorial_combat_tutor;

pub mod tutorial_combat;

pub mod tutorial_bank;
pub mod tutorial_lecture_tutor;
pub mod tutorial_magic_tutor;
pub mod tutorial_magic_ui;

pub mod tutorial_leave;

pub mod tower_chest;

pub mod tower_offering;

pub mod tower_staff;

pub mod tower_kit;

pub mod tower_restore;

pub mod tower_food;

pub mod tower_combat;
pub mod tower_tastes;
pub mod woodcutting;

pub mod counted_assembly;
pub mod assembly;
pub mod assembly_production;
pub mod held_item_conversion;
pub mod container_emptying;
pub mod facility_crafting;
pub mod gathering;
pub mod gathering_outfits;
pub mod item_packs;
pub mod mining;
pub mod shop;

pub mod milking;
pub mod water_sources;
pub mod cooking;
pub mod grinding;
pub mod dough;
pub mod swamp_tar;
pub mod chemistry;
pub mod chemistry_modifier;
pub mod potion_upgrades;
pub mod combat_mixing;
pub mod thieving;
pub mod tool_crafting;
pub mod tool_crafting_counts;
pub mod tool_crafting_materials;
