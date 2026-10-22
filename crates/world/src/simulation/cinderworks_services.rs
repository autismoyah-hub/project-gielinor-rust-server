//! Progress follows committed shared-engine outcomes, never a click or request.
use super::*;
use crate::tower_run::Service;
pub(super) fn credit(
    p: &mut Player,
    config: &crate::tower_cinderworks::Config,
    service: Service,
    events: &mut Vec<WorldEvent>,
) {
    if let Some(text) = config.credit(&mut p.data, p.movement.coords, service) {
        events.push(WorldEvent::CinderworksProgress {
            actor: p.id,
            run: p.data.tower_run,
        });
        events.push(WorldEvent::GameMessage {
            actor: p.id,
            text: text.into(),
        });
    }
}
pub(super) fn observe(
    p: &mut Player,
    config: Option<&crate::tower_cinderworks::Config>,
    from: usize,
    events: &mut Vec<WorldEvent>,
) {
    let Some(config) = config else {
        return;
    };
    let mut flags = 0;
    for event in &events[from..] {
        match event {
            WorldEvent::SmithingProductMade {
                actor,
                product,
                count,
            } if *actor == p.id && *count > 0 && config.forged_weapon(*product) => flags |= 1,
            WorldEvent::CookingMade { actor, trace } if *actor == p.id && !trace.outcome.burned => {
                if config.work.is_some_and(|w| {
                    w.cooking_loc == trace.job.target.id
                        && w.raw == trace.job.raw
                        && w.cooked == trace.outcome.output
                }) {
                    flags |= 2;
                }
            }
            _ => {}
        }
    }
    for (bit, service) in [(1, Service::Forge), (2, Service::Kitchen)] {
        if flags & bit != 0 {
            credit(p, config, service, events);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cooking::{Config as Cooking, Recipe},
        inventory::{Inventory, Item, StackType},
        loc_collision::LocCollision,
        locations::Location,
        npc::{BlockWalk, Mode, Npc, Restrict},
        tower_cinderworks::{Config, WorkConfig},
    };
    fn fixture(capacity: usize, burn: bool) -> (World, ActorId, NpcId, Location) {
        let mut w = World::default();
        let origin = crate::cinderworks_floor::origin();
        for x in (32..80).step_by(8) {
            for z in (32..80).step_by(8) {
                w.collision.allocate(origin.x() + x, origin.z() + z, 0);
            }
        }
        let at = origin.translate(56, 56, 0).unwrap();
        let loc = Location::new(at.translate(0, 1, 0).unwrap(), 5, 10, 0).unwrap();
        w.locations.load(loc);
        w.location_types.insert(
            5,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
        let content=Config {
            origin,
            spatial: crate::cinderworks_floor::Spatial::parse("CINDERWORKS_SPATIAL 1\nM 67 53 56 57\n").unwrap(),
            outside: CoordGrid::new(3225, 3218, 0).unwrap(),
            plaque: 1,
            portal: 2,
            chest: 3,
            tools: vec![],
            scimitar: 12,
            dagger: 13,
            coins: 995,
            beer: 1917,
            imp: 201,
            warden: 202,
            quench: 4,
            quench_tiles: vec![],
            warning_graphic: 86,
            strike_graphic: 78,
            death_animation: 836,
            charge_animations: [169, 4666],
            quench_animation: 4664,
            work: Some(WorkConfig {
                fishing_spot: 200,
                net: 303,
                raw: 317,
                cooked: 315,
                cooking_loc: 5,
                cooking_source: 6,
                fish_skill: 10,
                cook_skill: 7,
                fish_animation: 621,
                cook_animation: 897,
                queue: 123,
            }),
        };
        // Explicit synthetic curves isolate service credit; production keeps
        // its existing Wiki-backed recipes and random burn behavior.
        w.cooking = Some(Cooking {
            gauntlets: None,
            no_burn_cape: None,
            member_inputs: Default::default(),
            surfaces: Default::default(),
            fire: 5,
            animation: 897,
            skill: 7,
            max_level: 99,
            tutorial: None,
            recipes: [(
                317,
                Recipe {
                    raw: 317,
                    cooked: 315,
                    burnt: 323,
                    level: 1,
                    stop_burn: if burn { None } else { Some(1) },
                    low: 0,
                    high: 0,
                    xp: 30,
                    fine_xp_override: None,
                    ticks: 1,
                },
            )]
            .into(),
        });
        let cooking=w.cooking.as_mut().unwrap();
        cooking.fire=7;
        cooking.surfaces.insert(6,crate::cooking::Surface {animation:cooking.animation,recipes:cooking.recipes.clone()});
        w.configure_cinderworks(content).unwrap();
        w.held_use_types.insert(
            317,
            crate::held_use::Type {
                id: 317,
                content: 0,
            },
        );
        let mut data = PlayerData::new(1, "Net fixture".into(), at, Stats::default());
        assert!(data.tower_run.begin());
        data.inventory = Inventory::new(capacity, StackType::Normal, false);
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 303,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        let mut npc = Npc::new(
            200,
            at.translate(1, 0, 0).unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::None,
        )
        .unwrap();
        npc.mode = Mode::None;
        let target = w.spawn_npc(npc).unwrap();
        w.drain_events();
        (w, actor, target, loc)
    }
    fn shore_fixture(capacity:usize)->(World,ActorId,NpcId,CoordGrid) {
        let (mut w,actor,target,_)=fixture(capacity,false);
        let at=w.data(actor).unwrap().coords;
        let shore=at.translate(-1,0,0).unwrap();
        w.players.get_mut(actor.slot as usize).unwrap().movement.teleport(&mut w.collision,shore,w.clock.0,true).unwrap();
        let origin=w.cinderworks.as_ref().unwrap().origin;
        for x in at.x()..origin.x()+80 {for z in origin.z()+32..origin.z()+80 {w.collision.add(x,z,0,0x200000);}}
        w.npc_options.types.insert(200,crate::npc_options::Definition {operations:[Some("Net".into()),None,None,None,None],..Default::default()});
        assert_eq!(w.npc_handlers.get(&(200,1)),Some(&crate::npc_interaction::Handler::CinderworksFishing));
        w.drain_events();(w,actor,target,shore)
    }
    #[test]
    fn ordinary_scoped_net_action_catches_from_blocked_water_shore_without_walking_into_water() {
        let (mut w,actor,target,shore)=shore_fixture(2);
        assert!(!w.collision.can_step(shore,1,0,1,0,crate::collision::Strategy::Normal));
        w.submit(actor,Action::OperateNpc {target,operation:1,control:false}).unwrap();
        for _ in 0..7 {w.tick().unwrap();}
        assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id,317);
        assert!(w.data(actor).unwrap().stats.fine_xp(10)>0);
        assert_eq!(w.observe(actor).unwrap().coords,shore);
        assert!(!w.collision.can_step(shore,1,0,1,0,crate::collision::Strategy::Normal));
    }
    #[test]
    fn ordinary_shore_net_requires_scoped_live_target_and_inventory_capacity() {
        for mode in 0..4 {
            let (mut w,actor,target,_)=shore_fixture(if mode==3 {1}else{2});
            let before=w.data(actor).unwrap().clone();
            if mode==0 {w.cinderworks=None;}
            if mode==1 {w.cinderworks.as_mut().unwrap().work.as_mut().unwrap().fishing_spot=201;}
            w.submit(actor,Action::OperateNpc {target,operation:1,control:false}).unwrap();
            if mode==2 {w.despawn_npc(target);}
            for _ in 0..7 {w.tick().unwrap();}
            assert_eq!(w.data(actor).unwrap().inventory,before.inventory);
            assert_eq!(w.data(actor).unwrap().stats,before.stats);
        }
    }
    #[test]
    fn content_configuration_owns_net_operation_without_replacing_tutorial_or_live_handlers() {
        let (mut w,_,_,_)=fixture(2,false);let config=w.cinderworks.clone().unwrap();
        assert!(w.configure_cinderworks(config.clone()).is_err());
        let mut clean=World::default();clean.npc_handlers.insert((9478,1),crate::npc_interaction::Handler::TutorialFishing);
        clean.cooking=w.cooking.clone();clean.cooking.as_mut().unwrap().surfaces.remove(&5);
        clean.configure_cinderworks(config.clone()).unwrap();
        assert_eq!(clean.npc_handlers.get(&(9478,1)),Some(&crate::npc_interaction::Handler::TutorialFishing));
        assert_eq!(clean.npc_handlers.get(&(200,1)),Some(&crate::npc_interaction::Handler::CinderworksFishing));
        let mut conflict=World::default();conflict.npc_handlers.insert((200,1),crate::npc_interaction::Handler::Bank);
        assert!(conflict.configure_cinderworks(config).is_err());assert!(conflict.cinderworks.is_none());
        assert_eq!(conflict.npc_handlers.get(&(200,1)),Some(&crate::npc_interaction::Handler::Bank));
    }
    #[test]
    fn range_alias_preserves_source_and_configuration_failure_is_atomic() {
        let (w,_,_,_)=fixture(2,false);let config=w.cinderworks.clone().unwrap();
        assert_eq!(w.cooking.as_ref().unwrap().recipe(5,317).unwrap().xp,30);
        assert_eq!(w.cooking.as_ref().unwrap().recipe(6,317).unwrap().xp,30);
        for mode in 0..3 {
            let mut clean=World::default();clean.cooking=w.cooking.clone();
            if mode!=2 {clean.cooking.as_mut().unwrap().surfaces.remove(&5);}
            if mode==0 {clean.cooking.as_mut().unwrap().surfaces.remove(&6);}
            if mode==1 {clean.cooking.as_mut().unwrap().surfaces.get_mut(&6).unwrap().recipes.get_mut(&317).unwrap().cooked=999;}
            assert!(clean.configure_cinderworks(config.clone()).is_err());
            assert!(clean.cinderworks.is_none());assert!(!clean.npc_handlers.contains_key(&(200,1)));
            assert_eq!(clean.cooking.as_ref().unwrap().surfaces.contains_key(&5),mode==2);
        }
    }
    #[test]
    fn alias_cannot_cook_at_another_location_with_the_same_id() {
        let (mut w,actor,_,loc)=fixture(2,false);
        let elsewhere=Location::new(loc.coords.translate(1,0,0).unwrap(),loc.id,10,0).unwrap();w.locations.load(elsewhere);
        w.data_mut(actor).unwrap().inventory.restore_slot(1,Some(Item {id:317,count:1,vars:0})).unwrap();
        let before=w.data(actor).unwrap().clone();
        w.players.get_mut(actor.slot as usize).unwrap().location_revision=w.locations.revision(elsewhere);
        w.start_cooking(actor.slot as usize,elsewhere,1,317);
        for _ in 0..4 {w.tick().unwrap();}
        assert_eq!(w.data(actor).unwrap().inventory,before.inventory);assert_eq!(w.data(actor).unwrap().stats,before.stats);
        assert_eq!(w.data(actor).unwrap().tower_run.services,0);
    }
    #[test]
    fn full_inventory_and_stale_npc_do_not_create_fish_or_xp() {
        for stale in [false, true] {
            let (mut w, actor, target, _) = fixture(if stale { 2 } else { 1 }, false);
            let inventory = w.data(actor).unwrap().inventory.clone();
            let xp = w.data(actor).unwrap().stats.fine_xp(10);
            w.start_cinderworks_fishing(actor.slot as usize, target);
            if stale {
                w.despawn_npc(target);
            }
            for _ in 0..7 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(actor).unwrap().inventory, inventory);
            assert_eq!(w.data(actor).unwrap().stats.fine_xp(10), xp);
        }
    }
    #[test]
    fn caught_fish_uses_shared_cooking_and_credits_only_successful_kitchen_output() {
        let (mut w, actor, target, loc) = fixture(2, false);
        w.start_cinderworks_fishing(actor.slot as usize, target);
        for _ in 0..3 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(actor).unwrap().inventory.get(1).unwrap().id, 317);
        assert!(w.data(actor).unwrap().stats.fine_xp(10) > 0);
        w.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .location_revision = w.locations.revision(loc);
        w.submit(
            actor,
            Action::UseLocation {
                coords: loc.coords,
                id: loc.id,
                slot: 1,
                object: 317,
                control: false,
            },
        )
        .unwrap();
        for _ in 0..4 {
            w.tick().unwrap();
        }
        let data = w.data(actor).unwrap();
        assert_eq!(data.inventory.get(1).unwrap().id, 315);
        assert!(data.stats.fine_xp(7) > 0);
        assert_eq!(data.tower_run.services, 2);
    }
    #[test]
    fn a_real_burned_cooking_commit_does_not_restore_the_kitchen() {
        let (mut w, actor, _, loc) = fixture(2, true);
        w.data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(
                1,
                Some(Item {
                    id: 317,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .location_revision = w.locations.revision(loc);
        w.submit(
            actor,
            Action::UseLocation {
                coords: loc.coords,
                id: loc.id,
                slot: 1,
                object: 317,
                control: false,
            },
        )
        .unwrap();
        for _ in 0..4 {
            w.tick().unwrap();
        }
        let data = w.data(actor).unwrap();
        assert_eq!(data.inventory.get(1).unwrap().id, 323);
        assert_eq!(data.stats.fine_xp(7), 0);
        assert_eq!(data.tower_run.services, 0);
    }
    #[test]
    fn actual_queued_smelting_then_forging_restores_workshop_only_after_eligible_output() {
        let (mut w,actor,_,_)=fixture(28,false);
        for (slot,id) in [(1,436),(2,438),(3,2347)] {
            w.data_mut(actor).unwrap().inventory.restore_slot(slot,Some(Item {id,count:1,vars:0})).unwrap();
        }
        w.smithing=Some(crate::smithing_make::Config {queue:900,skill:13,max_level:99,xp_rate:1.0,tutorial:None});
        let request=|product,consumed,xp,tools|std::sync::Arc::new(crate::smithing_make::Request {
            product,produced:1,consumed,xp,animation:899,cycles:2,impurity:false,members:false,repeat_delay:None,required_level:1,required_tools:tools,
        });
        w.start_smithing(actor,request(2349,vec![(436,1),(438,1)],6.2,vec![]),1).unwrap();
        assert_eq!(w.data(actor).unwrap().tower_run.services,0);
        for _ in 0..4 {w.tick().unwrap();}
        assert_eq!(w.data(actor).unwrap().tower_run.services,0,"a bronze bar alone does not restore the workshop");
        assert_eq!(crate::smithing_make::total(&w.data(actor).unwrap().inventory,&w.object_types,2349),1);
        w.drain_events();
        w.start_smithing(actor,request(13,vec![(2349,1)],12.5,vec![2347]),1).unwrap();
        assert_eq!(w.data(actor).unwrap().tower_run.services,0,"UI/job admission does not credit a service");
        let mut progress=0;
        for _ in 0..4 {w.tick().unwrap();progress+=w.drain_events().iter().filter(|e|matches!(e,WorldEvent::CinderworksProgress {actor:a,..} if *a==actor)).count();}
        let data=w.data(actor).unwrap();
        assert_eq!(data.tower_run.services,Service::Forge as u8);assert_eq!(progress,1);
        assert_eq!(data.stats.fine_xp(13),187);
        for (id,count) in [(436,0),(438,0),(2349,0),(13,1),(2347,1),(303,1)] {
            assert_eq!(crate::smithing_make::total(&data.inventory,&w.object_types,id),count);
        }
    }
    #[test]
    fn forge_credit_does_not_manufacture_items_xp_or_accept_other_actor_events() {
        let (mut w, actor, _, _) = fixture(2, false);
        let config = w.cinderworks.clone().unwrap();
        let p = w.players.get_mut(actor.slot as usize).unwrap();
        let inventory = p.data.inventory.clone();
        let xp = p.data.stats.fine_xp(13);
        let mut events = vec![WorldEvent::SmithingProductMade {
            actor: ActorId {
                generation: actor.generation + 1,
                ..actor
            },
            product: 13,
            count: 1,
        }];
        observe(p, Some(&config), 0, &mut events);
        assert_eq!(p.data.tower_run.services, 0);
        events.push(WorldEvent::SmithingProductMade {
            actor,
            product: 13,
            count: 1,
        });
        observe(p, Some(&config), 1, &mut events);
        assert_eq!(p.data.tower_run.services, 1);
        assert_eq!(p.data.inventory, inventory);
        assert_eq!(p.data.stats.fine_xp(13), xp);
    }
    #[test]
    fn all_six_preparation_orders_require_real_committed_shared_outputs() {
        for order in [[0,1,2],[0,2,1],[1,0,2],[1,2,0],[2,0,1],[2,1,0]] {
            let (mut w,actor,_,range)=fixture(28,false);
            let at=w.data(actor).unwrap().coords;let origin=w.cinderworks.as_ref().unwrap().origin;
            let altar_at=w.cinderworks.as_ref().unwrap().spatial.at(origin,60,52).unwrap();
            let altar=Location::new(altar_at,8,10,0).unwrap();w.locations.load(altar);
            w.location_types.insert(8,w.location_types[&5]);
            w.prayer=Some(crate::prayer::Config {skill:5,animation:827,bones:[(526,4.5)].into(),necklace:None,restore:Default::default(),worn_xp_percent:Default::default(),chaos:None});
            w.tower_offering=Some(crate::tower_offering::HandlerConfig {location:8,origin,animation:827,accepted:[526,532],bones:crate::tower_offering::Bones {named:[526].into()},xp:[(526,Some(4.5))].into(),skills:[12,13,11],drop_bones:526});
            w.smithing=Some(crate::smithing_make::Config {queue:900,skill:13,max_level:99,xp_rate:1.0,tutorial:None});
            for (slot,id) in [(1,317),(2,526),(3,436),(4,438),(5,2347)] {
                w.data_mut(actor).unwrap().inventory.restore_slot(slot,Some(Item{id,count:1,vars:0})).unwrap();
            }
            w.held_use_types.insert(526,crate::held_use::Type{id:526,content:0});
            let mut expected=0;
            for service in order {
                let before=w.data(actor).unwrap().tower_run.services;
                if service==0 {
                    let request=|product,consumed,xp,tools|std::sync::Arc::new(crate::smithing_make::Request {product,produced:1,consumed,xp,animation:899,cycles:2,impurity:false,members:false,repeat_delay:None,required_level:1,required_tools:tools});
                    w.start_smithing(actor,request(2349,vec![(436,1),(438,1)],6.2,vec![]),1).unwrap();
                    for _ in 0..4 {w.tick().unwrap();}
                    assert_eq!(w.data(actor).unwrap().tower_run.services,before);
                    w.start_smithing(actor,request(13,vec![(2349,1)],12.5,vec![2347]),1).unwrap();
                } else {
                    let (target,slot,object)=if service==1 {(range,1,317)}else{(altar,2,526)};
                    let destination=if service==1 {at}else{altar_at.translate(0,1,0).unwrap()};
                    w.submit(actor,Action::Walk {destination,run:false}).unwrap();
                    for _ in 0..25 {w.tick().unwrap();}
                    assert_eq!(w.observe(actor).unwrap().coords,destination);
                    w.submit(actor,Action::UseLocation {coords:target.coords,id:target.id,slot,object,control:false}).unwrap();
                }
                assert_eq!(w.data(actor).unwrap().tower_run.services,before,"admission alone never restores a service");
                for _ in 0..12 {w.tick().unwrap();}
                expected|=1<<service;assert_eq!(w.data(actor).unwrap().tower_run.services,expected,"order {order:?}");
            }
            let data=w.data(actor).unwrap();assert_eq!(data.tower_run.services,7);
            assert_eq!(data.tower_floors.achieved(crate::tower_progression::FloorId::CINDERWORKS),7);
            for id in [317,526,436,438,2349] {assert_eq!(crate::smithing_make::total(&data.inventory,&w.object_types,id),0);}
            for id in [315,13,2347,303] {assert_eq!(crate::smithing_make::total(&data.inventory,&w.object_types,id),1);}
            assert!(data.stats.fine_xp(5)>0 && data.stats.fine_xp(7)>0 && data.stats.fine_xp(13)>0);
            let before=data.clone();
            w.submit(actor,Action::UseLocation {coords:altar.coords,id:altar.id,slot:2,object:526,control:false}).unwrap();
            for _ in 0..12 {w.tick().unwrap();}
            assert_eq!(w.data(actor).unwrap().inventory,before.inventory);assert_eq!(w.data(actor).unwrap().stats,before.stats);
            assert_eq!(w.data(actor).unwrap().tower_run.services,7);
        }
    }

}
