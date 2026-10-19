//! Real native239 item definitions and independently mutated last-stage/other-stage copies.
use gielinor_cache::{definitions::ItemType,game::{Definitions,GameCache},symbols};
use gielinor_server::content::{giant_crab_food,native_consumables};
use gielinor_world::{consumable_clocks::Kind,food::Edible};
use std::{collections::BTreeMap,path::Path};
const IDS:[i32;5]=[7521,7523,7524,7525,7526];
#[test]
fn real_cache_configured_world_two_tick_stages_stale_identity_and_walk(){
 use gielinor_world::{simulation::{World,PlayerData,Action,WorldEvent},npc::RandomSource,inventory::Item,stats::Stats,CoordGrid};
 struct Fixed;impl RandomSource for Fixed{fn below(&mut self,_:u32)->u32{panic!("fixed crab used RNG")}}
 let names=std::env::var("PG_SYMBOLS").unwrap();let names=symbols::load(&[Path::new(&names)],"obj",None).unwrap();let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();
 let mut d=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};for id in IDS{d.items.insert(id as u32,ItemType::decode(&raw[&(id as u32)]).unwrap());}
 let mut foods=BTreeMap::new();let selected=giant_crab_food(&names,&d,&mut foods).unwrap();let rules=native_consumables(&names,&d,&foods,&BTreeMap::new()).unwrap();let mut count=0;
 for slot in [0,13,27]{for hp in [50i8,98,99,115]{for attack in [0,20]{
  let mut w=World::default();w.collision.allocate(3224,3216,0);w.foods=foods.clone();w.member_foods=selected.clone();w.slot_preserving_foods=selected.clone();w.consumable_rules=Some(rules.clone());w.eating_animation=829;
  let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,hp);stats.set_fine_xp(3,130344311).unwrap();let mut data=PlayerData::new(1,"Independent crab World".into(),CoordGrid::new(3224,3218,0).unwrap(),stats);
  for s in 0..28{data.inventory.restore_slot(s,Some(Item{id:if s==slot||s==(slot+1)%28{IDS[0]}else{1931},count:1,vars:0})).unwrap();}
  w.queue_login(1,data).unwrap();w.tick_with_random(&mut Fixed).unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.set_next_attack_delay(a,attack).unwrap();w.drain_events();
  for (i,&id)in IDS.iter().enumerate(){
   w.submit(a,Action::EatItemSlot{slot,item:id}).unwrap();w.tick_with_random(&mut Fixed).unwrap();let rows:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::FoodTransition{trace,..}=e{Some(trace)}else{None}).collect();assert_eq!(rows.len(),1);assert_eq!(rows[0].attack_delay,Some(if attack>0{22}else{0}));
   let p=w.data(a).unwrap();assert_eq!(p.inventory.get(slot).map(|v|v.id),IDS.get(i+1).copied());assert_eq!(p.inventory.get((slot+1)%28).unwrap().id,IDS[0]);assert_eq!(p.stats.current(3),if hp<99{(hp as i32+2*(i as i32+1)).min(99)as i8}else{hp});assert_eq!(p.stats.fine_xp(3),130344311);
   if i<4{
    let inventory=p.inventory.clone();let stats=p.stats.clone();w.submit(a,Action::EatItemSlot{slot,item:id}).unwrap();w.submit(a,Action::EatItemSlot{slot,item:IDS[i+1]}).unwrap();let x=if i%2==0{3225}else{3224};w.submit(a,Action::Walk{destination:CoordGrid::new(x,3218,0).unwrap(),run:false}).unwrap();w.tick_with_random(&mut Fixed).unwrap();assert_eq!(w.data(a).unwrap().coords.x(),x);assert_eq!(w.data(a).unwrap().inventory.slots(),inventory.slots());assert_eq!(w.data(a).unwrap().stats,stats);assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::FoodTransition{..})));
   }
   count+=1;
  }
 }}}
 assert_eq!(count,120);println!("PG_CRAB_CACHE_WORLD120 realfactory5stage fullbag/capoverheal selectedduplicate active/readyattack22or0 trace;192stale/cooldown refusals and96realWalksPASS");
}
#[test]
fn actual_five_stage_factory_and_atomic_invalid_catalog(){
 let names=std::env::var("PG_SYMBOLS").unwrap();let symbols=symbols::load(&[Path::new(&names)],"obj",None).unwrap();let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();
 let mut d=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
 for id in IDS{d.items.insert(id as u32,ItemType::decode(&raw[&(id as u32)]).unwrap());}
 let mut f=BTreeMap::new();let selected=giant_crab_food(&symbols,&d,&mut f).unwrap();assert_eq!(selected.into_iter().collect::<Vec<_>>(),IDS);
 let rules=native_consumables(&symbols,&d,&f,&BTreeMap::new()).unwrap();assert_eq!(rules.foods.len(),5);assert!(rules.potions.is_empty());
 for (index,id)in IDS.into_iter().enumerate(){let food=&f[&id];assert_eq!((food.id,food.heal,food.delay,food.replacement),(id,2,2,IDS.get(index+1).copied()));let p=rules.foods[&id];assert_eq!(p.kind,Kind::Food);assert_eq!((p.consume_delay,p.combat_delay,p.members),(2,2,true));}
 let mut count=0;
 for (index,id)in IDS.into_iter().enumerate(){for mode in 0..11{
  let old=d.items[&(id as u32)].clone();let mut symbols=symbols.clone();let symbol=format!("hundred_pirate_giant_crab_meat_{}",5-index);let mut out=BTreeMap::new();out.insert(1931,Edible{id:1931,heal:1,delay:3,replacement:None,message:"Sentinel".into()});
  match mode{0=>{symbols.remove(&symbol);},1=>{symbols.insert(symbol,id+100);},2=>{d.items.remove(&(id as u32));},3=>d.items.get_mut(&(id as u32)).unwrap().name="Wrong crab".into(),4=>d.items.get_mut(&(id as u32)).unwrap().members=false,5=>d.items.get_mut(&(id as u32)).unwrap().stackable=true,6=>d.items.get_mut(&(id as u32)).unwrap().dummy=1,7=>d.items.get_mut(&(id as u32)).unwrap().cert.template=1,8=>d.items.get_mut(&(id as u32)).unwrap().placeholder.template=1,9=>d.items.get_mut(&(id as u32)).unwrap().inventory_ops[0]=Some("Drop".into()),_=>{out.insert(id,f[&id].clone());}}
  let before=format!("{out:?}");assert!(giant_crab_food(&symbols,&d,&mut out).is_err(),"index={index} mode={mode}");assert_eq!(format!("{out:?}"),before);d.items.insert(id as u32,old);count+=1;
 }}
 assert_eq!(count,55);println!("PG_CRAB_NATIVE_BINDINGS5 exactselectedstage/HP2/consume2/combat2 FOODmember;55atomicallstagefaults includinglaststage preserve existingcatalog PASS");
}
