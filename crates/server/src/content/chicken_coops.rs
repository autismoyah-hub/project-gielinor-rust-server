//! Ordinary F2P coops use the existing reached single-item grant transaction.
use super::{Definitions, Symbols};
use gielinor_world::{location_content::Handler, pickables::Rule};
use std::collections::{BTreeMap, BTreeSet};
const NODES: [(&str, u32); 3] = [
    ("chicken_coop_empty", 5571),
    ("chicken_coop_bare", 5572),
    ("chicken_coop_bare2", 5573),
];
const FOUND: &str = "You search the coop and find an egg.";
const FULL: &str = "You search the coop and find an egg but you don't have room to take it.";
pub fn chicken_coops(
    objects: &Symbols,
    locations: &Symbols,
    contents: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<(u32, u8), Handler>, String> {
    if objects.get("egg") != Some(&1944)
        || contents.get("chicken_coop") != Some(&47)
        || sequences.get("human_pickuptable") != Some(&832)
        || sounds.get("pick") != Some(&2581)
        || defs.items.get(&1944).is_none_or(|d| {
            d.name != "Egg" || d.members || d.stackable || d.dummy == 1
                || d.cert.template != 0 || d.placeholder.template != 0 || d.transform.template != 0
        })
    {
        return Err("Unattested ordinary coop product or presentation".into());
    }
    let actual: BTreeSet<_> = defs.locations.iter()
        .filter(|(_, d)| d.content_group == 47).map(|(&id, _)| id).collect();
    if actual != NODES.iter().map(|v| v.1).collect() {
        return Err("Unattested ordinary coop content group".into());
    }
    for (symbol, id) in NODES {
        if locations.get(symbol) != Some(&(id as i32))
            || defs.locations.get(&id).is_none_or(|d| {
                d.name != "Coop" || d.content_group != 47
                    || d.ops[0].as_deref() != Some("Search") || d.ops[1..].iter().any(Option::is_some)
                    || d.collision.width != 2 || d.collision.length != 1
                    || !d.multi.is_empty() || d.multi_default != -1 || d.multi_varbit != -1 || d.multi_varp != -1
            })
        {
            return Err(format!("Unattested ordinary coop {symbol}"));
        }
    }
    let rule = Rule { item: 1944, members: false, animation: 832, sound: 2581,
        picked_message: FOUND, full_message: FULL };
    Ok(NODES.into_iter().map(|(_, id)| ((id, 1), Handler::Pickable(rule))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{inventory::Item, locations::Location, npc::RandomSource,
        simulation::{Action, ActorId, PlayerData, World, WorldEvent}, stats::Stats, CoordGrid};
    struct Never;
    impl RandomSource for Never { fn below(&mut self, _: u32) -> u32 { panic!("coop search drew RNG") } }
    fn fixture() -> (Symbols, Symbols, Symbols, Symbols, Symbols, Definitions) {
        let mut defs = Definitions { inventories: Default::default(), locations: Default::default(),
            npcs: Default::default(), items: Default::default(), varbits: Default::default(),
            varps: Default::default(), stats: Default::default() };
        let mut locations = Symbols::new();
        for (symbol, id) in NODES {
            locations.insert(symbol.into(), id as i32);
            let mut d = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            d.name = "Coop".into(); d.content_group = 47; d.collision.width = 2;
            d.ops[0] = Some("Search".into()); defs.locations.insert(id, d);
        }
        let mut egg = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        egg.name = "Egg".into(); defs.items.insert(1944, egg);
        ([("egg".into(),1944)].into(),locations,[("chicken_coop".into(),47)].into(),
            [("human_pickuptable".into(),832)].into(),[("pick".into(),2581)].into(),defs)
    }
    fn setup(id: u32, angle: u8, x: i32) -> (World, ActorId, Location) {
        let (o,l,c,a,s,d) = fixture(); let mut w = World::default();
        for x in (3192..3224).step_by(8) { for z in (3192..3224).step_by(8) {w.collision.allocate(x,z,0);} }
        w.object_types = d.inventory_catalog();
        w.location_handlers = chicken_coops(&o,&l,&c,&a,&s,&d).unwrap();
        for (&id,t) in &d.locations {w.location_types.insert(id,t.collision);}
        let loc=Location::new(CoordGrid::new(3201,3200,0).unwrap(),id,10,angle).unwrap();
        w.locations.load(loc);
        gielinor_world::loc_collision::toggle(&mut w.collision,loc.coords,loc.shape,loc.angle,d.locations[&id].collision,true).unwrap();
        w.queue_login(1,PlayerData::new(1,"Coop test".into(),CoordGrid::new(x,3200,0).unwrap(),Stats::default())).unwrap();
        w.tick_with_random(&mut Never).unwrap(); let actor=w.actors().next().unwrap(); w.drain_events();
        (w,actor,loc)
    }
    fn search(loc: Location, operation: u8) -> Action {
        Action::OperateLocation { coords:loc.coords,id:loc.id,operation,control:false }
    }
    fn count(w:&World, actor:ActorId)->i32 {
        w.data(actor).unwrap().inventory.slots().iter().flatten().filter(|v|v.id==1944).map(|v|v.count).sum()
    }
    #[test]
    fn chicken_coops_exact_f2p_group_product_search_and_presentation_and_late_invalid_bindings() {
        let(o,l,c,a,s,d)=fixture();let h=chicken_coops(&o,&l,&c,&a,&s,&d).unwrap();assert_eq!(h.len(),3);
        for (_,id) in NODES {assert!(h.contains_key(&(id,1)));assert!(!h.contains_key(&(id,2)));assert!(h[&(id,1)].arrival_delay());}
        for mode in 0..12 {
            let(mut o,mut l,mut c,mut a,mut s,mut d)=fixture();
            match mode {
                0=>{o.insert("egg".into(),1945);},1=>d.items.get_mut(&1944).unwrap().members=true,
                2=>d.items.get_mut(&1944).unwrap().stackable=true,3=>d.items.get_mut(&1944).unwrap().cert.template=799,
                4=>{l.insert("chicken_coop_bare2".into(),5572);},5=>d.locations.get_mut(&5573).unwrap().ops[1]=Some("Search".into()),
                6=>d.locations.get_mut(&5573).unwrap().collision.width=1,7=>d.locations.get_mut(&5573).unwrap().multi_varp=1,
                8=>{c.clear();},9=>{a.clear();},10=>{s.clear();},
                _=>{let extra=d.locations[&5573].clone();d.locations.insert(5574,extra);}
            }
            assert!(chicken_coops(&o,&l,&c,&a,&s,&d).is_err(),"mode {mode}");
        }
    }
    #[test]
    fn chicken_coops_actual_search_grants_one_per_click_without_stock_xp_rng_or_auto_repeat() {
        for (_,id) in NODES { for angle in 0..4 { for member in [false,true] {
            let(mut w,actor,loc)=setup(id,angle,3200);w.set_membership(actor,member).unwrap();
            let stats=w.data(actor).unwrap().stats.clone();let revision=w.locations.revision(loc);
            for expected in [1,2] {
                w.submit(actor,search(loc,1)).unwrap();w.tick_with_random(&mut Never).unwrap();
                assert_eq!(count(&w,actor),expected);
                let events=w.drain_events();
                assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::PickableCollected{actor:a,item:1944,..}if *a==actor)).count(),1);
                assert!(events.iter().any(|e|matches!(e,WorldEvent::PlayerAnimation{actor:a,sequence:832}if *a==actor)));
                assert!(events.iter().any(|e|matches!(e,WorldEvent::PlayerSound{actor:a,sound:2581}if *a==actor)));
                assert!(events.iter().any(|e|matches!(e,WorldEvent::LocationMessage{actor:a,message:FOUND}if *a==actor)));
                for _ in 0..4 {w.tick_with_random(&mut Never).unwrap();}
                assert_eq!(count(&w,actor),expected);assert_eq!(w.data(actor).unwrap().stats,stats);
                assert_eq!(w.locations.revision(loc),revision);
            }
            w.submit(actor,search(loc,2)).unwrap();w.tick_with_random(&mut Never).unwrap();assert_eq!(count(&w,actor),2);
        }}}
    }
    #[test]
    fn chicken_coops_full_pack_refuses_without_presentation_or_reward_and_stale_generation_cancels() {
        for (_,id) in NODES {
            let(mut w,actor,loc)=setup(id,0,3200);
            for slot in 0..28 {w.data_mut(actor).unwrap().inventory.restore_slot(slot,Some(Item{id:1944,count:1,vars:0})).unwrap();}
            let before=w.data(actor).unwrap().inventory.clone();w.submit(actor,search(loc,1)).unwrap();w.tick_with_random(&mut Never).unwrap();
            assert_eq!(w.data(actor).unwrap().inventory.slots(),before.slots());let events=w.drain_events();
            assert!(events.iter().any(|e|matches!(e,WorldEvent::LocationMessage{message:FULL,..})));
            assert!(!events.iter().any(|e|matches!(e,WorldEvent::PickableCollected{..}|WorldEvent::PlayerAnimation{..}|WorldEvent::PlayerSound{..})));
            let(mut w,actor,loc)=setup(id,0,3197);w.submit(actor,search(loc,1)).unwrap();w.tick_with_random(&mut Never).unwrap();
            w.change_location(loc,false).unwrap();w.change_location(loc,true).unwrap();
            for _ in 0..8 {w.tick_with_random(&mut Never).unwrap();}assert_eq!(count(&w,actor),0);
        }
    }
    #[test]
    fn chicken_coops_zero_hp_actual_death_and_restored_visible_hp_do_not_grant() {
        use gielinor_world::hits::{Target,Hit,Style};
        for (_,id) in NODES {for phase in 0..3 {
            let(mut w,actor,loc)=setup(id,0,3200);
            if phase==0 {w.data_mut(actor).unwrap().stats.set_current(3,0);}else{
                w.queue_hit(Target::Player(actor),Hit{display:None,source:None,style:Style::Typeless,damage:99},0).unwrap();
                w.tick_with_random(&mut Never).unwrap();assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(a))if *a==actor)));
                if phase==2 {w.data_mut(actor).unwrap().stats.set_current(3,10);}
            }
            w.submit(actor,search(loc,1)).unwrap();for _ in 0..4 {w.tick_with_random(&mut Never).unwrap();}
            assert_eq!(count(&w,actor),0);
            assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::PickableCollected{..}|WorldEvent::PlayerAnimation{..}|WorldEvent::PlayerSound{..})));
        }}
    }
    #[test]
    fn chicken_coops_route_arrival_delay_and_late_capacity_or_replacement_walk() {
        for (_,id) in NODES {
            let(mut w,actor,loc)=setup(id,0,3197);w.submit(actor,search(loc,1)).unwrap();
            let mut arrival=None;let mut reward=None;
            for step in 1..=12 {
                w.tick_with_random(&mut Never).unwrap();
                if w.observe(actor).unwrap().coords==CoordGrid::new(3200,3200,0).unwrap() && arrival.is_none(){arrival=Some(step);}
                if count(&w,actor)>0 {reward=Some(step);break;}
            }
            assert_eq!(reward,arrival.map(|step|step+2));assert!(reward.is_some());
            for late_full in [false,true] {
                let(mut w,actor,loc)=setup(id,0,3197);w.submit(actor,search(loc,1)).unwrap();w.tick_with_random(&mut Never).unwrap();
                assert_eq!(count(&w,actor),0);
                if late_full {
                    for slot in 0..28 {w.data_mut(actor).unwrap().inventory.restore_slot(slot,Some(Item{id:1944,count:1,vars:0})).unwrap();}
                }else{w.submit(actor,Action::Walk{destination:CoordGrid::new(3196,3200,0).unwrap(),run:false}).unwrap();}
                for _ in 0..12 {w.tick_with_random(&mut Never).unwrap();}
                assert_eq!(count(&w,actor),if late_full{28}else{0});
                assert!(!w.drain_events().iter().any(|e|matches!(e,WorldEvent::PickableCollected{..})));
            }
        }
    }
}
