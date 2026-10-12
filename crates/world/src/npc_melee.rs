//! Live NPC melee definitions and the defending player's equipment/prayer state.
use crate::{
    bank_actions::Binding, bonuses, inventory::Inventory, melee, npc::Npc, npc_melee_roll::Inputs,
    simulation::PlayerData, variables::Varp,
};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy)]
pub struct Definition {
    pub retreat_range: i32,
    pub give_chase: bool,
    pub attack_type: u8,
    pub attack_bonus: i32,
    pub strength_bonus: i32,
    pub animation: u16,
    pub sound: Option<u16>,
}
pub struct Config {
    pub player_mark: crate::hit_display::Builder,
    pub definitions: BTreeMap<u16, Definition>,
    pub prayers: [Binding; 11],
    pub torag: [BTreeSet<i32>; 4],
    pub damned: i32,
    pub elysian: i32,
    pub justiciar: [i32; 3],
    pub bulwarks: BTreeSet<i32>,
    pub passive_delay: Varp,
    pub defence_stat: u8,
    pub health_stat: u8,
    pub unarmed_defend: u16,
    pub item_defend: BTreeMap<i32, u16>,
}
fn is(worn: &Inventory, slot: usize, id: i32) -> bool {
    worn.get(slot).is_some_and(|item| item.id == id)
}
impl Config {
    pub fn inputs(
        &self,
        npc: &Npc,
        target: &PlayerData,
        melee: &melee::Config,
        style: Option<u8>,
        clock: i32,
    ) -> Result<Inputs, String> {
        let definition = self
            .definitions
            .get(&npc.type_id)
            .ok_or("Missing NPC melee definition")?;
        let mut prayer = [false; 11];
        for (active, binding) in prayer.iter_mut().zip(self.prayers) {
            *active = binding
                .get(&target.variables)
                .map_err(|e| format!("{e:?}"))?
                == 1;
        }
        let prayer = if prayer[0] || prayer[1] || prayer[2] {
            1.25
        } else if prayer[3] {
            1.2
        } else if prayer[4] {
            1.15
        } else if prayer[5] {
            1.1
        } else if prayer[6] || (prayer[7] && prayer[8]) || (prayer[9] && prayer[10]) {
            1.05
        } else {
            1.0
        };
        let torag = is(&target.equipment, 2, self.damned)
            && [0, 4, 7, 3]
                .into_iter()
                .zip(&self.torag)
                .all(|(slot, ids)| {
                    target
                        .equipment
                        .get(slot)
                        .is_some_and(|i| ids.contains(&i.id))
                });
        let armour = if torag {
            let hp = target.stats.current(self.health_stat) as u8 as i32;
            let max = target.stats.base(self.health_stat) as u8 as i32;
            1.0 + (max - hp) as f64 / max as f64
        } else {
            1.0
        };
        let bonuses = bonuses::calculate(
            &target.equipment,
            &melee.bonuses,
            target.variables.get(melee.stance.id) == 0,
        );
        let defence_bonus = bonuses.values[4 + definition.attack_type as usize];
        Ok(Inputs {
            attack_level: npc.stats[0],
            strength_level: npc.stats[2],
            attack_bonus: definition.attack_bonus,
            strength_bonus: definition.strength_bonus,
            defence_level: target.stats.current(self.defence_stat) as u8 as i32,
            defence_bonus,
            defence_style: style,
            defence_prayer: prayer,
            defence_armour: armour,
            elysian: is(&target.equipment, 5, self.elysian),
            bulwark: target
                .equipment
                .get(3)
                .is_some_and(|i| self.bulwarks.contains(&i.id))
                && style == Some(3)
                && clock >= target.variables.get(self.passive_delay.id),
            justiciar: [0, 4, 7]
                .into_iter()
                .zip(self.justiciar)
                .all(|(slot, id)| is(&target.equipment, slot, id)),
        })
    }
    pub fn defend_animation(&self, data: &PlayerData) -> u16 {
        let animation = |slot| {
            data.equipment
                .get(slot)
                .and_then(|i| self.item_defend.get(&i.id))
                .copied()
        };
        animation(5)
            .filter(|id| *id != self.unarmed_defend)
            .or_else(|| animation(3))
            .unwrap_or(self.unarmed_defend)
    }
}
