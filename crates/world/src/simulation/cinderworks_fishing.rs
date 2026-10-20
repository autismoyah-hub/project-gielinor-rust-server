//! Authored net practice, using existing queues and atomic gathering rewards.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Job {
    pub target: NpcId,
    pub coords: CoordGrid,
    origin: CoordGrid,
    last_movement: i32,
    run_id: u32,
}
impl World {
    pub(super) fn start_cinderworks_fishing(&mut self, slot: usize, target: NpcId) {
        let Some(config) = self.cinderworks.as_ref() else {
            return;
        };
        let Some(work) = config.work else {
            return;
        };
        let Some(npc) = self.npc(target).filter(|n| n.type_id == work.fishing_spot) else {
            return;
        };
        let coords = npc.movement.coords;
        let p = self.players.get_mut(slot).unwrap();
        if !crate::tower_layout::contains(config.origin, p.movement.coords) {
            return;
        }
        if p.data.tower_run.id == 0 {
            self.events.push(WorldEvent::GameMessage {
                actor: p.id,
                text: p.data.tower_run.guide().into(),
            });
            return;
        }
        if !p
            .data
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| i.id == work.net)
        {
            self.events.push(WorldEvent::GameMessage{actor:p.id,text:"Bring your small fishing net. Missing tools can be recovered from the sanctuary plaque.".into()});
            return;
        }
        let job = Job {
            target,
            coords,
            origin: p.movement.coords,
            last_movement: p.movement.last_movement,
            run_id: p.data.tower_run.id,
        };
        p.queues.clear_weak();
        if p.queues
            .add(
                work.queue,
                crate::queues::Category::Weak,
                2,
                Deferred::CinderworksFishing(job),
            )
            .is_err()
        {
            self.events.push(WorldEvent::ActionRejected {
                actor: p.id,
                error: ActionError::QueueFull,
            });
            return;
        }
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor: p.id,
            sequence: work.fish_animation,
        });
        self.events.push(WorldEvent::GameMessage {
            actor: p.id,
            text: "You cast your net into the water. Moving away stops fishing.".into(),
        });
    }
}
pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    job: Job,
    config: &crate::tower_cinderworks::Config,
    target_valid: bool,
    items: &crate::inventory::Catalog,
    tutorial_stats: bool,
    events: &mut Vec<WorldEvent>,
) {
    let Some(work) = config.work else {
        return;
    };
    if !target_valid
        || p.saving
        || p.pending_logout
        || p.has_modal
        || p.movement.coords != job.origin
        || p.movement.last_movement != job.last_movement
        || p.data.tower_run.id != job.run_id
        || !crate::tower_layout::contains(config.origin, p.movement.coords)
        || p.movement.coords.chebyshev_distance(job.coords) > 3
        || !p
            .data
            .inventory
            .slots()
            .iter()
            .flatten()
            .any(|i| i.id == work.net)
    {
        return;
    }
    let reward = match crate::gathering::reward(&p.data, items, work.fish_skill, work.raw, 10.0) {
        Ok(reward) => reward,
        Err(error) => {
            events.push(WorldEvent::ActionRejected { actor: p.id, error });
            return;
        }
    };
    let (advancement, _) = reward.commit(&mut p.data);
    if tutorial_stats
        && p.stat_events
            .advance(
                &crate::tutorial_rules::bindings(),
                work.fish_skill,
                advancement,
            )
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
        skill: work.fish_skill,
        advancement,
    });
    events.push(WorldEvent::PlayerCombatAnimation {
        actor: p.id,
        sequence: work.fish_animation,
    });
    events.push(WorldEvent::GameMessage {
        actor: p.id,
        text: "You catch some raw shrimp.".into(),
    });
    if queues
        .add(
            work.queue,
            crate::queues::Category::Weak,
            4,
            Deferred::CinderworksFishing(job),
        )
        .is_err()
    {
        p.pending_logout = true;
        events.push(WorldEvent::ActionRejected {
            actor: p.id,
            error: ActionError::QueueFull,
        });
    }
}
