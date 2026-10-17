use super::*;
use crate::held_use::{Handler, Request};

impl World {
    fn resend_held_use(&mut self, actor: ActorId) {
        let inv = &mut self
            .players
            .get_mut(actor.slot as usize)
            .unwrap()
            .data
            .inventory;
        if !inv.slots().is_empty() {
            let first = inv.get(0);
            let _ = inv.restore_slot(0, first);
        }
    }
    /// HeldUOpScript resolves nullable types before clearPendingAction. The host
    /// must then close actual modal interfaces (including their callbacks) before
    /// finish_held_use. This split preserves ifClose ordering across the UI host.
    pub fn prepare_held_use(
        &mut self,
        actor: ActorId,
        request: Request,
    ) -> Result<Option<crate::held_use::Preparation>, ActionError> {
        let p = self.player(actor)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if !self.held_use_types.contains_key(&request.selected_id)
            || !self.held_use_types.contains_key(&request.target_id)
        {
            self.resend_held_use(actor);
            return Ok(None);
        }
        // Kotlin clearPendingAction requires !isDelayed before any mutation.
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let abort_input = self.close_dialogs(actor)?;
        let abort_pause = self.cancel_dialogue(actor)?;
        Ok(Some(crate::held_use::Preparation {
            abort_dialog: abort_input || abort_pause,
        }))
    }
    pub fn finish_held_use(&mut self, actor: ActorId, request: Request) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        // ifClose callbacks have completed. clearPendingAction now cancels any
        // remaining coroutine and interaction, then HeldUOpScript resets facing.
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.location_interaction = None;
        p.object_interaction = None;
        p.npc_interaction = None;
        p.bank_count = None;
        p.movement.interacting = false;
        self.events.push(WorldEvent::ResetFaceEntity { actor });
        let p = self.player(actor)?;
        if p.protection.access_protected(p.has_modal) {
            self.resend_held_use(actor);
            return Err(ActionError::Protected);
        }
        let handler = match self.held_use_handlers.resolve(
            &p.data.inventory,
            &self.held_use_types,
            request.selected_slot,
            request.selected_id,
            request.target_slot,
            request.target_id,
        ) {
            Ok(selected) => selected.map(|s| *s.handler),
            Err(_) => {
                self.resend_held_use(actor);
                return Err(ActionError::InvalidItem);
            }
        };
        // Tutorial's exact bucket/flour binding keeps its existing progression.
        // Ordinary choices use the same validated held-use admission afterwards.
        let tutorial_active = matches!(handler, Some(Handler::TutorialDough(config)) if config.progression.active(&p.data.variables));
        let ordinary_handler = handler.is_none() || matches!(handler, Some(Handler::TutorialDough(_)));
        if ordinary_handler {
            if let Some(job)=self.swamp_tar.as_ref().and_then(|config|config.job(request)) {
                self.start_swamp_tar(actor,job)?;
                return Ok(());
            }
        }
        if ordinary_handler {
            if let Some(config) = &self.unfinished_potions {
                if let Some(choice) = config.request(request,p.movement.coords,p.movement.last_movement) {
                    let recipe = config.recipes.get(&(choice.herb, choice.vessel)).unwrap();
                    let message = if !p.member { Some("You need to be a member to make potions.") }
                        else if p.data.variables.get(config.quest_progress) != config.quest_complete { Some("You must complete Druidic Ritual before making potions.") }
                        else if i32::from(p.data.stats.current(15)) < i32::from(recipe.level) { Some("You need a higher Herblore level to make that potion.") }
                        else { None };
                    if let Some(message)=message {self.events.push(WorldEvent::LocationMessage{actor,message});}
                    else {self.players.get_mut(actor.slot as usize).unwrap().unfinished_potion_request=Some(choice);}
                    return Ok(());
                }
            }
        }
        if ordinary_handler {
            if let Some(config) = &self.finished_potions {
                if let Some(choice) = config.request(request,p.movement.coords,p.movement.last_movement) {
                    let recipe = config.recipes.get(&(choice.unfinished, choice.secondary)).unwrap();
                    let message = if !p.member { Some("You need to be a member to make potions.") }
                        else if p.data.variables.get(config.quest_progress) != config.quest_complete { Some("You must complete Druidic Ritual before making potions.") }
                        else if i32::from(p.data.stats.current(15)) < i32::from(recipe.level) || (!recipe.boostable && i32::from(p.data.stats.base(15)) < i32::from(recipe.level)) { Some("You need a higher Herblore level to make that potion.") }
                        else { None };
                    if let Some(message)=message {self.events.push(WorldEvent::LocationMessage{actor,message});}
                    else {self.players.get_mut(actor.slot as usize).unwrap().finished_potion_request=Some(choice);}
                    return Ok(());
                }
            }
        }
        if ordinary_handler {
            if let Some(config) = &self.potion_upgrades {
                if let Some(choice) = config.request(request,p.movement.coords,p.movement.last_movement) {
                    let recipe = config.recipes.get(&(choice.base, choice.secondary)).unwrap();
                    let message = if !p.member { Some("You need to be a member to make potions.") }
                        else if p.data.variables.get(config.quest_progress) != config.quest_complete { Some("You must complete Druidic Ritual before making potions.") }
                        else if i32::from(p.data.stats.current(15)) < i32::from(recipe.level) { Some("You need a higher Herblore level to make that potion.") }
                        else if p.data.inventory.get(choice.secondary_slot).is_none_or(|i|i.count<i32::from(recipe.doses)*i32::from(recipe.secondary_per_dose)) {Some("You do not have enough secondary ingredients.")}
                        else { None };
                    if let Some(message)=message {self.events.push(WorldEvent::LocationMessage{actor,message});}
                    else {self.players.get_mut(actor.slot as usize).unwrap().potion_upgrade_request=Some(choice);}
                    return Ok(());
                }
            }
        }
        if ordinary_handler {
            if let Some(config) = &self.combat_mixing {
                if let Some(choice) = config.request(request,&p.data.inventory,p.movement.coords,p.movement.last_movement) {
                    let message = if !p.member { Some("You need to be a member to make potions.") }
                        else if p.data.variables.get(config.quest_progress) != config.quest_complete { Some("You must complete Druidic Ritual before making potions.") }
                        else if i32::from(p.data.stats.current(15)) < 90 { Some("You need a higher Herblore level to make that potion.") }
                        else { None };
                    if let Some(message)=message {self.events.push(WorldEvent::LocationMessage{actor,message});}
                    else {self.players.get_mut(actor.slot as usize).unwrap().combat_mixing_request=Some(choice);}
                    return Ok(());
                }
            }
        }
        if ordinary_handler && !tutorial_active && !self.dough.as_ref().and_then(|c|c.tutorial).is_some_and(|t|t.active(&p.data.variables)) {
            if let Some(choice) = self.dough.as_ref().and_then(|config| config.request(request,p.movement.coords,p.movement.last_movement)) {
                self.players.get_mut(actor.slot as usize).unwrap().dough_request=Some(choice);
                return Ok(());
            }
        }
        match handler {
            Some(Handler::Grinding(recipe)) => {
                let (tool_slot,input_slot)=if request.selected_id==recipe.tool {
                    (request.selected_slot as usize,request.target_slot as usize)
                } else {(request.target_slot as usize,request.selected_slot as usize)};
                self.start_grinding(actor,recipe,tool_slot,input_slot)?;
            }
            Some(Handler::Assembly(recipe)) | Some(Handler::AssemblyTimed{recipe,..}) => {
                let ticks=match handler {Some(Handler::AssemblyTimed{ticks,..})=>ticks,_=>2};
                if ticks==0 {return Err(ActionError::InvalidItem);}

                let p = self.players.get_mut(actor.slot as usize).unwrap();
                if recipe.members && !p.member {
                    self.events.push(WorldEvent::LocationMessage { actor, message: "You need to be a member to make that item." });
                    return Ok(());
                }
                if i32::from(p.data.stats.current(recipe.skill)) < i32::from(recipe.level) {
                    self.events.push(WorldEvent::LocationMessage {
                        actor,
                        message: "You need a higher Cooking level to make that.",
                    });
                    return Ok(());
                }
                let (first_slot, second_slot) = if request.selected_id == recipe.first {
                    (request.selected_slot as usize, request.target_slot as usize)
                } else {
                    (request.target_slot as usize, request.selected_slot as usize)
                };
                p.queues.clear_weak();
                p.queues
                    .add(
                        crate::assembly::QUEUE,
                        crate::queues::Category::Weak,
                        // Input's queue decrements in its stamped cycle.
                        // Preserve default2; optional content timing is in clocks.
                        i32::from(ticks)+1,
                        Deferred::Assembly(crate::assembly::Pending {
                            job: crate::assembly::Job { recipe, first_slot, second_slot },
                            queued_at: self.clock.0,
                        }),
                    )
                    .map_err(|_| ActionError::QueueFull)?;
            }
            Some(Handler::TutorialFire(config)) => self.start_tutorial_fire(actor, config),
            None => self.events.push(WorldEvent::EngineMessage {
                actor,
                message: "Nothing interesting happens.",
            }),
            Some(Handler::TutorialDough(config)) => {
                let p = self.players.get_mut(actor.slot as usize).unwrap();
                if let Some(result) = config.mix(
                    &mut p.data.inventory,
                    &self.object_types,
                    &mut p.data.variables,
                    p.protection.processed_clock > 0,
                ) {
                    self.events.push(WorldEvent::LocationMessage {
                        actor,
                        message: result.message,
                    });
                    self.events.push(WorldEvent::TutorialInventoryProgress {
                        actor,
                        effects: result.progression,
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        held_use::{Scope, Type},
        inventory::{Insert, StackType},
        tutorial_dough::Config,
        tutorial_progression,
        variables::{Transmit, Varp},
    };
    fn setup(capacity: usize) -> (World, ActorId, Request) {
        let mut world = World::default();
        for x in (3192..3216).step_by(8) {
            for z in (3192..3216).step_by(8) {
                world.collision.allocate(x, z, 0);
            }
        }
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let config = Config {
            progression: tutorial_progression::Config {
                step: varp(1),
                completed: varp(2),
            },
            flour: 10,
            water: 20,
            pot: 30,
            bucket: 40,
            dough: 50,
        };
        world.held_use_types = [10, 20, 30, 40, 50, 60]
            .map(|id| (id, Type { id, content: 0 }))
            .into();
        world
            .held_use_handlers
            .register(Scope::Types, 10, 20, Handler::TutorialDough(config))
            .unwrap();
        let mut data = PlayerData::new(
            1,
            "Dough".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            crate::stats::Stats::default(),
        );
        data.inventory = Inventory::new(capacity, StackType::Normal, false);
        data.inventory
            .insert(&world.object_types, Insert::new(10, 1))
            .unwrap();
        data.inventory
            .insert(&world.object_types, Insert::new(20, 1))
            .unwrap();
        data.variables.set(config.progression.step, 16, false);
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        world.drain_events();
        (
            world,
            actor,
            Request {
                selected_slot: 0,
                selected_id: 10,
                target_slot: 1,
                target_id: 20,
            },
        )
    }
    #[test]
    fn reverse_pair_runs_only_after_host_closes_modal_and_rechecks_slots() {
        let (mut w, actor, r) = setup(3);
        w.set_modal_open(actor, true).unwrap();
        w.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .protection
            .suspended = true;
        assert!(w.prepare_held_use(actor, r).unwrap().is_some());
        assert_eq!(w.finish_held_use(actor, r), Err(ActionError::Protected));
        assert_eq!(w.observe(actor).unwrap().inventory[0].unwrap().id, 10);
        w.set_modal_open(actor, false).unwrap();
        let reverse = Request {
            selected_slot: 1,
            selected_id: 20,
            target_slot: 0,
            target_id: 10,
        };
        w.finish_held_use(actor, reverse).unwrap();
        let observed = w.observe(actor).unwrap();
        assert_eq!(
            observed
                .inventory
                .into_iter()
                .flatten()
                .map(|i| i.id)
                .collect::<Vec<_>>(),
            [30, 40, 50]
        );
        assert_eq!(w.data_mut(actor).unwrap().variables.get(1), 17);
        assert!(w.drain_events().iter().any(|e| matches!(e,
            WorldEvent::TutorialInventoryProgress { effects, .. } if !effects.is_empty())));
    }
    #[test]
    fn missing_type_preserves_pending_action_and_delayed_clear_is_rejected() {
        let (mut w, actor, r) = setup(3);
        w.players
            .get_mut(actor.slot as usize)
            .unwrap()
            .protection
            .suspended = true;
        assert!(w
            .prepare_held_use(
                actor,
                Request {
                    selected_id: -1,
                    ..r
                }
            )
            .unwrap()
            .is_none());
        assert!(w.protection(actor).unwrap().suspended);
        w.delay_player(actor, 5).unwrap();
        assert_eq!(w.prepare_held_use(actor, r), Err(ActionError::Delayed));
        assert!(w.protection(actor).unwrap().suspended);
        assert!(!w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::ResetFaceEntity { .. })));
    }
    #[test]
    fn pause_dialog_aborts_before_modal_callbacks_and_facing_resets_afterward() {
        let (mut w, actor, r) = setup(3);
        let p = w.players.get_mut(actor.slot as usize).unwrap();
        p.dialogue = Some(crate::dialogue::Pending {
            token: 1,
            npc: NpcId {
                slot: 1,
                generation: 1,
            },
            conversation: crate::dialogue::Conversation::person(),
        });
        p.protection.suspended = true;
        let prepared = w.prepare_held_use(actor, r).unwrap().unwrap();
        assert!(prepared.abort_dialog);
        assert!(w.dialogue_token(actor).unwrap().is_none());
        assert!(!w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::ResetFaceEntity { .. })));
        // A close callback may impose a delay. Cancellation/facing still occur,
        // but the subsequent normal protected launch must reject execution.
        w.delay_player(actor, 5).unwrap();
        assert_eq!(w.finish_held_use(actor, r), Err(ActionError::Protected));
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::ResetFaceEntity { .. })));
        assert_eq!(w.observe(actor).unwrap().inventory[0].unwrap().id, 10);
    }
    #[test]
    fn stale_or_same_slot_resends_without_recipe_and_unbound_pair_reports_engine_message() {
        let (mut w, actor, r) = setup(3);
        assert!(w.prepare_held_use(actor, r).unwrap().is_some());
        assert_eq!(
            w.finish_held_use(
                actor,
                Request {
                    target_slot: 0,
                    ..r
                }
            ),
            Err(ActionError::InvalidItem)
        );
        w.data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(
                1,
                Some(crate::inventory::Item {
                    id: 60,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        assert_eq!(w.finish_held_use(actor, r), Err(ActionError::InvalidItem));
        w.finish_held_use(actor, Request { target_id: 60, ..r })
            .unwrap();
        assert_eq!(w.data_mut(actor).unwrap().variables.get(1), 16);
        assert!(w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::EngineMessage {
                message: "Nothing interesting happens.",
                ..
            }
        )));
    }
}

