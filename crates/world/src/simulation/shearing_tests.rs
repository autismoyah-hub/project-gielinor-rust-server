use super::*;
use crate::{inventory::Item,npc::{BlockWalk,Mode,Restrict},npc_options::Definition,stats::Stats};
fn fixture(full:bool,tool:bool)->(World,ActorId,NpcId) {
    let mut w=World::default();for x in(3192..3224).step_by(8){for z in(3192..3224).step_by(8){w.collision.allocate(x,z,0);}}
    w.shearing=Some(Config{targets:[(2693,1178)].into(),shears:1735,wool:1737,sequence:893,sound:761});
    w.npc_handlers.insert((2693,1),Handler::Shear{raw:2693,selected:None});
    w.npc_options.types.insert(2693,Definition{operations:[Some("Shear".into()),None,None,None,None],..Default::default()});w.npc_options.types.insert(1178,Definition::default());
    w.held_use_types.insert(1735,crate::held_use::Type{id:1735,content:-1});
    let mut npc=Npc::new(2693,CoordGrid::new(3201,3200,0).unwrap(),1,Restrict::Normal,BlockWalk::Npc).unwrap();npc.mode=Mode::None;
    let n=w.spawn_npc(npc).unwrap();
    let mut data=PlayerData::new(1,"Shearing fixture".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());
    for slot in 0..28{if full||slot!=0{data.inventory.restore_slot(slot,Some(Item{id:1513,count:1,vars:0})).unwrap();}}
    if tool{data.inventory.restore_slot(13,Some(Item{id:1735,count:1,vars:0})).unwrap();}
    w.queue_login(1,data).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();w.set_observer_id(a,42).unwrap();w.drain_events();(w,a,n)
}
fn count(w:&World,a:ActorId,id:i32)->i32{w.data(a).unwrap().inventory.slots().iter().flatten().filter(|i|i.id==id).map(|i|i.count).sum()}
fn begin(w:&mut World,a:ActorId,n:NpcId,held:bool) {
    w.submit(a,if held{Action::UseNpc{target:n,slot:13,object:1735,control:false}}else{Action::OperateNpc{target:n,operation:1,control:false}}).unwrap();
    for _ in 0..8{w.tick().unwrap();if w.players.get(a.slot as usize).unwrap().pending_shearing.is_some(){break;}}
    assert!(w.players.get(a.slot as usize).unwrap().pending_shearing.is_some(),"reached action starts shearing");
}
#[test]
fn shearing_two_ticks_both_dispatches_members_and_regrowth_preserve_tools_and_xp() {
    for member in [false,true]{for held in [false,true]{
        let(mut w,a,n)=fixture(false,true);w.set_membership(a,member).unwrap();let before=w.data(a).unwrap().clone();let npc_before=w.npc(n).unwrap().clone();
        begin(&mut w,a,n,held);let start=w.clock.0-1;
        assert_eq!(count(&w,a,1737),0);assert_eq!(w.npc(n).unwrap().type_id,2693);
        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::PlayerDelayedSound{sound:761,delay:10,..})));
        w.tick().unwrap();assert_eq!(count(&w,a,1737),0);assert_eq!(w.npc(n).unwrap().dialogue_face,Some(a));
        w.tick().unwrap();assert_eq!(w.clock.0-1,start+2);assert_eq!(count(&w,a,1737),1);assert_eq!(count(&w,a,1735),1);assert_eq!(w.npc(n).unwrap().visible_type(),1178);
        assert_eq!(w.npc(n).unwrap().type_id,npc_before.type_id);assert_eq!(w.npc(n).unwrap().stats,npc_before.stats);assert_eq!(w.npc(n).unwrap().spawn,npc_before.spawn);
        let regrow=w.drain_events().into_iter().find_map(|e|if let WorldEvent::SheepSheared{regrow_at,in_inventory:true,..}=e{Some(regrow_at)}else{None}).unwrap();assert_eq!(regrow,start+2+50);
        while w.clock.0<regrow{w.tick().unwrap();w.drain_events();assert_eq!(w.npc(n).unwrap().visible_type(),1178);}
        assert_eq!(w.npc(n).unwrap().face_player,None);w.tick().unwrap();assert_eq!(w.npc(n).unwrap().type_id,2693);
        assert_eq!(w.data(a).unwrap().stats,before.stats);assert_eq!(w.data(a).unwrap().bank,before.bank);assert_eq!(w.data(a).unwrap().equipment,before.equipment);
    }}
}
#[test]
fn shearing_full_inventory_drops_one_private_wool_without_consuming_shears() {
    let(mut w,a,n)=fixture(true,true);let before=w.data(a).unwrap().clone();begin(&mut w,a,n,true);w.tick().unwrap();w.tick().unwrap();
    assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.npc(n).unwrap().visible_type(),1178);
    let objects=w.objects.at(before.coords).map(|h|h.snapshot()).collect::<Vec<_>>();assert_eq!(objects.len(),1);let obj=&objects[0];
    assert_eq!((obj.item,obj.count,obj.owner,obj.receiver),(1737,1,Some(42),Some(42)));assert!(obj.visible_to(Some(42)));assert!(!obj.visible_to(Some(43)));
    assert_eq!(w.data(a).unwrap().stats,before.stats);
}
#[test]
fn shearing_missing_late_tools_and_stale_selected_slot_do_not_change_sheep() {
    let(mut w,a,n)=fixture(false,false);w.submit(a,Action::OperateNpc{target:n,operation:1,control:false}).unwrap();for _ in 0..5{w.tick().unwrap();}
    assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::EngineMessage{message:"You need a set of shears to do this.",..})));
    assert_eq!(w.npc(n).unwrap().type_id,2693);
    for late in [false,true] {
        let(mut w,a,n)=fixture(false,true);begin(&mut w,a,n,true);if late{w.tick().unwrap();}
        w.data_mut(a).unwrap().inventory.restore_slot(13,None).unwrap();for _ in 0..4{w.tick().unwrap();}
        assert_eq!(w.npc(n).unwrap().type_id,2693);assert_eq!(count(&w,a,1737),0);assert!(w.npc(n).unwrap().dialogue_face.is_none());
    }
    let(mut w,a,n)=fixture(false,true);w.data_mut(a).unwrap().inventory.restore_slot(13,Some(Item{id:1513,count:1,vars:0})).unwrap();
    w.submit(a,Action::UseNpc{target:n,slot:13,object:1735,control:false}).unwrap();for _ in 0..4{w.tick().unwrap();}assert_eq!(count(&w,a,1737),0);assert_eq!(w.npc(n).unwrap().type_id,2693);
}
#[test]
fn shearing_modal_move_and_npc_generation_cancellation_release_owned_face() {
    for case in 0..4 {
        let(mut w,a,n)=fixture(false,true);begin(&mut w,a,n,false);w.tick().unwrap();
        let mut replacement=None;
        match case {
            0=>w.set_modal_open(a,true).unwrap(),
            1=>{let p=w.players.get_mut(a.slot as usize).unwrap();p.movement.teleport(&mut w.collision,CoordGrid::new(3199,3200,0).unwrap(),w.clock.0,true).unwrap();p.data.coords=p.movement.coords;},
            2=>{w.despawn_npc(n);let mut npc=Npc::new(2693,CoordGrid::new(3201,3200,0).unwrap(),1,Restrict::Normal,BlockWalk::Npc).unwrap();npc.mode=Mode::None;replacement=Some(w.spawn_npc(npc).unwrap());},
            _=>{w.npc_mut(n).unwrap().type_id=1178;},
        }
        for _ in 0..5{w.tick().unwrap();}assert_eq!(count(&w,a,1737),0,"case{case}");
        if let Some(n)=replacement{assert_eq!(w.npc(n).unwrap().type_id,2693);assert_ne!(n.generation,1);}else{assert!(w.npc(n).unwrap().dialogue_face.is_none());}
    }
}

