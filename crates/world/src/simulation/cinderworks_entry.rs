//! Opt-in authored-floor arrival. Character possessions and legacy diary state
//! stay independent of the new run; recovery tools are claimed at the plaque.
use super::*;
impl World {
    /// Install explicit content operations before player admission. The ordinary
    /// tutorial fishing owner remains independent of the authored net spots.
    pub fn configure_cinderworks(&mut self,config:crate::tower_cinderworks::Config)->Result<(),&'static str> {
        if self.actors().next().is_some() || self.cinderworks.is_some() {return Err("Configure Cinderworks once before admission");}
        if let Some(work)=config.work {
            if self.npc_handlers.contains_key(&(work.fishing_spot,1)) {return Err("Cinderworks fishing operation already owned");}
            let cooking=self.cooking.as_mut().ok_or("Cinderworks requires shared cooking")?;
            if work.cooking_loc==cooking.fire || cooking.surfaces.contains_key(&work.cooking_loc) {
                return Err("Cinderworks cooking surface already owned");
            }
            let surface=cooking.surfaces.get(&work.cooking_source).ok_or("Cinderworks canonical range missing")?.clone();
            if cooking.skill!=work.cook_skill || !surface.recipes.get(&work.raw).is_some_and(|r|r.raw==work.raw && r.cooked==work.cooked) {
                return Err("Cinderworks cooking recipe mismatch");
            }
            config.spatial.at(config.origin,67,53).map_err(|_|"Invalid Cinderworks cooking site")?;
            // Commit only after all bindings validate. Keep ordinary range recipes,
            // timing, burns, membership and live equipment modifiers unchanged.
            if let Some(g)=cooking.gauntlets.as_mut() {
                if g.surfaces.contains(&work.cooking_source) {g.surfaces.insert(work.cooking_loc);}
            }
            cooking.surfaces.insert(work.cooking_loc,surface);
            self.npc_handlers.insert((work.fishing_spot,1),crate::npc_interaction::Handler::CinderworksFishing);
        }
        self.cinderworks=Some(config);Ok(())
    }
    pub(super) fn process_cinderworks_arrival(&mut self, slot: usize) {
        let Some(config) = self.cinderworks.as_ref() else {
            return;
        };
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        if !crate::tower_floor_catalogue::CINDERWORKS.definition.eligible(&p.data.tower_floors) {
            return;
        }
        let age = self.clock.0.wrapping_sub(p.login_cycle);
        if p.saving
            || p.pending_logout
            || p.cinderworks_checkpoint.is_some()
            || !(age == 0 || age == 2 || age > 0 && age % 8 == 0)
            || !crate::tower_arrival::recovery_pad(p.movement.coords)
        {
            return;
        }
        let Ok(destination) = config.spatial.at(config.origin, 56, 56) else {
            return;
        };
        if let Err(error) =
            p.movement
                .teleport(&mut self.collision, destination, self.clock.0, true)
        {
            self.events.push(WorldEvent::LocationError(error));
            return;
        }
        p.data.coords = destination;
        p.npc_interaction = None;
        p.location_interaction = None;
        p.object_interaction = None;
        if p.pending_location_action.take().is_some() {
            p.protection.suspended = false;
        }
        self.events.push(WorldEvent::ClearMapFlag(p.id));
        self.events.push(WorldEvent::GameMessage{actor:p.id,text:"You enter the Cinderworks sanctuary. Read the plaque for your restoration work and any missing tools.".into()});
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, tower_cinderworks::Config};
    fn fixture(allocate: bool) -> (World, ActorId) {
        let mut world = World::default();
        let origin = crate::cinderworks_floor::origin();
        let spatial = crate::cinderworks_floor::Spatial::default();
        let at = CoordGrid::new(3222, 3218, 0).unwrap();
        let spawn = spatial.at(origin, 56, 56).unwrap();
        world.collision.allocate(at.x(), at.z(), 0);
        if allocate {
            world.collision.allocate(spawn.x(), spawn.z(), 0);
        }
        world.cinderworks = Some(Config {
            origin,
            spatial,
            outside: CoordGrid::new(3225, 3218, 0).unwrap(),
            plaque: 1,
            portal: 2,
            chest: 3,
            tools: vec![10, 11],
            scimitar: 12,
            dagger: 13,
            coins: 995,
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
        });
        let mut data = PlayerData::new(1, "Restorer".into(), at, Stats::default());
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 995,
                    count: 123,
                    vars: 0,
                }),
            )
            .unwrap();
        data.equipment
            .restore_slot(
                0,
                Some(Item {
                    id: 100,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.bank
            .restore_slot(
                0,
                Some(Item {
                    id: 995,
                    count: 999,
                    vars: 0,
                }),
            )
            .unwrap();
        data.tower_arrival.staff_taken = true;
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        (world, actor)
    }
    #[test]
    fn authored_arrival_preserves_possessions_stats_legacy_flags_and_unstarted_run() {
        let (mut world, actor) = fixture(true);
        let before = world.data(actor).unwrap().clone();
        for _ in 0..4 {
            world.tick().unwrap();
        }
        let after = world.data(actor).unwrap();
        assert_eq!(after.coords, CoordGrid::new(10420, 110, 0).unwrap());
        assert_eq!(after.inventory, before.inventory);
        assert_eq!(after.equipment, before.equipment);
        assert_eq!(after.bank, before.bank);
        assert_eq!(after.tower_arrival, before.tower_arrival);
        assert_eq!(after.tower_run, before.tower_run);
        assert_eq!(after.chemistry, before.chemistry);
        for skill in 0..23 {
            assert_eq!(after.stats.fine_xp(skill), before.stats.fine_xp(skill));
            assert_eq!(after.stats.base(skill), before.stats.base(skill));
            assert_eq!(after.stats.current(skill), before.stats.current(skill));
        }
        assert!(!world.cinderworks_checkpoint_pending(actor));
    }
    #[test]
    fn missing_destination_does_not_move_strip_or_begin_the_character() {
        let (mut world, actor) = fixture(false);
        let before = world.data(actor).unwrap().clone();
        for _ in 0..4 {
            world.tick().unwrap();
        }
        let after = world.data(actor).unwrap();
        assert_eq!(after.coords, before.coords);
        assert_eq!(after.inventory, before.inventory);
        assert_eq!(after.equipment, before.equipment);
        assert_eq!(after.bank, before.bank);
        assert_eq!(after.tower_arrival, before.tower_arrival);
        assert_eq!(after.tower_run, before.tower_run);
    }
}
