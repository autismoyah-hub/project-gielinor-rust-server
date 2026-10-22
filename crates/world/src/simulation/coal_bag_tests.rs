use super::*;
use crate::{coal_bag::Config,inventory::Item,stats::Stats,variables::{Varp,Varbit,Transmit}};
fn fixture(stored:i32,coal:usize,cape:Option<i32>)->(World,ActorId){
    let mut w=World::default();let config=Config{closed:12019,open:24480,coal:453,capes:[9795,9796,13280],base:Varp{id:65531,permanent:true,transmit:Transmit::Never,bit_protect:true,client_code:-1},stored:Varbit{base:65531,lsb:25,msb:30}};w.coal_bag=Some(config);w.object_types.stackable.extend([12019,24480]);w.collision.allocate(3200,3200,0);
    let mut p=PlayerData::new(1,"Coal bag".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());p.inventory.restore_slot(27,Some(Item{id:12019,count:1,vars:0})).unwrap();for slot in 0..coal{p.inventory.restore_slot(slot,Some(Item{id:453,count:1,vars:0})).unwrap();}if let Some(id)=cape{p.equipment.restore_slot(1,Some(Item{id,count:1,vars:0})).unwrap();}p.variables.set(config.base,0x81000001u32 as i32,false);p.variables.set_bit(config.base,config.stored,stored,false).unwrap();w.queue_login(1,p).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,true).unwrap();w.drain_events();(w,a)
}
fn stored(w:&World,a:ActorId)->i32{w.data(a).unwrap().variables.bit(w.coal_bag.unwrap().stored).unwrap()}
fn coal(w:&World,a:ActorId)->usize{w.data(a).unwrap().inventory.slots().iter().flatten().filter(|v|v.id==453).count()}
fn action(w:&mut World,a:ActorId,item:i32,op:u8){w.submit(a,Action::CoalBagItem{slot:27,item,operation:op}).unwrap();w.tick().unwrap();}
#[test]
fn coal_bag_actual_actions_fill_check_toggle_empty_conserve_items_and_packed_neighbors(){
    for cape in[None,Some(9795),Some(9796),Some(13280)]{for initial in[0,26,27,35,36]{let(worn_capacity,available)=(if cape.is_some(){36}else{27},27);let(mut w,a)=fixture(initial,available,cape);let before=w.data(a).unwrap().stats.clone();let total=initial+available as i32;
        action(&mut w,a,12019,2);assert_eq!(stored(&w,a),initial+(worn_capacity-initial).max(0).min(available as i32));assert_eq!(stored(&w,a)+coal(&w,a)as i32,total);
        action(&mut w,a,12019,4);action(&mut w,a,12019,3);assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id,24480);action(&mut w,a,24480,6);assert_eq!(stored(&w,a)+coal(&w,a)as i32,total);action(&mut w,a,24480,3);assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().id,12019);
        assert_eq!(w.data(a).unwrap().variables.get(65531)as u32&!(0x3fu32<<25),0x81000001);assert_eq!(w.data(a).unwrap().stats,before);
    }}
}
#[test]
fn coal_bag_no_coal_full_bag_partial_empty_and_cape_removal_keep_stored_coal(){
    let(mut w,a)=fixture(36,0,Some(9795));for slot in 0..27{w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:1513,count:1,vars:0})).unwrap();}w.data_mut(a).unwrap().equipment.restore_slot(1,None).unwrap();action(&mut w,a,12019,6);assert_eq!(stored(&w,a),36);assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="You don't have any free space in your inventory.")));
    for slot in[4,12]{w.data_mut(a).unwrap().inventory.restore_slot(slot,None).unwrap();}action(&mut w,a,12019,6);assert_eq!(stored(&w,a),34);assert_eq!(coal(&w,a),2);action(&mut w,a,12019,2);assert_eq!(stored(&w,a),34);assert_eq!(coal(&w,a),2);
    let(mut w,a)=fixture(0,0,None);action(&mut w,a,12019,2);assert_eq!(stored(&w,a),0);assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="You don't have any coal to fill your coal bag with.")));
}
#[test]
fn coal_bag_current_slot_life_member_modal_delay_and_operation_guards_do_not_transfer(){for case in 0..10{let(mut w,a)=fixture(5,5,None);match case{0=>w.set_membership(a,false).unwrap(),1=>w.data_mut(a).unwrap().stats.set_current(3,0),2=>w.players.get_mut(a.slot as usize).unwrap().death_queued=true,3=>w.players.get_mut(a.slot as usize).unwrap().has_modal=true,4=>w.players.get_mut(a.slot as usize).unwrap().protection.delay(2),5=>w.players.get_mut(a.slot as usize).unwrap().saving=true,6=>w.players.get_mut(a.slot as usize).unwrap().pending_logout=true,7=>{w.data_mut(a).unwrap().inventory.restore_slot(27,None).unwrap();},_=>()};let inv=w.data(a).unwrap().inventory.clone();let vars=w.data(a).unwrap().variables.get(65531);let result=w.coal_bag_item(a,27,if case==8{24480}else{12019},if case==9{5}else{2});assert!(result.is_err(),"case{case}");assert_eq!(w.data(a).unwrap().inventory,inv);assert_eq!(w.data(a).unwrap().variables.get(65531),vars);}}
#[test]
fn coal_bag_saved_variable_round_trip_and_open_form_do_not_duplicate_contents(){let(mut w,a)=fixture(0,12,None);action(&mut w,a,12019,2);action(&mut w,a,12019,3);let snapshot=w.data(a).unwrap().clone();let mut restored=PlayerData::new(1,"Restored coal bag".into(),snapshot.coords,snapshot.stats.clone());restored.inventory=snapshot.inventory;restored.variables=crate::variables::Variables::restore(snapshot.variables.persistent(&[(65531,w.coal_bag.unwrap().base)].into()));let mut next=World::default();next.coal_bag=w.coal_bag;next.object_types.stackable.extend([12019,24480]);next.collision.allocate(3200,3200,0);next.queue_login(2,restored).unwrap();next.tick().unwrap();let b=next.actors().next().unwrap();next.set_membership(b,true).unwrap();assert_eq!(stored(&next,b),12);action(&mut next,b,24480,6);action(&mut next,b,24480,6);assert_eq!(stored(&next,b),0);assert_eq!(coal(&next,b),12);}
#[test]
fn coal_bag_carried_cape_noted_coal_and_duplicate_stack_do_not_expand_authority(){
    let(mut w,a)=fixture(26,0,None);w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:9795,count:1,vars:0})).unwrap();w.data_mut(a).unwrap().inventory.restore_slot(1,Some(Item{id:454,count:100,vars:0})).unwrap();w.data_mut(a).unwrap().stats.set_current(3,128u8 as i8);action(&mut w,a,12019,2);assert_eq!(stored(&w,a),26);assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="You don't have any coal to fill your coal bag with.")));
    for slot in[2,3]{w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:453,count:1,vars:0})).unwrap();}action(&mut w,a,12019,2);assert_eq!(stored(&w,a),27);assert_eq!(coal(&w,a),1);assert_eq!(w.data(a).unwrap().inventory.get(1).unwrap().count,100);
    w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:12019,count:2,vars:0})).unwrap();assert!(w.coal_bag_item(a,27,12019,3).is_err());assert_eq!(w.data(a).unwrap().inventory.get(27).unwrap().count,2);assert_eq!(stored(&w,a),27);
}
