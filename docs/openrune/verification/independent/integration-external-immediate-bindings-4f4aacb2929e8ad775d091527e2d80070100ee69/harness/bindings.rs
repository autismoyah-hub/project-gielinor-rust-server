//! Independently loaded cache definitions; no synthetic positive metadata.
use gielinor_cache::{game::{Definitions,GameCache},symbols};
use gielinor_server::content::immediate_tool_crafting;
use std::{collections::BTreeMap,path::Path};
fn copy(d:&Definitions)->Definitions{Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:d.items.clone(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:d.stats.clone()}}
#[test]
fn actual_immediate_bindings_and_eighty_one_fail_closed_mutations(){
 let dir=std::env::var("PG_SYMBOLS").unwrap();let objects=symbols::load(&[Path::new(&dir)],"obj",None).unwrap();let seq=symbols::load(&[Path::new(&dir)],"seq",None).unwrap();let stat=symbols::load(&[Path::new(&dir)],"stat",None).unwrap();
 let cache=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&cache),0xff_ffff).unwrap();let items=cache.files(2,10,None).unwrap().unwrap();let stats=cache.files(2,200,None).unwrap().unwrap();
 let ids=[("knife",946),("dramen_branch",771),("dramen_staff",772),("chisel",1755),("limestone",3211),("limestonebrick",3420),("rock",968)];
 let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
 for (_,id) in ids{defs.items.insert(id as u32,gielinor_cache::definitions::ItemType::decode(&items[&(id as u32)]).unwrap());}defs.stats.insert(12,gielinor_cache::definitions::StatType::decode(&stats[&12]).unwrap());
 let r=immediate_tool_crafting(&objects,&seq,&stat,&defs).unwrap();assert_eq!(r.len(),2);
 for (r,(tool,raw,out,level,xp,anim)) in r.iter().zip([(946,771,772,31,0,10937),(1755,3211,3420,12,60,1309)]){assert_eq!((r.exchange.tool,r.exchange.input,r.exchange.output,r.level,r.xp_tenths,r.ticks,r.exchange.animation,r.exchange.members),(tool,raw,out,level,xp,0,Some(anim),true));}
 assert!(r[0].failure.is_none());let f=r[1].failure.unwrap();assert_eq!((f.output,f.xp_tenths,f.low,f.high),(968,15,137,434));
 let mut refused=0;
 for (symbol,id) in ids{for case in 0..10{let mut d=copy(&defs);let mut o=objects.clone();match case{0=>{d.items.remove(&(id as u32));},1=>d.items.get_mut(&(id as u32)).unwrap().name="Independent wrong name".into(),2=>{let v=d.items.get_mut(&(id as u32)).unwrap();v.members=!v.members;},3=>d.items.get_mut(&(id as u32)).unwrap().stackable=true,4=>d.items.get_mut(&(id as u32)).unwrap().dummy=1,5=>d.items.get_mut(&(id as u32)).unwrap().cert.template=799,6=>d.items.get_mut(&(id as u32)).unwrap().placeholder.template=14401,7=>d.items.get_mut(&(id as u32)).unwrap().transform.template=1,8=>{o.insert(symbol.into(),id+100000);},_=>{o.remove(symbol);}}assert!(immediate_tool_crafting(&o,&seq,&stat,&d).is_err(),"{symbol} case{case}");refused+=1;}}
 for case in 0..11{let mut d=copy(&defs);let mut s=stat.clone();let mut q=seq.clone();match case{0=>{s.remove("crafting");},1=>{s.insert("crafting".into(),13);},2=>{d.stats.remove(&12);},3=>d.stats.get_mut(&12).unwrap().name="Wrong".into(),4=>d.stats.get_mut(&12).unwrap().unreleased=true,5=>d.stats.get_mut(&12).unwrap().min_level=0,6=>d.stats.get_mut(&12).unwrap().max_level=98,7=>{q.remove("human_cutting_knife");},8=>{q.insert("human_cutting_knife".into(),10938);},9=>{q.remove("human_limestonecutting");},_=>{q.insert("human_limestonecutting".into(),1310);}}assert!(immediate_tool_crafting(&objects,&q,&s,&d).is_err(),"stat/effect case{case}");refused+=1;}
 assert_eq!(refused,81);assert_eq!(immediate_tool_crafting(&objects,&seq,&stat,&defs).unwrap().len(),2);println!("PG_IMMEDIATE_BINDINGS actual_items=7 actual_stat=12 exact_sequences=2 independently_refused=81");
}
