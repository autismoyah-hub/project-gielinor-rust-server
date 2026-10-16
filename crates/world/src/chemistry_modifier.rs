//! Atomic ordinary chemistry and prescription-goggles modifiers.
//! Recipe dose mappings and amulet definitions must be validated at startup.
use crate::{chemistry, finished_potions::{Job, Outcome}, inventory::{Catalog, Delete, Insert, Inventory}, npc::RandomSource, stats::Stats};

#[derive(Clone, Copy, Debug)]
pub struct Binding { pub amulet: i32, pub extra_output: i32 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Result { pub outcome: Outcome, pub secondary_saved: bool, pub chemistry: chemistry::Proc, pub output: i32 }

impl Job {
    pub fn complete_with_modifiers(
        &self, inventory: &mut Inventory, catalog: &Catalog, stats: &mut Stats,
        equipment: &mut Inventory, state: &mut chemistry::State,
        member: bool, quest: bool, rate: i32, goggles: bool,
        binding: Option<Binding>, random: &mut impl RandomSource,
    ) -> Result {
        let none = chemistry::Proc { extra_dose: false, crumbled: false, stop: false };
        let refused = |outcome| Result { outcome, secondary_saved: false, chemistry: none, output: self.recipe.output };
        let mut ordinary = inventory.clone();
        let mut next_stats = stats.clone();
        let outcome = self.complete(&mut ordinary, catalog, &mut next_stats, member, quest, rate);
        if !matches!(outcome, Outcome::Made(_)) { return refused(outcome); }
        let binding = binding.filter(|b| equipment.get(2).is_some_and(|i| i.id == b.amulet && i.count == 1 && i.vars == 0));
        if let Some(b) = binding {
            if b.amulet < 0 || b.extra_output < 0 || [self.recipe.unfinished, self.recipe.secondary, self.recipe.output, b.amulet].contains(&b.extra_output)
                || [self.recipe.unfinished, self.recipe.secondary, self.recipe.output].contains(&b.amulet)
                || [b.amulet, b.extra_output, self.recipe.output].iter().any(|id| catalog.stackable.contains(id) || catalog.notes.contains_key(id) || catalog.placeholders.contains_key(id) || catalog.transforms.contains_key(id) || catalog.dummy.contains(id)) {
                return refused(Outcome::InvalidRecipe);
            }
        }
        // Prepare every possible inventory result before consuming random draws.
        // This also avoids selecting an unrelated pre-existing output potion.
        let candidate = |output, saved| -> Option<Inventory> {
            let mut next = inventory.clone();
            for (id, slot) in [(self.recipe.unfinished, self.unfinished_slot), (self.recipe.secondary, self.secondary_slot)] {
                if saved && slot == self.secondary_slot { continue; }
                let mut query = Delete::new(id, 1); query.strict_slot = Some(slot);
                if next.delete(catalog, query).ok()?.completed != 1 { return None; }
            }
            if next.insert(catalog, Insert::new(output, 1)).ok()?.completed != 1 { return None; }
            Some(next)
        };
        let Some(saved_ordinary) = (if goggles { candidate(self.recipe.output, true) } else { Some(ordinary.clone()) }) else { return refused(Outcome::Capacity); };
        let mut next_equipment = equipment.clone();
        let extra = if let Some(b) = binding {
            let (Some(normal), Some(saved)) = (candidate(b.extra_output, false), candidate(b.extra_output, goggles)) else { return refused(Outcome::Capacity); };
            let mut query = Delete::new(b.amulet, 1); query.strict_slot = Some(2);
            if next_equipment.delete(catalog, query).is_err() { return refused(Outcome::Stale); }
            Some((normal, saved, b.extra_output))
        } else { None };
        // Keep the established goggles draw first; chemistry adds a draw only
        // for a live ordinary amulet and a recipe with an explicit extra dose.
        let saved = goggles && random.below(10) == 0;
        let mut next_state = *state;
        let proc = next_state.roll(binding.is_some(), random);
        let (next_inventory, output) = if proc.extra_dose {
            let (normal, retained, output) = extra.expect("validated chemistry candidate");
            (if saved { retained } else { normal }, output)
        } else { (if saved { saved_ordinary } else { ordinary }, self.recipe.output) };
        *inventory = next_inventory;
        *stats = next_stats;
        *state = next_state;
        if proc.crumbled { *equipment = next_equipment; }
        Result { outcome, secondary_saved: saved, chemistry: proc, output }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType};
    struct Draw { values: std::collections::VecDeque<(u32,u32)> }
    impl RandomSource for Draw { fn below(&mut self, upper: u32) -> u32 { let (expected,value)=self.values.pop_front().expect("unexpected random draw"); assert_eq!(upper,expected); value } }
    fn fixture(reverse: bool) -> (Job, Inventory, Inventory, Stats) {
        let mut inv=Inventory::new(28,StackType::Normal,false);
        let (a,b)=if reverse {(1,0)}else{(0,1)};
        for slot in 0..28 {inv.restore_slot(slot,Some(Item{id:if slot==a{93}else if slot==b{235}else if slot==2{2446}else{1931},count:1,vars:0})).unwrap();}
        let mut equip=Inventory::new(14,StackType::Normal,false);
        equip.restore_slot(2,Some(Item{id:21163,count:1,vars:0})).unwrap();
        let mut stats=Stats::default();stats.set_base(15,5);stats.set_current(15,5);stats.set_fine_xp(15,3881).unwrap();
        (Job{recipe:crate::finished_potions::Recipe{unfinished:93,secondary:235,output:175,level:5,boostable:true,base_fine_xp:375},unfinished_slot:a,secondary_slot:b},inv,equip,stats)
    }
    const BINDING: Option<Binding> = Some(Binding{amulet:21163,extra_output:2446});
    #[test]
    fn simultaneous_modifiers_full_inventory_both_orders_and_last_charge() {
        for reverse in [false,true] {for saved in [false,true] {for extra in [false,true] {for remaining in [1,5] {for stop in [false,true] {
            let (job,mut inv,mut equip,mut stats)=fixture(reverse);let untouched=inv.slots()[2..].to_vec();
            let mut state=chemistry::State::restore(remaining,stop as i32).unwrap();
            let mut rng=Draw{values:[(10,if saved{0}else{1}),(100,if extra{4}else{5})].into()};
            let r=job.complete_with_modifiers(&mut inv,&Catalog::default(),&mut stats,&mut equip,&mut state,true,true,500,true,BINDING,&mut rng);
            assert!(matches!(r.outcome,Outcome::Made(_)));assert_eq!(r.secondary_saved,saved);assert_eq!(r.chemistry.extra_dose,extra);
            assert_eq!(r.chemistry.crumbled,extra && remaining==1);assert_eq!(r.chemistry.stop,extra && remaining==1 && stop);
            assert_eq!(state.remaining(),if extra && remaining>1{remaining as u8-1}else if extra{5}else{remaining as u8});
            assert_eq!(equip.get(2).is_none(),extra && remaining==1);assert_eq!(r.output,if extra{2446}else{175});
            assert_eq!(inv.slots()[2..],untouched);assert_eq!(inv.slots().iter().flatten().filter(|i|i.id==235).count(),saved as usize);
            assert_eq!(inv.slots().iter().flatten().filter(|i|i.id==r.output).count(),if extra{2}else{1});
            assert_eq!(stats.fine_xp(15),5756);assert!(rng.values.is_empty());
        }}}}}
    }
    #[test]
    fn invalid_transactions_do_not_draw_or_mutate_any_state() {
        for mode in 0..10 {
            let (job,mut inv,mut equip,mut stats)=fixture(false);let mut catalog=Catalog::default();let mut binding=BINDING;
            match mode {3=>stats.set_current(15,4),4=>{inv.restore_slot(0,None).unwrap();},5=>{catalog.dummy.insert(2446);},6=>{catalog.stackable.insert(2446);},7=>binding=Some(Binding{amulet:21163,extra_output:93}),8=>binding=Some(Binding{amulet:21163,extra_output:175}),9=>binding=Some(Binding{amulet:21163,extra_output:21163}),_=>()}
            let mut state=chemistry::State::restore(1,1).unwrap();let before=(inv.clone(),equip.clone(),stats.clone(),state);let mut rng=Draw{values:Default::default()};
            let r=job.complete_with_modifiers(&mut inv,&catalog,&mut stats,&mut equip,&mut state,mode!=0,mode!=1,if mode==2{0}else{500},true,binding,&mut rng);
            assert!(!matches!(r.outcome,Outcome::Made(_)),"mode{mode}");assert_eq!((inv,equip,stats,state),before);
        }
    }
    #[test]
    fn unsupported_recipe_or_stale_equipment_has_no_chemistry_draw_and_cap_preserves_proc() {
        for mode in 0..6 {
            let (job,mut inv,mut equip,mut stats)=fixture(false);let mut state=chemistry::State::restore(1,1).unwrap();
            match mode {1=>equip.restore_slot(2,None).unwrap(),2=>equip.restore_slot(2,Some(Item{id:29990,count:1,vars:0})).unwrap(),3=>equip.restore_slot(2,Some(Item{id:21163,count:1,vars:1})).unwrap(),4=>equip.restore_slot(2,Some(Item{id:21163,count:2,vars:0})).unwrap(),5=>{stats.set_base(15,99);stats.set_current(15,99);stats.set_fine_xp(15,Stats::MAX_FINE_XP).unwrap();},_=>()}
            let mut rng=Draw{values:if mode==5{[(100,0)].into()}else{Default::default()}};
            let r=job.complete_with_modifiers(&mut inv,&Catalog::default(),&mut stats,&mut equip,&mut state,true,true,500,false,if mode==0{None}else{BINDING},&mut rng);
            assert!(matches!(r.outcome,Outcome::Made(_)));assert_eq!(r.chemistry.extra_dose,mode==5);assert_eq!(r.chemistry.stop,mode==5);
            if mode==5{assert_eq!(stats.fine_xp(15),Stats::MAX_FINE_XP);assert!(equip.get(2).is_none());}else{assert_eq!(state.remaining(),1);}
            assert!(rng.values.is_empty());
        }
    }
}
