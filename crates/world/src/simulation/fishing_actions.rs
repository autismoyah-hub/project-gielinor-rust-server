//! Ordinary fishing spots: OpenRune Fishing.fish/attemptCatch over a weak queue.
use super::*;
use crate::fishing::{Binding, Config, Refusal, CATCH_CYCLE, QUEUE};
#[derive(Clone, Copy, Debug)]
pub(super) struct Job {
    pub(super) target: NpcId,
    type_id: u16,
    binding: Binding,
    npc_origin: CoordGrid,
    origin: CoordGrid,
    last_movement: i32,
}
fn message(events: &mut Vec<WorldEvent>, actor: ActorId, text: impl Into<String>) {
    events.push(WorldEvent::GameMessage {
        actor,
        text: text.into(),
    });
}
fn live(n: &NpcEntry) -> bool {
    !n.death_queued && n.npc.visible && !n.npc.delayed && n.npc.stats[3] > 0
}
impl World {
    pub(super) fn start_fishing(&mut self, slot: usize, target: NpcId, operation: u8) {
        let Some(c) = &self.fishing else { return };
        let Some(n) = self
            .npcs
            .get(target.slot as usize)
            .filter(|n| n.id == target && live(n))
        else {
            return;
        };
        let Some(&binding) = c.spots.get(&(n.npc.type_id, operation)) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if p.protection.access_protected(p.has_modal) || p.pending_logout || p.saving {
            return;
        }
        let level = p.data.stats.current(c.skill) as i32;
        let attempt = match c.prepare(binding, &p.data.inventory, &p.data.equipment, level, p.member) {
            Ok(a) => a,
            Err(refusal) => {
                message(&mut self.events, p.id, c.refusal_text(binding, refusal));
                if refusal == Refusal::Capacity {
                    self.events.push(WorldEvent::CombatSound {
                        actor: p.id,
                        sound: c.full_sound,
                    });
                }
                return;
            }
        };
        let job = Job {
            target,
            type_id: n.npc.type_id,
            binding,
            npc_origin: n.npc.movement.coords,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        };
        p.queues.remove_all(QUEUE);
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                CATCH_CYCLE,
                Deferred::Fishing(job),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::NpcInteraction {
            actor: p.id,
            event: crate::npc_interaction::Event::Face(target),
        });
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: attempt.animation,
        });
        for &text in c.methods[&binding.method].cast {
            message(&mut self.events, p.id, text);
        }
    }
}
/// One catch cycle. Any stop condition ends the loop silently, like the queued OpenRune
/// re-check; the player's animation is reset so the idle pose returns.
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    job: Job,
    c: &Config,
    items: &crate::inventory::Catalog,
    npc: Option<&NpcEntry>,
    tutorial: bool,
    events: &mut Vec<WorldEvent>,
    random: &mut impl RandomSource,
) {
    let stop = |events: &mut Vec<WorldEvent>, p: &Player| {
        events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: u16::MAX,
        });
    };
    if p.pending_logout
        || p.saving
        || p.has_modal
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || p.movement.coords != job.origin
        || p.movement.last_movement != job.last_movement
        || p.npc_interaction.is_some()
        || p.location_interaction.is_some()
        || p.object_interaction.is_some()
    {
        return;
    }
    // The spot can despawn or move while the cycle is pending.
    if !npc.is_some_and(|n| {
        n.id == job.target
            && n.npc.type_id == job.type_id
            && live(n)
            && n.npc.movement.coords == job.npc_origin
    }) {
        stop(events, p);
        return;
    }
    let level = p.data.stats.current(c.skill) as i32;
    let Ok(attempt) = c.prepare(job.binding, &p.data.inventory, &p.data.equipment, level, p.member)
    else {
        stop(events, p);
        return;
    };
    events.push(WorldEvent::PlayerCombatAnimation {
        actor: p.id,
        sequence: attempt.animation,
    });
    if let Some(row) = c.roll(&attempt, level, random) {
        let catch = &c.catches[row].1;
        match crate::gathering::reward_consuming(
            &p.data,
            items,
            c.skill,
            catch.product,
            f64::from(catch.xp_tenths) / 10.0,
            c.bait(job.binding, row),
        ) {
            Err(error) => {
                events.push(WorldEvent::ActionRejected { actor: p.id, error });
                stop(events, p);
                return;
            }
            Ok(reward) => {
                let (advancement, _) = reward.commit(&mut p.data);
                if tutorial
                    && p.stat_events
                        .advance(&crate::tutorial_rules::bindings(), c.skill, advancement)
                        .is_err()
                {
                    p.pending_logout = true;
                    events.push(WorldEvent::ActionRejected {
                        actor: p.id,
                        error: ActionError::QueueFull,
                    });
                    return;
                }
                events.push(WorldEvent::SkillAdvanced {
                    actor: p.id,
                    skill: c.skill,
                    advancement,
                });
                message(events, p.id, catch.message.clone());
            }
        }
    }
    if queues
        .add(
            QUEUE,
            crate::queues::Category::Weak,
            CATCH_CYCLE,
            Deferred::Fishing(job),
        )
        .is_err()
    {
        events.push(WorldEvent::ActionRejected {
            actor: p.id,
            error: ActionError::QueueFull,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing::tests::*;
    use crate::inventory::Item;
    use crate::npc::{BlockWalk, Restrict};
    /// Every 256-roll returns `value`; counts draws.
    struct Draw {
        value: u32,
        attempts: usize,
    }
    impl RandomSource for Draw {
        fn below(&mut self, n: u32) -> u32 {
            if n == 256 {
                self.attempts += 1;
                self.value
            } else {
                0
            }
        }
    }
    fn fixture(level: i8, items: &[(i32, i32)]) -> (World, ActorId, NpcId) {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let at = CoordGrid::new(3200, 3200, 0).unwrap();
        let mut stats = Stats::default();
        stats.set_base(3, 10);
        stats.set_current(3, 10);
        stats.set_base(10, level);
        stats.set_current(10, level);
        let mut data = PlayerData::new(1, "Fisher".into(), at, stats);
        for (slot, &(id, count)) in items.iter().enumerate() {
            data.inventory
                .restore_slot(slot, Some(Item { id, count, vars: 0 }))
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        let mut n = Npc::new(
            SPOT,
            CoordGrid::new(3201, 3200, 0).unwrap(),
            1,
            Restrict::NoMove,
            BlockWalk::None,
        )
        .unwrap();
        n.mode = Mode::None;
        let n = w.spawn_npc(n).unwrap();
        let mut d = crate::npc_options::Definition::default();
        d.operations[0] = Some("Net".into());
        d.operations[2] = Some("Bait".into());
        w.npc_options.types.insert(SPOT, d);
        let c = config();
        for &(npc, op) in c.spots.keys() {
            w.npc_handlers
                .insert((npc, op), crate::npc_interaction::Handler::Fishing);
        }
        w.fishing = Some(c);
        w.object_types.stackable.extend([BAIT, FEATHER, STRIPY]);
        (w, a, n)
    }
    fn operate(w: &mut World, a: ActorId, n: NpcId, operation: u8, r: &mut Draw) {
        w.submit(
            a,
            Action::OperateNpc {
                target: n,
                operation,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(r).unwrap();
    }
    fn messages(w: &mut World) -> Vec<String> {
        w.drain_events()
            .into_iter()
            .filter_map(|e| match e {
                WorldEvent::GameMessage { text, .. } => Some(text),
                _ => None,
            })
            .collect()
    }
    fn count(w: &World, a: ActorId, id: i32) -> i32 {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .map(|i| i.count)
            .sum()
    }
    fn queued(w: &World, a: ActorId) -> bool {
        w.players
            .get(a.slot as usize)
            .unwrap()
            .queues
            .snapshot()
            .iter()
            .any(|q| q.id == QUEUE)
    }
    fn ticks(w: &mut World, r: &mut Draw, n: usize) {
        for _ in 0..n {
            w.tick_with_random(r).unwrap();
        }
    }
    #[test]
    fn level_refusal_names_lowest_spot_level_without_rolls() {
        let (mut w, a, n) = fixture(4, &[(ROD, 1), (BAIT, 5)]);
        let mut r = Draw { value: 0, attempts: 0 };
        operate(&mut w, a, n, 3, &mut r);
        assert!(messages(&mut w).contains(&"You need a Fishing level of 5 to fish here.".into()));
        assert!(!queued(&w, a));
        ticks(&mut w, &mut r, 6);
        assert_eq!(r.attempts, 0);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(10), 0);
    }
    #[test]
    fn missing_tool_and_missing_bait_refuse() {
        let (mut w, a, n) = fixture(10, &[(BAIT, 5)]);
        let mut r = Draw { value: 0, attempts: 0 };
        operate(&mut w, a, n, 1, &mut r);
        assert!(messages(&mut w).contains(&"You need a small fishing net to fish here.".into()));
        assert!(!queued(&w, a));
        let (mut w, a, n) = fixture(10, &[(ROD, 1)]);
        operate(&mut w, a, n, 3, &mut r);
        assert!(messages(&mut w).contains(&"You don't have any bait left.".into()));
        assert!(!queued(&w, a));
        assert_eq!(r.attempts, 0);
    }
    #[test]
    fn net_catch_awards_fish_and_xp_after_one_cycle_and_continues() {
        let (mut w, a, n) = fixture(1, &[(NET, 1)]);
        let mut r = Draw { value: 0, attempts: 0 };
        operate(&mut w, a, n, 1, &mut r);
        assert!(messages(&mut w).contains(&"You cast out your net...".into()));
        assert!(queued(&w, a));
        ticks(&mut w, &mut r, CATCH_CYCLE as usize - 1);
        assert_eq!(r.attempts, 0);
        assert_eq!(count(&w, a, SHRIMP), 0);
        ticks(&mut w, &mut r, 1);
        assert_eq!(r.attempts, 1);
        assert_eq!(count(&w, a, SHRIMP), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(10), 100);
        assert!(messages(&mut w).contains(&"You catch some raw shrimps.".into()));
        // Continuation: the next cycle rolls again without another operation.
        ticks(&mut w, &mut r, CATCH_CYCLE as usize);
        assert_eq!(count(&w, a, SHRIMP), 2);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(10), 200);
        assert!(queued(&w, a));
    }
    #[test]
    fn failed_roll_keeps_fishing_without_reward() {
        let (mut w, a, n) = fixture(1, &[(NET, 1)]);
        let mut r = Draw { value: 255, attempts: 0 };
        operate(&mut w, a, n, 1, &mut r);
        ticks(&mut w, &mut r, CATCH_CYCLE as usize * 2);
        assert_eq!(r.attempts, 2);
        assert_eq!(count(&w, a, SHRIMP), 0);
        assert!(queued(&w, a));
    }
    #[test]
    fn bait_is_consumed_per_catch_and_running_out_stops() {
        let (mut w, a, n) = fixture(5, &[(ROD, 1), (BAIT, 2)]);
        let mut r = Draw { value: 0, attempts: 0 };
        operate(&mut w, a, n, 3, &mut r);
        ticks(&mut w, &mut r, CATCH_CYCLE as usize);
        assert_eq!(count(&w, a, SARDINE), 1);
        assert_eq!(count(&w, a, BAIT), 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(10), 200);
        ticks(&mut w, &mut r, CATCH_CYCLE as usize);
        assert_eq!(count(&w, a, SARDINE), 2);
        assert_eq!(count(&w, a, BAIT), 0);
        let rolls = r.attempts;
        ticks(&mut w, &mut r, CATCH_CYCLE as usize * 2);
        assert_eq!(r.attempts, rolls);
        assert_eq!(count(&w, a, SARDINE), 2);
        assert!(!queued(&w, a));
    }
    #[test]
    fn full_inventory_refuses_start_and_stops_loop() {
        let mut items = vec![(NET, 1)];
        items.extend((0..26).map(|_| (SHRIMP, 1)));
        let (mut w, a, n) = fixture(1, &items);
        let mut r = Draw { value: 0, attempts: 0 };
        operate(&mut w, a, n, 1, &mut r);
        ticks(&mut w, &mut r, CATCH_CYCLE as usize);
        assert_eq!(count(&w, a, SHRIMP), 27);
        assert_eq!(w.data(a).unwrap().inventory.free_space(), 0);
        ticks(&mut w, &mut r, CATCH_CYCLE as usize * 2);
        assert_eq!(count(&w, a, SHRIMP), 27);
        assert_eq!(r.attempts, 1);
        assert!(!queued(&w, a));
        messages(&mut w);
        operate(&mut w, a, n, 1, &mut r);
        assert!(messages(&mut w).contains(&"Your inventory is too full to hold any more fish.".into()));
        assert!(!queued(&w, a));
    }
    #[test]
    fn moving_or_despawned_spot_interrupts() {
        let (mut w, a, n) = fixture(1, &[(NET, 1)]);
        let mut r = Draw { value: 0, attempts: 0 };
        operate(&mut w, a, n, 1, &mut r);
        w.npc_mut(n).unwrap().movement.coords = CoordGrid::new(3202, 3200, 0).unwrap();
        ticks(&mut w, &mut r, CATCH_CYCLE as usize * 2);
        assert_eq!(r.attempts, 0);
        assert_eq!(count(&w, a, SHRIMP), 0);
        let (mut w, a, n) = fixture(1, &[(NET, 1)]);
        operate(&mut w, a, n, 1, &mut r);
        w.despawn_npc(n);
        ticks(&mut w, &mut r, CATCH_CYCLE as usize * 2);
        assert_eq!(r.attempts, 0);
        let (mut w, a, n) = fixture(1, &[(NET, 1)]);
        operate(&mut w, a, n, 1, &mut r);
        w.submit(
            a,
            Action::Walk {
                destination: CoordGrid::new(3198, 3200, 0).unwrap(),
                run: false,
            },
        )
        .unwrap();
        ticks(&mut w, &mut r, CATCH_CYCLE as usize * 2);
        assert_eq!(r.attempts, 0);
        assert!(!queued(&w, a));
    }
}
