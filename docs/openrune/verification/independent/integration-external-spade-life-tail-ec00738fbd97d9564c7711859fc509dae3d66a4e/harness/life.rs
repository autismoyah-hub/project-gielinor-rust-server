//! Actual DeathRequested is not undone by restoring only visible HP.
use gielinor_world::{CoordGrid,inventory::Item,stats::Stats,npc::RandomSource,spade::Rule,hits::{Hit,Target,Style},simulation::{World,PlayerData,Action,WorldEvent}};
struct Never;impl RandomSource for Never{fn below(&mut self,_:u32)->u32{panic!("Dig must not draw RNG")}}
fn tick(w:&mut World)->Vec<WorldEvent>{w.tick_with_random(&mut Never).unwrap();w.drain_events()}
fn run(during:bool,restore:bool){
 let mut w=World::default();w.collision.allocate(3200,3200,0);w.spade=Some(Rule{item:952,animation:830,sound:1470,delay:2});let mut stats=Stats::default();stats.set_base(3,99);stats.set_current(3,99);let mut p=PlayerData::new(1,"Independent spade life tail".into(),CoordGrid::new(3203,3203,0).unwrap(),stats);p.inventory.restore_slot(7,Some(Item{id:952,count:1,vars:0})).unwrap();w.queue_login(1,p).unwrap();tick(&mut w);let a=w.actors().next().unwrap();
 if during{w.submit(a,Action::DigItemSlot{slot:7,item:952}).unwrap();let e=tick(&mut w);assert!(e.iter().any(|e|matches!(e,WorldEvent::PlayerAnimation{sequence:830,..})));}
 w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:150},1).unwrap();let death=tick(&mut w);assert!(death.iter().any(|e|matches!(e,WorldEvent::DeathRequested(_))),"real lethal hit must request death");assert_eq!(w.data(a).unwrap().stats.current(3),0);
 if restore{w.data_mut(a).unwrap().stats.set_current(3,99);}
 let before=w.data(a).unwrap().clone();if !during{w.submit(a,Action::DigItemSlot{slot:7,item:952}).unwrap();}
 let mut events=Vec::new();for _ in 0..8{events.extend(tick(&mut w));}
 let messages=events.iter().filter(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="You dig a hole in the ground... but find nothing.")).count();let animations=events.iter().filter(|e|matches!(e,WorldEvent::PlayerAnimation{sequence:830,..})).count();println!("A5_SPADE_LIFE during={during} restore_visible_HP={restore} messages={messages} new_dig_animations={animations}");assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(messages,0,"unresolved DeathRequested cannot be bypassed by visible HP restoration");if !during{assert_eq!(animations,0);}
}
#[test]fn before_admission_death_hp_zero(){run(false,false)}
#[test]fn before_admission_death_visible_hp_restored(){run(false,true)}
#[test]fn during_delay_death_hp_zero(){run(true,false)}
#[test]fn during_delay_death_visible_hp_restored(){run(true,true)}
