use super::*;
use crate::tool_crafting::{Error, Job, Pending, QUEUE};
impl World {
    pub(super) fn start_tool_crafting(
        &mut self,
        actor: ActorId,
        job: Job,
    ) -> Result<(), ActionError> {
        if job.recipe.ticks == 0 {
            return Err(ActionError::InvalidItem);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if let Err(e) = job.prepare(
            &p.data.inventory,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
        ) {
            refusal(&mut self.events, actor, e, job.recipe.skill);
            return Ok(());
        }
        p.queues.clear_weak();
        let pending = Pending {
            job,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            batch: None,
            worn_tool_slot:None,
            input_count:1,
        };
        // Input precedes queue processing, so compensate for this clock's decrement.
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                i32::from(job.recipe.ticks) + 1,
                Deferred::ToolCrafting(pending),
            )
            .map_err(|_| ActionError::QueueFull)?;
        if let Some(sequence) = job.recipe.exchange.animation {
            self.events
                .push(WorldEvent::PlayerCombatAnimation { actor, sequence });
        }
        Ok(())
    }
}
fn refusal(events: &mut Vec<WorldEvent>, actor: ActorId, error: Error, skill: u8) {
    let text = match (skill, error) {
        (11, Error::Members) => "You need to be a member to light that item.",
        (11, Error::Level) => "You need a higher Firemaking level to light that item.",
        (11, _) => "You can't light that item right now.",
        (_, Error::Members) => "You need to be a member to craft that item.",
        (_, Error::Level) => "You need a higher Crafting level to craft that item.",
        _ => "You can't craft that item right now.",
    };
    events.push(WorldEvent::GameMessage {
        actor,
        text: text.into(),
    });
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    pending: Pending,
    items: &crate::inventory::Catalog,

    tutorial: bool,
    events: &mut Vec<WorldEvent>,
    random: &mut impl RandomSource,
) {
    if p.pending_logout
        || p.saving
        || p.has_modal
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || p.movement.coords != pending.origin
        || p.movement.last_movement != pending.last_movement
        || p.npc_interaction.is_some()
        || p.location_interaction.is_some()
        || p.object_interaction.is_some()
    {
        return;
    }
    let mixed_parts=pending.batch.as_ref().and_then(|batch|batch.expected_ids.as_ref().and_then(|ids|{
        let end=batch.index.checked_add(batch.input_stride)?;
        let counts=batch.expected_counts.as_ref()?.get(batch.index..end)?;
        Some(ids.get(batch.index..end)?.iter().copied().zip(batch.slots.get(batch.index..end)?.iter().copied()).zip(counts.iter().copied()).map(|((id,slot),n)|(id,slot,n)).collect::<Vec<_>>())
    }));
    let result=if pending.batch.as_ref().is_some_and(|batch|batch.expected_ids.is_some()){mixed_parts.ok_or(Error::Policy).and_then(|parts|(if let Some(slot)=pending.worn_tool_slot{crate::tool_crafting_counts::prepare_material_counts_worn(pending.job,&parts,&p.data.inventory,&p.data.equipment,slot,items,&p.data.stats,p.data.xp_rate_hundreds,p.member)}else{crate::tool_crafting_counts::prepare_material_counts(pending.job,&parts,&p.data.inventory,items,&p.data.stats,p.data.xp_rate_hundreds,p.member)}).map(|prepared|(prepared,true)))}else if pending.input_count==1{pending.job.resolve(
        &p.data.inventory,
        items,
        &p.data.stats,
        p.data.xp_rate_hundreds,
        p.member,
        random,
    )}else{
        let slots=pending.batch.as_ref().and_then(|batch|batch.slots.get(batch.index..batch.index.saturating_add(batch.input_stride))).unwrap_or(std::slice::from_ref(&pending.job.input_slot));
        crate::tool_crafting_counts::prepare_with_slots(pending.job,pending.input_count,slots,&p.data.inventory,items,&p.data.stats,p.data.xp_rate_hundreds,p.member).map(|prepared|(prepared,true))
    };
    let committed = match result {
        Ok((prepared, success)) => {
            publish(p, pending.job, prepared, success, tutorial, events);
            true
        }
        Err(e) => {
            refusal(events, p.id, e, pending.job.recipe.skill);
            false
        }
    };
    if !committed || p.pending_logout {
        return;
    }
    if let Some(mut batch) = pending.batch {
        batch.index += batch.input_stride;
        if let Some(&input_slot) = batch.slots.get(batch.index) {
            let job = Job {
                input_slot,
                ..pending.job
            };
            let delay = i32::from(batch.cycle_ticks) + 1;
            let next = Pending {
                job,
                batch: Some(batch),
                worn_tool_slot:pending.worn_tool_slot,
                input_count:pending.input_count,
                origin: pending.origin,
                last_movement: pending.last_movement,
            };
            if queues
                .add(
                    QUEUE,
                    crate::queues::Category::Weak,
                    delay,
                    Deferred::ToolCrafting(next),
                )
                .is_err()
            {
                events.push(WorldEvent::ActionRejected {
                    actor: p.id,
                    error: ActionError::QueueFull,
                });
            } else if let Some(sequence) = job.recipe.exchange.animation {
                events.push(WorldEvent::PlayerCombatAnimation {
                    actor: p.id,
                    sequence,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        tool_crafting::Recipe,
    };
    fn fixture() -> (World, ActorId, Request) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 43);
        stats.set_current(12, 43);
        let mut data = PlayerData::new(
            1,
            "Crafter".into(),
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
        data.inventory
            .restore_slot(
                5,
                Some(Item {
                    id: 1755,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.inventory
            .restore_slot(
                19,
                Some(Item {
                    id: 1617,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        for id in [1755, 1617] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        w.held_use_handlers
            .register(
                Scope::Types,
                1755,
                1617,
                Handler::ToolCrafting(Recipe {
                    skill: 12,
                    exchange: crate::grinding::Recipe {
                        tool: 1755,
                        input: 1617,
                        output: 1601,
                        members: false,
                        animation: Some(8833),
                    },
                    level: 43,
                    xp_tenths: 1075,
                    ticks: 1,
                    name: "diamond",
                    action: "cut",
                    output_count: 1,
                    failure: None,
                }),
            )
            .unwrap();
        (
            w,
            a,
            Request {
                selected_slot: 19,
                selected_id: 1617,
                target_slot: 5,
                target_id: 1755,
            },
        )
    }
    fn select(w: &mut World, a: ActorId, r: Request) {
        assert!(w.prepare_held_use(a, r).unwrap().is_some());
        w.finish_held_use(a, r).unwrap();
    }
    #[test]
    fn gem_reverse_native_held_dispatch_keeps_tool_and_credits_selected_diamond_after_one_clock() {
        let (mut w, a, r) = fixture();
        select(&mut w, a, r);
        for _ in 0..1 {
            w.tick().unwrap();
            assert_eq!(w.data(a).unwrap().inventory.get(19).unwrap().id, 1617);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
        }
        w.tick().unwrap();
        let data = w.data(a).unwrap();
        assert_eq!(data.inventory.get(19).unwrap().id, 1601);
        assert_eq!(data.inventory.get(5).unwrap().id, 1755);
        assert_eq!(data.stats.fine_xp(12), 1343);
        w.tick().unwrap();
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 1343);
    }
    #[test]
    fn gem_changed_tool_level_modal_and_failed_credit_cancel_without_inventory_or_xp() {
        for case in 0..4 {
            let (mut w, a, r) = fixture();
            select(&mut w, a, r);
            match case {
                0 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        5,
                        Some(Item {
                            id: 1755,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap(),
                1 => w.data_mut(a).unwrap().stats.set_current(12, 42),
                2 => {
                    w.set_modal_open(a, true).unwrap();
                    w.set_modal_open(a, false).unwrap();
                }
                _ => {
                    w.object_types.dummy.insert(1601);
                }
            };
            for _ in 0..4 {
                w.tick().unwrap();
            }
            assert_eq!(
                w.data(a).unwrap().inventory.get(19).unwrap().id,
                1617,
                "case{case}"
            );
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0, "case{case}");
        }
    }
}

impl World {
    pub(super) fn request_tool_crafting(
        &mut self,
        actor: ActorId,
        recipes: std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,
        cycle_ticks: u8,
        request: crate::held_use::Request,
    ) -> Result<(), ActionError> {
        self.request_tool_crafting_counted(actor,recipes,cycle_ticks,request,1)
    }
    pub(super) fn request_tool_crafting_counted(&mut self,actor:ActorId,recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,request:crate::held_use::Request,input_count:u16)->Result<(),ActionError>{
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let Some(choice) = crate::tool_crafting::Request::bind_counted(
            recipes,
            cycle_ticks,
            request,
            &p.data.inventory,
            p.movement.coords,
            p.movement.last_movement,
            input_count,
        ) else {
            return Err(ActionError::InvalidItem);
        };
        if choice.recipes.iter().all(|r| r.exchange.members) && !p.member {
            refusal(
                &mut self.events,
                actor,
                Error::Members,
                choice.recipes[0].skill,
            );
            return Ok(());
        }
        p.tool_crafting_request = Some(choice);
        Ok(())
    }
    pub(super) fn request_tool_crafting_materials(&mut self,actor:ActorId,recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,request:crate::held_use::Request,input_counts:std::sync::Arc<Vec<u16>>)->Result<(),ActionError>{
        let p=self.players.get_mut(actor.slot as usize).unwrap();
        let choice=crate::tool_crafting::Request::bind_materials(recipes,cycle_ticks,request,&p.data.inventory,p.movement.coords,p.movement.last_movement,input_counts).ok_or(ActionError::InvalidItem)?;
        if choice.recipes.iter().all(|r|r.exchange.members)&&!p.member{refusal(&mut self.events,actor,Error::Members,choice.recipes[0].skill);return Ok(());}
        p.tool_crafting_request=Some(choice);Ok(())
    }
    pub(super) fn request_tool_crafting_equivalent_mixed(&mut self,actor:ActorId,recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,request:crate::held_use::Request,specs:crate::tool_crafting_materials::Specifications,tools:std::sync::Arc<Vec<i32>>,worn_tools:std::sync::Arc<Vec<(i32,usize)>>)->Result<(),ActionError>{
        if tools.is_empty()||tools.len()>18||tools.iter().copied().collect::<std::collections::BTreeSet<_>>().len()!=tools.len()||tools.iter().any(|&id|!self.object_types.ordinary_nonstackable(id))||recipes.iter().any(|r|r.exchange.tool!=tools[0])||worn_tools.iter().any(|&(id,slot)|!tools.contains(&id)||slot>=15)||worn_tools.iter().copied().collect::<std::collections::BTreeSet<_>>().len()!=worn_tools.len(){return Err(ActionError::InvalidItem);}
        let p=self.player(actor)?;
        let held=[request.selected_id,request.target_id].into_iter().find(|id|tools.contains(id)).or_else(||tools.iter().copied().find(|id|p.data.inventory.slots().iter().flatten().any(|i|i.id==*id&&i.count==1&&i.vars==0)));
        let(selected,worn_slot)=if let Some(id)=held{(id,None)}else{let(id,slot)=worn_tools.iter().copied().find(|&(id,slot)|p.data.equipment.get(slot).is_some_and(|i|i.id==id&&i.count==1&&i.vars==0)).ok_or(ActionError::InvalidItem)?;(id,Some(slot))};
        let recipes=std::sync::Arc::new(recipes.iter().map(|r|{let mut r=*r;r.exchange.tool=selected;r}).collect());
        if let Some(slot)=worn_slot{
            let p=self.players.get_mut(actor.slot as usize).unwrap();
            let choice=crate::tool_crafting_materials::bind_worn(recipes,cycle_ticks,request,&p.data.inventory,&self.object_types,&p.data.equipment,slot,p.movement.coords,p.movement.last_movement,specs).ok_or(ActionError::InvalidItem)?;
            if choice.recipes.iter().all(|r|r.exchange.members)&&!p.member{refusal(&mut self.events,actor,Error::Members,choice.recipes[0].skill);return Ok(());}
            p.tool_crafting_request=Some(choice);Ok(())
        }else{self.request_tool_crafting_mixed(actor,recipes,cycle_ticks,request,specs)}
    }
    pub(super) fn request_tool_crafting_mixed(&mut self,actor:ActorId,recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,request:crate::held_use::Request,specs:crate::tool_crafting_materials::Specifications)->Result<(),ActionError>{
        let p=self.players.get_mut(actor.slot as usize).unwrap();
        let choice=crate::tool_crafting_materials::bind(recipes,cycle_ticks,request,&p.data.inventory,&self.object_types,p.movement.coords,p.movement.last_movement,specs).ok_or(ActionError::InvalidItem)?;
        if choice.recipes.iter().all(|r|r.exchange.members)&&!p.member{refusal(&mut self.events,actor,Error::Members,choice.recipes[0].skill);return Ok(());}
        p.tool_crafting_request=Some(choice);Ok(())
    }
    pub fn take_tool_crafting_request(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<crate::tool_crafting::Request>, ActionError> {
        self.player(actor)?;
        Ok(self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .tool_crafting_request
            .take())
    }
    pub fn start_tool_crafting_choice(
        &mut self,
        actor: ActorId,
        request: crate::tool_crafting::Request,
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
        if !(1..=28).contains(&count)
            || p.movement.coords != request.origin
            || p.movement.last_movement != request.last_movement
            || request.cycle_ticks == 0
        {
            return Err(ActionError::InvalidItem);
        }
        let recipe = *request.recipes.get(index).ok_or(ActionError::InvalidItem)?;
        if recipe.ticks == 0 {
            return Err(ActionError::InvalidItem);
        }
        let mut job = Job {
            recipe,
            tool_slot: request.tool_slot,
            input_slot: request.input_slot,
        };
        let input_count=request.input_count_for(index).ok_or(ActionError::InvalidItem)?;
        let (slots,input_stride,expected_ids,expected_counts)=if request.material_specs.is_some(){
            let capture=crate::tool_crafting_materials::capture(&request,index,&p.data.inventory,count as usize).ok_or(ActionError::InvalidItem)?;
            job.input_slot=*capture.slots.first().ok_or(ActionError::InvalidItem)?;
            let parts=capture.ids.iter().copied().zip(capture.slots.iter().copied()).zip(capture.counts.iter().copied()).take(capture.stride).map(|((id,slot),n)|(id,slot,n)).collect::<Vec<_>>();
            let prepared=if let Some(slot)=request.worn_tool_slot{crate::tool_crafting_counts::prepare_material_counts_worn(job,&parts,&p.data.inventory,&p.data.equipment,slot,&self.object_types,&p.data.stats,p.data.xp_rate_hundreds,p.member)}else{crate::tool_crafting_counts::prepare_material_counts(job,&parts,&p.data.inventory,&self.object_types,&p.data.stats,p.data.xp_rate_hundreds,p.member)};
            if let Err(e)=prepared{refusal(&mut self.events,actor,e,job.recipe.skill);return Ok(());}
            (capture.slots,capture.stride,Some(capture.ids),Some(capture.counts))
        }else{
        let mut slots = vec![request.input_slot];
        for (slot, item) in p.data.inventory.slots().iter().enumerate() {
            if slot != request.input_slot
                && slot != request.tool_slot
                && item
                    .is_some_and(|i| i.id == recipe.exchange.input && i.count == 1 && i.vars == 0)
            {
                slots.push(slot);
            }
        }
        let input_stride=if input_count>1&&!self.object_types.stackable.contains(&recipe.exchange.input){usize::from(input_count)}else{1};
        if input_count>1&&input_stride==1{slots=vec![request.input_slot;request.available_for(index,&p.data.inventory).min(count as usize)];}
        let jobs=(slots.len()/input_stride).min(count as usize);slots.truncate(jobs*input_stride);
        let first_slots=slots.get(..input_stride).ok_or(ActionError::InvalidItem)?;
        if let Err(e)=crate::tool_crafting_counts::prepare_with_slots(job,input_count,first_slots,&p.data.inventory,&self.object_types,&p.data.stats,p.data.xp_rate_hundreds,p.member){refusal(&mut self.events,actor,e,job.recipe.skill);return Ok(());}
            (slots,input_stride,None,None)
        };
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        let pending = Pending {
            job,
            input_count,
            worn_tool_slot:request.worn_tool_slot,
            origin: request.origin,
            last_movement: request.last_movement,
            batch: Some(crate::tool_crafting::Batch {
                input_stride,
                expected_ids,
                expected_counts,
                slots: std::sync::Arc::new(slots),
                index: 0,
                cycle_ticks: request.cycle_ticks,
            }),
        };
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                i32::from(recipe.ticks) + 1,
                Deferred::ToolCrafting(pending),
            )
            .map_err(|_| ActionError::QueueFull)?;
        if let Some(sequence) = recipe.exchange.animation {
            self.events
                .push(WorldEvent::PlayerCombatAnimation { actor, sequence });
        }
        Ok(())
    }
}

#[cfg(test)]
mod choice_tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request as Held, Scope, Type},
        inventory::Item,
        tool_crafting::{Recipe, Request},
    };
    fn fixture() -> (World, ActorId, Request) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 33);
        stats.set_current(12, 33);
        let mut data = PlayerData::new(
            1,
            "Glass maker".into(),
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
        for (slot, id) in [(0, 1785), (17, 1775), (3, 1775), (8, 1775)] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        for id in [1785, 1775] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        let recipes = std::sync::Arc::new(vec![
            Recipe {
                skill: 12,
                exchange: crate::grinding::Recipe {
                    tool: 1785,
                    input: 1775,
                    output: 1919,
                    members: true,
                    animation: Some(884),
                },
                level: 1,
                xp_tenths: 175,
                ticks: 2,
                name: "Beer glass",
                action: "blow",
                output_count: 1,
                failure: None,
            },
            Recipe {
                skill: 12,
                exchange: crate::grinding::Recipe {
                    tool: 1785,
                    input: 1775,
                    output: 229,
                    members: true,
                    animation: Some(884),
                },
                level: 33,
                xp_tenths: 350,
                ticks: 2,
                name: "Vial",
                action: "blow",
                output_count: 1,
                failure: None,
            },
        ]);
        w.held_use_handlers
            .register(
                Scope::Types,
                1785,
                1775,
                Handler::ToolCraftingChoices {
                    recipes,
                    cycle_ticks: 3,
                },
            )
            .unwrap();
        let held = Held {
            selected_slot: 17,
            selected_id: 1775,
            target_slot: 0,
            target_id: 1785,
        };
        w.prepare_held_use(a, held).unwrap();
        w.finish_held_use(a, held).unwrap();
        let req = w.take_tool_crafting_request(a).unwrap().unwrap();
        assert_eq!(req.available(&w.data(a).unwrap().inventory), 3);
        assert_eq!(req.input_slot, 17);
        (w, a, req)
    }
    fn count(w: &World, a: ActorId, id: i32) -> usize {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|v| v.id == id)
            .count()
    }
    #[test]
    fn glass_choices_capture_selected_input_and_batch_at_two_then_three_clock_intervals() {
        let (mut w, a, req) = fixture();
        let start = w.clock.0;
        w.start_tool_crafting_choice(a, req, 0, 28).unwrap();
        let mut commits = Vec::new();
        let mut old = 0;
        for _ in 0..13 {
            w.tick().unwrap();
            let now = count(&w, a, 1919);
            if now != old {
                assert_eq!(now, old + 1);
                commits.push(w.clock.0 - 1);
                if now == 1 {
                    assert_eq!(w.data(a).unwrap().inventory.get(17).unwrap().id, 1919);
                    assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().id, 1775);
                }
            }
            old = now;
        }
        assert_eq!(commits, vec![start + 2, start + 5, start + 8]);
        assert_eq!(old, 3);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 654);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 1785);
        assert_eq!(count(&w, a, 1775), 0);
        assert_eq!(w.data(a).unwrap().inventory.free_space(), 0);
    }
    #[test]
    fn glass_choice_invalid_indices_counts_and_live_eligibility_preserve_materials() {
        for case in 0..5 {
            let (mut w, a, req) = fixture();
            let before = w.data(a).unwrap().inventory.clone();
            match case {
                0 => assert!(w.start_tool_crafting_choice(a, req, 2, 1).is_err()),
                1 => assert!(w.start_tool_crafting_choice(a, req, 0, 0).is_err()),
                2 => assert!(w.start_tool_crafting_choice(a, req, 0, 29).is_err()),
                3 => {
                    w.set_membership(a, false).unwrap();
                    w.start_tool_crafting_choice(a, req, 0, 1).unwrap();
                }
                _ => {
                    w.data_mut(a).unwrap().stats.set_current(12, 32);
                    w.start_tool_crafting_choice(a, req, 1, 1).unwrap();
                }
            }
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(a).unwrap().inventory, before, "case{case}");
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0, "case{case}");
        }
    }
    #[test]
    fn glass_batch_later_stale_slot_membership_tool_and_close_stop_without_new_loss() {
        for case in 0..4 {
            let (mut w, a, req) = fixture();
            w.start_tool_crafting_choice(a, req, 0, 3).unwrap();
            for _ in 0..3 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, a, 1919), 1);
            match case {
                0 => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(3, None)
                        .unwrap();
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            5,
                            Some(Item {
                                id: 1775,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                1 => w.set_membership(a, false).unwrap(),
                2 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 1785,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap(),
                _ => {
                    w.close_dialogs(a).unwrap();
                }
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..10 {
                w.tick().unwrap();
            }
            assert_eq!(count(&w, a, 1919), 1, "case{case}");
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
mod prior_production_tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request as Held, Scope, Type},
        inventory::Item,
        tool_crafting::Recipe,
    };
    #[test]
    fn legacy_tool_production_single_option_preserves_gem_f2p_and_shell_member_cadences() {
        for (input, output, level, xp, first, cycle, member, action, expected, total) in [
            (1621, 1605, 27, 675, 1, 2, false, "cut", vec![1, 3, 5], 2529),
            (
                3345,
                3327,
                15,
                325,
                2,
                3,
                true,
                "carve",
                vec![2, 5, 8],
                1218,
            ),
        ] {
            let mut w = World::default();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            stats.set_base(12, level);
            stats.set_current(12, level);
            let mut data = PlayerData::new(
                1,
                "Tool production".into(),
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
            for (slot, id) in [(5, 1755), (21, input), (3, input), (11, input)] {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, member).unwrap();
            for id in [1755, input] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            let recipe = Recipe {
                skill: 12,
                exchange: crate::grinding::Recipe {
                    tool: 1755,
                    input,
                    output,
                    members: member,
                    animation: None,
                },
                level: level as u8,
                xp_tenths: xp,
                ticks: first,
                name: "product",
                action,
                output_count: 1,
                failure: None,
            };
            w.held_use_handlers
                .register(
                    Scope::Types,
                    1755,
                    input,
                    Handler::ToolCraftingChoices {
                        recipes: std::sync::Arc::new(vec![recipe]),
                        cycle_ticks: cycle,
                    },
                )
                .unwrap();
            let request = Held {
                selected_slot: 21,
                selected_id: input,
                target_slot: 5,
                target_id: 1755,
            };
            w.prepare_held_use(a, request).unwrap();
            w.finish_held_use(a, request).unwrap();
            let choices = w.take_tool_crafting_request(a).unwrap().unwrap();
            assert_eq!(choices.recipes.len(), 1);
            assert_eq!(choices.available(&w.data(a).unwrap().inventory), 3);
            let start = w.clock.0;
            w.start_tool_crafting_choice(a, choices, 0, 3).unwrap();
            let mut commits = Vec::new();
            let mut previous = 0;
            for _ in 0..12 {
                w.tick().unwrap();
                let count = w
                    .data(a)
                    .unwrap()
                    .inventory
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|v| v.id == output)
                    .count();
                if count != previous {
                    commits.push(w.clock.0 - 1 - start);
                    if count == 1 {
                        assert_eq!(w.data(a).unwrap().inventory.get(21).unwrap().id, output);
                        assert_eq!(w.data(a).unwrap().inventory.get(3).unwrap().id, input);
                    }
                }
                previous = count;
            }
            assert_eq!(commits, expected);
            assert_eq!(previous, 3);
            assert_eq!(w.data(a).unwrap().stats.fine_xp(12), total);
            assert_eq!(w.data(a).unwrap().inventory.get(5).unwrap().id, 1755);
            assert_eq!(w.data(a).unwrap().inventory.free_space(), 0);
        }
    }
}

