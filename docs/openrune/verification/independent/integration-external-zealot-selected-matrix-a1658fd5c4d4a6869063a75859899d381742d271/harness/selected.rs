//! Independently authored transaction matrix; target is a normal library dependency.
use gielinor_world::{prayer::{Config,Pending},prayer_preservation::{Rule,finish_selected},simulation::PlayerData,inventory::{Catalog,Item,StackType},stats::Stats,npc::RandomSource,CoordGrid};
const PIECES:[(i32,usize);4]=[(25438,0),(25434,4),(25436,7),(25440,10)];
const ASH:[i32;5]=[25766,25769,25772,25775,25778];
struct Draw{value:u32,calls:usize}impl RandomSource for Draw{fn below(&mut self,n:u32)->u32{assert_eq!(n,80);self.calls+=1;self.value}}
fn fixture(mask:u32,rate:i32,bone:i32)->(Config,PlayerData,Rule,Catalog){
 let mut stats=Stats::default();stats.set_base(5,10);stats.set_current(5,0);stats.set_fine_xp(5,11540).unwrap();
 let mut p=PlayerData::new(1,"Independent selected-slot matrix".into(),CoordGrid::new(3224,3218,0).unwrap(),stats);p.xp_rate_hundreds=rate;
 for s in 0..28{p.inventory.restore_slot(s,Some(Item{id:bone,count:1,vars:0})).unwrap();}
 for (i,&(id,slot))in PIECES.iter().enumerate(){if mask&(1<<i)!=0{p.equipment.restore_slot(slot,Some(Item{id,count:1,vars:0})).unwrap();}}
 let c=Config{skill:5,animation:827,bones:std::iter::once((526,4.5)).chain(ASH.map(|id|(id,4.5))).collect(),necklace:None,restore:Default::default(),worn_xp_percent:Default::default(),chaos:None};
 (c,p,Rule{pieces:PIECES,excluded:ASH.into()},Catalog::default())
}
#[test]fn all_subsets_intervals_selected_slots_rates_and_membership(){
 let mut cases=0;
 for mask in 0u32..16{for rate in [125,500]{for slot in [0,7,27]{for member in [false,true]{
 let (c,p,rule,items)=fixture(mask,rate,526);let count=if member{mask.count_ones()}else{0};let mut saved=0;
 for draw in 0..80{let mut next=p.clone();let mut rng=Draw{value:draw,calls:0};let done=finish_selected(&c,&mut next,&items,Pending{bone:526,restore:3},slot,Some(&rule),member,&mut rng).unwrap().unwrap();
 let keep=count>0&&draw>=80-count;let mut inv=p.inventory.clone();if !keep{inv.restore_slot(slot,None).unwrap();}assert_eq!(next.inventory,inv,"mask{mask} rate{rate} slot{slot} member{member} draw{draw}");
 let mut stats=p.stats.clone();stats.set_current(5,3);stats.set_fine_xp(5,11540+if rate==125{56}else{225}).unwrap();assert_eq!(next.stats,stats);assert!(done.restore.is_some());assert_eq!(next.equipment,p.equipment);assert_eq!(next.coords,p.coords);assert_eq!(rng.calls,usize::from(count>0));saved+=u32::from(keep);cases+=1;}
 assert_eq!(saved,count);
 }}}}assert_eq!(cases,15360);println!("PG_ZEALOT_MATRIX {cases} authoritative selected transactions; all16subsets x80draws x3slots x2rates x2membership; fullbag exactselected consumption/saving sameXPandrestore noRNGwhenzero PASS");
}
#[test]fn excluded_ashes_all_slots_and_draws_never_use_robe_random(){
 let mut cases=0;for bone in ASH{for slot in [0,7,27]{for draw in 0..80{let(c,mut p,rule,items)=fixture(15,125,bone);let before=p.clone();let mut rng=Draw{value:draw,calls:0};assert!(finish_selected(&c,&mut p,&items,Pending{bone,restore:0},slot,Some(&rule),true,&mut rng).unwrap().is_some());let mut inv=before.inventory;inv.restore_slot(slot,None).unwrap();assert_eq!(p.inventory,inv);let mut stats=before.stats;stats.set_fine_xp(5,11596).unwrap();assert_eq!(p.stats,stats);assert_eq!(p.equipment,before.equipment);assert_eq!(rng.calls,0);cases+=1;}}}assert_eq!(cases,1200);println!("PG_ZEALOT_ASH {cases} excludedash selected transactions sameXP noRandom PASS");
}
#[test]fn atomic_failure_precedes_draw_and_does_not_substitute_another_bone(){
 let mut cases=0;for slot in [0,7,27]{for bad in 0..14{
 let(mut c,mut p,rule,mut items)=fixture(15,125,526);let mut pending=Pending{bone:526,restore:3};
 match bad{0=>p.inventory.restore_slot(slot,None).unwrap(),1=>p.inventory.restore_slot(slot,Some(Item{id:532,count:1,vars:0})).unwrap(),2=>p.inventory.restore_slot(slot,Some(Item{id:526,count:2,vars:0})).unwrap(),3=>p.inventory.restore_slot(slot,Some(Item{id:526,count:1,vars:8})).unwrap(),4=>{items.dummy.insert(526);},5=>{items.stackable.insert(526);},6=>p.inventory.stack=StackType::Always,7=>p.inventory.placeholders=true,8=>{c.bones.insert(526,f64::NAN);},9=>{c.bones.insert(526,-1.0);},10=>p.xp_rate_hundreds=0,11=>pending.restore=-1,12=>p.stats.set_current(5,11),_=>{c.bones.remove(&526);}}
 let before=p.clone();let mut rng=Draw{value:79,calls:0};let out=finish_selected(&c,&mut p,&items,pending,slot,Some(&rule),true,&mut rng);assert!(!matches!(out,Ok(Some(_))),"bad{bad} slot{slot}");assert_eq!(p.inventory,before.inventory);assert_eq!(p.stats,before.stats);assert_eq!(p.equipment,before.equipment);assert_eq!(rng.calls,0);cases+=1;
 }}assert_eq!(cases,42);println!("PG_ZEALOT_REFUSAL {cases} selectedmissing/changed/count/vars/catalog/bag/XP/restore atomicallyrefuse beforeRNG PASS");
}
