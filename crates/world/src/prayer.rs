//! Prayer burial and chapel offerings: delayed consumption and source XP modifiers.
use crate::{
    inventory::{Catalog, Delete, Inventory},
    simulation::PlayerData,
    skills, stat_ops, CoordGrid,
};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Config {
    pub skill: u8,
    pub animation: u16,
    pub bones: BTreeMap<i32, f64>,
    pub necklace: Option<i32>,
    pub restore: BTreeMap<i32, i32>,
    /// WornXpModifiers values for the Prayer stat, summed before dividing by100.
    pub worn_xp_percent: BTreeMap<i32, i32>,
    pub chaos: Option<[i32; 4]>,
}
#[derive(Clone, Copy, Debug)]
pub struct Pending {
    pub bone: i32,
    pub restore: i32,
}
#[derive(Debug)]
pub enum Error {
    MissingBone,
    Inventory,
    Restore,
    Xp,
}
#[derive(Debug)]
pub struct Completed {
    pub restore: Option<stat_ops::Update>,
    pub advancement: skills::Advancement,
}
#[derive(Clone, Copy, Debug)]
pub enum State {
    Confirm {
        token: u64,
        bone: i32,
        slot: Option<usize>,
    },
    Bury { pending: Pending, slot: Option<usize> },
    Offer {
        bone: i32,
        animation: u16,
        offered: bool,
        xp: Option<f64>,
        continuation: Option<crate::tower_offering::Continuation>,
    },
    CrateWalk {
        step: u8,
        attempts: u8,
        pending: Option<CoordGrid>,
    },
}
#[derive(Debug)]
pub enum Event {
    Confirm {
        token: u64,
    },
    Closed,
    Animation(u16),
    Completed {
        skill: u8,
        result: Completed,
    },
    Message(&'static str),
    /// Coroutine return value, emitted only on normal completion, never cancellation.
    OfferingFinished {
        offered: bool,
    },
}
pub(crate) fn contains(inv: &Inventory, bone: i32) -> bool {
    inv.slots()
        .iter()
        .flatten()
        .any(|i| i.id == bone && i.count > 0)
}
fn selected_ordinary(player:&PlayerData,catalog:&Catalog,bone:i32,slot:usize)->bool {
    player.inventory.stack==crate::inventory::StackType::Normal && !player.inventory.placeholders
        && player.inventory.get(slot).is_some_and(|item|item.id==bone && item.count==1 && item.vars==0)
        && bone>=0 && !catalog.stackable.contains(&bone) && !catalog.dummy.contains(&bone)
        && !catalog.notes.get(&bone).is_some_and(|v|v.template!=0)
        && !catalog.placeholders.get(&bone).is_some_and(|v|v.template!=0)
        && !catalog.transforms.get(&bone).is_some_and(|v|v.template!=0)
}
impl Config {
    pub fn requires_confirmation(&self, coords: CoordGrid) -> bool {
        self.chaos.is_some_and(|[x0, z0, x1, z1]| {
            (x0..=x1).contains(&coords.x()) && (z0..=z1).contains(&coords.z())
        })
    }
    pub fn prepare(&self, player: &PlayerData, bone: i32) -> Result<Pending, Error> {
        if !self.bones.contains_key(&bone) || !contains(&player.inventory, bone) {
            return Err(Error::MissingBone);
        }
        let necklace = self
            .necklace
            .is_some_and(|id| player.equipment.get(2).is_some_and(|i| i.id == id));
        Ok(Pending {
            bone,
            restore: if necklace {
                self.restore.get(&bone).copied().unwrap_or(0)
            } else {
                0
            },
        })
    }
    /// Selected gameplay admission; `prepare` remains the original type-search
    /// reference API. Configured member eligibility is checked by the world.
    pub fn prepare_selected(&self, player:&PlayerData, catalog:&Catalog, bone:i32, slot:usize)->Result<Pending,Error>{
        if !selected_ordinary(player,catalog,bone,slot){return Err(Error::MissingBone);}
        self.prepare(player,bone)
    }
    /// Commit the selected inventory/Stats result together. Legacy reference
    /// and chapel offering callers retain their prior APIs and ordering.
    pub fn finish_selected(&self, player:&mut PlayerData, catalog:&Catalog, pending:Pending, slot:usize)->Result<Option<Completed>,Error>{
        let xp=self.bones.get(&pending.bone).copied().ok_or(Error::MissingBone)?;
        if !xp.is_finite() || xp<0.0 || player.xp_rate_hundreds<=0 {return Err(Error::Xp);}
        if pending.restore<0 {return Err(Error::Restore);}
        let percent=player.equipment.slots().iter().flatten().try_fold(0i32,|sum,item|sum.checked_add(self.worn_xp_percent.get(&item.id).copied().unwrap_or(0))).ok_or(Error::Xp)?;
        if percent < -100 {return Err(Error::Xp);}
        let mut next=player.clone();
        let result=self.finish_with_multiplier(&mut next,catalog,pending,1.0,self.bones.get(&pending.bone).copied(),Some(slot))?;
        if result.is_some(){*player=next;}
        Ok(result)
    }

