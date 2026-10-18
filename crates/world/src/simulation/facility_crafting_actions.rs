use super::*;
use crate::facility_crafting::{Error, Pending, Request, QUEUE};
use crate::locations::Location;
impl World {
    pub(super) fn request_facility_crafting(
        &mut self,
        slot: usize,
        location: Location,
        selected: Option<(usize, i32)>,
    ) {
        let Some(config) = self.facility_crafting.get(&location.id) else {
            return;
        };
        let Some(revision) = self.locations.revision(location) else {
            return;
        };
        let p = self.players.get(slot).unwrap();
        if p.location_revision != Some(revision) || !config.locations.contains_key(&location.id) {
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
                r.input_count > 0
                    && selected.is_none_or(|(_, input)| input == r.input)
                    && p.data
                        .inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == r.input && i.count == 1 && i.vars == 0)
                        .count()
                        >= usize::from(r.input_count)
            })
            .collect();
        if recipes.is_empty() {
            self.events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: config.empty_message.into(),
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
            action: config.action,
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
            .get(&request.location.id)
            .ok_or(ActionError::InvalidLocation)?;
        if !(1..=28).contains(&count)
            || self.locations.revision(request.location) != Some(request.revision)
            || !config.locations.contains_key(&request.location.id)
            || request.animation != config.animation
            || request.action != config.action
            || p.movement.coords != request.origin
            || p.movement.last_movement != request.last_movement
        {
            return Err(ActionError::InvalidLocation);
        }
        let recipe = *request.recipes.get(index).ok_or(ActionError::InvalidItem)?;
        if !config.recipes.contains(&recipe) {
            return Err(ActionError::InvalidItem);
        }
        let groups = request
            .groups(recipe, &p.data.inventory, count as usize)
            .ok_or(ActionError::InvalidItem)?;
        if let Err(error) = crate::facility_crafting::prepare_group(
            recipe,
            &groups[0],
            &p.data.inventory,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
        ) {
            refusal(actor, error, request.action, &mut self.events);
            return Ok(());
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        let pending = Pending {
            request,
            recipe,
            groups: std::sync::Arc::new(groups),
            index: 0,
        };
        enqueue(&mut p.queues, &pending)?;
        animate(actor, &pending.request, &mut self.events);
        Ok(())
    }
}
fn refusal(actor: ActorId, error: Error, action: &str, events: &mut Vec<WorldEvent>) {
    let text = match error {
        Error::Members => format!("You need to be a member to {action} that item."),
        Error::Level => format!("You need a higher Crafting level to {action} that item."),
        _ => format!("You can't {action} that item right now."),
    };
    events.push(WorldEvent::GameMessage { actor, text });
}
fn enqueue(
    queues: &mut crate::queues::Scheduler<Deferred>,
    pending: &Pending,
) -> Result<(), ActionError> {
    let ticks = if pending.index == 0 {
        pending.recipe.first_ticks
    } else {
        pending.recipe.cycle_ticks
    };
    queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            i32::from(ticks) + 1,
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
    let Some(slots) = pending.groups.get(pending.index) else {
        return;
    };
    match crate::facility_crafting::prepare_group(
        pending.recipe,
        slots,
        &p.data.inventory,
        catalog,
        &p.data.stats,
        p.data.xp_rate_hundreds,
        p.member,
    ) {
        Err(error) => {
            refusal(p.id, error, pending.request.action, events);
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
                text: format!(
                    "You {} the {}.",
                    pending.request.action, pending.recipe.label
                ),
            });
        }
    }
    pending.index += 1;
    if !p.pending_logout && pending.index < pending.groups.len() {
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
    use crate::{
        facility_crafting::{Config, Recipe},
        inventory::Item,
        loc_collision::LocCollision,
    };
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
        w.facility_crafting.insert(
            14889,
            Config {
                locations: [(14889, 2)].into(),
                recipes: std::sync::Arc::new(vec![
                    Recipe {
                        input: 1737,
                        input_count: 1,
                        first_ticks: 3,
                        cycle_ticks: 3,
                        output_count: 1,
                        output: 1759,
                        level: 1,
                        xp_tenths: 25,
                        members: false,
                        label: "ball of wool",
                    },
                    Recipe {
                        input: 1779,
                        input_count: 1,
                        first_ticks: 3,
                        cycle_ticks: 3,
                        output_count: 1,
                        output: 1777,
                        level: 10,
                        xp_tenths: 150,
                        members: true,
                        label: "bow string",
                    },
                ]),
                action: "spin",
                empty_message: "No suitable materials.",
                animation: 894,
                location_animation: Some(466),
            },
        );
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
#[cfg(test)]
mod grouped_tests {
    use super::*;
    use crate::{
        facility_crafting::{Config, Recipe},
        inventory::Item,
        loc_collision::LocCollision,
    };
    fn fixture(basket: bool) -> (World, ActorId, Location, Request) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 8717, 10, 0).unwrap();
        let ty = LocCollision {
            width: 1,
            length: 1,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(8717, ty);
        w.locations.load(loc);
        crate::loc_collision::toggle(&mut w.collision, loc.coords, loc.shape, loc.angle, ty, true)
            .unwrap();
        let recipe = Recipe {
            input: if basket { 5933 } else { 1759 },
            input_count: if basket { 6 } else { 4 },
            output_count: 1,
            output: if basket { 5376 } else { 3224 },
            level: if basket { 36 } else { 10 },
            xp_tenths: if basket { 560 } else { 120 },
            members: true,
            first_ticks: 4,
            cycle_ticks: 3,
            label: if basket { "basket" } else { "strip of cloth" },
        };
        w.facility_crafting.insert(
            8717,
            Config {
                locations: [(8717, 2)].into(),
                recipes: std::sync::Arc::new(vec![recipe]),
                action: "weave",
                empty_message: "No suitable materials.",
                animation: 2270,
                location_animation: None,
            },
        );
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 36);
        stats.set_current(12, 36);
        let mut data = PlayerData::new(
            1,
            "Weaver".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        for slot in 0..28 {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if slot < 12 { recipe.input } else { 1511 },
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.held_use_types.insert(
            recipe.input,
            crate::held_use::Type {
                id: recipe.input,
                content: -1,
            },
        );
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        w.drain_events();
        w.submit(
            a,
            Action::UseLocation {
                coords: loc.coords,
                id: 8717,
                slot: 11,
                object: recipe.input,
                control: false,
            },
        )
        .unwrap();
        w.tick().unwrap();
        let req = w
            .drain_events()
            .into_iter()
            .find_map(|e| {
                if let WorldEvent::FacilityCraftingRequested { request, .. } = e {
                    Some(request)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(req.selected, Some((11, recipe.input)));
        assert_eq!(
            req.available(&w.data(a).unwrap().inventory),
            if basket { 2 } else { 3 }
        );
        (w, a, loc, req)
    }
    #[test]
    fn loom_actual_material_route_captures_whole_groups_and_uses_first_four_then_three() {
        for basket in [false, true] {
            let (mut w, a, _, req) = fixture(basket);
            let start = w.clock.0;
            let groups = req
                .groups(req.recipes[0], &w.data(a).unwrap().inventory, 28)
                .unwrap();
            assert_eq!(groups[0][0], 11);
            assert_eq!(groups[0].len(), if basket { 6 } else { 4 });
            w.start_facility_crafting(a, req, 0, 28).unwrap();
            let mut commits = Vec::new();
            let mut old = 0;
            for _ in 0..13 {
                w.tick().unwrap();
                let fine = w.data(a).unwrap().stats.fine_xp(12);
                if fine != old {
                    commits.push(w.clock.0 - 1);
                    old = fine;
                }
            }
            assert_eq!(
                commits,
                if basket {
                    vec![start + 4, start + 7]
                } else {
                    vec![start + 4, start + 7, start + 10]
                }
            );
            assert_eq!(old, if basket { 1400 } else { 450 });
            assert_eq!(
                w.data(a).unwrap().inventory.free_space(),
                if basket { 10 } else { 9 }
            );
        }
    }
    #[test]
    fn loom_missing_part_of_later_group_or_failed_credit_never_takes_partial_materials() {
        for case in 0..4 {
            let (mut w, a, loc, req) = fixture(false);
            w.start_facility_crafting(a, req, 0, 3).unwrap();
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 150);
            match case {
                0 => {
                    let inv = &mut w.data_mut(a).unwrap().inventory;
                    inv.restore_slot(4, None).unwrap();
                    inv.restore_slot(
                        27,
                        Some(Item {
                            id: 1759,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                }
                1 => {
                    w.object_types.dummy.insert(3224);
                }
                2 => {
                    w.locations.delete(loc);
                    w.locations.add(loc).unwrap();
                }
                _ => {
                    w.close_dialogs(a).unwrap();
                }
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..13 {
                w.tick().unwrap();
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
#[cfg(test)]
mod pottery_tests {
    use super::*;
    use crate::{
        facility_crafting::{Config, Recipe},
        inventory::Item,
        loc_collision::LocCollision,
    };
    #[test]
    fn pottery_actual_use_and_clay_route_credit_four_cups_only_when_whole_output_fits() {
        for space in [0, 3, 6] {
            let mut w = World::default();
            for x in (3192..3224).step_by(8) {
                for z in (3192..3224).step_by(8) {
                    w.collision.allocate(x, z, 0);
                }
            }
            let loc = Location::new(CoordGrid::new(3201, 3200, 0).unwrap(), 14887, 10, 0).unwrap();
            let ty = LocCollision {
                width: 1,
                length: 1,
                block_walk: 2,
                block_range: true,
                break_route_finding: false,
            };
            w.location_types.insert(14887, ty);
            w.locations.load(loc);
            crate::loc_collision::toggle(
                &mut w.collision,
                loc.coords,
                loc.shape,
                loc.angle,
                ty,
                true,
            )
            .unwrap();
            let recipes = vec![
                Recipe {
                    input: 1761,
                    input_count: 1,
                    output: 1787,
                    output_count: 1,
                    level: 1,
                    xp_tenths: 63,
                    members: false,
                    first_ticks: 2,
                    cycle_ticks: 3,
                    label: "Unfired pot",
                },
                Recipe {
                    input: 1761,
                    input_count: 1,
                    output: 28193,
                    output_count: 4,
                    level: 3,
                    xp_tenths: 85,
                    members: true,
                    first_ticks: 2,
                    cycle_ticks: 3,
                    label: "Unfired cup",
                },
            ];
            w.facility_crafting.insert(
                14887,
                Config {
                    locations: [(14887, 1)].into(),
                    recipes: std::sync::Arc::new(recipes),
                    action: "shape",
                    empty_message: "No clay.",
                    animation: 883,
                    location_animation: None,
                },
            );
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            stats.set_base(12, 3);
            stats.set_current(12, 3);
            let mut data = PlayerData::new(
                1,
                "Potter".into(),
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
            for slot in [19, 3, 11] {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1761,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            for &slot in [0, 1, 2, 4, 5, 6].iter().take(space) {
                data.inventory.restore_slot(slot, None).unwrap();
            }
            w.held_use_types.insert(
                1761,
                crate::held_use::Type {
                    id: 1761,
                    content: -1,
                },
            );
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, true).unwrap();
            w.drain_events();
            assert!(w.has_location_handler(14887, 1));
            let action = if space == 0 {
                Action::OperateLocation {
                    coords: loc.coords,
                    id: 14887,
                    operation: 1,
                    control: false,
                }
            } else {
                Action::UseLocation {
                    coords: loc.coords,
                    id: 14887,
                    slot: 19,
                    object: 1761,
                    control: false,
                }
            };
            w.submit(a, action).unwrap();
            w.tick().unwrap();
            let req = w
                .drain_events()
                .into_iter()
                .find_map(|e| {
                    if let WorldEvent::FacilityCraftingRequested { request, .. } = e {
                        Some(request)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(req.recipes.len(), 2);
            assert_eq!(req.available(&w.data(a).unwrap().inventory), 3);
            let before = w.data(a).unwrap().inventory.clone();
            w.start_facility_crafting(a, req, 1, 3).unwrap();
            for _ in 0..13 {
                w.tick().unwrap();
            }
            let d = w.data(a).unwrap();
            let made = space / 3;
            assert_eq!(d.stats.fine_xp(12), made as i32 * 106);
            assert_eq!(
                d.inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == 28193)
                    .count(),
                made * 4
            );
            if space == 0 {
                assert_eq!(d.inventory, before);
            } else {
                assert_eq!(d.inventory.get(11).unwrap().id, 1761);
                if space == 3 {
                    assert_eq!(d.inventory.get(3).unwrap().id, 1761);
                }
            }
        }
    }
}
