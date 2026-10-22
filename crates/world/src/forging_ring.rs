//! Player-owned forging-ring wear. The counter belongs to the character, not
//! an item instance; swapping rings cannot refill it.
use crate::{inventory::{Catalog,Item},simulation::PlayerData,smithing_make::{Job,Outcome,Request},variables::Varp};
use std::sync::Arc;
#[derive(Clone,Copy,Debug)]
pub struct Config {pub ring:i32,pub used:Varp}
impl Config {
 pub fn remaining(self,data:&PlayerData)->Option<i32>{let used=data.variables.get(self.used.id);(0..140).contains(&used).then(||140-used)}
 pub fn worn(self,data:&PlayerData,catalog:&Catalog)->bool{catalog.ordinary_nonstackable(self.ring)&&data.equipment.get(12)==Some(Item{id:self.ring,count:1,vars:0})}
 fn recipe(self,r:&Request)->bool{r.product==2351&&r.produced==1&&r.consumed==[(440,1)]&&r.xp==12.5&&r.impurity&&!r.members&&r.required_level==15&&r.required_tools.is_empty()}
}
impl Job {
 pub(crate) fn complete_with_forging_ring(&self,data:&mut PlayerData,catalog:&Catalog,bag:Option<crate::coal_bag::Config>,catalyst:Option<&crate::smithing_catalyst::Config>,ring:Option<Config>,skill:u8,rate:f64,max_level:u8,member:bool,xp:f64,random:&mut impl crate::npc::RandomSource)->Result<Outcome,crate::stats::InvalidXp>{
  let ring=ring.filter(|c|self.furnace&&skill==13&&c.recipe(&self.request)&&c.worn(data,catalog));
  let Some(ring)=ring else{return self.complete_catalyzed(data,catalog,bag,catalyst,skill,rate,max_level,member,xp,random);};
  let Some(charges)=ring.remaining(data)else{return Ok(Outcome::RequirementsChanged);};
  if !catalog.ordinary_nonstackable(440)||!catalog.ordinary_nonstackable(2351)||data.inventory.slots().iter().flatten().any(|i|i.id==440&&(i.count!=1||i.vars!=0)){return Ok(Outcome::MissingInputs);}
  let mut effective=(*self.request).clone();effective.impurity=false;
  let temporary=Job{request:Arc::new(effective),remaining:1,furnace:true};let mut staged=data.clone();
  let outcome=temporary.complete_catalyzed(&mut staged,catalog,bag,catalyst,skill,rate,max_level,member,xp,random)?;
  match outcome {
   Outcome::Made{product,count,advancement,..}=>{
    if crate::smithing_make::total(&data.inventory,catalog,440)-crate::smithing_make::total(&staged.inventory,catalog,440)!=1||crate::smithing_make::total(&staged.inventory,catalog,2351)-crate::smithing_make::total(&data.inventory,catalog,2351)!=1{return Ok(Outcome::MissingInputs);}
    let broken=charges==1;
    staged.variables.set(ring.used,if broken{0}else{140-charges+1},false);
    if broken{staged.equipment.restore_slot(12,None).expect("attested worn slot");}
    data.inventory=staged.inventory;data.stats=staged.stats;data.variables=staged.variables;data.equipment=staged.equipment;
    let remaining=self.remaining.wrapping_sub(1);let next=(!broken&&remaining>0&&self.request.has_inputs(&data.inventory,catalog)).then(||Job{request:self.request.clone(),remaining,furnace:true});
    Ok(Outcome::Made{product,count,advancement,next})
   },other=>Ok(other)
  }
 }
}
