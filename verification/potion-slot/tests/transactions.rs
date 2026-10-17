use gielinor_world::{inventory::{Catalog, Inventory, Item, StackType, Template}, potions::{self, Potion, ResolvedEffect, Event}, stats::Stats};

fn dose() -> Potion {
    Potion { id: 100, next: 101, vial: 102, sip: "sip".into(), remaining: "3 left".into(),
        effects: vec![ResolvedEffect::Energy(20), ResolvedEffect::Boost {skill:0,constant:5,percent:15}] }
}
fn full(selected:usize)->Inventory {
    let mut inv=Inventory::new(28,StackType::Normal,false);
    for i in 0..28 {inv.restore_slot(i,Some(Item{id:if i==selected || i%3==0 {100} else {999},count:1,vars:0})).unwrap();}
    inv.clear_modified();inv
}
fn stats()->Stats {let mut s=Stats::default();s.set_base(0,50);s.set_current(0,40);s.set_fine_xp(0,12345).unwrap();s}

#[test]
fn every_selected_slot_with_duplicates_and_full_bag_drink_and_empty() {
    for selected in 0..28 {for empty in [false,true] {
        let mut inv=full(selected);let before=inv.clone();let mut s=stats();let original=s.clone();
        let mut energy=5000;let mut events=Vec::new();
        assert_eq!(potions::consume_slot(&dose(),selected,empty,&mut inv,&mut s,&mut energy,&Catalog::default(),829,|e|events.push(e)).unwrap(),true);
        for i in 0..28 {assert_eq!(inv.get(i),if i==selected {Some(Item{id:if empty{102}else{101},count:1,vars:0})}else{before.get(i)});}
        assert_eq!(inv.modified().iter().copied().collect::<Vec<_>>(),vec![selected]);
        for skill in 0..=255 {assert_eq!(s.fine_xp(skill),original.fine_xp(skill));}
        if empty {assert_eq!(s,original);assert_eq!(energy,5000);assert_eq!(events,vec![Event::Message(potions::EMPTY_MESSAGE.into())]);}
        else {assert_eq!(s.current(0),52);assert_eq!(energy,7000);assert_eq!(events.first(),Some(&Event::Animation(829)));assert_eq!(events.last(),Some(&Event::Delay(3)));}
        let state=(inv.clone(),s.clone(),energy);events.clear();
        assert!(!potions::consume_slot(&dose(),selected,empty,&mut inv,&mut s,&mut energy,&Catalog::default(),829,|e|events.push(e)).unwrap());
        assert_eq!((inv,s,energy),state);assert!(events.is_empty());
    }}
}

#[test]
fn malformed_or_stale_identity_and_nonordinary_inputs_or_outputs_are_lossless() {
    for empty in [false,true] {for mode in 0..18 {
        let mut inv=full(27);let mut c=Catalog::default();let mut p=dose();
        let mut slot=27;
        let output=if empty{102}else{101};
        match mode {
            0=>slot=usize::MAX,1=>{inv.restore_slot(27,None).unwrap();},
            2=>{inv.restore_slot(27,Some(Item{id:999,count:1,vars:0})).unwrap();},
            3..=5=>{inv.restore_slot(27,Some(Item{id:100,count:[0,-1,2][mode-3],vars:0})).unwrap();},
            6=>{c.stackable.insert(100);},7=>{c.stackable.insert(output);},
            8=>{c.dummy.insert(100);},9=>{c.dummy.insert(output);},
            10=>{c.notes.insert(100,Template{link:88,template:1});},
            11=>{c.notes.insert(output,Template{link:88,template:1});},
            12=>{c.placeholders.insert(100,Template{link:88,template:1});},
            13=>{c.placeholders.insert(output,Template{link:88,template:1});},
            14=>{c.transforms.insert(100,Template{link:88,template:1});},
            15=>{c.transforms.insert(output,Template{link:88,template:1});},
            16=>{if empty {p.vial=-1}else{p.next=-1}},
            _=>{if empty {p.vial=p.id}else{p.next=p.id}},
        }
        let mut s=stats();let mut energy=5000;let state=(inv.clone(),s.clone(),energy);let mut events=Vec::new();
        assert!(!potions::consume_slot(&p,slot,empty,&mut inv,&mut s,&mut energy,&c,829,|e|events.push(e)).unwrap(),"mode {mode} empty {empty}");
        assert_eq!((inv,s,energy),state);assert!(events.is_empty());
    }}
}

#[test]
fn late_effect_failure_rolls_back_prior_stat_energy_and_all_events() {
    for bad_at in 0..4 {
        let mut p=dose();p.effects=vec![ResolvedEffect::Energy(20),ResolvedEffect::Boost{skill:0,constant:5,percent:15},ResolvedEffect::Energy(10)];
        p.effects.insert(bad_at,ResolvedEffect::Boost{skill:1,constant:-1,percent:0});
        let mut inv=full(27);let mut s=stats();let mut energy=5000;let state=(inv.clone(),s.clone(),energy);let mut events=Vec::new();
        assert!(potions::consume_slot(&p,27,false,&mut inv,&mut s,&mut energy,&Catalog::default(),829,|e|events.push(e)).is_err());
        assert_eq!((inv,s,energy),state);assert!(events.is_empty());
    }
}

#[test]
fn stack_always_cannot_redirect_replacement_or_overflow_an_existing_stack() {
    for count in [1,i32::MAX] {
        let mut inv=full(27);inv.stack=StackType::Always;
        inv.restore_slot(0,Some(Item{id:101,count,vars:0})).unwrap();
        let mut s=stats();let mut energy=5000;let state=(inv.clone(),s.clone(),energy);let mut events=Vec::new();
        assert!(!potions::consume_slot(&dose(),27,false,&mut inv,&mut s,&mut energy,&Catalog::default(),829,|e|events.push(e)).unwrap());
        assert_eq!((inv,s,energy),state);assert!(events.is_empty());
    }
}