#[cfg(test)]
mod fallible_world_tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request as Held, Scope, Type},
        inventory::Item,
        tool_crafting::{Failure, Recipe},
    };
    struct Draw {
        calls: usize,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            if n == 256 {
                let value = [0, 255, 0][self.calls];
                self.calls += 1;
                value
            } else {
                0
            }
        }
    }
    #[test]
    fn semiprecious_actual_choice_batch_mixes_cut_crush_and_stops_stale_without_a_draw() {
        for stale in [false, true] {
            let mut w = World::default();
            w.collision.allocate(3200, 3200, 0);
            let mut stats = Stats::default();
            stats.set_base(3, 10);
            stats.set_current(3, 10);
            let mut data = PlayerData::new(
                1,
                "Gem cutter".into(),
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
            for (slot, id) in [(0, 1755), (17, 1625), (3, 1625), (8, 1625)] {
                data.inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let a = w.actors().next().unwrap();
            w.set_membership(a, true).unwrap();
            for id in [1755, 1625] {
                w.held_use_types.insert(id, Type { id, content: -1 });
            }
            let recipe = Recipe {
                skill: 12,
                exchange: crate::grinding::Recipe {
                    tool: 1755,
                    input: 1625,
                    output: 1609,
                    members: true,
                    animation: Some(890),
                },
                level: 1,
                xp_tenths: 150,
                ticks: 1,
                name: "Opal",
                action: "cut",
                output_count: 1,
                failure: Some(Failure {
                    output: 1633,
                    xp_tenths: 38,
                    low: 128,
                    high: 250,
                    message: "Crushed",
                }),
            };
            w.held_use_handlers
                .register(
                    Scope::Types,
                    1755,
                    1625,
                    Handler::ToolCraftingChoices {
                        recipes: std::sync::Arc::new(vec![recipe]),
                        cycle_ticks: 2,
                    },
                )
                .unwrap();
            let held = Held {
                selected_slot: 17,
                selected_id: 1625,
                target_slot: 0,
                target_id: 1755,
            };
            w.prepare_held_use(a, held).unwrap();
            w.finish_held_use(a, held).unwrap();
            let req = w.take_tool_crafting_request(a).unwrap().unwrap();
            w.start_tool_crafting_choice(a, req, 0, 3).unwrap();
            let mut random = Draw { calls: 0 };
            w.tick_with_random(&mut random).unwrap();
            assert_eq!(random.calls, 0);
            w.tick_with_random(&mut random).unwrap();
            assert_eq!(random.calls, 1);
            assert_eq!(w.data(a).unwrap().inventory.get(17).unwrap().id, 1609);
            if stale {
                w.data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        3,
                        Some(Item {
                            id: 1625,
                            count: 1,
                            vars: 7,
                        }),
                    )
                    .unwrap();
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
            );
            for _ in 0..10 {
                w.tick_with_random(&mut random).unwrap();
            }
            let d = w.data(a).unwrap();
            if stale {
                assert_eq!(random.calls, 1);
                assert_eq!((d.inventory.clone(), d.stats.clone()), before);
            } else {
                assert_eq!(random.calls, 3);
                assert_eq!(d.inventory.get(3).unwrap().id, 1633);
                assert_eq!(d.inventory.get(8).unwrap().id, 1609);
                assert_eq!(d.stats.fine_xp(12), 421);
                assert_eq!(d.inventory.free_space(), 0);
            }
            assert_eq!(d.inventory.get(0).unwrap().id, 1755);
        }
    }
}