#[test]
fn shearing_protected_walk_refuses_input_and_completes_original_work_once() {
    let(mut w,a,n)=fixture(false,true);begin(&mut w,a,n,false);w.tick().unwrap();let before=w.data(a).unwrap().coords;w.drain_events();
    w.submit(a,Action::Walk{destination:CoordGrid::new(3199,3200,0).unwrap(),run:false}).unwrap();w.tick().unwrap();
    assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::ActionRejected{actor,error:ActionError::Delayed}if *actor==a)));
    assert_eq!(count(&w,a,1737),1);assert_eq!(w.data(a).unwrap().coords,before);for _ in 0..4{w.tick().unwrap();}assert_eq!(count(&w,a,1737),1);
}

#[test]
fn shearing_two_players_cannot_duplicate_a_single_fleece() {
    let(mut w,a,n)=fixture(false,true);let mut data=PlayerData::new(2,"Other shearer".into(),CoordGrid::new(3201,3201,0).unwrap(),Stats::default());data.inventory.restore_slot(13,Some(Item{id:1735,count:1,vars:0})).unwrap();w.queue_login(2,data).unwrap();w.tick().unwrap();
    let b=w.actors().find(|&id|w.data(id).unwrap().account_id==2).unwrap();w.set_observer_id(b,43).unwrap();w.drain_events();
    for actor in [a,b]{w.submit(actor,Action::OperateNpc{target:n,operation:1,control:false}).unwrap();}
    for _ in 0..8{w.tick().unwrap();}
    assert_eq!(count(&w,a,1737)+count(&w,b,1737),1);assert_eq!(count(&w,a,1735),1);assert_eq!(count(&w,b,1735),1);assert_eq!(w.npc(n).unwrap().visible_type(),1178);
    assert_eq!(w.drain_events().iter().filter(|e|matches!(e,WorldEvent::SheepSheared{..})).count(),1);
}

