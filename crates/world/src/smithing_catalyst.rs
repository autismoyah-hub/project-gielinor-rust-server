//! Normal-furnace catalyst plan. Keep the registered request as the continuation
//! identity; evaluate the optional consumable again for each completed bar.
use crate::{coal_bag,inventory::{Catalog,Inventory,StackType},simulation::PlayerData,smithing_make::{Job,Outcome,Request}};
use std::{collections::BTreeMap,sync::Arc};
#[derive(Clone,Copy,Debug)]
pub struct Profile {pub ore:i32,pub coal:i32,pub level:i32,pub xp:f64}
#[derive(Clone,Debug)]
pub struct Config {pub item:i32,pub coal:i32,pub profiles:BTreeMap<i32,Profile>}
impl Config {
    pub(crate) fn effective(&self,request:&Request,inv:&Inventory,catalog:&Catalog,member:bool)->Option<Request>{
        let p=self.profiles.get(&request.product)?;
        if !member||inv.stack!=StackType::Normal||inv.placeholders||request.produced!=1||request.impurity||request.required_level!=p.level||request.xp!=p.xp||!request.required_tools.is_empty()||request.consumed!=[(p.ore,1),(self.coal,p.coal)]||p.coal<2||p.coal%2!=0{return None;}
        if [self.item,self.coal,p.ore,request.product].into_iter().any(|id|!catalog.ordinary_nonstackable(id)){return None;}
        let mut present=false;
        for item in inv.slots().iter().flatten().filter(|i|i.id==self.item){present=true;if item.count!=1||item.vars!=0{return None;}}
        if !present{return None;}
        let mut result=request.clone();result.consumed[1].1=p.coal/2;result.consumed.push((self.item,1));Some(result)
    }
}
impl Job {
    pub(crate) fn complete_catalyzed(&self,data:&mut PlayerData,catalog:&Catalog,bag:Option<coal_bag::Config>,catalyst:Option<&Config>,skill:u8,xp_rate:f64,max_level:u8,member:bool,base_xp:f64,random:&mut impl crate::npc::RandomSource)->Result<Outcome,crate::stats::InvalidXp>{
        let config=catalyst.filter(|_|self.furnace&&skill==13);
        let effective=config.and_then(|c|c.effective(&self.request,&data.inventory,catalog,member));
        let Some(effective)=effective else{return self.complete_furnace(data,catalog,bag,skill,xp_rate,max_level,member,base_xp,random);};
        // The last catalyst still earns double XP: eligibility belongs to this
        // transaction, before the consumable disappears from the inventory.
        if data.inventory.slots().iter().flatten().any(|i|effective.consumed.iter().any(|(id,_)|*id==i.id)&&(i.count!=1||i.vars!=0)){return Ok(Outcome::MissingInputs);}
        let temporary=Job{request:Arc::new(effective),remaining:1,furnace:true};let mut staged=data.clone();
        let outcome=temporary.complete_furnace(&mut staged,catalog,bag,skill,xp_rate,max_level,member,base_xp*2.0,random)?;
        match outcome {
            Outcome::Made{product,count,advancement,..}=>{
                let stock=|d:&PlayerData,id|crate::smithing_make::total(&d.inventory,catalog,id)+bag.filter(|c|c.coal==id).map_or(0,|c|d.variables.bit(c.stored).unwrap_or(0));
                if temporary.request.consumed.iter().any(|&(id,n)|stock(data,id)-stock(&staged,id)!=n){return Ok(Outcome::MissingInputs);}
                if stock(&staged,product)-stock(data,product)!=count{return Ok(Outcome::NoSpace);}
                data.inventory=staged.inventory;data.stats=staged.stats;data.variables=staged.variables;
                let remaining=self.remaining.wrapping_sub(1);
                let next_cost=config.and_then(|c|c.effective(&self.request,&data.inventory,catalog,member));let next_cost=next_cost.as_ref().unwrap_or(self.request.as_ref());
                let stock=bag.filter(|c|catalog.ordinary_nonstackable(c.coal)).map(|c|(c.coal,c.consumable(&data.inventory,&data.variables,member)));
                let next=(remaining>0&&next_cost.affordable_count_with_stock(&data.inventory,catalog,1,stock).unwrap_or(0)>0).then(||Job{request:self.request.clone(),remaining,furnace:true});
                Ok(Outcome::Made{product,count,advancement,next})
            }
            Outcome::Impure{..}=>Ok(Outcome::RequirementsChanged), // accepted profiles never use impurity RNG
            other=>Ok(other),
        }
    }
}
