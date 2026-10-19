//! Native239 baseline food catalog clock classification; potion map intentionally empty.
use gielinor_cache::{definitions::ItemType,game::{Definitions,GameCache},symbols};
use gielinor_server::content::{foods,native_consumables};
use gielinor_world::consumable_clocks::Kind;
use std::{collections::BTreeMap,path::Path};
#[test]
fn actual_baseline_food_factory_classifies_king_worm_without_phantom_candidates(){
 let path=std::env::var("PG_GAME_CACHE").unwrap();let names=std::env::var("PG_SYMBOLS").unwrap();let symbols=symbols::load(&[Path::new(&names)],"obj",None).unwrap();
 let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let raw=cache.files(2,10,None).unwrap().unwrap();
 let mut d=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
 for (&id,bytes)in &raw{d.items.insert(id,ItemType::decode(bytes).unwrap());}
 let f=foods(&symbols,&d).unwrap();let r=native_consumables(&symbols,&d,&f,&BTreeMap::new()).unwrap();assert_eq!(f.len(),r.foods.len());assert!(r.potions.is_empty());
 assert_eq!(symbols.get("king_worm"),Some(&2162));let k=r.foods[&2162];assert_eq!(k.kind,Kind::ComboFood);assert_eq!((k.consume_delay,k.combat_delay,k.members),(3,2,true));assert_eq!(f[&2162].heal,2);
 for (&id,p)in &r.foods{assert_eq!(p.members,d.items[&(id as u32)].members);assert_eq!(p.consume_delay,f[&id].delay);if id!=2162{assert_eq!(p.kind,Kind::Food);assert_eq!(p.combat_delay,3);}}
 for absent in [3144,7521,7523,7524,7525,7526,2152,2217,2243]{assert!(!f.contains_key(&absent));assert!(!r.foods.contains_key(&absent));}
 println!("PG_NATIVE_BASELINE_FOOD_BINDINGS count={} combo2162only allactualmemberflags/durations;no phantom crab/gnome/karambwan/toad registration PASS",f.len());
 let mut count=0;
 for mode in 0..8{
  let mut f=f.clone();let old=d.items[&2162].clone();let mut symbols=symbols.clone();
  match mode{0=>{symbols.insert("king_worm".into(),2163);},1=>d.items.get_mut(&2162).unwrap().name="Wrong worm".into(),2=>d.items.get_mut(&2162).unwrap().members=false,3=>d.items.get_mut(&2162).unwrap().stackable=true,4=>d.items.get_mut(&2162).unwrap().inventory_ops[0]=None,5=>f.get_mut(&2162).unwrap().delay=2,6=>f.get_mut(&2162).unwrap().id=2163,_=>d.items.get_mut(&2162).unwrap().dummy=1}
  assert!(native_consumables(&symbols,&d,&f,&BTreeMap::new()).is_err(),"mode={mode}");d.items.insert(2162,old);count+=1;
 }
 assert_eq!(count,8);println!("PG_NATIVE_BASELINE_FOOD_REFUSALS8 cachealias/name/member/stack/operation/delay/id/dummy atomicfactory refusal PASS");
}
