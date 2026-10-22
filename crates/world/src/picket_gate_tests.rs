use super::*;
use crate::{loc_collision::LocCollision,simulation::{Action,PlayerData,WorldEvent},stats::Stats};
fn catalog()->Catalog {
    use Kind::*;
    [(1558,Door{kind:GateClosedLeft,next:1559,sound:67}),(1559,Door{kind:GateOpenedLeft,next:1558,sound:66}),
     (1560,Door{kind:GateClosedRight,next:1567,sound:67}),(1567,Door{kind:GateOpenedRight,next:1560,sound:66})].into()
}
fn setup(angle:u8,pair:bool)->(World,Location,Option<Location>) {
    let mut w=World::default();w.doors=catalog();
    for x in(3192..3224).step_by(8) {for z in(3192..3224).step_by(8) {w.collision.allocate(x,z,0);}}
    let l=Location::new(CoordGrid::new(3208,3208,0).unwrap(),1558,0,angle).unwrap();
    let p=begin(l,w.doors[&1558],CoordGrid::NULL).unwrap().pending;
    let right=pair.then(||Location::new(p.pair.unwrap().0,1560,0,angle).unwrap());
    for id in [1558,1559,1560,1567] {w.location_types.insert(id,LocCollision{width:1,length:1,block_walk:2,block_range:true,break_route_finding:false});}
    for loc in std::iter::once(l).chain(right) {w.locations.load(loc);crate::loc_collision::toggle(&mut w.collision,loc.coords,0,angle,w.location_types[&loc.id],true).unwrap();}
    (w,l,right)
}
#[test]
fn picket_hinge_translations_all_angles_both_halves_round_trip() {
    for angle in 0..4 {for(id,kind,next)in [(1558,Kind::GateClosedLeft,1559),(1560,Kind::GateClosedRight,1567)] {
        let source=Location::new(CoordGrid::new(3208,3208,0).unwrap(),id,0,angle).unwrap();
        let started=begin(source,Door{kind,next,sound:67},source.coords).unwrap();
        assert!(started.step_away.is_none());assert_eq!(started.pending.destination.angle,(angle+3)&3);
        let expected=if id==1558{[(3207,3208),(3208,3209),(3209,3208),(3208,3207)]}else{[(3206,3207),(3207,3210),(3210,3209),(3209,3206)]};
        assert_eq!((started.pending.destination.coords.x(),started.pending.destination.coords.z()),expected[angle as usize]);
        let close=begin(started.pending.destination,catalog()[&next],source.coords).unwrap();assert_eq!(close.pending.destination,source);
    }}
    let invalid=Location::new(CoordGrid::new(3208,3208,0).unwrap(),1558,9,0).unwrap();
    assert!(begin(invalid,catalog()[&1558],CoordGrid::NULL).is_err());
    for x in [6431,6432] {
        let edge=Location::new(CoordGrid::new(x,3200,0).unwrap(),1558,0,2).unwrap();
        assert!(begin(edge,catalog()[&1558],CoordGrid::NULL).is_err());
    }
}
#[test]
fn picket_pair_mutation_captures_both_before_delete_and_closes_from_either_half() {
    for angle in 0..4 {for right_first in [false,true] {for close_right in [false,true] {
        let(mut w,l,r)=setup(angle,true);let r=r.unwrap();let c=catalog();
        let source=if right_first{r}else{l};let p=begin(source,c[&source.id],CoordGrid::NULL).unwrap().pending;
        let(dx,dz)=[(-1,0),(0,1),(1,0),(0,-1)][angle as usize];
        assert!(!w.collision.can_step(l.coords,dx,dz,1,0,crate::collision::Strategy::Normal));
        let ol=begin(l,c[&l.id],CoordGrid::NULL).unwrap().pending.destination;
        let or=begin(r,c[&r.id],CoordGrid::NULL).unwrap().pending.destination;
        finish(&mut w,&c,p).unwrap();
        assert!(w.collision.can_step(l.coords,dx,dz,1,0,crate::collision::Strategy::Normal));
        assert_eq!(w.locations.find_type(ol.coords,ol.id),Some(ol));assert_eq!(w.locations.find_type(or.coords,or.id),Some(or));
        assert!(w.locations.find_type(l.coords,l.id).is_none());assert!(w.locations.find_type(r.coords,r.id).is_none());
        let source=if close_right{or}else{ol};finish(&mut w,&c,begin(source,c[&source.id],CoordGrid::NULL).unwrap().pending).unwrap();
        assert!(!w.collision.can_step(l.coords,dx,dz,1,0,crate::collision::Strategy::Normal));
        assert_eq!(w.locations.find_type(l.coords,l.id),Some(l));assert_eq!(w.locations.find_type(r.coords,r.id),Some(r));
    }}}
}
#[test]
fn picket_single_half_and_invalid_partner_destination_leave_unrelated_scenery() {
    let(mut w,l,_)=setup(0,false);let c=catalog();let p=begin(l,c[&l.id],CoordGrid::NULL).unwrap().pending;
    finish(&mut w,&c,p).unwrap();assert_eq!(w.locations.find_type(p.destination.coords,1559),Some(p.destination));
    for bad in 0..4 {
        let(mut w,l,r)=setup(0,true);let r=r.unwrap();let p=begin(l,c[&l.id],CoordGrid::NULL).unwrap().pending;
        if bad==0 {w.location_types.remove(&1567);}
        if bad==1 {let blocker=Location::new(p.destination.coords,1234,0,0).unwrap();w.locations.load(blocker);}
        if bad==2 {w.clock.0=i32::MAX-100;}
        if bad==3 {let blocker=Location::new(p.destination.coords,1234,1,0).unwrap();w.locations.load(blocker);}
        assert!(finish(&mut w,&c,p).is_err());assert_eq!(w.locations.find_type(l.coords,l.id),Some(l));assert_eq!(w.locations.find_type(r.coords,r.id),Some(r));
    }
}

