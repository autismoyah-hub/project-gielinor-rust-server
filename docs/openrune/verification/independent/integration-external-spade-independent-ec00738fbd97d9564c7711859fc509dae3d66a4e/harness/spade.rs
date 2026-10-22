//! Independent source-derived no-hook Dig oracle; public authoritative actions only.
use gielinor_world::{CoordGrid,inventory::Item,stats::Stats,npc::RandomSource,spade::{Rule,Hook},simulation::{World,PlayerData,ActorId,Action,WorldEvent,ActionError}};
const TEXT:&str="You dig a hole in the ground... but find nothing.";
struct Never;impl RandomSource for Never{fn below(&mut self,_:u32)->u32{panic!("generic Dig must not draw RNG")}}
fn tick(w:&mut World)->Vec<WorldEvent>{w.tick_with_random(&mut Never).unwrap();w.drain_events()}
fn fixture(slot:usize,full:bool,member:bool,hooks:&[Hook])->(World,ActorId){
 let mut w=World::default();w.collision.allocate(3200,3200,0);w.spade=Some(Rule{item:952,animation:830,sound:1470,delay:2});
 for h in hooks{w.register_spade_hook(*h).unwrap();}
 let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,99);let mut p=PlayerData::new(1,"Independent Dig".into(),CoordGrid::new(3203,3203,0).unwrap(),stats);
 for s in 0..28{if full||s==slot||s==(slot+1)%28{p.inventory.restore_slot(s,Some(Item{id:if s==slot||s==(slot+1)%28{952}else{1931},count:1,vars:0})).unwrap();}}
 w.queue_login(1,p).unwrap();tick(&mut w);let a=w.actors().next().unwrap();w.set_membership(a,member).unwrap();w.drain_events();(w,a)
}
fn messages(e:&[WorldEvent])->usize{e.iter().filter(|x|matches!(x,WorldEvent::GameMessage{text,..}if text==TEXT)).count()}
fn same(w:&World,a:ActorId,b:&PlayerData){let p=w.data(a).unwrap();assert_eq!(p.inventory,b.inventory);assert_eq!(p.equipment,b.equipment);assert_eq!(p.stats,b.stats);assert_eq!(p.coords,b.coords);assert_eq!(p.energy,b.energy);}
#[test]fn all_selected_slots_both_memberships_full_and_open_pack_two_tick_no_reward_or_repeat(){
 let mut count=0;for slot in 0..28{for full in [false,true]{for member in [false,true]{
  let(mut w,a)=fixture(slot,full,member,&[]);let before=w.data(a).unwrap().clone();w.submit(a,Action::DigItemSlot{slot,item:952}).unwrap();let first=tick(&mut w);let admitted=w.clock.0;
  assert_eq!(first.iter().filter(|e|matches!(e,WorldEvent::PlayerAnimation{sequence:830,..})).count(),1);assert_eq!(first.iter().filter(|e|matches!(e,WorldEvent::PlayerSound{sound:1470,..})).count(),1);assert_eq!(messages(&first),0);
  assert_eq!(messages(&tick(&mut w)),0);let done=tick(&mut w);assert_eq!(w.clock.0-admitted,2);assert_eq!(messages(&done),1);assert!(done.iter().any(|e|matches!(e,WorldEvent::PlayerAnimation{sequence:65535,..})));
  for _ in 0..8{let e=tick(&mut w);assert_eq!(messages(&e),0);assert!(!e.iter().any(|x|matches!(x,WorldEvent::PlayerAnimation{..}|WorldEvent::PlayerSound{..})));}same(&w,a,&before);count+=1;
 }}}assert_eq!(count,112);println!("A5_SPADE normal112 all28slots full/open member/F2P exact_delay2 no_RNG_reward_repeat=true");
}
#[test]fn stale_selected_duplicate_and_invalid_initial_state_refuse_without_effects(){
 let mut cases=0;for slot in [0,7,27]{for mode in 0..8{
  let(mut w,a)=fixture(slot,true,true,&[]);
  match mode{0=>w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:1931,count:1,vars:0})).unwrap(),1=>w.data_mut(a).unwrap().inventory.restore_slot(slot,Some(Item{id:952,count:1,vars:1})).unwrap(),2=>{w.object_types.stackable.insert(952);},3=>{w.object_types.dummy.insert(952);},4=>w.spade=None,5=>w.spade.as_mut().unwrap().delay=0,6=>{w.data_mut(a).unwrap().stats.set_current(3,0);},_=>w.set_modal_open(a,true).unwrap()}
  let before=w.data(a).unwrap().clone();w.submit(a,Action::DigItemSlot{slot,item:952}).unwrap();let mut events=Vec::new();for _ in 0..6{events.extend(tick(&mut w));}assert_eq!(messages(&events),0);assert!(!events.iter().any(|e|matches!(e,WorldEvent::PlayerAnimation{..}|WorldEvent::PlayerSound{..})));same(&w,a,&before);cases+=1;
 }}assert_eq!(cases,24);println!("A5_SPADE initial24 wrong_selected_with_real_duplicate vars catalog missing_rule invalid_delay zeroHP modal refuse=true");
}
#[test]fn actual_held_close_and_logout_cancel_completion_without_replay(){
 for mode in 0..3{let(mut w,a)=fixture(7,true,true,&[]);let before=w.data(a).unwrap().clone();w.submit(a,Action::DigItemSlot{slot:7,item:952}).unwrap();let first=tick(&mut w);assert_eq!(messages(&first),0);
  match mode{0=>w.submit(a,Action::EatItemSlot{slot:7,item:952}).unwrap(),1=>{w.close_dialogs(a).unwrap();},_=>w.request_logout(a).unwrap()}
  let mut e=Vec::new();for _ in 0..8{e.extend(tick(&mut w));}assert_eq!(messages(&e),0);same(&w,a,&before);println!("A5_SPADE cancel mode={mode} no_late_message_reward=true");
 }
}
fn claims(_: &World,_:ActorId)->bool{true}
fn allows(_: &mut World,_:ActorId)->Result<bool,ActionError>{Ok(true)}
fn veto(_: &mut World,_:ActorId)->Result<bool,ActionError>{Ok(false)}
fn first(w:&mut World,a:ActorId)->Result<(),ActionError>{w.data_mut(a)?.character_id=4321;Ok(())}
fn second(w:&mut World,a:ActorId)->Result<(),ActionError>{w.data_mut(a)?.character_id=1234;Ok(())}
#[test]fn independent_first_claim_callback_and_veto_do_not_fall_back_to_generic_text(){
 for deny in [false,true]{let hooks=[Hook{name:"independent-first",claims,before:if deny{veto}else{allows},dig:first},Hook{name:"independent-second",claims,before:allows,dig:second}];let(mut w,a)=fixture(0,true,true,&hooks);let before=w.data(a).unwrap().clone();w.submit(a,Action::DigItemSlot{slot:0,item:952}).unwrap();let mut e=Vec::new();for _ in 0..8{e.extend(tick(&mut w));}assert_eq!(messages(&e),0);assert_eq!(w.data(a).unwrap().character_id,if deny{before.character_id}else{4321});assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(e.iter().filter(|e|matches!(e,WorldEvent::PlayerAnimation{sequence:830,..})).count(),usize::from(!deny));println!("A5_SPADE hook deny={deny} first_claim_only=true");}
}
#[test]fn actual_native_cache_binding_identity_and_whole_factory_contradictions(){
 use gielinor_cache::{game::GameCache,symbols};use std::path::Path;
 let root=std::env::var("PG_GAME_CACHE").unwrap();let syms=std::env::var("PG_SYMBOLS").unwrap();let mut cache=GameCache::open(Path::new(&root),0xff_ffff).unwrap();let mut defs=cache.definitions_all().unwrap();let names=|kind|symbols::load(&[Path::new(&syms)],kind,None).unwrap();let obj=names("obj");let seq=names("seq");let synth=names("synth");let original=defs.items[&952].clone();assert_eq!(original.name,"Spade");assert_eq!(original.inventory_ops[0].as_deref(),Some("Dig"));
 let rule=gielinor_server::content::spade_tool(&obj,&seq,&synth,&defs).unwrap();assert_eq!((rule.item,rule.animation,rule.sound,rule.delay),(952,830,1470,2));
 for mode in 0..6{let mut item=original.clone();match mode{0=>item.members=true,1=>item.stackable=true,2=>item.name="Fake spade".into(),3=>item.inventory_ops[0]=Some("Use".into()),4=>item.inventory_ops[1]=Some("Dig".into()),_=>item.dummy=1}defs.items.insert(952,item);assert!(gielinor_server::content::spade_tool(&obj,&seq,&synth,&defs).is_err());}
 println!("A5_SPADE native952 firstDig seq830 source1470 delay2 factory6contradictions=true; sound_alias={:?}",synth.get("digspade"));
}
