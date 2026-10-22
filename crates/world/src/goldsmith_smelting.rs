//! Worn gold-smelting XP bonus for the maintained normal-furnace path.
use crate::{inventory::{Catalog,Inventory,Item},smithing_make::Request};
#[derive(Clone,Copy,Debug)]
pub struct Config {pub ore:i32,pub bar:i32,pub gauntlets:i32,pub capes:[i32;3]}
impl Config {
    /// Resolve at each completion; never cache equipment authority in a job.
    pub fn xp(self,request:&Request,worn:&Inventory,member:bool,catalog:&Catalog)->f64 {
        if !member || request.product!=self.bar || request.produced!=1 || request.consumed!=[(self.ore,1)] || request.xp!=22.5 || request.impurity || !catalog.ordinary_nonstackable(self.ore) || !catalog.ordinary_nonstackable(self.bar){return request.xp;}
        let equipped=|slot,id| catalog.ordinary_nonstackable(id)&&worn.get(slot)==Some(Item{id,count:1,vars:0});
        if equipped(9,self.gauntlets)||self.capes.into_iter().any(|id|equipped(1,id)){56.2}else{request.xp}
    }
}
