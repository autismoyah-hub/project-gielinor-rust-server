//! Selected held Dig uses the existing weak queue and player protection clock.
use super::*;
pub(super) const QUEUE: i32 = -2391173;
#[derive(Clone, Copy)]
pub(super) struct Pending {
    hook: Option<&'static str>,
    dialog_generation: u64,
}
impl World {
    pub fn register_spade_hook(&mut self, hook: crate::spade::Hook) -> Result<(), &'static str> {
        if self.players.iter().next().is_some()
            || hook.name.is_empty()
            || self.spade_hooks.iter().any(|h| h.name == hook.name)
        {
            return Err("Register unique spade hooks before admission");
        }
        self.spade_hooks.push(hook);
        Ok(())
    }
    pub(super) fn start_spade(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let rule = self.spade.ok_or(ActionError::InvalidItem)?;
        if p.saving
            || p.pending_logout
            || p.has_modal
            || p.data.stats.current(3) <= 0
            || !rule.valid()
            || rule.item != item
            || !self.object_types.ordinary_nonstackable(item)
            || p.data
                .inventory
                .get(slot)
                .is_none_or(|v| v.id != item || v.count != 1 || v.vars != 0)
            || self.clock.0 < 0
            || self.clock.0.checked_add(rule.delay).is_none()
        {
            return Err(ActionError::InvalidItem);
        }
        let hook = self
            .spade_hooks
            .iter()
            .find(|hook| (hook.claims)(self, actor))
            .copied();
        if let Some(hook) = hook {
            if !(hook.before)(self, actor)? {
                return Ok(());
            }
        }
        // Hook callbacks own their effects; revalidate the live actor/tool before
        // admitting the protected generic animation and completion.
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        if p.id != actor
            || p.saving
            || p.pending_logout
            || p.has_modal
            || p.data.stats.current(3) <= 0
            || self.clock.0 < 0
            || self.clock.0.checked_add(rule.delay).is_none()
            || p.data
                .inventory
                .get(slot)
                .is_none_or(|v| v.id != item || v.count != 1 || v.vars != 0)
        {
            return Err(ActionError::InvalidItem);
        }
        p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                rule.delay,
                Deferred::Spade(Pending {
                    hook: hook.map(|h| h.name),
                    dialog_generation: p.dialog_generation,
                }),
            )
            .map_err(|_| ActionError::QueueFull)?;
        p.protection.processed_clock = self.clock.0;
        p.protection.delay(rule.delay);
        self.events.push(WorldEvent::PlayerAnimation {
            actor,
            sequence: rule.animation,
        });
        self.events.push(WorldEvent::PlayerSound {
            actor,
            sound: rule.sound,
        });
        Ok(())
    }
    pub(super) fn complete_spade_hook(&mut self, actor: ActorId, job: Pending) {
        if self.player(actor).is_err()
            || !ready(self.players.get(actor.slot as usize).unwrap(), job)
        {
            return;
        }
        let Some(name) = job.hook else {
            return;
        };
        // A missing/removed binding never falls back to generic text after a
        // hook already claimed the action. Native modules register before login.
        if let Some(hook) = self.spade_hooks.iter().find(|h| h.name == name).copied() {
            if let Err(error) = (hook.dig)(self, actor) {
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
            }
        }
    }
}
fn ready(p: &Player, job: Pending) -> bool {
    !p.saving
        && !p.pending_logout
        && !p.has_modal
        && p.data.stats.current(3) > 0
        && p.location_interaction.is_none()
        && p.npc_interaction.is_none()
        && p.object_interaction.is_none()
        && p.dialog_generation == job.dialog_generation
}
pub(super) fn complete(
    p: &mut Player,
    job: Pending,
    events: &mut Vec<WorldEvent>,
) -> Option<(ActorId, Pending)> {
    if !ready(p, job) {
        return None;
    }
    events.push(WorldEvent::PlayerAnimation {
        actor: p.id,
        sequence: u16::MAX,
    });
    if job.hook.is_some() {
        Some((p.id, job))
    } else {
        events.push(WorldEvent::GameMessage {
            actor: p.id,
            text: crate::spade::NOTHING.into(),
        });
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        spade::{Hook, Rule},
        stats::Stats,
    };
    struct Never;
    impl RandomSource for Never {
        fn below(&mut self, _: u32) -> u32 {
            panic!("spade core must not roll")
        }
    }
    fn fixture(full: bool, hooks: &[Hook]) -> (World, ActorId) {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.spade = Some(Rule {
            item: 952,
            animation: 830,
            sound: 1470,
            delay: 2,
        });
        for &hook in hooks {
            w.register_spade_hook(hook).unwrap();
        }
        let mut p = PlayerData::new(
            1,
            "Digger".into(),
            CoordGrid::new(3202, 3202, 0).unwrap(),
            Stats::default(),
        );
        for slot in 0..if full { 28 } else { 2 } {
            p.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: if slot < 2 { 952 } else { 5000 },
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, p).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        (w, a)
    }
    fn dig(w: &mut World, a: ActorId) {
        w.submit(a, Action::DigItemSlot { slot: 0, item: 952 })
            .unwrap();
    }
    #[test]
    fn spade_default_selected_tool_full_pack_two_ticks_sound_reset_and_no_item_xp_rng_or_movement()
    {
        for full in [false, true] {
            let (mut w, a) = fixture(full, &[]);
            let before = w.data(a).unwrap().clone();
            dig(&mut w, a);
            w.tick_with_random(&mut Never).unwrap();
            let e = w.drain_events();
            assert!(e
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerAnimation { sequence: 830, .. })));
            assert!(e
                .iter()
                .any(|e| matches!(e, WorldEvent::PlayerSound { sound: 1470, .. })));
            assert!(!e
                .iter()
                .any(|e| matches!(e, WorldEvent::GameMessage { .. })));
            w.tick_with_random(&mut Never).unwrap();
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::GameMessage { .. })));
            w.tick_with_random(&mut Never).unwrap();
            let e = w.drain_events();
            assert!(e.iter().any(|e| matches!(
                e,
                WorldEvent::PlayerAnimation {
                    sequence: u16::MAX,
                    ..
                }
            )));
            assert_eq!(e.iter().filter(|e|matches!(e,WorldEvent::GameMessage{text,..}if text==crate::spade::NOTHING)).count(),1);
            for _ in 0..2 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::GameMessage { .. })));
            assert_eq!(w.data(a).unwrap().inventory, before.inventory);
            assert_eq!(w.data(a).unwrap().stats, before.stats);
            assert_eq!(w.data(a).unwrap().coords, before.coords);
        }
    }
    fn claims(_: &World, _: ActorId) -> bool {
        true
    }
    fn accepts(_: &mut World, _: ActorId) -> Result<bool, ActionError> {
        Ok(true)
    }
    fn denies(_: &mut World, _: ActorId) -> Result<bool, ActionError> {
        Ok(false)
    }
    fn hook_finish(w: &mut World, a: ActorId) -> Result<(), ActionError> {
        w.events.push(WorldEvent::GameMessage {
            actor: a,
            text: "developer hook completed".into(),
        });
        Ok(())
    }
    #[test]
    fn spade_registered_claim_before_veto_and_completed_hook_do_not_fabricate_native_quest_fallback(
    ) {
        for deny in [false, true] {
            let hook = Hook {
                name: "developer-fixture-only",
                claims,
                before: if deny { denies } else { accepts },
                dig: hook_finish,
            };
            let (mut w, a) = fixture(false, &[hook]);
            assert!(w
                .register_spade_hook(Hook {
                    name: "late",
                    ..hook
                })
                .is_err());
            dig(&mut w, a);
            let mut events = vec![];
            for _ in 0..4 {
                w.tick_with_random(&mut Never).unwrap();
                events.extend(w.drain_events());
            }
            assert!(!events.iter().any(
                |e| matches!(e,WorldEvent::GameMessage{text,..}if text==crate::spade::NOTHING)
            ));
            assert_eq!(events.iter().filter(|e|matches!(e,WorldEvent::GameMessage{text,..}if text=="developer hook completed")).count(),if deny{0}else{1});
            if deny {
                assert!(!events.iter().any(|e| matches!(
                    e,
                    WorldEvent::PlayerAnimation { .. } | WorldEvent::PlayerSound { .. }
                )));
            }
        }
    }
    #[test]
    fn spade_stale_selected_claim_invalid_catalog_vars_delay_and_lifecycle_refuse_before_effects() {
        for mode in 0..7 {
            let (mut w, a) = fixture(false, &[]);
            match mode {
                0 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 5000,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                1 => {
                    w.object_types.stackable.insert(952);
                }
                2 => {
                    w.object_types.dummy.insert(952);
                }
                3 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 952,
                            count: 1,
                            vars: 1,
                        }),
                    )
                    .unwrap(),
                4 => {
                    w.spade.as_mut().unwrap().delay = 0;
                }
                5 => {
                    w.players.get_mut(a.slot as usize).unwrap().has_modal = true;
                }
                _ => {
                    w.data_mut(a).unwrap().stats.set_current(3, 0);
                }
            }
            let inventory = w.data(a).unwrap().inventory.clone();
            dig(&mut w, a);
            for _ in 0..4 {
                w.tick_with_random(&mut Never).unwrap();
            }
            let e = w.drain_events();
            assert!(!e.iter().any(|e| matches!(
                e,
                WorldEvent::PlayerAnimation { .. }
                    | WorldEvent::PlayerSound { .. }
                    | WorldEvent::GameMessage { .. }
            )));
            assert_eq!(w.data(a).unwrap().inventory, inventory);
        }
    }
    #[test]
    fn spade_held_cancel_logout_and_changed_dialog_epoch_do_not_finish_later() {
        for mode in 0..3 {
            let (mut w, a) = fixture(false, &[]);
            dig(&mut w, a);
            w.tick_with_random(&mut Never).unwrap();
            w.drain_events();
            match mode {
                0 => {
                    w.submit(a, Action::EatItemSlot { slot: 0, item: 999 })
                        .unwrap();
                }
                1 => {
                    w.request_logout(a).unwrap();
                }
                _ => {
                    w.players
                        .get_mut(a.slot as usize)
                        .unwrap()
                        .dialog_generation += 1;
                }
            }
            for _ in 0..4 {
                w.tick_with_random(&mut Never).unwrap();
            }
            assert!(!w
                .drain_events()
                .iter()
                .any(|e| matches!(e, WorldEvent::GameMessage { .. })));
        }
    }
}
