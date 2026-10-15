use super::*;
use crate::smithing_make::{Config, Job, Outcome, Request};
use std::sync::Arc;

impl World {
    /// Trusted smithing content entry, after recipe/level/UI validation. The
    /// player is never delayed; subsequent input can cancel the weak job.
    pub fn start_smithing(
        &mut self,
        actor: ActorId,
        request: Arc<Request>,
        count: i32,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        let config = self.smithing.ok_or(ActionError::InvalidItem)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.saving {
            return Err(ActionError::LoggingOut);
        }
        let Some(job) = request.start(count) else {
            return Ok(());
        };
        self.events.push(WorldEvent::PlayerCombatAnimation {
            actor,
            sequence: request.animation,
        });
        p.queues
            .add(
                config.queue,
                crate::queues::Category::Weak,
                request.cycles,
                Deferred::Smithing(job),
            )
            .map_err(|_| ActionError::QueueFull)
    }
}

pub(super) fn complete(
    p: &mut Player,
    queues: &mut crate::queues::Scheduler<Deferred>,
    config: Config,
    job: Job,
    catalog: &crate::inventory::Catalog,
    tutorial_stats: bool,
    events: &mut Vec<WorldEvent>,
) -> Result<(), ActionError> {
    match job
        .complete(
            &mut p.data.inventory,
            catalog,
            &mut p.data.stats,
            config.skill,
            config.xp_rate,
            config.max_level,
        )
        .map_err(|_| ActionError::InvalidStat)?
    {
        Outcome::MissingInputs => (),
        Outcome::NoSpace => events.push(WorldEvent::LocationMessage {
            actor: p.id,
            message: "You don't have enough inventory space to hold that item.",
        }),
        Outcome::Made {
            product,
            count,
            advancement,
            next,
        } => {
            if tutorial_stats {
                p.stat_events
                    .advance(
                        &crate::tutorial_rules::bindings(),
                        config.skill,
                        advancement,
                    )
                    .map_err(|_| ActionError::QueueFull)?;
            }
            events.push(WorldEvent::SkillAdvanced {
                actor: p.id,
                skill: config.skill,
                advancement,
            });
            events.push(WorldEvent::SmithingProductMade {
                actor: p.id,
                product,
                count,
            });
            // The original synchronously publishes SmithingProductMade before
            // deciding whether to animate and enqueue another iteration.
            if let Some((progression, bar, dagger)) = config.tutorial {
                let step = if product == bar {
                    Some(crate::tutorial_steps::Step::MineHammer)
                } else if product == dagger {
                    Some(crate::tutorial_steps::Step::MineGate)
                } else {
                    None
                };
                if let Some(step) = step {
                    let effects = progression.advance(
                        &mut p.data.variables,
                        step,
                        None,
                        p.protection.processed_clock > 0,
                    );
                    if !effects.is_empty() {
                        events.push(WorldEvent::TutorialInventoryProgress {
                            actor: p.id,
                            effects,
                        });
                    }
                }
            }
            if let Some(next) = next {
                events.push(WorldEvent::PlayerCombatAnimation {
                    actor: p.id,
                    sequence: next.request.animation,
                });
                queues
                    .add(
                        config.queue,
                        crate::queues::Category::Weak,
                        next.request.cycles,
                        Deferred::Smithing(next),
                    )
                    .map_err(|_| ActionError::QueueFull)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn prepared() -> (World, ActorId, Arc<Request>) {
        let mut world = World::default();
        let coords = CoordGrid::new(3200, 3200, 0).unwrap();
        world.collision.allocate(3200, 3200, 0);
        world
            .queue_login(
                1,
                PlayerData::new(1, "Smith".into(), coords, Stats::default()),
            )
            .unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        world.smithing = Some(Config {
            queue: 900,
            skill: 13,
            max_level: 99,
            xp_rate: 1.0,
            tutorial: None,
        });
        for id in [436, 438, 436, 438] {
            world
                .data_mut(actor)
                .unwrap()
                .inventory
                .insert(
                    &crate::inventory::Catalog::default(),
                    crate::inventory::Insert::new(id, 1),
                )
                .unwrap();
        }
        let request = Arc::new(Request {
            product: 2349,
            produced: 1,
            consumed: vec![(436, 1), (438, 1)],
            xp: 6.2,
            animation: 899,
            cycles: 5,
        });
        world.drain_events();
        (world, actor, request)
    }
    #[test]
    fn weak_make_animates_before_output_and_repeats_without_delaying_player() {
        let (mut world, actor, request) = prepared();
        world.start_smithing(actor, request, 28).unwrap();
        assert!(!world.protection(actor).unwrap().delayed());
        assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 0);
        assert!(world
            .drain_events()
            .iter()
            .any(|e| matches!(e, WorldEvent::PlayerCombatAnimation { sequence: 899, .. })));
        for _ in 0..4 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 0);
        world.tick().unwrap();
        assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 62);
        for _ in 0..5 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 124);
        assert!(!world.player(actor).unwrap().queues.contains(900));
        assert!(!world.protection(actor).unwrap().delayed());
    }
    #[test]
    fn normal_smelting_characterizes_single_and_repeat_queue_phases() {
        // Characterization of the existing PG-compatible weak scheduler, not
        // acceptance of the Wiki first/repeat/single 4/5/6-tick profile.
        for (count, regular_jobs, expected) in [
            (1, 0, vec![5]),
            (2, 0, vec![5, 9]),
            (2, 1, vec![5, 10]),
            (2, 2, vec![5, 9]),
        ] {
            let (mut world, actor, request) = prepared();
            for index in 0..regular_jobs {
                world
                    .players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .queues
                    .add(
                        910 + index,
                        crate::queues::Category::Normal,
                        100,
                        Deferred::Preserve,
                    )
                    .unwrap();
            }
            world.start_smithing(actor, request, count).unwrap();
            let mut made_at = Vec::new();
            for tick in 1..=12 {
                world.tick().unwrap();
                for event in world.drain_events() {
                    if let WorldEvent::SmithingProductMade {
                        actor: made_by,
                        product,
                        count,
                    } = event
                    {
                        assert_eq!(made_by, actor);
                        assert_eq!((product, count), (2349, 1));
                        made_at.push(tick);
                    }
                }
                let data = world.data(actor).unwrap();
                assert_eq!(data.stats.fine_xp(13), made_at.len() as i32 * 62);
                for ore in [436, 438] {
                    assert_eq!(
                        crate::smithing_make::total(&data.inventory, &world.object_types, ore),
                        2 - made_at.len() as i32
                    );
                }
            }
            assert_eq!(
                made_at, expected,
                "count={count}, regular_jobs={regular_jobs}"
            );
            assert!(!world.player(actor).unwrap().queues.contains(900));
        }
    }

    #[test]
    fn clicking_away_cancels_remaining_smithing_before_consumption() {
        let (mut world, actor, request) = prepared();
        world.start_smithing(actor, request, 28).unwrap();
        world.tick().unwrap();
        world
            .submit(
                actor,
                Action::MoveClick {
                    destination: CoordGrid::new(3202, 3200, 0).unwrap(),
                    control: false,
                },
            )
            .unwrap();
        for _ in 0..10 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 0);
        assert_eq!(
            world
                .data(actor)
                .unwrap()
                .inventory
                .slots()
                .iter()
                .flatten()
                .count(),
            4
        );
        assert_eq!(world.observe(actor).unwrap().coords.x(), 3202);
    }
}
