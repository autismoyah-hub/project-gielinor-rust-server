//! Typed in-process adapter calls; no network or hand-built wire payloads.
use gielinor_protocol::buttons::Button;
use gielinor_world::{worn_stat_boost::Rule,simulation::Action};
#[test]fn mapped_cape_component_claims_do_not_alias_remove_or_teleport(){
 let rules=[9780,9781].into_iter().map(|id|(id,Rule{skill:12,slot:1,button_operation:2,members:true})).collect();let mut n=0;
 for cape in [9780,9781]{for object in [-1,cape,if cape==9780{9781}else{9780},-2,995]{for claimed_slot in [-1,0,1,27,28]{for operation in [0,1,2,3,4,5,6,10]{for actual_slot in [0,1,4,14]{
  let button=Button{component:(387<<16)|16,slot:claimed_slot,object,op:operation};let expected=claimed_slot== -1&&(object== -1||object==cape)&&operation==2&&actual_slot==1;
  let routed=pg_cape_boundary::route(&button,actual_slot,cape,&rules);assert_eq!(routed.is_some(),expected,"cape{cape} object{object} claim{claimed_slot} op{operation} actualslot{actual_slot}");
  if let Some(action)=routed{assert!(matches!(action,Action::BoostWornItem{slot:1,item}if item==cape));}n+=1;
 }}}}}
 assert_eq!(n,1600);
 for item in [9782,995,-1,24704]{for op in [1,2,3]{let button=Button{component:(387<<16)|16,slot:-1,object:-1,op};assert!(pg_cape_boundary::route(&button,1,item,&rules).is_none());n+=1;}}
 assert_eq!(n,1612);println!("PG_CAPE_BUTTONS1612 pinned mapped-component adapter actualtypedButtons:bothcapes,currentexplicit-or-1object,canonical-1inventoryslot,actualwearslot1,op2only;Remove1Teleport3neverBoost;12unregisteredcurrentitemcontrols PASS");
}
