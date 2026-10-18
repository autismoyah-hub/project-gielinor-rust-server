//! Real local target cache plus isolated malformed-definition copies.
use gielinor_cache::{game::{Definitions,GameCache},symbols,definitions::{ItemType,StatType,Param}};
use gielinor_server::content::crafting_cape_boost;
use std::{collections::BTreeMap,path::Path};
fn copy(d:&Definitions)->Definitions{Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:d.items.clone(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:d.stats.clone()}}
#[test]fn actual_crafting_cape_boost_profile_and_fifty_five_binding_refusals(){
 let dir=std::env::var("PG_SYMBOLS").unwrap();let names=|name|symbols::load(&[Path::new(&dir)],name,None).unwrap();let obj=names("obj");let stats=names("stat");let params=names("param");
 let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let items=cache.files(2,10,None).unwrap().unwrap();let stat_bytes=cache.files(2,200,None).unwrap().unwrap();
 let mut d=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
 for id in [9780,9781]{d.items.insert(id,ItemType::decode(&items[&id]).unwrap());}d.stats.insert(12,StatType::decode(&stat_bytes[&12]).unwrap());
 let profile=crafting_cape_boost(&obj,&stats,&params,&d).unwrap();assert_eq!(profile.len(),2);
 for id in [9780,9781]{let r=profile[&id];assert_eq!((r.skill,r.slot,r.button_operation,r.members),(12,1,2,true));let def=&d.items[&(id as u32)];assert_eq!(def.params[&451],Param::String("Boost".into()));assert_eq!(def.params[&452],Param::String("Teleport".into()));}
 let mut n=0;for id in [9780,9781]{for case in 0..20{let mut q=copy(&d);let mut o=obj.clone();
  if case==0{q.items.remove(&(id as u32));}else if case==19{o.retain(|_,v|*v!=id);}else{let v=q.items.get_mut(&(id as u32)).unwrap();match case{
   1=>v.name="Wrong".into(),2=>v.members=false,3=>v.stackable=true,4=>v.dummy=1,5=>v.cert.template=799,6=>v.placeholder.template=14401,7=>v.transform.template=1,
   8=>v.wear[0]=4,9=>v.wear[1]=1,10=>v.inventory_ops[1]=None,11=>v.inventory_ops[2]=None,
   12=>{v.params.remove(&451);},13=>{v.params.insert(451,Param::String("Teleport".into()));},14=>{v.params.insert(452,Param::String("Boost".into()));},15=>{v.params.insert(434,Param::Integer(7));},16=>{v.params.insert(436,Param::Integer(98));},17=>{v.params.insert(451,Param::Integer(2));},18=>{v.params.insert(436,Param::String("99".into()));},_=>unreachable!()}}
  assert!(crafting_cape_boost(&o,&stats,&params,&q).is_err(),"item{id} case{case}");n+=1;
 }}
 for case in 0..15{let mut q=copy(&d);let mut s=stats.clone();let mut p=params.clone();match case{
  0=>{q.stats.remove(&12);},1=>q.stats.get_mut(&12).unwrap().name="Wrong".into(),2=>q.stats.get_mut(&12).unwrap().unreleased=true,3=>q.stats.get_mut(&12).unwrap().min_level=0,4=>q.stats.get_mut(&12).unwrap().max_level=100,
  5=>{s.remove("crafting");},6=>{s.insert("crafting".into(),7);},
  _=>{let name=["wear_op1","wear_op2","statreq1_skill","statreq1_level"][(case-7)/2];if case%2==1{p.remove(name);}else{p.insert(name.into(),99999);}}
 }assert!(crafting_cape_boost(&obj,&s,&p,&q).is_err(),"stat/paramcase{case}");n+=1;}
 assert_eq!(n,55);println!("PG_CAPE_BINDINGS actual2ordinarymembercapewear1/stat12base99/451Boost452Teleport/nativeop2 and55malformedbindingrefusals PASS");
}
