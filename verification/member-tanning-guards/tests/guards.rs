//! Independent paid service, reach/event/World quantity flow and life boundaries.
use gielinor_world::{inventory::Item,npc::{Npc,Restrict,BlockWalk},npc_interaction::{Handler,Event},tanning::{Recipe,Request},simulation::{World,PlayerData,ActorId,WorldEvent,Action},stats::Stats,CoordGrid};
fn fixture(kind:u16,coins:i32,member:bool,row:usize)->(World,ActorId,gielinor_world::simulation::NpcId,Request){let mut w=World::default();w.collision.allocate(3200,3200,0);w.object_types.stackable.insert(995);w.tanning.insert(kind,recipes());w.npc_handlers.insert((kind,3),Handler::Tanning);let mut options=gielinor_world::npc_options::Definition::default();options.operations[2]=Some("Trade".into());w.npc_options.types.insert(kind,options);let target=w.spawn_npc(Npc::new(kind,CoordGrid::new(3201,3200,0).unwrap(),1,Restrict::NoMove,BlockWalk::None).unwrap()).unwrap();let mut stats=Stats::default();stats.set_base(3,10);stats.set_current(3,10);stats.set_current(12,0);let mut data=PlayerData::new(1,"Independent paid tanning".into(),CoordGrid::new(3200,3200,0).unwrap(),stats);for slot in 0..27{data.inventory.restore_slot(slot,Some(Item{id:ROWS[row].0,count:1,vars:0})).unwrap();}data.inventory.restore_slot(27,Some(Item{id:if coins>0{995}else{1931},count:if coins>0{coins}else{1},vars:0})).unwrap();w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_membership(a,member).unwrap();w.drain_events();w.submit(a,Action::OperateNpc{target,operation:3,control:false}).unwrap();w.tick().unwrap();assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::NpcInteraction{actor,event:Event::Tanning(n)} if *actor==a && *n==target)));let req=w.tanning_request(a,target).unwrap();assert_eq!(req.recipes().collect::<Vec<_>>(),if member{recipes()}else{recipes()[..2].to_vec()});w.data_mut(a).unwrap().inventory.clear_modified();(w,a,target,req)}

const ROWS:[(i32,i32,i32);8]=[(1739,1741,1),(1739,1743,3),(7801,6289,20),(6287,6289,15),(1753,1745,20),(1751,2505,20),(1749,2507,20),(1747,2509,20)];
fn recipes()->Vec<Recipe>{ROWS.iter().enumerate().map(|(i,&(input,output,cost))|Recipe{input,output,cost,members:i>=2}).collect()}

#[test]fn one_hundred_forty_four_atomic_refusals(){let mut n=0;for kind in[3231,5809]{for row in 2..8{for case in 0..12{let(mut w,a,_,req)=fixture(kind,1000,true,row);let token=w.start_quantity_dialog(a).unwrap();let(input,output,_)=ROWS[row];match case{
0=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:input,count:1,vars:1})).unwrap();},
1=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:1931,count:1,vars:0})).unwrap();},
2=>{w.data_mut(a).unwrap().inventory.restore_slot(0,Some(Item{id:input,count:2,vars:0})).unwrap();},
3=>{w.data_mut(a).unwrap().inventory.restore_slot(27,Some(Item{id:995,count:1000,vars:1})).unwrap();},
4=>{w.data_mut(a).unwrap().inventory.restore_slot(26,Some(Item{id:995,count:10,vars:0})).unwrap();},
5=>{w.object_types.stackable.insert(output);},
6=>{w.object_types.dummy.insert(output);},
7=>{w.object_types.stackable.insert(input);},
8=>{w.object_types.stackable.remove(&995);},
9=>{w.object_types.dummy.insert(995);},
10=>{w.tanning.get_mut(&kind).unwrap()[row].cost+=1;},
_=>{w.set_membership(a,false).unwrap();}}
w.data_mut(a).unwrap().inventory.clear_modified();let before=w.data(a).unwrap().clone();assert!(w.resume_quantity_dialog(a,token).unwrap());let _=w.complete_tanning(a,req,row,5);assert!(!w.resume_quantity_dialog(a,token).unwrap());assert_eq!(w.data(a).unwrap().inventory,before.inventory,"npc{kind} row{row} case{case}");assert_eq!(w.data(a).unwrap().stats,before.stats);assert!(w.data(a).unwrap().inventory.modified().is_empty());assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::GameMessage{text,..}if text.starts_with("The tanner tans "))));for _ in 0..4{w.tick().unwrap();}assert_eq!(w.data(a).unwrap().inventory,before.inventory);n+=1;}}}assert_eq!(n,144);println!("PG_MEMBER_TANNING atomic_refusals={n}");}
