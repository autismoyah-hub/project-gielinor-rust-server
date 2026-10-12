//! Synchronous hand-slot listeners, before the next actor action is dispatched.
use crate::{autocast, bank_actions::Binding, combat_stance, magic, simulation::PlayerData};
pub struct Config {
    pub autocast: autocast::Config,
    pub runes: magic::Config,
    pub level: Binding,
    pub decimal: Binding,
    pub combat_stats: [u8; 7],
}
impl Config {
    pub fn changed(
        &self,
        data: &mut PlayerData,
        stance: &combat_stance::Config,
        slot: usize,
        decimal: i32,
        logged: bool,
    ) -> Result<Vec<autocast::Event>, String> {
        if slot != 3 && slot != 5 {
            return Ok(vec![]);
        }
        let weapon = data.equipment.get(3).map(|i| i.id);
        let mut vars = data.variables.clone();
        let mut events: Vec<_> = stance
            .load_saved(&mut vars, weapon, logged)?
            .into_iter()
            .map(autocast::Event::Variable)
            .collect();
        let rune_vars = vars.clone();
        events.extend(self.autocast.restore(&mut vars, weapon, logged, |spell| {
            self.runes
                .missing_message(&rune_vars, &data.inventory, &data.equipment, spell)
        })?);
        events.extend(
            stance
                .validate_worn(&mut vars, weapon, logged)?
                .into_iter()
                .map(autocast::Event::Variable),
        );
        let [a, d, s, h, r, m, p] = self.combat_stats.map(|id| data.stats.base(id) as u8 as i32);
        let whole = crate::skills::combat_level(a, d, s, h, r, m, p);
        for (binding, value) in [(self.level, whole), (self.decimal, decimal)] {
            events.extend(
                vars.set_bit(binding.base, binding.bit, value, logged)
                    .map_err(|e| format!("{e:?}"))?
                    .map(autocast::Event::Variable),
            );
        }
        data.variables = vars;
        Ok(events)
    }
}
