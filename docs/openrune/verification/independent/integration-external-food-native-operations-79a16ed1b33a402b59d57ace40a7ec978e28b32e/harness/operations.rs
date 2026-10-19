//! Exact152 ordinary food operation mapping over typed in-process Button values.
use gielinor_cache::{definitions::ItemType,game::{Definitions,GameCache}};
use gielinor_protocol::buttons::Button;
use gielinor_world::{food::Edible,simulation::Action};
use std::{collections::BTreeMap,path::Path};
#[test]
fn actual_cache_first_and_fourth_eat_and_preserved_claim_matrix(){
 let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();
 let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
 for id in [2128,2152,1931]{defs.items.insert(id,ItemType::decode(&raw[&id]).unwrap());}
 assert_eq!(defs.items[&2128].inventory_ops[0].as_deref(),Some("Eat"));assert_eq!(defs.items[&2152].inventory_ops[3].as_deref(),Some("Eat"));
 assert_eq!(pg_food_boundary::operation(&defs.items[&2128]),Some(2));assert_eq!(pg_food_boundary::operation(&defs.items[&2152]),Some(5));
 let mut foods=BTreeMap::new();for(id,heal)in[(2128,1),(2152,3)]{foods.insert(id,Edible{id,heal,delay:3,replacement:None,message:"Food".into()});}
 let expected=|id,op|id==2128&&op==2||id==2152&&op==5;let mut count=0;
 for slot in 0..28{for current in [2128,2152,1931]{for claimed in [2128,2152,1931]{for op in 0..=6{for native in [false,true]{
  let b=Button{component:149<<16,slot:slot as i32,object:claimed,op};
  let routed=pg_food_boundary::route(&b,slot,current,&foods,&defs,native);
  let allowed=if native{expected(current,op)||expected(claimed,op)}else{op==2&&(current!=1931||claimed!=1931)};
  assert_eq!(routed.is_some(),allowed,"slot={slot} current={current} claimed={claimed} op={op} native={native}");
  if let Some(Action::EatItemSlot{slot:actual,item})=routed{assert_eq!((actual,item),(slot,claimed));}else{assert!(!allowed);}
  count+=1;
 }}}}}
 assert_eq!(count,3528);println!("PG_FOOD_NATIVE_OP3528 realcacheEquaEat1=>2/ToadsEat4=>5;legacy2/nativeexactoperation;all28slots current/claimedidentities preserved PASS");
 for slot in 0..28{for wrong in [-1,28,999]{let b=Button{component:149<<16,slot:wrong,object:2152,op:5};assert!(pg_food_boundary::route(&b,slot,2152,&foods,&defs,true).is_none());}}
 println!("PG_FOOD_NATIVE_WRONG_SLOT84 typedslotmismatch refuses PASS");
 // Independent multi-option profiles prove first eligible option wins; option5 excluded.
 for label in ["Eat","Drink"]{for index in 0..5{let mut d=ItemType::decode(&[0]).unwrap();d.inventory_ops[index]=Some(label.into());assert_eq!(pg_food_boundary::operation(&d),if index<4{Some(index as u8+2)}else{None});for later in index+1..5{d.inventory_ops[later]=Some("Eat".into());assert_eq!(pg_food_boundary::operation(&d),if index<4{Some(index as u8+2)}else{None});}}}
}
