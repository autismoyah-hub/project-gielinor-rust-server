//! Raw dynamic parent admission uses the actor's current selector, including replies.
use super::*;
use crate::{bank_access::Request, bank_actions::Binding, inventory::Item, locations::Location,
    loc_collision::LocCollision, variables::{Transmit,Varbit,Varp}};

fn binding()->Binding {
    Binding{base:Varp{id:1293,permanent:true,transmit:Transmit::Always,bit_protect:false,client_code:-1},
        bit:Varbit{base:1293,lsb:0,msb:0}}
}
fn mode(w:&mut World,a:ActorId,value:i32) {w.data_mut(a).unwrap().variables.set(binding().base,value,true);}
fn setup(angle:u8,member:bool,raw:bool)->(World,ActorId,Location) {
    let mut w=World::default();
    w.collision.allocate(3200,3200,0);
    w.object_types=crate::banknote_exchange::tests::catalog();
    w.ordinary_bank_booths.insert(18491);
    w.normal_bank_booth_modes.insert(27291,binding());
    let l=Location::new(CoordGrid::new(3201,3200,0).unwrap(),if raw{27291}else{18491},10,angle).unwrap();
    let ty=LocCollision{width:1,length:1,block_walk:2,block_range:true,break_route_finding:false};
    w.location_types.insert(l.id,ty);w.locations.load(l);
    crate::loc_collision::toggle(&mut w.collision,l.coords,10,angle,ty,true).unwrap();
    w.held_use_types.insert(1512,crate::held_use::Type{id:1512,content:0});
    let mut data=PlayerData::new(1,"Normal booth".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());
    for i in 0..26 {data.inventory.restore_slot(i,Some(Item{id:1513,count:1,vars:0})).unwrap();}
    data.inventory.restore_slot(13,Some(Item{id:1512,count:5,vars:0})).unwrap();
    w.queue_login(1,data).unwrap();w.tick().unwrap();
    let a=w.actors().next().unwrap();w.set_membership(a,member).unwrap();w.drain_events();(w,a,l)
}
fn open(w:&mut World,a:ActorId,l:Location,op:u8)->Option<Request> {
    w.submit(a,Action::OperateLocation{coords:l.coords,id:l.id,operation:op,control:false}).unwrap();
    w.tick().unwrap();w.drain_events().into_iter().find_map(|e|if let WorldEvent::LocationBankRequested{request,..}=e{Some(request)}else{None})
}
fn use_note(w:&mut World,a:ActorId,l:Location)->Option<u64> {
    w.submit(a,Action::UseLocation{coords:l.coords,id:l.id,slot:13,object:1512,control:false}).unwrap();
    w.tick().unwrap();w.dialogue_token(a).unwrap()
}
fn reply(w:&mut World,a:ActorId,token:u64,choice:i32) {
    w.submit(a,Action::ResumeDialogue{token,choice}).unwrap();w.tick().unwrap();
}
fn total(w:&World,a:ActorId,id:i32)->i32 {
    w.data(a).unwrap().inventory.slots().iter().flatten().filter(|i|i.id==id).map(|i|i.count).sum()
}
#[test]
fn normal_dynamic_booth_op2_members_angles_and_only_selector_bit_zero() {
    for angle in 0..4 {for member in [false,true] {for bits in [0,2,4,128,-2,1,3,129,-1] {
        let(mut w,a,l)=setup(angle,member,true);mode(&mut w,a,bits);
        let before=w.data(a).unwrap().clone();
        let r=open(&mut w,a,l,2);
        assert_eq!(r.is_some(),bits&1==0,"angle {angle} member {member} bits {bits}");
        if let Some(r)=r {assert_eq!(r.kind(),crate::bank_access::Kind::Full);assert!(w.location_bank_request_valid(a,r));}
        assert_eq!(w.data(a).unwrap().inventory,before.inventory);
        assert_eq!(w.data(a).unwrap().bank,before.bank);
        assert_eq!(w.data(a).unwrap().stats,before.stats);
    }}}
}
#[test]
fn normal_dynamic_booth_rechecks_selector_at_pending_bank_presentation() {
    let(mut w,a,l)=setup(2,false,true);
    let r=open(&mut w,a,l,2).unwrap();mode(&mut w,a,1);
    assert!(!w.location_bank_request_valid(a,r));
    mode(&mut w,a,2);assert!(w.location_bank_request_valid(a,r));
    w.normal_bank_booth_modes.clear();assert!(!w.location_bank_request_valid(a,r));
    for op in [1,3,4,5] {let(mut w,a,l)=setup(0,true,true);assert!(open(&mut w,a,l,op).is_none());}
    let(mut w,a,l)=setup(0,false,false);mode(&mut w,a,1);
    assert!(open(&mut w,a,l,2).is_some(),"static booth retains its independent profile");
}
#[test]
fn normal_dynamic_booth_notes_partial_conversion_no_and_mode_switch_cancel() {
    for member in [false,true] {for phase in 0..4 {
        let(mut w,a,l)=setup(0,member,true);
        let before=w.data(a).unwrap().clone();
        if phase==0 {mode(&mut w,a,1);assert_eq!(use_note(&mut w,a,l),None);}
        else {
            let token=use_note(&mut w,a,l).expect("normal selector admits confirmation");
            if phase==1 {reply(&mut w,a,token,2);}
            if phase==2 {
                mode(&mut w,a,1);reply(&mut w,a,token,1);
                mode(&mut w,a,0);reply(&mut w,a,token,1);
                assert_eq!(w.dialogue_token(a).unwrap(),None);
            }
            if phase==3 {
                mode(&mut w,a,128);reply(&mut w,a,token,1);
                assert_eq!((total(&w,a,1512),total(&w,a,1511)),(3,2));
                let message=w.dialogue_token(a).unwrap().unwrap();
                let committed=w.data(a).unwrap().inventory.clone();
                mode(&mut w,a,129);w.tick().unwrap();
                assert_eq!(w.dialogue_token(a).unwrap(),None);
                mode(&mut w,a,0);reply(&mut w,a,message,0);reply(&mut w,a,token,1);
                assert_eq!(w.data(a).unwrap().inventory,committed);
            }
        }
        if phase!=3 {assert_eq!(w.data(a).unwrap().inventory,before.inventory);}
        assert_eq!(w.data(a).unwrap().bank,before.bank);
        assert_eq!(w.data(a).unwrap().stats,before.stats);
    }}
}
#[test]
fn normal_dynamic_booth_life_and_stale_location_admission() {
    for phase in 0..3 {
        let(mut w,a,l)=setup(0,false,true);
        if phase==0 {w.data_mut(a).unwrap().stats.set_current(3,0);}
        if phase==1 {w.locations=Default::default();}
        if phase==2 {w.normal_bank_booth_modes.clear();}
        let before=w.data(a).unwrap().inventory.clone();
        assert!(open(&mut w,a,l,2).is_none());assert_eq!(use_note(&mut w,a,l),None);
        assert_eq!(w.data(a).unwrap().inventory,before);
    }
}
