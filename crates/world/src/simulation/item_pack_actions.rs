use super::*;

// Internal server queue identifier, deliberately outside nonnegative cache IDs.
const PACK_QUEUE: i32 = -2391001;

pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    catalog: &crate::inventory::Catalog,
    pack: i32,
    rule: crate::item_packs::Rule,
    members: bool,
    events: &mut Vec<WorldEvent>,
) {
    if p.saving
        || p.pending_logout
        || p.death_queued
        || p.data.stats.current(3) as u8 == 0
        || (members && !p.member)
    {
        return;
    }
    let Some(slot) = p
        .data
        .inventory
        .slots()
        .iter()
        .position(|v| v.is_some_and(|i| i.id == pack))
    else {
        return;
    };
    open_and_queue(p, queues, catalog, slot, pack, rule, events);
}

fn open_and_queue(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    catalog: &crate::inventory::Catalog,
    slot: usize,
    pack: i32,
    rule: crate::item_packs::Rule,
    events: &mut Vec<WorldEvent>,
) {
    if crate::item_packs::open(&mut p.data.inventory, catalog, slot, pack, rule).is_err() {
        events.push(WorldEvent::LocationMessage {
            actor: p.id,
            message: "You don't have enough inventory space to open that.",
        });
        return;
    }
    if p.data
        .inventory
        .slots()
        .iter()
        .flatten()
        .any(|item| item.id == pack)
    {
        // Preserve the existing weak-queue lifecycle. The Wiki specifies one or
        // two ticks; the concrete scheduler phase is characterized in tests.
        let _ = queues.add(
            PACK_QUEUE,
            crate::queues::Category::Weak,
            2,
            Deferred::ItemPack { pack, rule },
        );
    }
}

