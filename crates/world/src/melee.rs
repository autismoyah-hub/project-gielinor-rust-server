use crate::{
    bank_actions::Binding,
    bonuses::{self, ItemBonuses},
    melee_equipment,
    melee_modifiers::Attributes,
    melee_roll::Inputs,
    npc::Npc,
    npc_combat,
    simulation::PlayerData,
    variables::Varp,
};
use std::collections::BTreeMap;
pub struct Config {
    pub npc_marks: BTreeMap<u16, crate::hit_display::Builder>,
    pub npc_defend: BTreeMap<u16, u16>,
    pub npc_defend_sounds: BTreeMap<u16, u16>,
    pub hit_threshold: Binding,
    pub equipment: melee_equipment::Catalog,
    pub bonuses: BTreeMap<i32, ItemBonuses>,
    pub stance: Varp,
    pub attack_prayers: [Binding; 5],
    pub strength_prayers: [Binding; 5],
    pub souls: Varp,
    pub invocation: Binding,
    pub max_hit: Varp,
    pub stats: [u8; 3], // attack, strength, hitpoints
    pub unarmed_fx: [(u16, u16); 4],
    pub weapon_fx: BTreeMap<i32, [(u16, u16); 4]>,
}
impl Config {
    pub fn inputs(
        &self,
        data: &PlayerData,
        target: &Npc,
        definition: &npc_combat::Definition,
        kind: Option<u8>,
        style: Option<u8>,
        speed: i32,
        skull: i32,
    ) -> Result<Inputs, String> {
        let [attack, strength, hp] = self.stats;
        let prayer = |bindings: [Binding; 5], values: [f64; 5]| -> Result<f64, String> {
            for (binding, value) in bindings.into_iter().zip(values) {
                if binding.get(&data.variables).map_err(|e| format!("{e:?}"))? == 1 {
                    return Ok(value);
                }
            }
            Ok(1.0)
        };
        let bonuses = bonuses::calculate(
            &data.equipment,
            &self.bonuses,
            data.variables.get(self.stance.id) == 0,
        );
        let index = kind.filter(|k| (1..=3).contains(k)).map(|k| k as usize - 1);
        Ok(Inputs {
            attack_level: data.stats.current(attack) as u8 as i32,
            strength_level: data.stats.current(strength) as u8 as i32,
            defence_level: target.stats[1],
            attack_bonus: index.map_or(0, |i| bonuses.values[i]),
            strength_bonus: bonuses.values[10],
            defence_bonus: index.map_or(0, |i| definition.defence[i]),
            style,
            attack_prayer: prayer(self.attack_prayers, [1.05, 1.1, 1.15, 1.15, 1.2])?,
            strength_prayer: prayer(self.strength_prayers, [1.05, 1.1, 1.15, 1.18, 1.23])?,
            void: self.equipment.void_multiplier(&data.equipment),
            soul: self
                .equipment
                .soul_multiplier(&data.equipment, data.variables.get(self.souls.id)),
            invocation: self
                .invocation
                .get(&data.variables)
                .map_err(|e| format!("{e:?}"))?,
            speed,
            health: data.stats.current(hp) as u8 as i32,
            max_health: data.stats.base(hp) as u8 as i32,
            attributes: Attributes {
                melee: self.equipment.collect(&data.equipment, kind, skull),
                npc: definition.attributes(target.stats[3], target.base_stats[3], false),
            },
        })
    }
    pub fn effects(&self, weapon: Option<i32>, stance: u8) -> (u16, u16) {
        let effects = weapon
            .and_then(|id| self.weapon_fx.get(&id))
            .unwrap_or(&self.unarmed_fx);
        effects.get(stance as usize).copied().unwrap_or(effects[0])
    }
}
