use super::*;
use crate::thieving::{Config, Error, QUEUE};
#[derive(Clone, Copy, Debug)]
pub(super) struct Job {
    pub(super) target: NpcId,
    type_id: u16,
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
fn refusal(events: &mut Vec<WorldEvent>, actor: ActorId, error: Error) {
    message(
        events,
        actor,
        match error {
            Error::Cap => {
                "You need to empty your coin pouches before you can continue pickpocketing."
            }
            Error::Capacity => "You don't have enough inventory space.",
            Error::Level => "You need a Thieving level of 1 to pickpocket this person.",
            _ => "You can't pickpocket right now.",
        },
    );
}
impl World {
    pub(super) fn start_pickpocket(&mut self, slot: usize, target: NpcId) {
        let Some(c) = &self.pickpocket else { return };
        let Some(n) = self.npcs.get(target.slot as usize).filter(|n| {
            n.id == target
                && !n.death_queued
                && n.npc.visible
                && !n.npc.delayed
                && n.npc.stats[3] > 0
        }) else {
            return;
        };
        let Some(profile) = c.targets.get(&n.npc.type_id) else {
            return;
        };
        let p = self.players.get_mut(slot).unwrap();
        if !p.member
            || p.protection.access_protected(p.has_modal)
            || p.pending_logout
            || p.saving
            || p.movement.rooted
            || p.pickpocket_stunned()
        {
            return;
        }
        if let Err(error) = c.prepare(
            &p.data.inventory,
            &p.data.stats,
            &self.object_types,
            p.data.xp_rate_hundreds,
        ) {
            refusal(&mut self.events, p.id, error);
            return;
        }
        let job = Job {
            target,
            type_id: n.npc.type_id,
            npc_origin: n.npc.movement.coords,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
        };
        p.queues.remove_all(QUEUE);
        if p.queues
            .add(
                QUEUE,
                crate::queues::Category::Weak,
                1,
                Deferred::Pickpocket(job),
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
        message(
            &mut self.events,
            p.id,
            format!("You attempt to pick the {}'s pocket.", profile.owner),
        );
    }
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    job: Job,
    c: &Config,
    items: &crate::inventory::Catalog,
    npc: Option<&NpcEntry>,
    hit_queues: Option<crate::hits::PlayerQueues>,
    clock: i32,
    tutorial: bool,
    events: &mut Vec<WorldEvent>,
    random: &mut impl RandomSource,
) -> bool {
    if p.pending_logout
        || p.saving
        || p.has_modal
        || !p.member
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || p.movement.rooted
        || p.pickpocket_stunned()
        || p.movement.coords != job.origin
        || p.movement.last_movement != job.last_movement
        || p.npc_interaction.is_some()
        || p.location_interaction.is_some()
        || p.object_interaction.is_some()
    {
        return false;
    }
    let Some(n) = npc.filter(|n| {
        n.id == job.target
            && !n.death_queued
            && n.npc.type_id == job.type_id
            && n.npc.visible
            && !n.npc.delayed
            && n.npc.stats[3] > 0
            && n.npc.movement.coords == job.npc_origin
    }) else {
        return false;
    };
    let Some(profile) = c.targets.get(&n.npc.type_id) else {
        return false;
    };
    let prepared = match c.prepare(
        &p.data.inventory,
        &p.data.stats,
        items,
        p.data.xp_rate_hundreds,
    ) {
        Ok(v) => v,
        Err(e) => {
            refusal(events, p.id, e);
            return false;
        }
    };
    if random.below(256) < prepared.threshold {
        p.data.inventory = prepared.inventory;
        p.data.stats = prepared.stats;
        if tutorial
            && p.stat_events
                .advance(
                    &crate::tutorial_rules::bindings(),
                    c.skill,
                    prepared.advancement,
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
            skill: c.skill,
            advancement: prepared.advancement,
        });
        events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: c.animation,
        });
        events.push(WorldEvent::CombatSound {
            actor: p.id,
            sound: c.sound,
        });
        message(
            events,
            p.id,
            format!("You pick the {}'s pocket.", profile.owner),
        );
        return false;
    }
    let hit = crate::hits::Hit {
        display: None,
        source: Some(crate::hits::Target::Npc(job.target)),
        style: crate::hits::Style::Typeless,
        damage: 1,
    };
    // Strong hits survive weak-action cancellation (including eating during the stun).
    let queued = if let Some(q) = hit_queues {
        queues
            .add(
                q.standard,
                crate::queues::Category::Strong,
                2,
                Deferred::StandardHit(hit),
            )
            .is_ok()
    } else {
        p.hits.push(hit, 2).is_ok()
    };
    if !queued {
        events.push(WorldEvent::ActionRejected {
            actor: p.id,
            error: ActionError::QueueFull,
        });
        return false;
    }
    p.pickpocket_stun_until = crate::tick::MapClock(clock).deadline(9);
    p.movement.abort();
    events.push(WorldEvent::PlayerCombatAnimation {
        actor: p.id,
        sequence: c.block,
    });
    events.push(WorldEvent::PlayerSpotAnimation {
        actor: p.id,
        spot: c.spot,
        delay: 0,
        height: 124,
        slot: 0,
    });
    message(
        events,
        p.id,
        format!("You fail to pick the {}'s pocket.", profile.owner),
    );
    message(events, p.id, "You've been stunned!");
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        npc::{BlockWalk, Restrict},
        thieving::Target,
    };
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
    fn fixture() -> (World, ActorId, NpcId) {
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
        w.queue_login(1, PlayerData::new(1, "Thief".into(), at, stats))
            .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.players.get_mut(a.slot as usize).unwrap().member = true;
        w.data_mut(a).unwrap().xp_rate_hundreds = 125;
        let mut n = Npc::new(
            3106,
            CoordGrid::new(3201, 3200, 0).unwrap(),
            1,
            Restrict::Normal,
            BlockWalk::None,
        )
        .unwrap();
        n.mode = Mode::None;
        let n = w.spawn_npc(n).unwrap();
        let mut d = crate::npc_options::Definition::default();
        d.operations[2] = Some("Pickpocket".into());
        w.npc_options.types.insert(3106, d);
        w.npc_handlers
            .insert((3106, 3), crate::npc_interaction::Handler::Pickpocket);
        w.pickpocket = Some(Config {
            targets: [(3106, Target { owner: "man" })].into(),
            skill: 17,
            pouch: 22521,
            animation: 881,
            block: 424,
            spot: 245,
            sound: 2581,
        });
        w.object_types.stackable.insert(22521);
        w.player_hit_queues = Some(crate::hits::PlayerQueues {
            standard: 800,
            death: 801,
        });
        (w, a, n)
    }
    fn arm(w: &mut World, a: ActorId, n: NpcId, r: &mut Draw) {
        w.submit(
            a,
            Action::OperateNpc {
                target: n,
                operation: 3,
                control: false,
            },
        )
        .unwrap();
        w.tick_with_random(r).unwrap();
        assert_eq!(r.attempts, 0);
        assert!(w
            .players
            .get(a.slot as usize)
            .unwrap()
            .queues
            .snapshot()
            .iter()
            .any(|q| q.id == QUEUE));
    }
    #[test]
    fn actual_npc_operation_awards_atomic_pouch_and_fine_xp_after_one_clock() {
        let (mut w, a, n) = fixture();
        let mut r = Draw {
            value: 0,
            attempts: 0,
        };
        arm(&mut w, a, n, &mut r);
        assert!(w
            .data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .all(Option::is_none));
        w.tick_with_random(&mut r).unwrap();
        assert_eq!(r.attempts, 1);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 22521);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(17), 100);
        for _ in 0..3 {
            w.tick_with_random(&mut r).unwrap();
        }
        assert_eq!(r.attempts, 1);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().count, 1);
    }
    #[test]
    fn live_refusals_cancel_without_rng_reward_or_replacement_target() {
        for case in 0..6 {
            let (mut w, a, n) = fixture();
            let mut r = Draw {
                value: 0,
                attempts: 0,
            };
            arm(&mut w, a, n, &mut r);
            match case {
                0 => w.players.get_mut(a.slot as usize).unwrap().member = false,
                1 => {
                    w.set_modal_open(a, true).unwrap();
                    w.set_modal_open(a, false).unwrap();
                }
                2 => w.npc_mut(n).unwrap().movement.coords = CoordGrid::new(3202, 3200, 0).unwrap(),
                3 => {
                    w.despawn_npc(n);
                }
                4 => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 22521,
                            count: 28,
                            vars: 0,
                        }),
                    )
                    .unwrap(),
                _ => w
                    .data_mut(a)
                    .unwrap()
                    .inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 22521,
                            count: 1,
                            vars: 9,
                        }),
                    )
                    .unwrap(),
            };
            w.tick_with_random(&mut r).unwrap();
            assert_eq!(r.attempts, 0, "case{case}");
            assert_eq!(w.data(a).unwrap().stats.fine_xp(17), 0, "case{case}");
        }
    }
    #[test]
    fn failure_stuns_movement_but_food_cannot_cancel_strong_damage_and_expiry() {
        let (mut w, a, n) = fixture();
        let mut r = Draw {
            value: 255,
            attempts: 0,
        };
        w.foods.insert(
            315,
            crate::food::Edible {
                id: 315,
                heal: 3,
                delay: 3,
                replacement: None,
                message: "You eat the shrimps.".into(),
            },
        );
        w.data_mut(a)
            .unwrap()
            .inventory
            .restore_slot(
                4,
                Some(Item {
                    id: 315,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        arm(&mut w, a, n, &mut r);
        w.tick_with_random(&mut r).unwrap();
        assert_eq!(r.attempts, 1);
        assert!(!w.movement_input_allowed(a).unwrap());
        assert_eq!(w.data(a).unwrap().stats.fine_xp(17), 0);
        let until = w
            .players
            .get(a.slot as usize)
            .unwrap()
            .pickpocket_stun_until;
        w.submit(a, Action::EatItemSlot { slot: 4, item: 315 })
            .unwrap();
        w.tick_with_random(&mut r).unwrap();
        assert!(w.data(a).unwrap().inventory.get(4).is_none());
        let mut damage = 0;
        for e in w.drain_events() {
            if let WorldEvent::Hit { impact, .. } = e {
                damage += impact.applied;
            }
        }
        while w.clock.0 <= until {
            w.tick_with_random(&mut r).unwrap();
            for e in w.drain_events() {
                if let WorldEvent::Hit { impact, .. } = e {
                    damage += impact.applied;
                }
            }
        }
        assert_eq!(damage, 1);
        assert!(w.movement_input_allowed(a).unwrap());
        assert_eq!(r.attempts, 1);
        assert_eq!(w.data(a).unwrap().stats.fine_xp(17), 0);
        assert!(w
            .data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .all(|i| i.id != 22521));
    }
}
