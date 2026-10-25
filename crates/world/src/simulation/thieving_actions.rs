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
        if !p.member {
            // Thieving is members-only ("NPCs cannot be pickpocketed when not on a Members
            // server", Wiki Thieving/Experience table). No refusal text is attested, so none
            // is sent; OpenRune's pickpocket has no membership check.
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::Members,
            });
            return;
        }
        if p.protection.access_protected(p.has_modal)
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
    #[test]
    fn a_man_killed_for_loot_respawns_and_can_still_be_pickpocketed() {
        use crate::{
            hits::{Hit, Style, Target as HitTarget},
            npc_death::{Config as Death, Definition, DropPolicy},
            npc_loot::{Outcome, Product, Profile, Row, Weighted},
        };
        let (mut w, a, n) = fixture();
        w.data_mut(a).unwrap().character_id = 400;
        w.set_observer_id(a, 900).unwrap();
        w.npc_death = Some(Death {
            definitions: [(3106, Definition { animation: 836, animation_ticks: 2, sound: None })].into(),
            bones: 526,
            chickens: Default::default(),
        });
        let main = Weighted::with_rows(1, vec![Row { weight: 1, outcome: Outcome::Range { item: 877, min: 2, max: 12 } }]).unwrap();
        w.npc_loot_profiles.insert(
            3106,
            Profile::new(DropPolicy::Man, vec![Product { item: 526, count: 1 }], Some(main)).unwrap(),
        );
        w.object_types.stackable.insert(877);
        let man = w.npc_mut(n).unwrap();
        man.stats[3] = 2;
        man.base_stats[3] = 2;
        man.regen_rate = 0;
        man.respawns = true;
        man.respawn_rate = 3;
        man.hero_points.credit(400, 2);
        let tile = man.movement.coords;
        w.queue_hit(
            HitTarget::Npc(n),
            Hit { source: Some(HitTarget::Player(a)), style: Style::Typeless, damage: 2, display: None },
            1,
        )
        .unwrap();
        let mut r = Draw { value: 0, attempts: 0 };
        let mut drops = vec![];
        let mut respawned = None;
        for _ in 0..12 {
            w.tick_with_random(&mut r).unwrap();
            for e in w.drain_events() {
                match e {
                    WorldEvent::NpcDrop(t) => drops.push(t),
                    // Respawn re-admits the slot under a new generation.
                    WorldEvent::NpcCombatAnimation { actor, sequence: u16::MAX } if actor.slot == n.slot => {
                        respawned = Some(actor)
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].products, [(526, 1), (877, 2)], "count draw 0 is the range minimum");
        assert!(w.objects.at(tile).all(|h| !h.snapshot().visible_to(Some(901))));
        let n = respawned.expect("the man respawned");
        let man = w.npc(n).unwrap();
        assert!(man.visible && man.type_id == 3106 && man.stats[3] == 2, "respawned with full hitpoints");
        // The respawned man is an ordinary pickpocket target again.
        arm(&mut w, a, n, &mut r);
        w.tick_with_random(&mut r).unwrap();
        assert_eq!(r.attempts, 1);
        let pouch = w.data(a).unwrap().inventory.slots().iter().flatten().find(|i| i.id == 22521).map(|i| i.count);
        assert_eq!(pouch, Some(1));
    }
    #[test]
    fn a_free_account_pickpocket_is_refused_without_a_message_or_any_change() {
        let (mut w, a, n) = fixture();
        w.set_membership(a, false).unwrap();
        let before = (w.data(a).unwrap().inventory.clone(), w.data(a).unwrap().stats.clone());
        let mut r = Draw { value: 0, attempts: 0 };
        w.submit(a, Action::OperateNpc { target: n, operation: 3, control: false }).unwrap();
        let mut events = vec![];
        for _ in 0..4 {
            w.tick_with_random(&mut r).unwrap();
            events.extend(w.drain_events());
        }
        let rejected: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, WorldEvent::ActionRejected { actor, error: ActionError::Members } if *actor == a))
            .collect();
        assert_eq!(rejected.len(), 1, "one explicit members refusal");
        assert!(!events.iter().any(|e| matches!(e, WorldEvent::GameMessage { .. })), "no unattested message");
        assert_eq!(r.attempts, 0, "no pickpocket roll");
        let p = w.players.get(a.slot as usize).unwrap();
        assert!(p.npc_interaction.is_none() && !p.movement.interacting, "interaction cleared");
        assert!(!p.queues.snapshot().iter().any(|q| q.id == QUEUE), "nothing queued");
        assert_eq!((w.data(a).unwrap().inventory.clone(), w.data(a).unwrap().stats.clone()), before);
        // The same player as a member pickpockets as before.
        w.set_membership(a, true).unwrap();
        arm(&mut w, a, n, &mut r);
        w.tick_with_random(&mut r).unwrap();
        assert_eq!(r.attempts, 1);
        assert_eq!(w.data(a).unwrap().inventory.get(0).unwrap().id, 22521);
    }
}
