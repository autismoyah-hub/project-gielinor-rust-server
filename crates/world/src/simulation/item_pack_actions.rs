use super::*;

// Internal server queue identifier, deliberately outside nonnegative cache IDs.
const PACK_QUEUE: i32 = -2391001;

pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    catalog: &crate::inventory::Catalog,
    pack: i32,
    rule: crate::item_packs::Rule,
    events: &mut Vec<WorldEvent>,
) {
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
