use super::*;

pub(super) fn hit(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    config: crate::hits::PlayerQueues,
    tutorial: bool,
    hit: crate::hits::Hit,
    clock: MapClock,
    alive_npcs: &std::collections::HashSet<NpcId>,
    events: &mut Vec<WorldEvent>,
) -> Result<(), ActionError> {
    if !hit.valid_player_source(|id| alive_npcs.contains(&id)) {
        return Ok(());
    }
    let impact = hit.impact(p.data.stats.current(3) as u8 as i32, false);
    if tutorial && impact.applied > 0 {
        p.stat_events
            .changed(&crate::tutorial_rules::bindings(), 3)
            .map_err(|_| ActionError::QueueFull)?;
    }
    p.data.stats.set_current(3, impact.health as i8);
    p.logout_blocked_until = clock.deadline(16);
    if impact.death && !p.death_queued && !queues.contains(config.death) {
        queues
            .add(
                config.death,
                crate::queues::Category::Normal,
                1,
                Deferred::PlayerDeath,
            )
            .map_err(|_| ActionError::QueueFull)?;
    }
    events.push(WorldEvent::Hit {
        target: crate::hits::Target::Player(p.id),
        impact,
    });
    Ok(())
}

pub(super) fn publish(
    p: &mut Player,
    completed_varp: Option<u16>,
    hit_queues: Option<crate::hits::PlayerQueues>,
    events: &mut Vec<WorldEvent>,
) {
    let Some(completed_varp) = completed_varp else {
        return;
    };
    let protected = p.protection.access_protected(p.has_modal);
    let mut pending = std::mem::take(&mut p.stat_events);
    let result = pending.process(protected, |kind, stat| {
        let effects = crate::tutorial_rules::deliver(
            &mut p.data.stats,
            p.data.variables.get(completed_varp),
            kind,
            stat,
        );
        if effects.clear_death_queue {
            if let Some(config) = hit_queues {
                p.queues.remove_all(config.death);
            }
        }
        if let Some(message) = effects.message {
            events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message,
            });
        }
        p.protection.access_protected(p.has_modal)
    });
    p.stat_events = pending;
    if result.is_err() {
        p.pending_logout = true;
        events.push(WorldEvent::ActionRejected {
            actor: p.id,
            error: ActionError::QueueFull,
        });
    }
}
