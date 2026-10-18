use super::*;
use crate::tool_crafting::{Error, Job, Pending, QUEUE};
impl World {
    pub(super) fn start_tool_crafting(
        &mut self,
        actor: ActorId,
        job: Job,
    ) -> Result<(), ActionError> {
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if let Err(e) = job.prepare(
            &p.data.inventory,
            &self.object_types,
            &p.data.stats,
            p.data.xp_rate_hundreds,
            p.member,
        ) {
            refusal(&mut self.events, actor, e);
            return Ok(());
        }
        p.queues.clear_weak();
        let pending = Pending {
            job,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
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
fn refusal(events: &mut Vec<WorldEvent>, actor: ActorId, error: Error) {
    events.push(WorldEvent::GameMessage {
        actor,
        text: match error {
            Error::Members => "You need to be a member to craft that item.",
            Error::Level => "You need a higher Crafting level to craft that item.",
            _ => "You can't craft that item right now.",
        }
        .into(),
    });
}
pub(super) fn complete(
    p: &mut Player,
    pending: Pending,
    items: &crate::inventory::Catalog,
    tutorial: bool,
    events: &mut Vec<WorldEvent>,
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
    match pending.job.prepare(
        &p.data.inventory,
        items,
        &p.data.stats,
        p.data.xp_rate_hundreds,
        p.member,
    ) {
        Ok(prepared) => {
            p.data.inventory = prepared.inventory;
            p.data.stats = prepared.stats;
            if tutorial
                && p.stat_events
                    .advance(&crate::tutorial_rules::bindings(), 12, prepared.advancement)
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
                advancement: prepared.advancement,
            });
            events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: format!("You cut the {}.", pending.job.recipe.name),
            });
        }
        Err(e) => refusal(events, p.id, e),
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
