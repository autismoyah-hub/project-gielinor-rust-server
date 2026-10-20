use super::*;
use crate::{inventory::Insert, tower_run::Reward};
impl World {
    /// Called only after ordinary location routing and approach validation.
    pub(super) fn cinderworks_location(
        &mut self,
        slot: usize,
        loc: crate::locations::Location,
        operation: u8,
    ) -> bool {
        let Some(config) = self
            .cinderworks
            .clone()
            .filter(|c| c.handles(loc.id, operation))
        else {
            return false;
        };
        if loc.id==crate::tower_cinderworks::BANK_BOOTH && operation==2 {
            if self.cinderworks_bank(loc) {self.request_location_bank(slot,loc);}
            return true; // The same cache object elsewhere is not this authored bank.
        }
        if loc.id == config.quench {
            return self.quench_cinderworks(slot,loc);
        }
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        let inside = crate::tower_layout::contains(config.origin, p.movement.coords);
        if loc.id == crate::tower_cinderworks::COAL_BRAZIER {
            if !inside {
                return false;
            }
            self.events.push(WorldEvent::GameMessage{actor,text:if loc.coords.z()>=config.origin.z()+64 {
                "Banked coal feeds the foundry. Search a coolant barrel while the Warden charges to interrupt its heat."
            } else {"Steady coal flames mark the sanctuary route. The plaque lists the work needed to restore the Cinderworks."}.into()});
            return true;
        }
        if loc.id == config.portal {
            // Only the authored return portal and the Tower portal are entrances.
            if !inside && loc.coords != config.outside {
                return false;
            }
            let destination = if inside {
                config.outside.translate(-1, 0, 0).unwrap()
            } else {
                config.spatial.at(config.origin, 56, 56).unwrap()
            };
            match p
                .movement
                .teleport(&mut self.collision, destination, self.clock.0, true)
            {
                Ok(()) => {
                    p.data.coords = destination;
                    p.location_interaction = None;
                    p.npc_interaction = None;
                    self.events.push(WorldEvent::ClearMapFlag(actor));
                    self.events.push(WorldEvent::GameMessage { actor, text:if inside {
                        "You leave the Cinderworks. Your services, unlocks and pending reward are preserved. Use the nearby portal to return."
                    } else { "You return to the Cinderworks sanctuary. Read the plaque for your next step." }.into() });
                }
                Err(error) => self.events.push(WorldEvent::LocationError(error)),
            }
            return true;
        }
        if !inside {
            return false;
        }
        if loc.id == config.plaque {
            let mut staged = p.data.inventory.clone();
            let mut grants = false;
            for &tool in &config.tools {
                let owned = p
                    .data
                    .inventory
                    .slots()
                    .iter()
                    .chain(p.data.equipment.slots())
                    .chain(p.data.bank.slots())
                    .flatten()
                    .any(|item| item.id == tool);
                if !owned {
                    if staged
                        .insert(&self.object_types, Insert::new(tool, 1))
                        .is_err()
                    {
                        self.events.push(WorldEvent::GameMessage { actor, text:"Make room for the sanctuary's recovery tools, then read the plaque again.".into() });
                        return true;
                    }
                    grants = true;
                }
            }
            // Both run admission and recovery tools use the same acknowledged
            // snapshot boundary as claims. Never expose an unsaved starter kit.
            let mut run = p.data.tower_run;
            let began = run.begin();
            if began || grants {
                let message = format!(
                    "{}{}{}",
                    if began {
                        format!("Cinderworks run {} begins. ",run.id)
                    } else {
                        String::new()
                    },
                    if grants {
                        "The sanctuary provides missing tools. "
                    } else {
                        ""
                    },
                    run.guide()
                );
                self.checkpoint_cinderworks(actor, staged, run, message);
            } else {
                self.events.push(WorldEvent::GameMessage {
                    actor,
                    text: run.guide().into(),
                });
            }
            return true;
        }
        if p.data.tower_run.id == 0 {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: p.data.tower_run.guide().into(),
            });
            return true; // Explicit Cinderworks ownership must not reach legacy bounty.
        }
        if let Some(reward) = p.data.tower_run.reward() {
            let count = if reward == Reward::FirstClear {
                500
            } else {
                25
            };
            let mut inventory = p.data.inventory.clone();
            if inventory
                .insert(&self.object_types, Insert::new(config.coins, count))
                .is_err()
                || inventory
                    .insert(&self.object_types, Insert::new(config.beer, 1))
                    .is_err()
            {
                self.events.push(WorldEvent::GameMessage { actor, text:"Your reward is safe. Make room for coins and a beer, then search the chest again.".into() });
                return true;
            }
            let mut run = p.data.tower_run;
            if run.paid(run.id) {
                self.checkpoint_cinderworks(actor, inventory, run,
                    format!("The restored sanctuary yields {count} coins and a beer. The portal leads home; read the plaque to begin a replay."));
            }
        } else {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: p.data.tower_run.guide().into(),
            });
        }
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        loc_collision::LocCollision, locations::Location, tower_cinderworks::Config,
        tower_run::Service,
    };
    fn fixture(capacity: usize) -> (World, ActorId, [Location; 3]) {
        let mut world = World::default();
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        for x in (32..80).step_by(8) {
            for z in (32..80).step_by(8) {
                world.collision.allocate(origin.x() + x, origin.z() + z, 0);
            }
        }
        world.collision.allocate(3224, 3216, 0);
        let config = Config {
            origin,
            spatial: crate::cinderworks_floor::Spatial::default(),
            outside: CoordGrid::new(3225, 3218, 0).unwrap(),
            plaque: 1,
            portal: 2,
            chest: 3,
            tools: vec![10, 11],
            scimitar: 12,
            dagger: 13,
            coins: 100,
            beer: 101,
            imp: 200,
            warden: 201,
            quench: 4,
            quench_tiles: vec![],
            warning_graphic: 86,
            strike_graphic: 78,
            work: None,
            death_animation: 836,
            charge_animations: [169, 4666],
            quench_animation: 4664,
        };
        let locations = [
            Location::new(origin.translate(55, 56, 0).unwrap(), 1, 10, 0).unwrap(),
            Location::new(origin.translate(57, 56, 0).unwrap(), 2, 10, 0).unwrap(),
            Location::new(origin.translate(56, 57, 0).unwrap(), 3, 10, 0).unwrap(),
        ];
        for loc in locations
            .into_iter()
            .chain([Location::new(config.outside, 2, 10, 0).unwrap()])
        {
            let ty = LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: false,
                break_route_finding: false,
            };
            world.location_types.insert(loc.id, ty);
            world.locations.load(loc);
            crate::loc_collision::toggle(
                &mut world.collision,
                loc.coords,
                loc.shape,
                loc.angle,
                ty,
                true,
            )
            .unwrap();
        }
        world.object_types.stackable.insert(100);
        world.cinderworks = Some(config);
        let mut data = PlayerData::new(
            1,
            "Restorer".into(),
            origin.translate(56, 56, 0).unwrap(),
            Stats::default(),
        );
        data.inventory = Inventory::new(capacity, StackType::Normal, false);
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        (world, actor, locations)
    }
    fn operate(world: &mut World, actor: ActorId, loc: Location) {
        world
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .location_revision = world.locations.revision(loc);
        world
            .submit(
                actor,
                Action::OperateLocation {
                    coords: loc.coords,
                    id: loc.id,
                    operation: 1,
                    control: false,
                },
            )
            .unwrap();
        for _ in 0..3 {
            world.tick().unwrap();
        }
    }
    fn bank_fixture()->(World,ActorId,Location) {
        let (mut world,actor,_)=fixture(28);
        let config=world.cinderworks.as_ref().unwrap();
        let at=config.spatial.at(config.origin,53,60).unwrap();
        assert_eq!((at.x(),at.z()),(10417,114));
        let bank=Location::new(at,crate::tower_cinderworks::BANK_BOOTH,10,0).unwrap();
        let ty=LocCollision {width:1,length:1,block_walk:2,block_range:false,break_route_finding:false};
        world.location_types.insert(bank.id,ty);world.locations.load(bank);
        crate::loc_collision::toggle(&mut world.collision,at,10,0,ty,true).unwrap();
        world.drain_events();(world,actor,bank)
    }
    fn request_bank(world:&mut World,actor:ActorId,bank:Location) {
        world.submit(actor,Action::OperateLocation {coords:bank.coords,id:bank.id,operation:2,control:false}).unwrap();
    }
    #[test]
    fn authored_bank_uses_shared_reached_request_and_preserves_character_state() {
        let (mut world,actor,bank)=bank_fixture();
        let before=world.data(actor).unwrap().clone();
        assert!(!world.has_location_handler(bank.id,1));assert!(world.has_location_handler(bank.id,2));
        request_bank(&mut world,actor,bank);world.tick().unwrap();
        assert!(!world.drain_events().iter().any(|event|matches!(event,WorldEvent::LocationBankRequested {..})));
        let mut requests=vec![];
        for _ in 0..30 {
            world.tick().unwrap();
            for event in world.drain_events() {if let WorldEvent::LocationBankRequested {actor:a,request}=event {assert_eq!(a,actor);requests.push(request);}}
        }
        assert_eq!(requests.len(),1);let request=requests[0];
        assert_eq!(request.kind(),crate::bank_access::Kind::Full);
        assert!(world.location_bank_request_valid(actor,request));
        let after=world.data(actor).unwrap();
        assert_eq!(after.inventory,before.inventory);assert_eq!(after.bank,before.bank);
        assert_eq!(after.stats,before.stats);assert_eq!(after.tower_run,before.tower_run);
        let config=world.cinderworks.take();assert!(!world.location_bank_request_valid(actor,request));world.cinderworks=config;
        world.locations.delete(bank);assert!(!world.location_bank_request_valid(actor,request));
    }
    #[test]
    fn authored_bank_rejects_same_type_elsewhere_wrong_shape_and_removed_targets() {
        for removed in [false,true] {
            let (mut world,actor,bank)=bank_fixture();
            let other=Location::new(bank.coords.translate(3,0,0).unwrap(),bank.id,10,0).unwrap();
            world.locations.load(other);
            assert!(!world.cinderworks_bank(other));
            assert!(!world.cinderworks_bank(Location::new(bank.coords,bank.id,10,1).unwrap()));
            request_bank(&mut world,actor,if removed {bank}else{other});
            if removed {world.locations.delete(bank);}
            for _ in 0..30 {world.tick().unwrap();assert!(!world.drain_events().iter().any(|event|matches!(event,WorldEvent::LocationBankRequested {..})));}
        }
    }
    #[test]
    fn plaque_kit_is_atomic_and_repeated_reads_do_not_restart_active_run() {
        let (mut world, actor, locations) = fixture(1);
        operate(&mut world, actor, locations[0]);
        assert_eq!(world.data(actor).unwrap().tower_run.id, 0);
        assert_eq!(world.data(actor).unwrap().inventory.free_space(), 1);
        world.data_mut(actor).unwrap().inventory = Inventory::new(3, StackType::Normal, false);
        operate(&mut world, actor, locations[0]);
        assert_eq!(world.data(actor).unwrap().tower_run.id, 0);
        assert_eq!(world.data(actor).unwrap().inventory.free_space(), 3);
        assert!(world.cinderworks_checkpoint_pending(actor));
        world.cinderworks_checkpoint_saved(actor).unwrap();
        assert_eq!(world.data(actor).unwrap().tower_run.id, 1);
        assert_eq!(world.data(actor).unwrap().inventory.free_space(), 1);
        operate(&mut world, actor, locations[0]);
        assert_eq!(world.data(actor).unwrap().tower_run.id, 1);
        assert_eq!(world.data(actor).unwrap().inventory.free_space(), 1);
    }
    #[test]
    fn actual_first_and_maximum_replay_plaque_messages_fit_standard_game_chat() {
        for previous in [0,u32::MAX-1] {
            let (mut world,actor,locations)=fixture(28);
            if previous>0 {
                let run=&mut world.data_mut(actor).unwrap().tower_run;
                run.id=previous;run.victory=true;run.first_reward_paid=true;run.clears=1;
            }
            world.drain_events();
            operate(&mut world,actor,locations[0]);
            assert!(world.cinderworks_checkpoint_pending(actor));
            assert!(world.data(actor).unwrap().inventory.slots().iter().all(Option::is_none));
            assert!(!world.drain_events().iter().any(|event|matches!(event,WorldEvent::GameMessage {text,..} if text.starts_with("Cinderworks run"))));
            world.cinderworks_checkpoint_saved(actor).unwrap();
            let messages:Vec<_>=world.drain_events().into_iter().filter_map(|event|if let WorldEvent::GameMessage {text,..}=event {Some(text)}else{None}).collect();
            assert_eq!(messages.len(),1);
            assert!(messages[0].contains(&format!("Cinderworks run {} begins.",previous+1)));
            assert!(messages[0].contains("provides missing tools"));
            assert!(messages[0].contains("Restore services in any order"));
            // MESSAGE_GAME kind0/no sender reserves two prefix bytes and NUL.
            assert!(messages[0].is_ascii()&&messages[0].len()<=252,"standard byte-length chat limit: {}",messages[0].len());
            assert_eq!(world.data(actor).unwrap().inventory.free_space(),26);
        }
    }
    #[test]
    fn reward_retry_does_not_mint_and_success_pays_only_once() {
        let (mut world, actor, locations) = fixture(1);
        let run = &mut world.data_mut(actor).unwrap().tower_run;
        run.begin();
        for service in [Service::Forge, Service::Kitchen, Service::Altar] {
            run.service(service);
        }
        run.defeat_imp();
        run.defeat_warden();
        let pending = *run;
        for _ in 0..3 {
            operate(&mut world, actor, locations[2]);
            assert_eq!(world.data(actor).unwrap().tower_run, pending);
            assert_eq!(world.data(actor).unwrap().inventory.free_space(), 1);
        }
        world.data_mut(actor).unwrap().inventory = Inventory::new(2, StackType::Normal, false);
        operate(&mut world, actor, locations[2]);
        // The proposed items are still unavailable until durable acknowledgement.
        assert_eq!(world.data(actor).unwrap().tower_run, pending);
        assert_eq!(world.data(actor).unwrap().inventory.free_space(), 2);
        assert!(world.cinderworks_checkpoint_pending(actor));
        assert!(!world.movement_input_allowed(actor).unwrap());
        assert_eq!(
            world.submit(
                actor,
                Action::Walk {
                    destination: locations[0].coords,
                    run: false
                }
            ),
            Err(ActionError::LoggingOut)
        );
        world.request_logout(actor).unwrap();
        assert!(!world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::Save { .. })));
        world.cinderworks_checkpoint_saved(actor).unwrap();
        assert!(world.cinderworks_checkpoint_saved(actor).is_err());
        // Logout cannot overwrite the payment with an older snapshot.
        world.tick().unwrap();
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e,WorldEvent::Save{data,..} if data.tower_run.first_reward_paid)));
        let data = world.data(actor).unwrap();
        assert!(data.tower_run.first_reward_paid);
        assert!(!data.tower_run.reward_pending);
        assert_eq!(data.inventory.get(0).unwrap().count, 500);
        assert_eq!(data.inventory.get(1).unwrap().id, 101);
    }
    #[test]
    fn ordinary_exit_and_return_preserve_run_and_avoid_recovery_pad() {
        let (mut world, actor, locations) = fixture(3);
        operate(&mut world, actor, locations[0]);
        world.cinderworks_checkpoint_saved(actor).unwrap();
        let before = world.data(actor).unwrap().tower_run;
        operate(&mut world, actor, locations[1]);
        let outside = world.cinderworks.as_ref().unwrap().outside;
        assert_eq!(
            world.data(actor).unwrap().coords,
            outside.translate(-1, 0, 0).unwrap()
        );
        assert!(!crate::tower_arrival::recovery_pad(
            world.data(actor).unwrap().coords
        ));
        let portal = Location::new(outside, 2, 10, 0).unwrap();
        operate(&mut world, actor, portal);
        assert_eq!(
            world.data(actor).unwrap().coords,
            CoordGrid::new(10420, 110, 0).unwrap()
        );
        assert_eq!(world.data(actor).unwrap().tower_run, before);
    }
    #[test]
    fn unstarted_cinderworks_chest_is_handled_without_legacy_bounty_fallback() {
        let (mut world, actor, locations) = fixture(3);
        assert!(world.cinderworks_location(actor.slot as usize, locations[2], 1));
        assert_eq!(world.data(actor).unwrap().tower_run.id, 0);
        assert_eq!(world.data(actor).unwrap().inventory.free_space(), 3);
        assert!(!world.cinderworks_checkpoint_pending(actor));
    }
    #[test]
    fn replay_chest_awards_twenty_five_once_after_real_bank_space_and_save_ack() {
        let (mut world,actor,locations)=fixture(4);
        operate(&mut world,actor,locations[0]);world.cinderworks_checkpoint_saved(actor).unwrap();
        let complete=|world:&mut World| {
            let run=&mut world.data_mut(actor).unwrap().tower_run;
            for service in [Service::Forge,Service::Kitchen,Service::Altar] {assert!(run.service(service));}
            assert!(run.defeat_imp());assert!(run.defeat_warden());
        };
        // Explicit synthetic completed-run preconditions isolate reward policy;
        // actual earned native combat is covered separately by its receipts.
        complete(&mut world);operate(&mut world,actor,locations[2]);
        world.cinderworks_checkpoint_saved(actor).unwrap();
        assert_eq!(world.data(actor).unwrap().inventory.get(2).unwrap().count,500);
        let first=world.data(actor).unwrap().inventory.clone();
        operate(&mut world,actor,locations[0]);assert_eq!(world.data(actor).unwrap().tower_run.id,1);
        world.cinderworks_checkpoint_saved(actor).unwrap();
        assert_eq!(world.data(actor).unwrap().tower_run.id,2);assert_eq!(world.data(actor).unwrap().inventory,first);
        operate(&mut world,actor,locations[2]);assert_eq!(world.data(actor).unwrap().inventory,first,"unearned replay claim must not pay");
        complete(&mut world);let pending=world.data(actor).unwrap().tower_run;
        for _ in 0..3 {
            operate(&mut world,actor,locations[2]);assert_eq!(world.data(actor).unwrap().inventory,first);
            assert_eq!(world.data(actor).unwrap().tower_run,pending);assert!(!world.cinderworks_checkpoint_pending(actor));
        }
        // Coins can stack, but the extra beer still needs room: no partial25
        // credit may escape when the pack is full. Move an actual tool to bank.
        let data=&mut world.players.get_mut(actor.slot as usize).unwrap().data;
        crate::inventory::transfer(&mut data.inventory,&mut data.bank,&world.object_types,0,1,Insert::new(10,1),false).unwrap();
        let before=world.data(actor).unwrap().clone();operate(&mut world,actor,locations[2]);
        assert!(world.cinderworks_checkpoint_pending(actor));assert_eq!(world.data(actor).unwrap().inventory,before.inventory);
        assert_eq!(world.submit(actor,Action::Walk {destination:locations[0].coords,run:false}),Err(ActionError::LoggingOut));
        for _ in 0..8 {world.tick().unwrap();}
        assert_eq!(world.data(actor).unwrap().inventory,before.inventory,"unacknowledged replay grant is unavailable");
        world.cinderworks_checkpoint_saved(actor).unwrap();assert!(world.cinderworks_checkpoint_saved(actor).is_err());
        let paid=world.data(actor).unwrap().clone();
        assert_eq!(paid.inventory.get(2).unwrap().count,525);assert_eq!(paid.inventory.slots().iter().flatten().filter(|i|i.id==101).map(|i|i.count).sum::<i32>(),2);
        assert_eq!(paid.bank.slots().iter().flatten().filter(|i|i.id==10).map(|i|i.count).sum::<i32>(),1);
        assert_eq!(paid.tower_run.clears,2);assert!(paid.tower_run.first_reward_paid);assert!(!paid.tower_run.reward_pending);
        for _ in 0..3 {operate(&mut world,actor,locations[2]);assert_eq!(world.data(actor).unwrap().inventory,paid.inventory);assert_eq!(world.data(actor).unwrap().tower_run,paid.tower_run);}
    }

}