    pub fn finish(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        pending: Pending,
    ) -> Result<Option<Completed>, Error> {
        self.finish_with_multiplier(
            player,
            catalog,
            pending,
            1.0,
            self.bones.get(&pending.bone).copied(),
            None,
        )
    }
    /// Prayer.offerOnAltar omits necklace restoration and applies the chapel
    /// multiplier before the worn modifier and account XP rate.
    pub fn finish_offering(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        bone: i32,
    ) -> Result<Option<Completed>, Error> {
        self.finish_offering_xp(player, catalog, bone, self.bones.get(&bone).copied())
    }
    pub(crate) fn finish_offering_xp(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        bone: i32,
        xp: Option<f64>,
    ) -> Result<Option<Completed>, Error> {
        self.finish_with_multiplier(player, catalog, Pending { bone, restore: 0 }, 2.0, xp, None)
    }
    fn finish_with_multiplier(
        &self,
        player: &mut PlayerData,
        catalog: &Catalog,
        pending: Pending,
        multiplier: f64,
        xp: Option<f64>,
        slot: Option<usize>,
    ) -> Result<Option<Completed>, Error> {
        if slot.is_some_and(|slot|!selected_ordinary(player,catalog,pending.bone,slot)) || !contains(&player.inventory, pending.bone) {
            return Ok(None);
        }
        // Only legacy/reference and offering calls search by type. Selected
        // gameplay retains the captured slot through its delayed completion.
        let mut debit=Delete::new(pending.bone,1);
        debit.slot=slot.unwrap_or(0);debit.strict_slot=slot;
        player
            .inventory
            .delete(catalog, debit)
            .map_err(|_| Error::Inventory)?;
        let restore = if pending.restore > 0 {
            Some(
                player
                    .stats
                    .apply(
                        self.skill,
                        stat_ops::Operation::Heal,
                        pending.restore,
                        0,
                        false,
                    )
                    .map_err(|_| Error::Restore)?,
            )
        } else {
            None
        };
        // Source resolves the XP parameter after deleting the bone.
        let xp = xp.ok_or(Error::MissingBone)?;
        let percent = player
            .equipment
            .slots()
            .iter()
            .flatten()
            .fold(0i32, |sum, item| {
                sum.wrapping_add(self.worn_xp_percent.get(&item.id).copied().unwrap_or(0))
            });
        let xp = xp * multiplier * (1.0 + percent as f64 / 100.0);
        let advancement = skills::advance(
            &mut player.stats,
            self.skill,
            xp,
            player.xp_rate_hundreds as f64 / 100.0,
            99,
        )
        .map_err(|_| Error::Xp)?;
        Ok(Some(Completed {
            restore,
            advancement,
        }))
    }
}

#[cfg(test)]
mod selected_tests {
    use super::*;
    use crate::{inventory::{Item, StackType}, stats::Stats};
    #[test]
    fn selected_burial_never_substitutes_duplicates_or_loses_items_on_xp_restore_failure() {
        let config=Config{skill:5,animation:827,bones:BTreeMap::from([(526,4.5)]),necklace:None,restore:BTreeMap::new(),worn_xp_percent:BTreeMap::new(),chaos:None};
        let mut player=PlayerData::new(1,"Selected burial".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());player.xp_rate_hundreds=125;
        for slot in 0..2{player.inventory.restore_slot(slot,Some(Item{id:526,count:1,vars:0})).unwrap();}
        let catalog=Catalog::default();let pending=config.prepare_selected(&player,&catalog,526,1).unwrap();
        let mut good=player.clone();assert!(config.finish_selected(&mut good,&catalog,pending,1).unwrap().is_some());assert_eq!(good.inventory.get(0).unwrap().id,526);assert!(good.inventory.get(1).is_none());assert_eq!(good.stats.fine_xp(5),56);
        for bad in 0..7 {
            let mut player=player.clone();let mut config=config.clone();let mut pending=pending;let mut catalog=Catalog::default();
            match bad{0=>player.inventory.restore_slot(1,None).unwrap(),1=>{config.bones.insert(526,f64::NAN);},2=>{player.xp_rate_hundreds=-1;},3=>{pending.restore=1;player.stats.set_base(5,10);player.stats.set_current(5,11);},4=>{catalog.dummy.insert(526);},5=>player.inventory.restore_slot(1,Some(Item{id:526,count:2,vars:0})).unwrap(),_=>{player.inventory.stack=StackType::Always;}}
            let before=(player.inventory.clone(),player.stats.clone());let outcome=config.finish_selected(&mut player,&catalog,pending,1);assert!(!matches!(outcome,Ok(Some(_))));assert_eq!((player.inventory,player.stats),before);
        }
    }
}