#[cfg(test)]
mod assembly_tests {
    use super::*;
    use crate::{
        assembly::Recipe,
        held_use::{Scope, Type},
        inventory::{Insert, StackType},
    };
    fn fixture() -> (World, ActorId, Request) {
        fixture_with_xp(0)
    }
    fn fixture_with_xp(base_xp: u16) -> (World, ActorId, Request) {
        fixture_with_policy(base_xp, false)
    }
    fn fixture_with_policy(base_xp: u16, members: bool) -> (World, ActorId, Request) {
        let mut world = World::default();
        world.collision.allocate(3200, 3200, 0);
        world.held_use_types = [10, 11, 12].map(|id| (id, Type { id, content: 0 })).into();
        world
            .held_use_handlers
            .register(
                Scope::Types,
                10,
                11,
                Handler::Assembly(Recipe {
                    first: 10,
                    second: 11,
                    output: 12,
                    skill: 7,
                    level: 1,
                    base_xp,
                    members,
                }),
            )
            .unwrap();
        let mut data = PlayerData::new(
            1,
            "Assembler".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.inventory = Inventory::new(3, StackType::Normal, false);
        for id in [10, 11] {
            data.inventory
                .insert(&world.object_types, Insert::new(id, 1))
                .unwrap();
        }
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        (
            world,
            actor,
            Request {
                selected_slot: 1,
                selected_id: 11,
                target_slot: 0,
                target_id: 10,
            },
        )
    }
    #[test]
    fn timed_assembly_uses_requested_processing_clock_and_live_requirements(){
        for ticks in [1,2] {for change in 0..4 {
            let (mut w,actor,request)=fixture();w.held_use_handlers=Default::default();w.held_use_handlers.register(Scope::Types,10,11,Handler::AssemblyTimed{recipe:Recipe{first:10,second:11,output:12,skill:7,level:70,base_xp:0,members:true},ticks}).unwrap();
            let p=w.players.get_mut(actor.slot as usize).unwrap();p.member=true;p.data.stats.set_base(7,69);p.data.stats.set_current(7,70);
            let before=w.data(actor).unwrap().stats.clone();w.prepare_held_use(actor,request).unwrap();w.finish_held_use(actor,request).unwrap();
            match change {1=>w.data_mut(actor).unwrap().stats.set_current(7,69),2=>w.players.get_mut(actor.slot as usize).unwrap().member=false,3=>{let inv=&mut w.data_mut(actor).unwrap().inventory;let i=inv.get(0);inv.restore_slot(0,None).unwrap();inv.restore_slot(2,i).unwrap();},_=>{}}
            for _ in 0..5{w.tick().unwrap();}
            let traces:Vec<_>=w.drain_events().into_iter().filter_map(|e|if let WorldEvent::AssemblyMade{trace,..}=e{Some(trace)}else{None}).collect();assert_eq!(traces.len(),if change==0{1}else{0});
            if change==0{assert_eq!(traces[0].committed_at-traces[0].queued_at,i32::from(ticks));assert_eq!(w.data(actor).unwrap().stats,before);}else{assert!(!w.data(actor).unwrap().inventory.slots().iter().flatten().any(|i|i.id==12));assert_eq!(w.data(actor).unwrap().stats.fine_xp(7),before.fine_xp(7));}
        }}
        let (mut w,actor,request)=fixture();w.held_use_handlers=Default::default();w.held_use_handlers.register(Scope::Types,10,11,Handler::AssemblyTimed{recipe:Recipe{first:10,second:11,output:12,skill:7,level:1,base_xp:0,members:false},ticks:0}).unwrap();let before=w.data(actor).unwrap().inventory.clone();w.prepare_held_use(actor,request).unwrap();assert_eq!(w.finish_held_use(actor,request),Err(ActionError::InvalidItem));for _ in 0..4{w.tick().unwrap();}assert_eq!(w.data(actor).unwrap().inventory,before);
    }
    #[test]
    fn assembly_reverse_pair_rechecks_slots_level_and_cancellation() {
        for change in 0..4 {
            let (mut w, actor, request) = fixture();
            let xp = w.data(actor).unwrap().stats.clone();
            w.prepare_held_use(actor, request).unwrap();
            w.finish_held_use(actor, request).unwrap();
            assert_eq!(w.data(actor).unwrap().inventory.get(0).unwrap().id, 10);
            match change {
                1 => {
                    // Same item elsewhere does not authorize a moved selected input.
                    let inv = &mut w.data_mut(actor).unwrap().inventory;
                    let old = inv.get(0);
                    inv.restore_slot(0, None).unwrap();
                    inv.restore_slot(2, old).unwrap();
                }
                2 => {
                    w.data_mut(actor).unwrap().stats.set_current(7, 0);
                }
                3 => {
                    w.submit(
                        actor,
                        Action::MoveClick {
                            destination: CoordGrid::new(3201, 3200, 0).unwrap(),
                            control: false,
                        },
                    )
                    .unwrap();
                }
                _ => (),
            }
            for tick in 1..=4 {
                w.tick().unwrap();
                if change == 0 {
                    assert_eq!(
                        w.data(actor)
                            .unwrap()
                            .inventory
                            .slots()
                            .iter()
                            .flatten()
                            .any(|i| i.id == 12),
                        tick >= 3
                    );
                }
            }
            let data = w.data(actor).unwrap();
            assert_eq!(
                data.inventory.slots().iter().flatten().any(|i| i.id == 12),
                change == 0
            );
            if change == 0 {
                assert_eq!(data.stats, xp);
                let inventory = data.inventory.clone();
                assert_eq!(
                    w.finish_held_use(actor, request),
                    Err(ActionError::InvalidItem)
                );
                for _ in 0..3 {
                    w.tick().unwrap();
                }
                assert_eq!(w.data(actor).unwrap().inventory, inventory);
            } else {
                assert!(data.inventory.slots().iter().flatten().any(|i| i.id == 10));
            }
            let traces: Vec<_> = w
                .drain_events()
                .into_iter()
                .filter_map(|e| {
                    if let WorldEvent::AssemblyMade { actor: who, trace } = e {
                        assert_eq!(who, actor);
                        Some(trace)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(traces.len(), if change == 0 { 1 } else { 0 });
            if change == 0 {
                assert_eq!(traces[0].committed_at - traces[0].queued_at, 2);
                assert_eq!(
                    (traces[0].job.first_slot, traces[0].job.second_slot),
                    (0, 1)
                );
            }
        }
    }
    #[test]
    fn nonpositive_current_cooking_cannot_admit_assembly() {
        for level in [-1, 0] {
            let (mut w, actor, request) = fixture();
            w.data_mut(actor).unwrap().stats.set_current(7, level);
            let before = w.data(actor).unwrap().inventory.clone();
            w.prepare_held_use(actor, request).unwrap();
            w.finish_held_use(actor, request).unwrap();
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(w.data(actor).unwrap().inventory, before);
            assert!(
                !w.drain_events()
                    .into_iter()
                    .any(|e| matches!(e, WorldEvent::AssemblyMade { .. }))
            );
        }
    }
    #[test]
    fn assembly_reward_is_once_only_and_cancelled_work_has_no_xp() {
        for cancel in [false, true] {
            let (mut w, actor, request) = fixture_with_xp(26);
            w.data_mut(actor).unwrap().xp_rate_hundreds = 500;
            w.prepare_held_use(actor, request).unwrap();
            w.finish_held_use(actor, request).unwrap();
            w.tick().unwrap();
            assert_eq!(w.data(actor).unwrap().stats.fine_xp(7), 0);
            if cancel {
                w.submit(
                    actor,
                    Action::MoveClick {
                        destination: CoordGrid::new(3201, 3200, 0).unwrap(),
                        control: false,
                    },
                )
                .unwrap();
            }
            for _ in 0..5 {
                w.tick().unwrap();
            }
            assert_eq!(
                w.data(actor).unwrap().stats.fine_xp(7),
                if cancel { 0 } else { 1300 }
            );
            let events = w.drain_events();
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::SkillAdvanced { .. }))
                    .count(),
                if cancel { 0 } else { 1 }
            );
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::AssemblyMade { .. }))
                    .count(),
                if cancel { 0 } else { 1 }
            );
        }
    }
    #[test]
    fn unsupported_held_pair_interrupts_pending_assembly_without_reward() {
        let (mut w, actor, request) = fixture_with_xp(45);
        w.data_mut(actor)
            .unwrap()
            .inventory
            .restore_slot(
                2,
                Some(crate::inventory::Item {
                    id: 12,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let before = w.data(actor).unwrap().inventory.clone();
        let stats = w.data(actor).unwrap().stats.clone();
        w.prepare_held_use(actor, request).unwrap();
        w.finish_held_use(actor, request).unwrap();
        // A second genuine pair resolves valid current items but no recipe.
        let unsupported = Request {
            selected_slot: 0,
            selected_id: 10,
            target_slot: 2,
            target_id: 12,
        };
        w.prepare_held_use(actor, unsupported).unwrap();
        w.finish_held_use(actor, unsupported).unwrap();
        for _ in 0..5 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(actor).unwrap().inventory, before);
        assert_eq!(w.data(actor).unwrap().stats, stats);
        let events = w.drain_events();
        assert!(events.iter().any(|e| matches!(
            e,
            WorldEvent::EngineMessage {
                message: "Nothing interesting happens.",
                ..
            }
        )));
        assert!(!events.iter().any(|e| matches!(
            e,
            WorldEvent::AssemblyMade { .. } | WorldEvent::SkillAdvanced { .. }
        )));
        // Cancellation does not poison a subsequent legitimate use.
        w.prepare_held_use(actor, request).unwrap();
        w.finish_held_use(actor, request).unwrap();
        for _ in 0..5 {
            w.tick().unwrap();
        }
        let events = w.drain_events();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::AssemblyMade { .. }))
                .count(),
            1
        );
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, WorldEvent::SkillAdvanced { .. }))
                .count(),
            1
        );
        assert_eq!(w.data(actor).unwrap().inventory.get(2), before.get(2));
    }
    #[test]
    fn member_assembly_rechecks_revocation_without_input_or_xp_loss() {
        for (admitted, retained) in [(false, false), (true, false), (true, true)] {
            let (mut w, actor, request) = fixture_with_policy(45, true);
            w.data_mut(actor).unwrap().xp_rate_hundreds = 500;
            w.set_membership(actor, admitted).unwrap();
            let before = w.data(actor).unwrap().inventory.clone();
            w.prepare_held_use(actor, request).unwrap();
            w.finish_held_use(actor, request).unwrap();
            w.tick().unwrap();
            w.set_membership(actor, retained).unwrap();
            for _ in 0..4 {
                w.tick().unwrap();
            }
            let made = admitted && retained;
            assert_eq!(
                w.data(actor).unwrap().stats.fine_xp(7),
                if made { 2250 } else { 0 }
            );
            if !made {
                assert_eq!(w.data(actor).unwrap().inventory, before);
            }
            let events = w.drain_events();
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::AssemblyMade { .. }))
                    .count(),
                usize::from(made)
            );
            assert_eq!(
                events
                    .iter()
                    .filter(|e| matches!(e, WorldEvent::SkillAdvanced { .. }))
                    .count(),
                usize::from(made)
            );
        }
    }
}
