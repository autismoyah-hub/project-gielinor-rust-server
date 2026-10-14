//! WearposScript listeners. Preserve redundant writes and their observable order.
use crate::{
    bank_actions::Binding,
    combat_stance,
    simulation::PlayerData,
    variables::{Transmit, Update, Varp},
};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub two_handed: bool,
    pub category: u8,
    pub sound: Option<u16>,
    pub player_op: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory,
        variables::{Varbit, Variables},
        CoordGrid,
    };
    fn setup() -> (Config, PlayerData) {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let binding = |id| Binding {
            base: varp(id),
            bit: Varbit {
                base: id,
                lsb: 0,
                msb: 7,
            },
        };
        let config = Config {
            special: varp(301),
            stance: varp(43),
            category: binding(843),
            level: binding(8),
            decimal: binding(9),
            combat_stats: [0, 1, 2, 3, 4, 6, 5],
            items: BTreeMap::from([
                (
                    1277,
                    Item {
                        category: 17,
                        sound: Some(2238),
                        ..Default::default()
                    },
                ),
                (
                    2,
                    Item {
                        category: 10,
                        two_handed: true,
                        sound: Some(20),
                        player_op: Some("Pelt".into()),
                    },
                ),
            ]),
        };
        let mut data = PlayerData::new(
            1,
            "test".into(),
            CoordGrid::new(3224, 3218, 0).unwrap(),
            Default::default(),
        );
        data.stats.set_base(3, 10);
        (config, data)
    }
    #[test]
    fn sword_write_preserves_explicit_zero_and_updates_category_on_remove() {
        let (config, mut data) = setup();
        assert!(!data.variables.contains(301));
        data.equipment
            .restore_slot(
                3,
                Some(inventory::Item {
                    id: 1277,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let equip = config.changed(&mut data, 3, 1277, 0, true).unwrap();
        assert!(data.variables.contains(301));
        assert_eq!(data.variables.get(301), 0);
        assert_eq!(data.variables.get(843), 17);
        assert_eq!(&equip[..2], &[Event::Sound(2238), Event::PlayerOp(None)]);
        data.equipment.restore_slot(3, None).unwrap();
        let remove = config.changed(&mut data, 3, 1277, 0, true).unwrap();
        assert_eq!(data.variables.get(843), 0);
        assert_eq!(&remove[..2], &[Event::Sound(2238), Event::PlayerOp(None)]);
    }
    #[test]
    fn shield_with_two_handed_weapon_clears_shield_spec_without_combat_refresh() {
        let (config, mut data) = setup();
        data.equipment
            .restore_slot(
                3,
                Some(inventory::Item {
                    id: 2,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.variables = Variables::restore(BTreeMap::from([(301, 2), (843, 17)]));
        let events = config.changed(&mut data, 5, 1277, 0, true).unwrap();
        assert_eq!(
            events,
            vec![
                Event::Variable(Update { id: 301, value: 0 }),
                Event::Sound(2238),
                Event::PlayerOp(Some("Pelt".into()))
            ]
        );
        assert_eq!(data.variables.get(843), 17);
        assert!(!data.variables.contains(8));
    }
}
pub struct Config {
    pub special: Varp,
    pub stance: Varp,
    pub category: Binding,
    pub level: Binding,
    pub decimal: Binding,
    pub combat_stats: [u8; 7],
    pub items: BTreeMap<i32, Item>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Variable(Update),
    Sound(u16),
    PlayerOp(Option<String>),
    CombatRange(i32),
}
impl Config {
    pub fn changed(
        &self,
        data: &mut PlayerData,
        slot: usize,
        changed_item: i32,
        decimal: i32,
        logged: bool,
    ) -> Result<Vec<Event>, String> {
        let weapon = data.equipment.get(3).and_then(|i| self.items.get(&i.id));
        let mut vars = data.variables.clone();
        let mut events = Vec::new();
        let special = vars.get(self.special.id);
        if (slot == 3 && special == 1) || (slot == 5 && special == 2) {
            events.extend(vars.set(self.special, 0, logged).map(Event::Variable));
        }
        if let Some(sound) = self.items.get(&changed_item).and_then(|i| i.sound) {
            events.push(Event::Sound(sound));
        }
        events.push(Event::PlayerOp(weapon.and_then(|i| i.player_op.clone())));
        if slot == 3 || slot == 5 {
            let two_handed = weapon.is_some_and(|w| w.two_handed);
            let update_combat = slot == 3 || !two_handed;
            if update_combat && self.stance.transmit != Transmit::Never {
                // resyncVar bypasses login status and on-difference suppression.
                events.push(Event::Variable(Update {
                    id: self.stance.id,
                    value: vars.get(self.stance.id),
                }));
            }
            if slot == 3 && !two_handed {
                events.extend(vars.set(self.special, 0, logged).map(Event::Variable));
            }
            if update_combat {
                let [a, d, s, h, r, m, p] =
                    self.combat_stats.map(|id| data.stats.base(id) as u8 as i32);
                let level = crate::skills::combat_level(a, d, s, h, r, m, p);
                for (binding, value) in [
                    (self.level, level),
                    (self.decimal, decimal),
                    (
                        self.category,
                        combat_stance::category(weapon.map(|w| w.category)),
                    ),
                ] {
                    events.extend(
                        vars.set_bit(binding.base, binding.bit, value, logged)
                            .map_err(|e| format!("{e:?}"))?
                            .map(Event::Variable),
                    );
                }
                events.extend([Event::CombatRange(level), Event::CombatRange(level)]);
            }
        }
        data.variables = vars;
        Ok(events)
    }
}
