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
        w.cinderworks = Some(Config {
            origin,
            spatial: crate::cinderworks_floor::Spatial::default(),
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
                fish_skill: 10,
                cook_skill: 7,
                fish_animation: 621,
                cook_animation: 897,
                queue: 123,
            }),
        });
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
}
