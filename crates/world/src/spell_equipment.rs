//! CombatMagicAttributeCollector.spellCollect with explicit ordered random draws.
use crate::{inventory::Inventory, npc::RandomSource, spell_modifiers::Spell};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default)]
pub struct Catalog {
    /// Resolver applies original exclusive spell/necklace/weapon/shield choices.
    pub items: BTreeMap<(usize, i32), u64>,
    pub spells: BTreeMap<i32, u64>,
    pub god_spells: BTreeSet<i32>,
    pub ahrim: [BTreeSet<i32>; 4], // hat, torso, legs, weapon
    pub damned: BTreeSet<i32>,
    pub brimstone: BTreeSet<i32>,
    pub void_helm: BTreeSet<i32>,
    pub void_top: BTreeSet<i32>,
    pub void_robe: BTreeSet<i32>,
    pub void_gloves: BTreeSet<i32>,
}
fn wearing(worn: &Inventory, slot: usize, ids: &BTreeSet<i32>) -> bool {
    worn.get(slot).is_some_and(|item| ids.contains(&item.id))
}
pub struct Context {
    pub spell: i32,
    pub standard_book: bool,
    pub sunfire: bool,
    pub skull: i32,
    pub mark_of_darkness: i32,
    pub charge_cooldown: i32,
}
impl Catalog {
    pub fn collect(
        &self,
        worn: &Inventory,
        context: Context,
        random: &mut impl RandomSource,
    ) -> u64 {
        let mut attributes = self.spells.get(&context.spell).copied().unwrap_or(0);
        if context.standard_book {
            attributes |= 1 << Spell::StandardBook as u8;
        }
        // Ahrim precedes Brimstone; absent/incomplete gear consumes no draw.
        if [0, 4, 7, 3]
            .into_iter()
            .zip(&self.ahrim)
            .all(|(slot, ids)| wearing(worn, slot, ids))
            && wearing(worn, 2, &self.damned)
            && random.below(4) == 0
        {
            attributes |= 1 << Spell::AhrimPassive as u8;
        }
        if wearing(worn, 12, &self.brimstone) && random.below(4) == 0 {
            attributes |= 1 << Spell::BrimstonePassive as u8;
        }
        for slot in [0, 2, 3, 5, 9, 12] {
            if let Some(item) = worn.get(slot) {
                attributes |= self.items.get(&(slot, item.id)).copied().unwrap_or(0);
            }
        }
        if context.mark_of_darkness == 1 {
            attributes |= 1 << Spell::MarkOfDarkness as u8;
        }
        if context.charge_cooldown > 0 && self.god_spells.contains(&context.spell) {
            attributes |= 1 << Spell::ChargeSpell as u8;
        }
        if context.skull == 3 {
            attributes |= 1 << Spell::ForinthrySurge as u8;
        }
        if context.sunfire && attributes & (1 << Spell::FireSpell as u8) != 0 {
            attributes |= 1 << Spell::SunfireRunePassive as u8;
        }
        attributes
    }
    pub fn accuracy_void(&self, worn: &Inventory) -> f64 {
        if wearing(worn, 0, &self.void_helm)
            && wearing(worn, 4, &self.void_top)
            && wearing(worn, 7, &self.void_robe)
            && wearing(worn, 9, &self.void_gloves)
        {
            1.45
        } else {
            1.0
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    struct Draws(std::collections::VecDeque<u32>);
    impl RandomSource for Draws {
        fn below(&mut self, b: u32) -> u32 {
            assert_eq!(b, 4);
            self.0.pop_front().unwrap()
        }
    }
    #[test]
    fn passives_draw_again_in_damage_stage_and_missing_gear_skips_draws() {
        let mut c = Catalog::default();
        c.ahrim = [[1].into(), [2].into(), [3].into(), [4].into()];
        c.damned = [5].into();
        c.brimstone = [6].into();
        let mut worn = Inventory::new(14, StackType::Normal, false);
        for (slot, id) in [(0, 1), (4, 2), (7, 3), (3, 4), (2, 5), (12, 6)] {
            worn.restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let context = || Context {
            spell: 0,
            standard_book: false,
            sunfire: false,
            skull: -1,
            mark_of_darkness: 0,
            charge_cooldown: 0,
        };
        let mut random = Draws([0, 1, 1, 0].into());
        assert_eq!(
            c.collect(&worn, context(), &mut random),
            1 << Spell::AhrimPassive as u8
        );
        assert_eq!(
            c.collect(&worn, context(), &mut random),
            1 << Spell::BrimstonePassive as u8
        );
        assert!(random.0.is_empty());
        worn.restore_slot(4, None).unwrap();
        worn.restore_slot(12, None).unwrap();
        assert_eq!(c.collect(&worn, context(), &mut random), 0);
    }
}
