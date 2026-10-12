//! WornBonuses aggregation. Display-only magic modifiers are separate from combat damage.
use crate::inventory::Inventory;
use std::collections::BTreeMap;

pub const PARAMS: [&str; 17] = [
    "attack_stab",
    "attack_slash",
    "attack_crush",
    "attack_magic",
    "attack_ranged",
    "defence_stab",
    "defence_slash",
    "defence_crush",
    "defence_ranged",
    "defence_magic",
    "melee_strength",
    "ranged_strength",
    "magic_damage",
    "item_prayer_bonus",
    "bonus_undead_buff",
    "bonus_slayer_buff",
    "additive_ranged_strength",
];
#[derive(Clone, Debug, Default)]
pub struct ItemBonuses {
    pub values: [i32; 17],
    pub undead_melee_only: bool,
    pub slayer_melee_only: bool,
    pub ignore_quiver: bool,
    pub bulwark: bool,
    pub shadow: bool,
    pub virtus_slots: u16,
    pub elite_void_slots: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bonuses {
    pub values: [i32; 16],
    pub magic_additive: i32,
    pub magic_multiplier: i32,
    pub undead_melee_only: bool,
    pub slayer_melee_only: bool,
}
impl Default for Bonuses {
    fn default() -> Self {
        Self {
            values: [0; 16],
            magic_additive: 0,
            magic_multiplier: 10,
            undead_melee_only: false,
            slayer_melee_only: false,
        }
    }
}
impl Bonuses {
    pub fn displayed_magic_damage(&self) -> i32 {
        self.values[12].wrapping_mul(self.magic_multiplier / 10)
    }
}
pub fn calculate(worn: &Inventory, types: &BTreeMap<i32, ItemBonuses>, pummel: bool) -> Bonuses {
    let item_type = |slot| worn.get(slot).and_then(|item| types.get(&item.id));
    let weapon = item_type(3);
    let ignore_quiver = weapon.is_some_and(|ty| ty.ignore_quiver);
    let mut result = Bonuses::default();
    for slot in 0..14 {
        if slot == 13 && ignore_quiver {
            continue;
        }
        let Some(ty) = item_type(slot) else {
            continue;
        };
        for (value, extra) in result.values.iter_mut().zip(ty.values) {
            *value = value.wrapping_add(extra);
        }
        result.values[11] = result.values[11].wrapping_add(ty.values[16]);
        // Reference overwrites these flags for each worn item, including false.
        result.undead_melee_only = ty.undead_melee_only;
        result.slayer_melee_only = ty.slayer_melee_only;
    }
    if weapon.is_some_and(|ty| ty.shadow) {
        result.magic_multiplier = 30;
        result.values[3] = (result.values[3] as f64 * 3.0) as i32;
    }
    if pummel && weapon.is_some_and(|ty| ty.bulwark) {
        let defence = [5, 6, 7, 8]
            .into_iter()
            .fold(0i32, |sum, i| sum.wrapping_add(result.values[i]));
        let extra = (defence.wrapping_sub(800) / 12).wrapping_sub(38).max(0);
        result.values[10] = result.values[10].wrapping_add(extra);
    }
    for slot in [0, 4, 7] {
        if item_type(slot).is_some_and(|ty| ty.virtus_slots & (1 << slot) != 0) {
            result.magic_additive += 3;
        }
    }
    if [0, 4, 7, 9]
        .into_iter()
        .all(|slot| item_type(slot).is_some_and(|ty| ty.elite_void_slots & (1 << slot) != 0))
    {
        result.values[12] = result.values[12].wrapping_add(50);
    }
    result
}

pub fn attack_speed(base: i32, rapid_ranged: bool) -> i32 {
    if rapid_ranged {
        base.wrapping_sub(1).max(1)
    } else {
        base
    }
}
/// Packed styles reserve zero and nibble values 10..15 for absent styles.
pub fn attack_style(packed: i32, stance: i32) -> Result<Option<u8>, &'static str> {
    if !(0..=3).contains(&stance) {
        return Err("Combat stance must be within range [0..3]");
    }
    let style = ((packed >> (stance * 4)) & 15) as u8;
    Ok((1..=9).contains(&style).then_some(style))
}