impl World {
    pub(super) fn open_item_pack(
        &mut self,
        actor: ActorId,
        slot: usize,
        pack: i32,
    ) -> Result<(), ActionError> {
        let rule = *self.item_packs.get(&pack).ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if p.death_queued || p.data.stats.current(3) as u8 == 0 {
            return Err(ActionError::Protected);
        }
        if self.member_item_packs.contains(&pack) && !p.member {
            return Err(ActionError::InvalidItem);
        }
        p.begin_held_action();
        p.queues.clear_weak();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if p.data
            .inventory
            .get(slot)
            .is_none_or(|item| item.id != pack)
        {
            return Err(ActionError::InvalidItem);
        }
        let mut queues = std::mem::take(&mut p.queues);
        open_and_queue(
            p,
            &mut queues,
            &self.object_types,
            slot,
            pack,
            rule,
            &mut self.events,
        );
        p.queues = queues;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (World, ActorId) {
        let mut w = World::default();
        let pos = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(3200, 3200, 0);
        w.queue_login(
            1,
            PlayerData::new(1, "Packer".into(), pos, Stats::default()),
        )
        .unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        w.object_types.stackable.insert(314);
        w.item_packs.insert(
            11881,
            crate::item_packs::Rule {
                output: 314,
                count: 100,
                noted: false,
            },
        );
        for slot in 0..3 {
            w.data_mut(actor)
                .unwrap()
                .inventory
                .restore_slot(
                    slot,
                    Some(crate::inventory::Item {
                        id: 11881,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        (w, actor)
    }
    #[test]
    fn item_pack_life_refuses_initial_or_queued_exchange_after_unresolved_death() {
        use crate::hits::{Hit, Style, Target};
        let mut cases = 0;
        for late in [false, true] {
            for phase in 0..3 {
                let (mut w, actor) = setup();
                for slot in 3..28 {
                    w.data_mut(actor)
                        .unwrap()
                        .inventory
                        .restore_slot(
                            slot,
                            Some(crate::inventory::Item {
                                id: 1511,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                let hit = Hit {
                    display: None,
                    source: None,
                    style: Style::Typeless,
                    damage: 99,
                };
                if late {
                    w.submit(
                        actor,
                        Action::OpenItemPack {
                            slot: 0,
                            item: 11881,
                        },
                    )
                    .unwrap();
                    if phase > 0 {
                        w.queue_hit(Target::Player(actor), hit, 0).unwrap();
                    }
                    w.tick().unwrap();
                    assert_eq!(
                        crate::smithing_make::total(
                            &w.data(actor).unwrap().inventory,
                            &w.object_types,
                            314
                        ),
                        100
                    );
                    if phase > 0 {
                        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(a))if *a==actor)));
                    }
                }
                if phase == 0 {
                    w.data_mut(actor).unwrap().stats.set_current(3, 0);
                } else {
                    if !late {
                        w.queue_hit(Target::Player(actor), hit, 0).unwrap();
                        w.tick().unwrap();
                        assert!(w.drain_events().iter().any(|e|matches!(e,WorldEvent::DeathRequested(Target::Player(a))if *a==actor)));
                    }
                    if phase == 2 {
                        w.data_mut(actor).unwrap().stats.set_current(3, 10);
                    }
                }
                let before = w.data(actor).unwrap().clone();
                if !late {
                    w.submit(
                        actor,
                        Action::OpenItemPack {
                            slot: 0,
                            item: 11881,
                        },
                    )
                    .unwrap();
                }
                for _ in 0..8 {
                    w.tick().unwrap();
                }
                let after = w.data(actor).unwrap();
                assert_eq!(
                    after.inventory.slots(),
                    before.inventory.slots(),
                    "case {cases}"
                );
                assert_eq!(after.stats, before.stats);
                assert!(!w.player(actor).unwrap().queues.contains(PACK_QUEUE));
                cases += 1;
            }
        }
        assert_eq!(cases, 6);
    }
    #[test]
    fn item_pack_membership_is_live_at_admission_and_repeat_and_cannot_revive_old_queue() {
        let (mut w, actor) = setup();
        w.member_item_packs.insert(11881);
        let before = w.data(actor).unwrap().clone();
        w.submit(
            actor,
            Action::OpenItemPack {
                slot: 0,
                item: 11881,
            },
        )
        .unwrap();
        for _ in 0..4 {
            w.tick().unwrap();
        }
        assert_eq!(
            w.data(actor).unwrap().inventory.slots(),
            before.inventory.slots()
        );
        assert_eq!(w.data(actor).unwrap().stats, before.stats);
        w.set_membership(actor, true).unwrap();
        w.submit(
            actor,
            Action::OpenItemPack {
                slot: 0,
                item: 11881,
            },
        )
        .unwrap();
        w.tick().unwrap();
        let opened = w.data(actor).unwrap().clone();
        assert_eq!(
            crate::smithing_make::total(&opened.inventory, &w.object_types, 314),
            100
        );
        w.set_membership(actor, false).unwrap();
        for _ in 0..5 {
            w.tick().unwrap();
        }
        assert_eq!(
            w.data(actor).unwrap().inventory.slots(),
            opened.inventory.slots()
        );
        assert!(!w.player(actor).unwrap().queues.contains(PACK_QUEUE));
        w.set_membership(actor, true).unwrap();
        for _ in 0..5 {
            w.tick().unwrap();
        }
        assert_eq!(
            w.data(actor).unwrap().inventory.slots(),
            opened.inventory.slots()
        );
        assert_eq!(w.data(actor).unwrap().stats, before.stats);
        w.submit(
            actor,
            Action::OpenItemPack {
                slot: 1,
                item: 11881,
            },
        )
        .unwrap();
        w.tick().unwrap();
        assert_eq!(
            crate::smithing_make::total(&w.data(actor).unwrap().inventory, &w.object_types, 314),
            200
        );
    }
    #[test]
    fn selected_pack_opens_then_repeats_and_movement_cancels() {
        for cancel in [false, true] {
            let (mut w, actor) = setup();
            let xp = w.data(actor).unwrap().stats.clone();
            w.submit(
                actor,
                Action::OpenItemPack {
                    slot: 0,
                    item: 11881,
                },
            )
            .unwrap();
            w.tick().unwrap();
            assert_eq!(
                crate::smithing_make::total(
                    &w.data(actor).unwrap().inventory,
                    &w.object_types,
                    314
                ),
                100
            );
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
            let mut previous = 100;
            let mut last_reward = 0;
            for tick in 1..=8 {
                w.tick().unwrap();
                let count = crate::smithing_make::total(
                    &w.data(actor).unwrap().inventory,
                    &w.object_types,
                    314,
                );
                if count != previous {
                    assert_eq!(count - previous, 100);
                    assert!((1..=2).contains(&(tick - last_reward)));
                    last_reward = tick;
                    previous = count;
                }
            }
            let data = w.data(actor).unwrap();
            assert_eq!(
                crate::smithing_make::total(&data.inventory, &w.object_types, 314),
                if cancel { 100 } else { 300 }
            );
            assert_eq!(data.stats, xp);
            assert!(!w.player(actor).unwrap().queues.contains(PACK_QUEUE));
        }
    }
    #[test]
    fn stale_slot_never_opens_a_matching_pack_elsewhere() {
        let (mut w, actor) = setup();
        let before = w.data(actor).unwrap().inventory.clone();
        w.submit(
            actor,
            Action::OpenItemPack {
                slot: 20,
                item: 11881,
            },
        )
        .unwrap();
        for _ in 0..6 {
            w.tick().unwrap();
        }
        assert_eq!(w.data(actor).unwrap().inventory, before);
    }
}

impl World {
    /// One atomic selected-stack exchange; no pack repeat queue or skill award.
    pub(super) fn open_coin_pouch(
        &mut self,
        actor: ActorId,
        slot: usize,
        item: i32,
        all: bool,
    ) -> Result<(), ActionError> {
        let spec = *self
            .coin_pouches
            .get(&item)
            .ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.queues.clear_weak();
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        if p.has_modal
            || (spec.members && !p.member)
            || p.data.inventory.stack != crate::inventory::StackType::Normal
            || p.data.inventory.placeholders
        {
            return Err(ActionError::InvalidItem);
        }
        let ordinary = |id| {
            id >= 0
                && !self.object_types.dummy.contains(&id)
                && !self
                    .object_types
                    .notes
                    .get(&id)
                    .is_some_and(|v| v.template != 0)
                && !self
                    .object_types
                    .placeholders
                    .get(&id)
                    .is_some_and(|v| v.template != 0)
                && !self
                    .object_types
                    .transforms
                    .get(&id)
                    .is_some_and(|v| v.template != 0)
        };
        if !ordinary(item)
            || !ordinary(spec.rule.output)
            || spec.rule.noted
            || !self.object_types.stackable.contains(&item)
            || !self.object_types.stackable.contains(&spec.rule.output)
        {
            return Err(ActionError::InvalidItem);
        }
        let selected = p
            .data
            .inventory
            .get(slot)
            .filter(|v| v.id == item && v.count > 0 && v.vars == 0)
            .ok_or(ActionError::InvalidItem)?;
        if p.data
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|v| v.id == spec.rule.output && (v.vars != 0 || v.count <= 0))
        {
            return Err(ActionError::InvalidItem);
        }
        let count = if all { selected.count } else { 1 };
        if crate::item_packs::open_count(
            &mut p.data.inventory,
            &self.object_types,
            slot,
            item,
            spec.rule,
            count,
        )
        .is_err()
        {
            self.events.push(WorldEvent::LocationMessage {
                actor,
                message: "You don't have enough inventory space to open that.",
            });
            return Ok(());
        }
        self.events.push(WorldEvent::LocationMessage {
            actor,
            message: if count > 1 {
                "You open all of the pouches."
            } else {
                "You open the coin pouch."
            },
        });
        Ok(())
    }
}

#[cfg(test)]
mod coin_pouch_tests {
    use super::*;
    use crate::{CoordGrid, inventory::Item, stats::Stats};
    fn setup(coins: Option<i32>) -> (World, ActorId) {
        let mut w = World::default();
        w.coin_pouches.insert(
            22521,
            crate::item_packs::PouchRule {
                rule: crate::item_packs::Rule {
                    output: 995,
                    count: 3,
                    noted: false,
                },
                members: true,
            },
        );
        w.object_types.stackable.extend([22521, 995]);
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(3200, 3200, 0);
        let mut data = PlayerData::new(1, "Pouch opener".into(), coords, Stats::default());
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
                27,
                Some(Item {
                    id: 22521,
                    count: 3,
                    vars: 0,
                }),
            )
            .unwrap();
        if let Some(count) = coins {
            data.inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: 995,
                        count,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.set_membership(a, true).unwrap();
        w.drain_events();
        (w, a)
    }
    fn coins(w: &World, a: ActorId) -> i32 {
        w.data(a)
            .unwrap()
            .inventory
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == 995)
            .map(|i| i.count)
            .sum()
    }
    #[test]
    fn coin_pouch_actions_open_one_or_all_immediately_without_repeat_xp_or_replay() {
        for (all, existing, expected_pouches, expected_coins) in [
            (false, Some(10), Some(2), 13),
            (true, Some(10), None, 19),
            (true, None, None, 9),
            (false, None, Some(3), 0),
        ] {
            let (mut w, a) = setup(existing);
            let before = w.data(a).unwrap().stats.clone();
            w.submit(
                a,
                Action::OpenCoinPouch {
                    slot: 27,
                    item: 22521,
                    all,
                },
            )
            .unwrap();
            w.tick().unwrap();
            assert_eq!(
                w.data(a)
                    .unwrap()
                    .inventory
                    .get(27)
                    .filter(|i| i.id == 22521)
                    .map(|i| i.count),
                expected_pouches
            );
            assert_eq!(coins(&w, a), expected_coins);
            for _ in 0..4 {
                w.tick().unwrap();
            }
            assert_eq!(coins(&w, a), expected_coins);
            assert_eq!(w.data(a).unwrap().stats, before);
        }
        let (mut w, a) = setup(None);
        for _ in 0..2 {
            w.submit(
                a,
                Action::OpenCoinPouch {
                    slot: 27,
                    item: 22521,
                    all: true,
                },
            )
            .unwrap();
        }
        w.tick().unwrap();
        assert_eq!(coins(&w, a), 9);
    }
    #[test]
    fn coin_pouch_refusals_preserve_selected_stack_currency_and_stats() {
        for bad in 0..6 {
            let (mut w, a) = setup(Some(if bad == 2 { i32::MAX - 8 } else { 10 }));
            if bad == 0 {
                w.set_membership(a, false).unwrap();
            }
            let p = w.data_mut(a).unwrap();
            if bad == 1 {
                p.inventory
                    .restore_slot(
                        27,
                        Some(Item {
                            id: 1511,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            if bad == 3 {
                p.inventory
                    .restore_slot(
                        27,
                        Some(Item {
                            id: 22521,
                            count: i32::MAX,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            if bad == 4 {
                p.inventory
                    .restore_slot(
                        27,
                        Some(Item {
                            id: 22521,
                            count: 3,
                            vars: 1,
                        }),
                    )
                    .unwrap();
            }
            if bad == 5 {
                p.inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 995,
                            count: 10,
                            vars: 1,
                        }),
                    )
                    .unwrap();
            }
            let before = (p.inventory.clone(), p.stats.clone());
            w.submit(
                a,
                Action::OpenCoinPouch {
                    slot: 27,
                    item: 22521,
                    all: true,
                },
            )
            .unwrap();
            w.tick().unwrap();
            let p = w.data(a).unwrap();
            assert_eq!((&p.inventory, &p.stats), (&before.0, &before.1));
        }
    }
}