// Immediate held recipes are single actions; they never enter a production queue.
impl World {
    pub(super) fn complete_tool_crafting_immediate(
        &mut self,
        actor: ActorId,
        job: Job,
    ) -> Result<(), ActionError> {
        if job.recipe.ticks != 0 {
            return Err(ActionError::InvalidItem);
        }
        let p = self.player(actor)?;
        if p.pending_logout || p.saving {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        let mut random = self.random;
        let result = job.resolve(
            &p.data.inventory,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
            &mut random,
        );
        self.random = random;
        match result {
            Ok((prepared, success)) => {
                let p = self.players.get_mut(actor.slot as usize).unwrap();
                if let Some(sequence) = job.recipe.exchange.animation {
                    self.events
                        .push(WorldEvent::PlayerCombatAnimation { actor, sequence });
                }
                publish(
                    p,
                    job,
                    prepared,
                    success,
                    self.tutorial_rules_completed_varp.is_some(),
                    &mut self.events,
                );
            }
            Err(error) => refusal(&mut self.events, actor, error, job.recipe.skill),
        }
        Ok(())
    }
}
fn publish(
    p: &mut Player,
    job: Job,
    prepared: crate::tool_crafting::Prepared,
    success: bool,
    tutorial: bool,
    events: &mut Vec<WorldEvent>,
) {
    p.data.inventory = prepared.inventory;
    p.data.stats = prepared.stats;
    if let Some(advancement) = prepared.advancement {
        if tutorial
            && p.stat_events
                .advance(
                    &crate::tutorial_rules::bindings(),
                    job.recipe.skill,
                    advancement,
                )
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
            skill: job.recipe.skill,
            advancement,
        });
    }
    events.push(WorldEvent::GameMessage {
        actor: p.id,
        text: if success {
            format!("You {} the {}.", job.recipe.action, job.recipe.name)
        } else {
            job.recipe.failure.unwrap().message.into()
        },
    });
}

