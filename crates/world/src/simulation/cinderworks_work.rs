use super::*;
use crate::{
    inventory::{Delete, Insert},
    locations::Location,
    tower_run::Service,
};
#[derive(Clone, Copy)]
pub(super) enum Work {
    Fish { target: NpcId, coords: CoordGrid },
    Cook { location: Location },
}
impl World {
    pub(super) fn start_cinderworks_fishing(&mut self, slot: usize, target: NpcId) {
        let Some(config) = &self.cinderworks else {
            return;
        };
        let Some(work) = config.work else { return };
        let Some(npc) = self
            .npc(target)
            .filter(|npc| npc.type_id == work.fishing_spot)
        else {
            return;
        };
        let coords = npc.movement.coords;
        let p = self.players.get_mut(slot).unwrap();
        if p.data.tower_run.id == 0
            || !crate::tower_layout::contains(config.origin, p.movement.coords)
        {
            return;
        }
        if !p
            .data
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|item| item.id == work.net)
        {
            self.events.push(WorldEvent::GameMessage {actor:p.id,text:"Bring your small fishing net. Missing tools can be recovered from the sanctuary plaque.".into()});
            return;
        }
        p.queues.remove_all(work.queue);
        if p.queues
            .add(
                work.queue,
                crate::queues::Category::Weak,
                2,
                Deferred::Cinderworks(Work::Fish { target, coords }),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: work.fish_animation,
        });
        self.events.push(WorldEvent::GameMessage {
            actor: p.id,
            text: "You cast your net into the water. Moving away stops fishing.".into(),
        });
    }
    pub(super) fn start_cinderworks_cooking(
        &mut self,
        slot: usize,
        location: Location,
        object: i32,
    ) -> bool {
        let Some(config) = &self.cinderworks else {
            return false;
        };
        let Some(work) = config
            .work
            .filter(|work| work.cooking_loc == location.id && work.raw == object)
        else {
            return false;
        };
        let p = self.players.get_mut(slot).unwrap();
        if p.data.tower_run.id == 0
            || !crate::tower_layout::contains(config.origin, p.movement.coords)
        {
            return false;
        }
        p.queues.remove_all(work.queue);
        if p.queues
            .add(
                work.queue,
                crate::queues::Category::Weak,
                2,
                Deferred::Cinderworks(Work::Cook { location }),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return true;
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: work.cook_animation,
        });
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        loc_collision::LocCollision,
        npc::{BlockWalk, Mode, Npc, Restrict},
        tower_cinderworks::{Config, WorkConfig},
    };
    fn setup(capacity: usize) -> (World, ActorId, NpcId, Location) {
        let mut world = World::default();
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        world.collision.allocate(10424, 120, 0);
        let work = WorkConfig {
            fishing_spot: 200,
            net: 303,
            raw: 317,
            cooked: 315,
            cooking_loc: 5,
            fish_skill: 10,
            cook_skill: 7,
            fish_animation: 621,
            cook_animation: 897,
            queue: 999,
        };
        world.cinderworks = Some(Config {
            origin,
            outside: CoordGrid::new(3225, 3218, 0).unwrap(),
            plaque: 1,
            portal: 2,
            chest: 3,
            tools: vec![],
            scimitar: 12,
            dagger: 13,
            coins: 100,
            beer: 101,
            imp: 201,
            warden: 202,
            quench: 4,
            quench_tiles: vec![],
            warning_graphic: 86,
            strike_graphic: 78,
            work: Some(work),
            death_animation: 836,
            charge_animations:[169,4666],quench_animation:4664,
        });
        let mut data = PlayerData::new(
            1,
            "Fisher".into(),
            CoordGrid::new(10424, 120, 0).unwrap(),
            Stats::default(),
        );
        data.tower_run.begin();
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
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        let mut npc = Npc::new(
            200,
            CoordGrid::new(10425, 120, 0).unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::None,
        )
        .unwrap();
        npc.mode = Mode::None;
        let target = world.spawn_npc(npc).unwrap();
        let location = Location::new(CoordGrid::new(10424, 121, 0).unwrap(), 5, 10, 0).unwrap();
        world.location_types.insert(
            5,
            LocCollision {
                width: 1,
                length: 1,
                block_walk: 0,
                block_range: false,
                break_route_finding: false,
            },
        );
        world.locations.load(location);
        (world, actor, target, location)
    }
    #[test]
    fn full_pack_cannot_award_fishing_xp_and_stale_target_cancels_work() {
        for stale in [false, true] {
            let (mut world, actor, target, _) = setup(if stale { 2 } else { 1 });
            let inventory = world.data(actor).unwrap().inventory.clone();
            let xp = world.data(actor).unwrap().stats.fine_xp(10);
            world.start_cinderworks_fishing(actor.slot as usize, target);
            if stale {
                world.despawn_npc(target);
            }
            for _ in 0..6 {
                world.tick().unwrap();
            }
            assert_eq!(world.data(actor).unwrap().inventory, inventory);
            assert_eq!(world.data(actor).unwrap().stats.fine_xp(10), xp);
        }
    }
    #[test]
    fn caught_ingredients_cook_once_award_xp_and_credit_kitchen_only_on_output() {
        let (mut world, actor, target, location) = setup(2);
        world.start_cinderworks_fishing(actor.slot as usize, target);
        for _ in 0..4 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().inventory.get(1).unwrap().id, 317);
        assert!(world.data(actor).unwrap().stats.fine_xp(10) > 0);
        let before = world.data(actor).unwrap().stats.fine_xp(7);
        assert!(world.start_cinderworks_cooking(actor.slot as usize, location, 317));
        for _ in 0..4 {
            world.tick().unwrap();
        }
        let data = world.data(actor).unwrap();
        assert_eq!(data.inventory.get(1).unwrap().id, 315);
        assert_eq!(data.tower_run.services, 2);
        assert!(data.stats.fine_xp(7) > before);
        let inventory = data.inventory.clone();
        let xp = data.stats.fine_xp(7);
        for _ in 0..8 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().inventory, inventory);
        assert_eq!(world.data(actor).unwrap().stats.fine_xp(7), xp);
    }
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    config: &crate::tower_cinderworks::Config,
    job: Work,
    target_valid: bool,
    catalog: &crate::inventory::Catalog,
    tutorial_stats: bool,
    events: &mut Vec<WorldEvent>,
) -> Result<(), ActionError> {
    let Some(work) = config.work else {
        return Ok(());
    };
    if !target_valid
        || p.data.tower_run.id == 0
        || !crate::tower_layout::contains(config.origin, p.movement.coords)
    {
        return Ok(());
    }
    let mut inventory = p.data.inventory.clone();
    let (skill, xp, animation, message) = match job {
        Work::Fish { coords, .. } => {
            if p.movement.coords.chebyshev_distance(coords) > 3 {
                return Ok(());
            }
            if !inventory
                .slots()
                .iter()
                .flatten()
                .any(|item| item.id == work.net)
            {
                return Ok(());
            }
            if inventory.insert(catalog, Insert::new(work.raw, 1)).is_err() {
                events.push(WorldEvent::GameMessage {
                    actor: p.id,
                    text: "Your pack is full. No more shrimp can be caught.".into(),
                });
                return Ok(());
            }
            (
                work.fish_skill,
                10.0,
                work.fish_animation,
                "You catch some raw shrimp.",
            )
        }
        Work::Cook { location } => {
            if p.movement.coords.chebyshev_distance(location.coords) > 3 {
                return Ok(());
            }
            if !inventory
                .slots()
                .iter()
                .flatten()
                .any(|item| item.id == work.raw)
            {
                return Ok(());
            }
            if inventory.delete(catalog, Delete::new(work.raw, 1)).is_err()
                || inventory
                    .insert(catalog, Insert::new(work.cooked, 1))
                    .is_err()
            {
                events.push(WorldEvent::GameMessage {
                    actor: p.id,
                    text: "There is no room for the cooked shrimp.".into(),
                });
                return Ok(());
            }
            (
                work.cook_skill,
                30.0,
                work.cook_animation,
                "You cook some shrimp. Moving away stops cooking.",
            )
        }
    };
    let mut stats = p.data.stats.clone();
    let advancement = crate::skills::advance(
        &mut stats,
        skill,
        xp,
        p.data.xp_rate_hundreds as f64 / 100.0,
        99,
    )
    .map_err(|_| ActionError::InvalidStat)?;
    p.data.inventory = inventory;
    p.data.stats = stats;
    if tutorial_stats {
        p.stat_events
            .advance(&crate::tutorial_rules::bindings(), skill, advancement)
            .map_err(|_| ActionError::QueueFull)?;
    }
    events.push(WorldEvent::SkillAdvanced {
        actor: p.id,
        skill,
        advancement,
    });
    events.push(WorldEvent::PlayerCombatAnimation {
        actor: p.id,
        sequence: animation,
    });
    events.push(WorldEvent::GameMessage {
        actor: p.id,
        text: message.into(),
    });
    if matches!(job, Work::Cook { .. }) {
        if let Some(text) = config.credit(&mut p.data, p.movement.coords, Service::Kitchen) {
            events.push(WorldEvent::CinderworksProgress {actor:p.id,run:p.data.tower_run});
            events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: text.into(),
            });
        }
    }
    queues
        .add(
            work.queue,
            crate::queues::Category::Weak,
            4,
            Deferred::Cinderworks(job),
        )
        .map_err(|_| ActionError::QueueFull)
}
