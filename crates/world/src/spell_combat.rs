//! Resolved spell formula inputs and stage-specific equipment attributes.
use crate::{
    bank_actions::Binding,
    bonuses, melee,
    npc::{Npc, RandomSource},
    npc_combat,
    simulation::PlayerData,
    spell_equipment,
    spell_modifiers::Attributes,
    spell_roll,
};
use std::collections::BTreeMap;
#[derive(Clone, Copy)]
pub struct NpcDefence {
    pub bonus: i32,
    pub uses_defence: bool,
    pub weakness_percent: i32,
}
pub struct Config {
    pub equipment: spell_equipment::Catalog,
    pub npcs: BTreeMap<u16, NpcDefence>,
    pub prayers: [Binding; 5],
    pub mark: Binding,
    pub charge: Binding,
    pub stat: u8,
}
impl Config {
    pub fn inputs(
        &self,
        common: &melee::Config,
        data: &PlayerData,
        target: &Npc,
        speed: i32,
    ) -> Result<spell_roll::Inputs, String> {
        let defence = self
            .npcs
            .get(&target.type_id)
            .ok_or("Missing spell NPC metadata")?;
        let mut flags = [0; 5];
        for (flag, binding) in flags.iter_mut().zip(self.prayers) {
            *flag = binding.get(&data.variables).map_err(|e| format!("{e:?}"))?;
        }
        let (accuracy_prayer, damage_prayer) = spell_roll::prayer_bonuses(flags);
        let bonus = bonuses::calculate(
            &data.equipment,
            &common.bonuses,
            data.variables.get(common.stance.id) == 0,
        );
        Ok(spell_roll::Inputs {
            magic: data.stats.current(self.stat) as u8 as i32,
            attack_bonus: bonus.values[3],
            defence_level: target.stats[if defence.uses_defence { 1 } else { 5 }],
            defence_bonus: defence.bonus,
            accuracy_prayer,
            accuracy_void: self.equipment.accuracy_void(&data.equipment),
            damage_bonus: bonus.values[12],
            damage_prayer,
            invocation: common
                .invocation
                .get(&data.variables)
                .map_err(|e| format!("{e:?}"))?,
            weakness_percent: defence.weakness_percent,
            speed,
        })
    }
    pub fn attributes(
        &self,
        data: &PlayerData,
        target: &Npc,
        definition: &npc_combat::Definition,
        spell: i32,
        book: Option<i32>,
        sunfire: bool,
        skull: i32,
        slayer: bool,
        random: &mut impl RandomSource,
    ) -> Result<Attributes, String> {
        let context = spell_equipment::Context {
            spell,
            standard_book: book == Some(0),
            sunfire,
            skull,
            mark_of_darkness: self
                .mark
                .get(&data.variables)
                .map_err(|e| format!("{e:?}"))?,
            charge_cooldown: self
                .charge
                .get(&data.variables)
                .map_err(|e| format!("{e:?}"))?,
        };
        Ok(Attributes {
            spell: self.equipment.collect(&data.equipment, context, random),
            npc: definition.attributes(target.stats[3], target.base_stats[3], slayer),
        })
    }
}