#[cfg(test)]
mod immediate_tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        tool_crafting::{Failure, Recipe},
    };
    fn fixture(limestone: bool) -> (World, ActorId, Request) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(12, 31);
        stats.set_current(12, 31);
        let mut data = PlayerData::new(
            1,
            "Instant crafter".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = if limestone { 125 } else { 0 };
        let (tool, raw, out) = if limestone {
            (1755, 3211, 3420)
        } else {
            (946, 771, 772)
        };
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
        for (slot, id) in [(5, tool), (19, raw), (21, raw)] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        for id in [tool, raw] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        w.held_use_handlers
            .register(
                Scope::Types,
                tool,
                raw,
                Handler::ToolCraftingImmediate(Recipe {
                    skill: 12,
                    exchange: crate::grinding::Recipe {
                        tool,
                        input: raw,
                        output: out,
                        members: true,
                        animation: None,
                    },
                    level: if limestone { 12 } else { 31 },
                    xp_tenths: if limestone { 60 } else { 0 },
                    ticks: 0,
                    name: "product",
                    action: "craft",
                    output_count: 1,
                    failure: if limestone {
                        Some(Failure {
                            output: 968,
                            xp_tenths: 15,
                            low: 137,
                            high: 434,
                            message: "Failed",
                        })
                    } else {
                        None
                    },
                }),
            )
            .unwrap();
        (
            w,
            a,
            Request {
                selected_slot: 19,
                selected_id: raw,
                target_slot: 5,
                target_id: tool,
            },
        )
    }
    #[test]
    fn actual_immediate_held_use_is_single_selected_action_with_no_xp_or_delayed_replay() {
        let (mut w, a, r) = fixture(false);
        let stats = w.data(a).unwrap().stats.clone();
        let seed = w.random.0;
        w.prepare_held_use(a, r).unwrap();
        w.finish_held_use(a, r).unwrap();
        let d = w.data(a).unwrap();
        assert_eq!(d.inventory.get(19).unwrap().id, 772);
        assert_eq!(d.inventory.get(21).unwrap().id, 771);
        assert_eq!(d.inventory.get(5).unwrap().id, 946);
        assert_eq!(d.inventory.free_space(), 0);
        assert_eq!(d.stats, stats);
        assert_eq!(w.random.0, seed);
        assert!(w.take_tool_crafting_request(a).unwrap().is_none());
        assert!(!w
            .events
            .iter()
            .any(|e| matches!(e, WorldEvent::SkillAdvanced { .. })));
        assert!(w.finish_held_use(a, r).is_err());
        for _ in 0..5 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(a).unwrap().inventory.get(21).unwrap().id, 771);
        assert_eq!(w.data(a).unwrap().stats, stats);
    }
    #[test]
    fn immediate_either_bad_output_or_member_level_tool_guard_refuses_without_rng_or_loss() {
        for case in 0..5 {
            let (mut w, a, r) = fixture(true);
            match case {
                0 => {
                    w.object_types.dummy.insert(3420);
                }
                1 => {
                    w.object_types.dummy.insert(968);
                }
                2 => w.set_membership(a, false).unwrap(),
                3 => w.data_mut(a).unwrap().stats.set_current(12, 11),
                _ => {
                    w.data_mut(a)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            5,
                            Some(Item {
                                id: 1755,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap();
                }
            }
            let before = (
                w.data(a).unwrap().inventory.clone(),
                w.data(a).unwrap().stats.clone(),
                w.random.0,
            );
            w.prepare_held_use(a, r).unwrap();
            let _ = w.finish_held_use(a, r);
            assert_eq!(
                (
                    w.data(a).unwrap().inventory.clone(),
                    w.data(a).unwrap().stats.clone(),
                    w.random.0
                ),
                before,
                "case{case}"
            );
        }
    }
    #[test]
    fn immediate_limestone_publishes_one_chosen_reward_before_a_world_tick() {
        for success in [false, true] {
            let (mut w, a, r) = fixture(true);
            w.data_mut(a).unwrap().stats.set_current(12, 12);
            let seed = (0..10000)
                .find(|s| {
                    let mut rng = SeededRandom(*s);
                    (rng.below(256) < 171) == success
                })
                .unwrap();
            let mut expected = SeededRandom(seed);
            expected.below(256);
            w.random = SeededRandom(seed);
            w.prepare_held_use(a, r).unwrap();
            w.finish_held_use(a, r).unwrap();
            let d = w.data(a).unwrap();
            assert_eq!(
                d.inventory.get(19).unwrap().id,
                if success { 3420 } else { 968 }
            );
            assert_eq!(d.stats.fine_xp(12), if success { 75 } else { 18 });
            assert_eq!(d.inventory.get(21).unwrap().id, 3211);
            assert_eq!(d.inventory.get(5).unwrap().id, 1755);
            assert_eq!(w.random.0, expected.0);
        }
    }
}

#[cfg(test)]
mod tool_skill_tests {
    use super::*;
    use crate::{
        held_use::{Handler, Request, Scope, Type},
        inventory::Item,
        tool_crafting::Recipe,
    };
    #[test]
    fn tool_skill_routes_noncrafting_positive_award_and_failure_to_selected_stat() {
        let r = Recipe {
            skill: 9,
            exchange: crate::grinding::Recipe {
                tool: 100,
                input: 101,
                output: 102,
                members: false,
                animation: None,
            },
            level: 5,
            xp_tenths: 15,
            output_count: 1,
            ticks: 0,
            name: "fixture",
            action: "make",
            failure: None,
        };
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        for id in [100, 101] {
            w.held_use_types.insert(id, Type { id, content: -1 });
        }
        w.held_use_handlers
            .register(Scope::Types, 100, 101, Handler::ToolCraftingImmediate(r))
            .unwrap();
        let mut stats = Stats::default();
        stats.set_base(9, 5);
        stats.set_current(9, 5);
        stats.set_current(12, 0);
        let mut data = PlayerData::new(
            1,
            "Tool fixture".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        for (slot, id) in [(0, 100), (1, 101)] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        let req = Request {
            selected_slot: 0,
            selected_id: 100,
            target_slot: 1,
            target_id: 101,
        };
        w.prepare_held_use(a, req).unwrap();
        w.finish_held_use(a, req).unwrap();
        assert_eq!(w.data(a).unwrap().stats.fine_xp(9), 18);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(12), 0);
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::SkillAdvanced { skill: 9, .. })));
        let mut r = r;
        r.failure = Some(crate::tool_crafting::Failure {
            output: 103,
            xp_tenths: 5,
            low: 0,
            high: 0,
            message: "fixture",
        });
        let mut inv = w.data(a).unwrap().inventory.clone();
        inv.restore_slot(
            1,
            Some(Item {
                id: 101,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let mut stats = w.data(a).unwrap().stats.clone();
        stats.set_fine_xp(9, 0).unwrap();
        struct Fail;
        impl RandomSource for Fail {
            fn below(&mut self, upper: u32) -> u32 {
                assert_eq!(upper, 256);
                255
            }
        }
        let (prepared, success) = Job {
            recipe: r,
            tool_slot: 0,
            input_slot: 1,
        }
        .resolve(&inv, &w.object_types, &stats, 125, false, &mut Fail)
        .unwrap();
        assert!(!success);
        assert_eq!(prepared.stats.fine_xp(9), 6);
        assert_eq!(prepared.stats.fine_xp(12), 0);
    }
}