#[test]
fn picket_stale_replaced_target_during_approach_cannot_open_new_revision() {
    let(mut w,l,r)=setup(0,true);let r=r.unwrap();
    w.queue_login(1,PlayerData::new(1,"Stale gate".into(),l.coords.translate(4,0,0).unwrap(),Stats::default())).unwrap();w.tick().unwrap();let a=w.actors().next().unwrap();
    w.submit(a,Action::OperateLocation{coords:l.coords,id:l.id,operation:1,control:false}).unwrap();w.tick().unwrap();
    assert_eq!(w.locations.find_type(l.coords,l.id),Some(l));
    w.change_location(l,false).unwrap();w.change_location(l,true).unwrap();w.drain_events();
    for _ in 0..10 {w.tick().unwrap();assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::DoorSound{..})));}
    assert_eq!(w.locations.find_type(l.coords,l.id),Some(l));assert_eq!(w.locations.find_type(r.coords,r.id),Some(r));
}
#[test]
fn picket_world_operation_reuses_reach_sound_and_500_tick_restoration() {
    let(mut w,l,r)=setup(0,true);let r=r.unwrap();let c=catalog();
    let ol=begin(l,c[&l.id],CoordGrid::NULL).unwrap().pending.destination;
    let or=begin(r,c[&r.id],CoordGrid::NULL).unwrap().pending.destination;
    w.queue_login(1,PlayerData::new(1,"Picket gate".into(),l.coords.translate(1,0,0).unwrap(),Stats::default())).unwrap();w.tick().unwrap();
    let a=w.actors().next().unwrap();let before=w.data(a).unwrap().clone();w.drain_events();
    w.submit(a,Action::OperateLocation{coords:l.coords,id:l.id,operation:1,control:false}).unwrap();
    let mut heard=false;
    for _ in 0..8 {w.tick().unwrap();heard|=w.drain_events().iter().any(|e|matches!(e,WorldEvent::DoorSound{sound:67,..}));if heard{break;}}
    assert!(heard,"reached gate operation must emit opening sound");
    assert_eq!(w.locations.find_type(ol.coords,ol.id),Some(ol));assert_eq!(w.locations.find_type(or.coords,or.id),Some(or));
    // Shared door timers run after Late advances the clock. The action was
    // scheduled at the preceding clock, so compare its deadline explicitly.
    let deadline=w.clock.0-1+DURATION;
    while w.clock.0<deadline-1 {w.tick().unwrap();w.drain_events();}
    assert!(w.locations.find_type(l.coords,l.id).is_none());
    w.tick().unwrap();assert_eq!(w.clock.0,deadline);assert_eq!(w.locations.find_type(l.coords,l.id),Some(l));assert_eq!(w.locations.find_type(r.coords,r.id),Some(r));
    assert!(w.locations.find_type(ol.coords,ol.id).is_none());assert!(w.locations.find_type(or.coords,or.id).is_none());
    assert_eq!(w.data(a).unwrap().inventory,before.inventory);assert_eq!(w.data(a).unwrap().stats,before.stats);
}

#[test]
fn picket_life_refuses_zero_health_and_unresolved_death_with_restored_visible_health() {
    use crate::hits::{Hit,Style,Target};
    for phase in 0..3 {
        let(mut w,l,r)=setup(0,true);let r=r.unwrap();
        w.queue_login(1,PlayerData::new(1,"Gate life".into(),l.coords.translate(1,0,0).unwrap(),Stats::default())).unwrap();w.tick().unwrap();
        let a=w.actors().next().unwrap();
        if phase==0{w.data_mut(a).unwrap().stats.set_current(3,0);}else{
            w.queue_hit(Target::Player(a),Hit{display:None,source:None,style:Style::Typeless,damage:99},0).unwrap();w.tick().unwrap();
            assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(id))if *id==a)));
            if phase==2{w.data_mut(a).unwrap().stats.set_current(3,10);}
        }
        w.drain_events();w.submit(a,Action::OperateLocation{coords:l.coords,id:l.id,operation:1,control:false}).unwrap();
        for _ in 0..8 {w.tick().unwrap();assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::DoorSound{..})));}
        assert_eq!(w.locations.find_type(l.coords,l.id),Some(l));assert_eq!(w.locations.find_type(r.coords,r.id),Some(r));
    }
}
