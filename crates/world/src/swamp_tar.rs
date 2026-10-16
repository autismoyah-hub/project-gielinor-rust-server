//! Wiki-backed fifteen-unit herb tar batches. Scheduling is owned by the caller.
//! Guam tar15183062, Marrentill tar15183124, Tarromin tar15184685,
//! Harralander tar15184745. No OpenRune expression is reused.
use crate::{inventory::{Catalog, Delete, Insert, Inventory}, skills::{self, Advancement}, stats::Stats};

pub const BATCH: i32 = 15;
#[derive(Clone, Copy, Debug)]
pub struct Recipe { pub herb:i32, pub tar:i32, pub output:i32, pub tool:i32, pub level:u8, pub base_fine_xp:u16 }
#[derive(Clone, Copy, Debug)]
pub struct Job { pub recipe:Recipe, pub herb_slot:usize, pub tar_slot:usize }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Made(Advancement), InvalidRecipe, Members, Quest, Level, Tool, Stale, Capacity, InvalidReward }
impl Job {
    pub fn complete(&self, inventory:&mut Inventory, catalog:&Catalog, stats:&mut Stats,
        member:bool, quest_complete:bool, rate_hundreds:i32) -> Outcome {
        let r=self.recipe;
        let ids=[r.herb,r.tar,r.output,r.tool];
        if ids.iter().any(|&id|id<0) || (0..4).any(|i|ids[i+1..].contains(&ids[i]))
            || !(1..=99).contains(&r.level) || r.base_fine_xp==0
            || !catalog.stackable.contains(&r.tar) || !catalog.stackable.contains(&r.output)
            || catalog.stackable.contains(&r.herb) || catalog.stackable.contains(&r.tool) {
            return Outcome::InvalidRecipe;
        }
        if !member {return Outcome::Members;}
        if !quest_complete {return Outcome::Quest;}
        if rate_hundreds<=0 {return Outcome::InvalidReward;}
        if i32::from(stats.current(15))<i32::from(r.level) {return Outcome::Level;}
        if !inventory.slots().iter().flatten().any(|i|i.id==r.tool && i.count==1 && i.vars==0) {return Outcome::Tool;}
        if self.herb_slot==self.tar_slot
            || inventory.get(self.herb_slot).is_none_or(|i|i.id!=r.herb || i.count!=1 || i.vars!=0)
            || inventory.get(self.tar_slot).is_none_or(|i|i.id!=r.tar || i.count<BATCH || i.vars!=0) {return Outcome::Stale;}
        let mut next_inventory=inventory.clone();
        for (id,count,slot) in [(r.herb,1,self.herb_slot),(r.tar,BATCH,self.tar_slot)] {
            let mut debit=Delete::new(id,count);debit.strict_slot=Some(slot);
            if next_inventory.delete(catalog,debit).is_err() {return Outcome::Stale;}
        }
        if next_inventory.insert(catalog,Insert::new(r.output,BATCH)).is_err() {return Outcome::Capacity;}
        let mut next_stats=stats.clone();
        let Ok(advancement)=skills::advance(&mut next_stats,15,f64::from(r.base_fine_xp)/10.0,f64::from(rate_hundreds)/100.0,99)
            else {return Outcome::InvalidReward;};
        *inventory=next_inventory;*stats=next_stats;
        Outcome::Made(advancement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item,StackType};
    fn profiles()->[Recipe;5] {
        [(249,10142,19,300),(6681,10142,19,300),(251,10143,31,425),(253,10144,39,550),(255,10145,44,725)]
            .map(|(herb,output,level,base_fine_xp)|Recipe{herb,tar:1939,output,tool:233,level,base_fine_xp})
    }
    fn fixture(recipe:Recipe,reverse:bool,tar_count:i32)->(Job,Inventory,Catalog,Stats) {
        let (herb_slot,tar_slot)=if reverse {(1,0)} else {(0,1)};
        let mut inv=Inventory::new(28,StackType::Normal,false);
        for slot in 0..28 {
            let (id,count)=if slot==herb_slot {(recipe.herb,1)} else if slot==tar_slot {(recipe.tar,tar_count)} else if slot==2 {(recipe.tool,1)} else {(995,slot as i32+100)};
            inv.restore_slot(slot,Some(Item{id,count,vars:0})).unwrap();
        }
        let mut catalog=Catalog::default();catalog.stackable.extend([recipe.tar,recipe.output,995]);
        let mut stats=Stats::default();stats.set_base(15,recipe.level as i8-1);stats.set_current(15,recipe.level as i8);stats.set_fine_xp(15,100_001).unwrap();
        (Job{recipe,herb_slot,tar_slot},inv,catalog,stats)
    }
    #[test]
    fn five_boosted_profiles_full_backpack_exact_batches_and_fractional_rewards() {
        for recipe in profiles() {for reverse in [false,true] {for count in [15,16,30] {
            let (job,mut inv,catalog,mut stats)=fixture(recipe,reverse,count);
            let before=inv.clone();let before_stats=stats.clone();
            assert!(matches!(job.complete(&mut inv,&catalog,&mut stats,true,true,500),Outcome::Made(_)));
            assert_eq!(stats.fine_xp(15),100_001+i32::from(recipe.base_fine_xp)*5);
            assert_eq!(inv.slots().iter().flatten().filter(|i|i.id==recipe.output).map(|i|i.count).sum::<i32>(),15);
            assert_eq!(inv.slots().iter().flatten().filter(|i|i.id==recipe.tar).map(|i|i.count).sum::<i32>(),count-15);
            for slot in 2..28 {assert_eq!(inv.get(slot),before.get(slot));}
            for skill in 0..25 {if skill!=15 {assert_eq!(stats.fine_xp(skill),before_stats.fine_xp(skill));}}
            let made=inv.clone();let rewarded=stats.clone();
            assert_eq!(job.complete(&mut inv,&catalog,&mut stats,true,true,500),Outcome::Stale);
            assert_eq!(inv,made);assert_eq!(stats,rewarded);
        }}}
    }
    #[test]
    fn all_refusals_and_stack_overflow_are_atomic() {
        for mode in 0..12 {
            let (mut job,mut inv,mut catalog,mut stats)=fixture(profiles()[2],false,30);
            let mut member=true;let mut quest=true;let mut rate=500;
            let expected=match mode {
                0=>{member=false;Outcome::Members},1=>{quest=false;Outcome::Quest},
                2=>{stats.set_current(15,30);Outcome::Level},
                3=>{inv.restore_slot(2,None).unwrap();Outcome::Tool},
                4=>{inv.restore_slot(2,Some(Item{id:233,count:1,vars:1})).unwrap();Outcome::Tool},
                5=>{job.herb_slot=3;Outcome::Stale},
                6=>{inv.restore_slot(1,Some(Item{id:1939,count:14,vars:0})).unwrap();Outcome::Stale},
                7=>{inv.restore_slot(1,Some(Item{id:1939,count:30,vars:1})).unwrap();Outcome::Stale},
                8=>{inv.restore_slot(3,Some(Item{id:10143,count:i32::MAX-14,vars:0})).unwrap();Outcome::Capacity},
                9=>{rate=0;Outcome::InvalidReward},
                10=>{catalog.stackable.remove(&1939);Outcome::InvalidRecipe},
                _=>{job.recipe.tool=job.recipe.herb;Outcome::InvalidRecipe},
            };
            let before=inv.clone();let before_stats=stats.clone();
            assert_eq!(job.complete(&mut inv,&catalog,&mut stats,member,quest,rate),expected,"mode{mode}");
            assert_eq!(inv,before,"mode{mode}");assert_eq!(stats,before_stats,"mode{mode}");
        }
    }
    #[test]
    fn existing_output_stack_merges_and_reward_caps_without_losing_batch() {
        let (job,mut inv,catalog,mut stats)=fixture(profiles()[4],false,15);
        inv.restore_slot(3,Some(Item{id:10145,count:123,vars:0})).unwrap();
        stats.set_fine_xp(15,Stats::MAX_FINE_XP-7).unwrap();
        assert!(matches!(job.complete(&mut inv,&catalog,&mut stats,true,true,500),Outcome::Made(_)));
        assert_eq!(inv.get(3),Some(Item{id:10145,count:138,vars:0}));
        assert_eq!(stats.fine_xp(15),Stats::MAX_FINE_XP);
        assert_eq!(inv.get(0),None);assert_eq!(inv.get(1),None);
    }
}
