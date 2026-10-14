//! Resolved inputs for the original PvN ranged accuracy and maximum-hit operations.
use crate::{
    bank_actions::Binding, bonuses, melee, npc::Npc, npc_combat, ranged_equipment,
    ranged_modifiers::Attributes, ranged_roll::Inputs, simulation::PlayerData,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct NpcDefence {
    /// Light, standard, heavy (global attack type ids 5, 6, 7).
    pub bonuses: [i32; 3],
    pub magic_attack: i32,
}
pub struct Config {
    pub equipment: ranged_equipment::Catalog,
    pub npcs: BTreeMap<u16, NpcDefence>,
    /// Sharp eye, hawk eye, eagle eye, rigour, deadeye unlocked.
    pub prayers: [Binding; 5],
    pub stat: u8,
}
/// Original when-expression priority, including malformed simultaneous prayer flags.
pub fn prayer_multipliers([sharp, hawk, eagle, rigour, deadeye]: [i32; 5]) -> (f64, f64) {
    if sharp == 1 {
        (1.05, 1.05)
    } else if hawk == 1 {
        (1.1, 1.1)
    } else if eagle == 1 {
        let value = if deadeye == 1 { 1.18 } else { 1.15 };
        (value, value)
    } else if rigour == 1 {
        (1.2, 1.23)
    } else {
        (1.0, 1.0)
    }
}
impl Config {
    pub fn inputs(
        &self,
        common: &melee::Config,
        data: &PlayerData,
        target: &Npc,
        definition: &npc_combat::Definition,
        kind: Option<u8>,
        style: Option<u8>,
        speed: i32,
        skull: i32,
        slayer_task: bool,
        source: crate::CoordGrid,
    ) -> Result<Inputs, String> {
        let defence = self
            .npcs
            .get(&target.type_id)
            .ok_or("Missing ranged NPC metadata")?;
        let mut flags = [0; 5];
        for (value, binding) in flags.iter_mut().zip(self.prayers) {
            *value = binding.get(&data.variables).map_err(|e| format!("{e:?}"))?;
        }
        let (accuracy_prayer, damage_prayer) = prayer_multipliers(flags);
        let (accuracy_void, damage_void) = self.equipment.void_multipliers(&data.equipment);
        let bonuses = bonuses::calculate(
            &data.equipment,
            &common.bonuses,
            data.variables.get(common.stance.id) == 0,
        );
        let index = kind.filter(|k| (5..=7).contains(k)).map(|k| k as usize - 5);
        Ok(Inputs {
            level: data.stats.current(self.stat) as u8 as i32,
            defence_level: target.stats[1],
            attack_bonus: bonuses.values[4],
            strength_bonus: bonuses.values[11],
            defence_bonus: index.map_or(0, |i| defence.bonuses[i]),
            style,
            accuracy_prayer,
            damage_prayer,
            accuracy_void,
            damage_void,
            invocation: common
                .invocation
                .get(&data.variables)
                .map_err(|e| format!("{e:?}"))?,
            speed,
            magic: target.stats[5].max(defence.magic_attack),
            // Fuse modifiers use origins; projectile timing separately uses bounds.
            distance: (source.x() - target.movement.coords.x())
                .abs()
                .max((source.z() - target.movement.coords.z()).abs()),
            accuracy_multiplier: 1.0,
            damage_multiplier: 1.0,
            bolt_damage: 0,
            attributes: Attributes {
                ranged: self.equipment.collect(&data.equipment, kind, style, skull),
                npc: definition.attributes(target.stats[3], target.base_stats[3], slayer_task),
            },
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prayer_priority_and_exact_enabled_value_follow_original() {
        assert_eq!(prayer_multipliers([1, 1, 1, 1, 1]), (1.05, 1.05));
        assert_eq!(prayer_multipliers([2, 1, 1, 1, 1]), (1.1, 1.1));
        assert_eq!(prayer_multipliers([0, 0, 1, 1, 0]), (1.15, 1.15));
        assert_eq!(prayer_multipliers([0, 0, 1, 1, 1]), (1.18, 1.18));
        assert_eq!(prayer_multipliers([0, 0, 1, 0, 2]), (1.15, 1.15));
        assert_eq!(prayer_multipliers([0, 0, 0, 1, 1]), (1.2, 1.23));
        assert_eq!(prayer_multipliers([0, 2, -1, 0, 1]), (1.0, 1.0));
    }
}
