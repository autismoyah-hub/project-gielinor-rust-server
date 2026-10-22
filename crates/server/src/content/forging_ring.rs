//! Server-owned counter, deliberately outside target-cache varp names.
use super::{Definitions,Symbols};
use gielinor_world::{forging_ring::Config,variables::{Varp,Transmit},smelting::Recipe};
pub const COUNTER:Varp=Varp{id:62400,permanent:true,transmit:Transmit::Never,bit_protect:true,client_code:-1};
pub fn forging_ring(items:&Symbols,varps:&Symbols,defs:&Definitions,recipes:&[Recipe])->Result<Config,String>{
 if varps.values().any(|&id|id==i32::from(COUNTER.id))||defs.varps.contains_key(&u32::from(COUNTER.id))||defs.varbits.values().any(|b|b.base==COUNTER.id){return Err("Forging-ring server counter collides with cache namespace".into());}
 for(sym,id,name,slot)in[("ring_of_forging",2568,"Ring of forging",12),("iron_ore",440,"Iron ore",-1),("iron_bar",2351,"Iron bar",-1)]{
  let t=defs.items.get(&(id as u32)).ok_or("Missing forging-ring type")?;
  if items.get(sym)!=Some(&id)||t.name!=name||t.wear!=[slot,-1,-1]||t.members||t.stackable||t.dummy==1||t.cert.template!=0||t.placeholder.template!=0||t.transform.template!=0||!t.objvars.is_empty(){return Err("Unattested forging-ring type".into());}
  if id==2568&&t.inventory_ops.iter().map(Option::as_deref).collect::<Vec<_>>()!=[None,Some("Wear"),None,Some("Break"),None]{return Err("Unattested ring operations".into());}
 }
 let mut rows=recipes.iter().filter(|r|r.bar==2351);let r=rows.next().ok_or("Configure iron furnace before ring")?;
 if rows.next().is_some()||r.ingredients!=[(440,1)]||r.level!=15||r.xp!=12.5||!r.impurity||r.members{return Err("Conflicting ordinary iron furnace".into());}
 Ok(Config{ring:2568,used:COUNTER})
}
#[cfg(test)]
mod tests{
 use super::*;
 fn prepared()->(Symbols,Definitions,Vec<Recipe>){let mut d=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};let mut n=Symbols::new();for(sym,id,name,slot)in[("ring_of_forging",2568,"Ring of forging",12),("iron_ore",440,"Iron ore",-1),("iron_bar",2351,"Iron bar",-1)]{let mut t=gielinor_cache::definitions::ItemType::decode(b"\0").unwrap();t.name=name.into();t.wear=[slot,-1,-1];if id==2568{t.inventory_ops[1]=Some("Wear".into());t.inventory_ops[3]=Some("Break".into());}d.items.insert(id,t);n.insert(sym.into(),id as i32);}let row=Recipe{bar:2351,label:"Iron".into(),level:15,xp:12.5,ingredients:vec![(440,1)],impurity:true,members:false,repeat_delay:None};(n,d,vec![row])}
 #[test]
 fn forging_ring_factory_attests_items_recipe_and_counter_isolation(){let(n,d,rows)=prepared();let c=forging_ring(&n,&Symbols::new(),&d,&rows).unwrap();assert_eq!((c.ring,c.used.id,c.used.permanent,c.used.transmit),(2568,62400,true,Transmit::Never));
 for id in[2568,440,2351]{for change in 0..11{let(mut n,mut d,rows)=prepared();let t=d.items.get_mut(&id).unwrap();match change{0=>{n.retain(|_,v|*v!=id as i32);},1=>t.name="Other".into(),2=>t.wear=[8,-1,-1],3=>t.members=true,4=>t.stackable=true,5=>t.dummy=1,6=>t.cert.template=799,7=>t.placeholder.template=799,8=>t.transform.template=799,9=>t.objvars.push(1),_=>{if id!=2568{continue;}t.inventory_ops.fill(None)}}assert!(forging_ring(&n,&Symbols::new(),&d,&rows).is_err());}}
 for change in 0..7{let(n,d,mut rows)=prepared();match change{0=>rows[0].xp=13.0,1=>rows[0].level=14,2=>rows[0].ingredients=vec![(440,2)],3=>rows[0].impurity=false,4=>rows[0].members=true,5=>rows.push(rows[0].clone()),_=>rows.clear()};assert!(forging_ring(&n,&Symbols::new(),&d,&rows).is_err());}
 for change in 0..3{let(n,mut d,rows)=prepared();let mut names=Symbols::new();match change{0=>{names.insert("Existing".into(),62400);},1=>{d.varps.insert(62400,COUNTER);},_=>{d.varbits.insert(1,gielinor_world::variables::Varbit{base:62400,lsb:0,msb:7});}}assert!(forging_ring(&n,&names,&d,&rows).is_err());}
 }
}
