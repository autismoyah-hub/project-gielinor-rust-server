use super::*;
use crate::facility_crafting::{Config, Error, Pending, Request, QUEUE};
use crate::locations::Location;
impl World {
    pub(super) fn request_facility_crafting(
        &mut self,
        slot: usize,
        location: Location,
        selected: Option<(usize, i32)>,
    ) {
        let Some(config) = &self.facility_crafting else {
            return;
        };
        let Some(revision) = self.locations.revision(location) else {
            return;
        };
        let p = self.players.get(slot).unwrap();
        if p.location_revision != Some(revision)
            || config.ticks == 0
            || !config.locations.contains_key(&location.id)
        {
            return;
        }
        if let Some((selected, input)) = selected {
            if p.data
                .inventory
                .get(selected)
                .is_none_or(|i| i.id != input || i.count != 1 || i.vars != 0)
            {
                return;
            }
        }
        let recipes: Vec<_> = config
            .recipes
            .iter()
            .copied()
            .filter(|r| {
                selected.is_none_or(|(_, input)| input == r.input)
                    && p.data
                        .inventory
                        .slots()
                        .iter()
                        .flatten()
                        .any(|i| i.id == r.input && i.count == 1 && i.vars == 0)
            })
            .collect();
        if recipes.is_empty() {
            self.events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: "You don't have anything suitable to spin at this spinning wheel.".into(),
            });
            return;
        }
        let request = Request {
            recipes: std::sync::Arc::new(recipes),
            location,
            revision,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            selected,
            ticks: config.ticks,
            animation: config.animation,
            location_animation: config.location_animation,
        };
        self.events.push(WorldEvent::FacilityCraftingRequested {
            actor: p.id,
            request,
        });
    }
    pub fn start_facility_crafting(
        &mut self,
        actor: ActorId,
        request: Request,
        index: usize,
        count: i32,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.pending_logout || p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        let config = self
            .facility_crafting
            .as_ref()
            .ok_or(ActionError::InvalidLocation)?;
        if !(1..=28).contains(&count)
            || request.ticks == 0
            || request.ticks != config.ticks
            || self.locations.revision(request.location) != Some(request.revision)
            || !config.locations.contains_key(&request.location.id)
            || p.movement.coords != request.origin
            || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidLocation);
        }
        let recipe = *request.recipes.get(index).ok_or(ActionError::InvalidItem)?;
        if !config.recipes.contains(&recipe) {
            return Err(ActionError::InvalidItem);
        }
        let slots = request
            .slots(recipe, &p.data.inventory, count as usize)
            .ok_or(ActionError::InvalidItem)?;
        if let Err(error) = crate::facility_crafting::prepare(
            recipe,
            slots[0],
            &p.data.inventory,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
        ) {
            refusal(actor, error, &mut self.events);
            return Ok(());
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        let pending = Pending {
            request,
            recipe,
            slots: std::sync::Arc::new(slots),
            index: 0,
        };
        enqueue(&mut p.queues, &pending)?;
        animate(actor, &pending.request, &mut self.events);
        Ok(())
    }
}
fn refusal(actor: ActorId, error: Error, events: &mut Vec<WorldEvent>) {
    events.push(WorldEvent::GameMessage {
        actor,
        text: match error {
            Error::Members => "You need to be a member to spin that item.",
            Error::Level => "You need a higher Crafting level to spin that item.",
            _ => "You can't spin that item right now.",
        }
        .into(),
    });
}
fn enqueue(
    queues: &mut crate::queues::Scheduler<Deferred>,
    pending: &Pending,
) -> Result<(), ActionError> {
    queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            i32::from(pending.request.ticks) + 1,
            Deferred::FacilityCrafting(pending.clone()),
        )
        .map_err(|_| ActionError::QueueFull)
}
fn animate(actor: ActorId, request: &Request, events: &mut Vec<WorldEvent>) {
    events.push(WorldEvent::PlayerCombatAnimation {
        actor,
        sequence: request.animation,
    });
    if let Some(sequence) = request.location_animation {
        events.push(WorldEvent::Location(crate::locations::Event::Animate {
            location: request.location,
            sequence,
        }));
    }
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    mut pending: Pending,
    catalog: &crate::inventory::Catalog,
    revision: Option<u64>,
    tutorial: bool,
    events: &mut Vec<WorldEvent>,
) {
    if revision != Some(pending.request.revision)
        || p.pending_logout
        || p.saving
        || p.has_modal
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || p.movement.coords != pending.request.origin
        || p.movement.last_movement != pending.request.last_movement
        || p.npc_interaction.is_some()
        || p.location_interaction.is_some()
        || p.object_interaction.is_some()
    {
        return;
    }
    let Some(&slot) = pending.slots.get(pending.index) else {
        return;
    };
    match crate::facility_crafting::prepare(
        pending.recipe,
        slot,
        &p.data.inventory,
        catalog,
        &p.data.stats,
        p.data.xp_rate_hundreds,
        p.member,
    ) {
        Err(error) => {
            refusal(p.id, error, events);
            return;
        }
        Ok((inventory, stats, advanced)) => {
            p.data.inventory = inventory;
            p.data.stats = stats;
            if let Some(advancement) = advanced {
                if tutorial
                    && p.stat_events
                        .advance(&crate::tutorial_rules::bindings(), 12, advancement)
                        .is_err()
                {
                    p.pending_logout = true;
                    events.push(WorldEvent::ActionRejected {
                        actor: p.id,
                        error: ActionError::QueueFull,
                    });
                }
                events.push(WorldEvent::SkillAdvanced {
                    actor: p.id,
                    skill: 12,
                    advancement,
                });
            }
            events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: format!("You spin the {}.", pending.recipe.label),
            });
        }
    }
    pending.index += 1;
    if !p.pending_logout && pending.index < pending.slots.len() {
        if let Err(error) = enqueue(queues, &pending) {
            events.push(WorldEvent::ActionRejected { actor: p.id, error });
        } else {
            animate(p.id, &pending.request, events);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{facility_crafting::Recipe, inventory::Item, loc_collision::LocCollision};
    struct NoDraw;
    impl RandomSource for NoDraw {
        fn below(&mut self, _: u32) -> u32 {
            panic!("Spinning must not draw RNG")
        }
    }
    fn fixture(member: bool) -> (World, ActorId, Location) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 14889, 10, 0).unwrap();
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(14889, ty);
        w.locations.load(loc);
        crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        w.facility_crafting = Some(Config {
            locations: [(14889, 2)].into(),
            recipes: std::sync::Arc::new(vec![
                Recipe {
                    input: 1737,
                    output: 1759,
                    level: 1,
                    xp_tenths: 25,
                    members: false,
                    label: "ball of wool",
                },
                Recipe {
                    input: 1779,
                    output: 1777,
                    level: 10,
                    xp_tenths: 150,
                    members: true,
                    label: "bow string",
                },
            ]),
            ticks: 3,
            animation: 894,
            location_animation: Some(466),
        });
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 10);
        stats.set_current(12, 10);
        let mut data = PlayerData::new(
            1,
            "Spinner".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 1511,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let input = if member { 1779 } else { 1737 };
        for slot in [19, 3, 11] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: input,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.held_use_types.insert(
            input,
            crate::held_use::Type {
                id: input,
                content: -1,
            },
        );
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, member).unwrap();
        w.drain_events();
        (w, a, loc)
    }
    fn request(w: &mut World, a: ActorId, loc: Location, material: bool, member: bool) -> Request {
        assert!(w.has_location_handler(14889, 2));
        assert!(!w.has_location_handler(14889, 1));
        let action = if material {
            Action::UseLocation {
                coords: loc.coords,
                id: loc.id,
                slot: 19,
                object: if member { 1779 } else { 1737 },
                control: false,
            }
        } else {
            Action::OperateLocation {
                coords: loc.coords,
                id: loc.id,
                operation: 2,
                control: false,
            }
        };
        w.submit(a, action).unwrap();
        w.tick().unwrap();
        let request = w
            .drain_events()
            .into_iter()
            .find_map(|e| {
                if let WorldEvent::FacilityCraftingRequested { request, .. } = e {
                    Some(request)
                } else {
                    None
                }
            })
            .expect("Reached spinning wheel must request actual quantity dialog");
        assert_eq!(request.available(&w.data(a).unwrap().inventory), 3);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
        request
    }
    #[test]
    fn facility_actual_wheel_and_material_routes_spin_full_bag_at_three_tick_intervals() {
        for material in [false, true] {
            let (mut w, a, loc) = fixture(false);
            let req = request(&mut w, a, loc, material, false);
            assert_eq!(req.selected, if material { Some((19, 1737)) } else { None });
            let start = w.clock.0;
            w.start_facility_crafting(a, req, 0, 28).unwrap();
            let mut commits = Vec::new();
            let mut fine = 0;
            for _ in 0..12 {
                w.tick_with_random(&mut NoDraw).unwrap();
                let now = w.data(a).unwrap().stats.fine_xp(12);
                if now != fine {
                    commits.push(w.clock.0 - 1);
                    if material && commits.len() == 1 {
                        assert_eq!(w.data(a).unwrap().inventory.get(19).unwrap().id, 1759);
                        assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().id, 1737);
                    }
                    fine = now;
                }
            }
            assert_eq!(commits, vec![start + 3, start + 6, start + 9]);
            assert_eq!(fine, 93);
            assert_eq!(w.data(a).unwrap().inventory.free_space(), 0);
            assert_eq!(
                w.data(a)
                    .unwrap()
                    .inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == 1759)
                    .count(),
                3
            );
            let events = w.drain_events();
            assert!(events
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 894, .. })));
            assert!(events.iter().any(|e| matches!(
                e,
                WorldEvent::Location(crate::locations::Event::Animate { sequence: 466, .. })
            )));
        }
    }
    #[test]
    fn facility_replaced_wheel_invalid_quantity_and_live_requirements_refuse_before_work() {
        for case in 0..7 {
            let (mut w, a, loc) = fixture(true);
            let req = request(&mut w, a, loc, true, true);
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            match case {
                0 => assert!(w.start_facility_crafting(a, req, 1, 1).is_err()),
                1 => assert!(w.start_facility_crafting(a, req, 0, 0).is_err()),
                2 => assert!(w.start_facility_crafting(a, req, 0, 29).is_err()),
                3 => {
                    w.locations.delete(loc);
                    w.locations.add(loc).unwrap();
                    assert!(w.start_facility_crafting(a, req, 0, 3).is_err());
                }
                4 => {
                    w.set_membership(a, false).unwrap();
                    w.start_facility_crafting(a, req, 0, 3).unwrap();
                }
                5 => {
                    w.data_mut(a).unwrap().xp_rate_hundreds = 0;
                    w.start_facility_crafting(a, req, 0, 3).unwrap();
                }
                _ => {
                    w.object_types.dummy.insert(1777);
                    w.start_facility_crafting(a, req, 0, 3).unwrap();
                }
            }
            for _ in 0..8 {
                w.tick_with_random(&mut NoDraw).unwrap();
            }
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone()
                ),
                before,
                "case{case}"
            );
        }
    }
    #[test]
    fn facility_later_target_revision_stale_slot_and_interrupts_preserve_committed_output() {
        for case in 0..8 {
            let (mut w, a, loc) = fixture(true);
            let req = request(&mut w, a, loc, true, true);
            w.start_facility_crafting(a, req, 0, 3).unwrap();
            for _ in 0..4 {
                w.tick_with_random(&mut NoDraw).unwrap();
            }
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 187);
            match case {
                0 => {
                    w.locations.delete(loc);
                    w.locations.add(loc).unwrap();
                }
                1 => {
                    let inv = &mut w.data_mut(a).unwrap().inventory;
                    inv.restore_slot(3, None).unwrap();
                    inv.restore_slot(
                        0,
                        Some(Item {
                            id: 1779,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                2 => {
                    w.close_dialogs(a).unwrap();
                }
                3 => {
                    w.set_modal_open(a, true).unwrap();
                    w.set_modal_open(a, false).unwrap();
                }
                4 => w.set_membership(a, false).unwrap(),
                5 => w.data_mut(a).unwrap().stats.set_current(12, 9),
                6 => w.data_mut(a).unwrap().xp_rate_hundreds = 0,
                _ => {
                    w.object_types.dummy.insert(1777);
                }
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..12 {
                w.tick_with_random(&mut NoDraw).unwrap();
            }
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone()
                ),
                before,
                "case{case}"
            );
        }
    }
}
