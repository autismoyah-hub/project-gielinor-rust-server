//! Independently specified contract149 stage/slot/HP oracles, not developer tests.
use gielinor_world::{food::{Edible,eat_slot},inventory::{Catalog,Inventory,Item,StackType},stats::Stats};
const IDS:[i32;5]=[7521,7523,7524,7525,7526];
fn food(i:usize)->Edible{Edible{id:IDS[i],heal:2,delay:2,replacement:IDS.get(i+1).copied(),message:"Crab".into()}}
#[test]
fn every_stage_every_selected_slot_caps_and_preserves_other_copy(){
 let mut count=0;
 for index in 0..5{for slot in 0..28{for hp in [50i8,98,99,115]{
  let mut inv=Inventory::new(28,StackType::Normal,false);let duplicate=(slot+1)%28;
  for s in 0..28{inv.restore_slot(s,Some(Item{id:if s==slot||s==duplicate{IDS[index]}else{1931},count:1,vars:0})).unwrap();}
  let before=inv.clone();let mut stats=Stats::default();for s in 0..23{stats.set_base(s,99);stats.set_current(s,99);stats.set_fine_xp(s,130344311).unwrap();}stats.set_current(3,hp);let old=stats.clone();let mut events=Vec::new();
  assert!(eat_slot(&food(index),slot,&mut inv,&mut stats,&Catalog::default(),3,829,|e|events.push(e)).unwrap());
  let mut expected=before;expected.restore_slot(slot,IDS.get(index+1).map(|&id|Item{id,count:1,vars:0})).unwrap();assert_eq!(inv.slots(),expected.slots());
  let mut expected=old;expected.set_current(3,if hp<99{(hp+2).min(99)}else{hp});assert_eq!(stats,expected);assert_eq!(inv.get(duplicate).unwrap().id,IDS[index]);
  assert_eq!(events.iter().filter(|e|matches!(e,gielinor_world::food::Event::Delay(2))).count(),1);count+=1;
 }}}
 assert_eq!(count,560);println!("PG_CRAB_SELECTED560 all5stages/all28slots/HP50,98,99,115 exactsingledebit/remainder/finalnone noXPotherstats fullbagduplicate PASS");
}
#[test]
fn five_bite_sequences_preserve_slot_and_total_ten_hp(){
 let mut count=0;
 for slot in 0..28{for hp in [50i8,98,99,115]{
  let mut inv=Inventory::new(28,StackType::Normal,false);for s in 0..28{inv.restore_slot(s,Some(Item{id:if s==slot||s==(slot+1)%28{IDS[0]}else{1931},count:1,vars:0})).unwrap();}
  let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,hp);
  for index in 0..5{assert_eq!(inv.get(slot).unwrap().id,IDS[index]);assert!(eat_slot(&food(index),slot,&mut inv,&mut stats,&Catalog::default(),3,829,|_|{}).unwrap());assert_eq!(stats.current(3),if hp<99{(hp as i32+2*(index as i32+1)).min(99)as i8}else{hp});count+=1;}
  assert!(inv.get(slot).is_none());assert_eq!(inv.get((slot+1)%28).unwrap().id,IDS[0]);
 }}
 assert_eq!(count,560);println!("PG_CRAB_SEQUENCES112 fullfivebitechains560transactions no inventedcontainer/tenHP belowcap PASS");
}