#[test]
fn shearing_capacity_is_live_and_naturally_sheared_sheep_do_not_regrow() {
    for full in [false,true] {
        let(mut w,a,n)=fixture(full,true);begin(&mut w,a,n,false);
        w.data_mut(a).unwrap().inventory.restore_slot(0,if full{None}else{Some(Item{id:1513,count:1,vars:0})}).unwrap();w.tick().unwrap();w.tick().unwrap();
        assert_eq!(count(&w,a,1737),i32::from(full));assert_eq!(w.objects.count(),usize::from(!full));assert_eq!(w.npc(n).unwrap().visible_type(),1178);
    }
    let(mut w,a,n)=fixture(false,true);w.npc_mut(n).unwrap().type_id=1178;
    w.submit(a,Action::OperateNpc{target:n,operation:1,control:false}).unwrap();for _ in 0..60{w.tick().unwrap();}
    assert_eq!(w.npc(n).unwrap().visible_type(),1178);assert_eq!(count(&w,a,1737),0);
}

#[test]
fn shearing_visible_change_preserves_base_and_cancels_modal_held_npc_interaction() {
    let(mut w,a,n)=fixture(false,true);let mut data=PlayerData::new(2,"Waiting shearer".into(),CoordGrid::new(3202,3200,0).unwrap(),Stats::default());data.inventory.restore_slot(13,Some(Item{id:1735,count:1,vars:0})).unwrap();w.queue_login(2,data).unwrap();w.tick().unwrap();
    let b=w.actors().find(|&id|w.data(id).unwrap().account_id==2).unwrap();w.set_modal_open(b,true).unwrap();begin(&mut w,a,n,false);
    w.submit(b,Action::OperateNpc{target:n,operation:1,control:false}).unwrap();w.tick().unwrap();assert!(w.players.get(b.slot as usize).unwrap().npc_interaction.is_some());
    w.tick().unwrap();assert!(w.players.get(b.slot as usize).unwrap().npc_interaction.is_none());assert_eq!(w.npc(n).unwrap().type_id,2693);assert_eq!(w.npc(n).unwrap().visible_type(),1178);
    for _ in 0..51{w.tick().unwrap();}assert_eq!(w.npc(n).unwrap().visible_type(),2693);assert_eq!(w.npc(n).unwrap().type_id,2693);w.set_modal_open(b,false).unwrap();
    for _ in 0..4{w.tick().unwrap();}assert_eq!(count(&w,b,1737),0);assert_eq!(count(&w,a,1737),1);
}
#[test]
fn shearing_unresolved_death_before_completion_never_rewards_or_revives() {
    use crate::hits::{Hit,Style,Target};
    for phase in 0..3 {
        let(mut w,a,n)=fixture(false,true);begin(&mut w,a,n,true);
        if phase==0{w.data_mut(a).unwrap().stats.set_current(3,0);}else{
            w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:99},0).unwrap();w.tick().unwrap();
            assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)));
            if phase==2{w.data_mut(a).unwrap().stats.set_current(3,10);}
        }
        for _ in 0..5{w.tick().unwrap();}assert_eq!(count(&w,a,1737),0);assert_eq!(w.npc(n).unwrap().type_id,2693);assert!(w.npc(n).unwrap().dialogue_face.is_none());
    }
}
