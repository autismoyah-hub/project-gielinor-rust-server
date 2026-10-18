//! Independently loaded cache definitions; no synthetic positive metadata.
use gielinor_cache::{game::{Definitions,GameCache},symbols};
use gielinor_server::content::fallible_gems;
use std::{collections::BTreeMap,path::Path};
fn copy(d:&Definitions)->Definitions{Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:d.items.clone(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:d.stats.clone()}}
#[test]
fn actual_immediate_bindings_and_ninety_three_fail_closed_mutations(){
 let dir=std::env::var("PG_SYMBOLS").unwrap();let objects=symbols::load(&[Path::new(&dir)],"obj",None).unwrap();let seq=symbols::load(&[Path::new(&dir)],"seq",None).unwrap();let stat=symbols::load(&[Path::new(&dir)],"stat",None).unwrap();
 let cache=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&cache),0xff_ffff).unwrap();let items=cache.files(2,10,None).unwrap().unwrap();let stats=cache.files(2,200,None).unwrap().unwrap();
 let ids=[("chisel",1755),("crushed_gemstone",1633),("uncut_opal",1625),("opal",1609),("uncut_jade",1627),("jade",1611),("uncut_red_topaz",1629),("red_topaz",1613)];
 let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
 for (_,id) in ids{defs.items.insert(id as u32,gielinor_cache::definitions::ItemType::decode(&items[&(id as u32)]).unwrap());}defs.stats.insert(12,gielinor_cache::definitions::StatType::decode(&stats[&12]).unwrap());
 let r=fallible_gems(&objects,&seq,&stat,&defs).unwrap();assert_eq!(r.len(),3);
 for (r,(raw,out,level,xp,fail,low,high,anim)) in r.iter().zip([(1625,1609,1,150,38,128,250,890),(1627,1611,13,200,50,100,245,891),(1629,1613,16,250,63,90,240,892)]){assert_eq!((r.exchange.tool,r.exchange.input,r.exchange.output,r.level,r.xp_tenths,r.ticks,r.exchange.animation,r.exchange.members),(1755,raw,out,level,xp,1,Some(anim),true));let f=r.failure.unwrap();assert_eq!((f.output,f.xp_tenths,f.low,f.high),(1633,fail,low,high));}
 let mut refused=0;
 for (symbol,id) in ids{for case in 0..10{let mut d=copy(&defs);let mut o=objects.clone();match case{0=>{d.items.remove(&(id as u32));},1=>d.items.get_mut(&(id as u32)).unwrap().name="Independent wrong name".into(),2=>{let v=d.items.get_mut(&(id as u32)).unwrap();v.members=!v.members;},3=>d.items.get_mut(&(id as u32)).unwrap().stackable=true,4=>d.items.get_mut(&(id as u32)).unwrap().dummy=1,5=>d.items.get_mut(&(id as u32)).unwrap().cert.template=799,6=>d.items.get_mut(&(id as u32)).unwrap().placeholder.template=14401,7=>d.items.get_mut(&(id as u32)).unwrap().transform.template=1,8=>{o.insert(symbol.into(),id+100000);},_=>{o.remove(symbol);}}assert!(fallible_gems(&o,&seq,&stat,&d).is_err(),"{symbol} case{case}");refused+=1;}}
 for case in 0..7{let mut d=copy(&defs);let mut s=stat.clone();let mut q=seq.clone();match case{0=>{s.remove("crafting");},1=>{s.insert("crafting".into(),13);},2=>{d.stats.remove(&12);},3=>d.stats.get_mut(&12).unwrap().name="Wrong".into(),4=>d.stats.get_mut(&12).unwrap().unreleased=true,5=>d.stats.get_mut(&12).unwrap().min_level=0,6=>d.stats.get_mut(&12).unwrap().max_level=98,_=>unreachable!()}assert!(fallible_gems(&objects,&q,&s,&d).is_err(),"stat/effect case{case}");refused+=1;}
 for (symbol,id) in [("human_opalcutting",890),("human_jadecutting",891),("human_redtopazcutting",892)]{for alias in [false,true]{let mut q=seq.clone();if alias{q.insert(symbol.into(),id+1);}else{q.remove(symbol);}assert!(fallible_gems(&objects,&q,&stat,&defs).is_err(),"accepted {symbol} alias{alias}");refused+=1;}}
 assert_eq!(refused,93);assert_eq!(fallible_gems(&objects,&seq,&stat,&defs).unwrap().len(),3);println!("PG_FALLIBLE_GEM_BINDINGS actual_items=8 actual_stat=12 exact_sequences=3 independently_refused=93");
}
