//! Actual152 ordinary Eat factories; no ingredient-use recipe or potion action.
use gielinor_cache::{definitions::ItemType,game::{Definitions,GameCache},symbols};
use gielinor_server::content::{gnome_ingredients,native_consumables};
use gielinor_world::{consumable_clocks::Kind,food::Edible};
use std::{collections::BTreeMap,path::Path};
#[test]
fn actual_native_first_and_fourth_eat_bind_atomically_with_selected_clocks(){
 let path=std::env::var("PG_GAME_CACHE").unwrap();let names=std::env::var("PG_SYMBOLS").unwrap();let symbols=symbols::load(&[Path::new(&names)],"obj",None).unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();
 let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};for id in[2128,2152]{defs.items.insert(id,ItemType::decode(&raw[&id]).unwrap());}
 let mut foods=BTreeMap::new();let selected=gnome_ingredients(&symbols,&defs,&mut foods).unwrap();assert_eq!(selected.into_iter().collect::<Vec<_>>(),[2128,2152]);let rules=native_consumables(&symbols,&defs,&foods,&BTreeMap::new()).unwrap();
 for(id,hp,kind,combat)in[(2128,1,Kind::Food,3),(2152,3,Kind::ComboFood,2)]{let f=&foods[&id];assert_eq!((f.id,f.heal,f.delay,f.replacement),(id,hp,3,None));let p=rules.foods[&id];assert_eq!((p.kind,p.consume_delay,p.combat_delay,p.members),(kind,3,combat,true));}assert!(rules.potions.is_empty());
 let mut count=0;
 for(id,symbol,option)in[(2128u32,"equa_leaves",0),(2152,"toads_legs",3)]{for mode in 0..13{
  let old=defs.items[&id].clone();let mut symbols=symbols.clone();let mut out=BTreeMap::new();out.insert(1931,Edible{id:1931,heal:1,delay:3,replacement:None,message:"Sentinel".into()});
  match mode{0=>{symbols.remove(symbol);},1=>{symbols.insert(symbol.into(),id as i32+1);},2=>{defs.items.remove(&id);},3=>defs.items.get_mut(&id).unwrap().name="Wrong food".into(),4=>defs.items.get_mut(&id).unwrap().members=false,5=>defs.items.get_mut(&id).unwrap().stackable=true,6=>defs.items.get_mut(&id).unwrap().dummy=1,7=>defs.items.get_mut(&id).unwrap().cert.template=1,8=>defs.items.get_mut(&id).unwrap().placeholder.template=1,9=>defs.items.get_mut(&id).unwrap().transform.template=1,10=>defs.items.get_mut(&id).unwrap().inventory_ops[option]=None,11=>{let d=defs.items.get_mut(&id).unwrap();if id==2152{d.inventory_ops[0]=Some("Eat".into());}else{d.inventory_ops[0]=Some("Use".into());d.inventory_ops[1]=Some("Eat".into());}},_=>{out.insert(id as i32,foods[&(id as i32)].clone());}}
  let before=format!("{out:?}");assert!(gnome_ingredients(&symbols,&defs,&mut out).is_err(),"id={id} mode={mode}");assert_eq!(format!("{out:?}"),before);defs.items.insert(id,old);count+=1;
 }}
 assert_eq!(count,26);println!("PG_FOOD_GNOME_NATIVE_BINDINGS2 realcacheEatfirst/fourth ordinaryselectedheal1/3 FOOD3/3andCOMBO3/2member;26atomicfaults includingwrongearlierconsumeoption PASS");
}
