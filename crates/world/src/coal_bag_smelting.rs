//! Furnace-only virtual coal consumption. Physical coal is spent first (Wiki),
//! unlike the donor's StorageFirst hook. No virtual items are inserted into slots.
use crate::{coal_bag::Config, inventory::{Catalog,Delete,Insert}, simulation::PlayerData, smithing_make::{Job,Outcome,total}};
impl Job {
    pub(crate) fn complete_furnace(&self,data:&mut PlayerData,catalog:&Catalog,bag:Option<Config>,skill:u8,xp_rate:f64,max_level:u8,member:bool,random:&mut impl crate::npc::RandomSource)->Result<Outcome,crate::stats::InvalidXp>{
        let bag=bag.filter(|c|self.furnace && c.consumable(&data.inventory,&data.variables,member)>0 && !self.request.impurity && self.request.consumed.iter().any(|&(id,n)|id==c.coal && n>0));
        let Some(bag)=bag else{return self.complete_for_member(&mut data.inventory,catalog,&mut data.stats,skill,xp_rate,max_level,member,random);};
        let request=&self.request;
        if (request.members&&!member)||i32::from(data.stats.current(skill)as u8)<request.required_level||request.required_tools.iter().any(|&id|total(&data.inventory,catalog,id)<=0){return Ok(Outcome::RequirementsChanged);}
        // A bad cache identity must never turn stored variables into spendable ore.
        if !catalog.ordinary_nonstackable(bag.coal){return Ok(Outcome::MissingInputs);}
        let stored=bag.consumable(&data.inventory,&data.variables,member);
        if request.affordable_count_with_stock(&data.inventory,catalog,1,Some((bag.coal,stored))).unwrap_or(0)<1{return Ok(Outcome::MissingInputs);}
        let mut stats=data.stats.clone();
        let advancement=crate::skills::advance(&mut stats,skill,request.xp,xp_rate,max_level)?;
        let mut inventory=data.inventory.clone();let mut variables=data.variables.clone();let mut remaining=stored;
        for &(id,amount)in &request.consumed {
            if amount<=0{return Ok(Outcome::MissingInputs);}
            let physical=if id==bag.coal{amount.min(total(&inventory,catalog,id))}else{amount};
            if physical>0 && inventory.delete(catalog,Delete::new(id,physical)).is_err(){return Ok(Outcome::MissingInputs);}
            if id==bag.coal {let virtual_count=amount-physical;if virtual_count>remaining{return Ok(Outcome::MissingInputs);}remaining-=virtual_count;}
        }
        if remaining!=stored && variables.set_bit(bag.base,bag.stored,remaining,false).is_err(){return Ok(Outcome::MissingInputs);}
        if inventory.insert(catalog,Insert::new(request.product,request.produced)).is_err(){return Ok(Outcome::NoSpace);}
        // Inputs, output, permanent packed bits and XP commit together.
        data.inventory=inventory;data.variables=variables;data.stats=stats;
        let remaining_jobs=self.remaining.wrapping_sub(1);
        let available=bag.consumable(&data.inventory,&data.variables,member);
        let next=(remaining_jobs>0&&request.affordable_count_with_stock(&data.inventory,catalog,1,Some((bag.coal,available))).unwrap_or(0)>0).then(||Job{request:request.clone(),remaining:remaining_jobs,furnace:true});
        Ok(Outcome::Made{product:request.product,count:request.produced,advancement,next})
    }
}
