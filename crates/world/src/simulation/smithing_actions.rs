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
    cinderworks: Option<&crate::tower_cinderworks::Config>,
    events: &mut Vec<WorldEvent>,
    random: &mut impl RandomSource,
) -> Result<(), ActionError> {
    let next = match job
        .complete(
            &mut p.data.inventory,
            catalog,
            &mut p.data.stats,
            config.skill,
            config.xp_rate * p.data.xp_rate_hundreds as f64 / 100.0,
            config.max_level,
            random,
        )
        .map_err(|_| ActionError::InvalidStat)?
    {
        Outcome::MissingInputs => None,
        Outcome::RequirementsChanged => {
            events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "You no longer meet the requirements to make that item.",
            });
            None
        }
        Outcome::NoSpace => {
            events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "You don't have enough inventory space to hold that item.",
            });
            None
        }
        Outcome::Impure { next } => {
            events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "The ore is too impure to make a bar.",
            });
            next
        }
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
            if let Some(config) = cinderworks.filter(|config| config.forged_weapon(product)) {
                if let Some(text) = config.credit(&mut p.data, p.movement.coords, crate::tower_run::Service::Forge) {
                    events.push(WorldEvent::CinderworksProgress {actor:p.id,run:p.data.tower_run});
                    events.push(WorldEvent::GameMessage { actor:p.id, text:text.into() });
                }
            }
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
            next
        }
    };
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
            impurity: false,
            required_level: 1,
            required_tools: vec![],
        });
        world.drain_events();
        (world, actor, request)
    }
    #[test]
    fn queued_smithing_applies_character_rate_in_addition_to_global_rate() {
        for (rate,expected) in [(100,62),(500,310)] {
            let (mut world,actor,request)=prepared();
            world.data_mut(actor).unwrap().xp_rate_hundreds=rate;
            world.start_smithing(actor,request,1).unwrap();
            for _ in 0..7 {world.tick().unwrap();}
            assert_eq!(world.data(actor).unwrap().stats.fine_xp(13),expected);
        }
    }
    #[test]
    fn changed_level_or_removed_tool_stops_pending_and_repeated_jobs() {
        for after_first in [false, true] {
            for remove_tool in [false, true] {
                let (mut world, actor, mut request) = prepared();
                Arc::get_mut(&mut request).unwrap().required_tools = vec![2347];
                world
                    .data_mut(actor)
                    .unwrap()
                    .inventory
                    .insert(
                        &crate::inventory::Catalog::default(),
                        crate::inventory::Insert::new(2347, 1),
                    )
                    .unwrap();
                world.start_smithing(actor, request, 2).unwrap();
                if after_first {
                    for _ in 0..5 {
                        world.tick().unwrap();
                    }
                    assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 62);
                }
                if remove_tool {
                    world
                        .data_mut(actor)
                        .unwrap()
                        .inventory
                        .delete(
                            &crate::inventory::Catalog::default(),
                            crate::inventory::Delete::new(2347, 1),
                        )
                        .unwrap();
                } else {
                    world.data_mut(actor).unwrap().stats.set_current(13, 0);
                }
                let before = world.data(actor).unwrap().clone();
                world.drain_events();
                for _ in 0..15 {
                    world.tick().unwrap();
                }
                let after = world.data(actor).unwrap();
                assert_eq!(after.inventory, before.inventory);
                assert_eq!(after.stats.fine_xp(13), before.stats.fine_xp(13));
                assert!(!world.player(actor).unwrap().queues.contains(900));
                let events = world.drain_events();
                assert!(!events.iter().any(|event| matches!(
                    event,
                    WorldEvent::SmithingProductMade { .. } | WorldEvent::SkillAdvanced { .. }
                )));
                assert!(events.iter().any(|event| matches!(
                    event,
                    WorldEvent::LocationMessage {
                        message: "You no longer meet the requirements to make that item.",
                        ..
                    }
                )));
            }
        }
    }
    #[test]
    fn ordinary_iron_failure_continues_or_cancels_through_existing_weak_queue() {
        struct Draw(VecDeque<u32>);
        impl RandomSource for Draw {
            fn below(&mut self, bound: u32) -> u32 {
                assert_eq!(bound, 2);
                self.0
                    .pop_front()
                    .expect("no draw after completion/cancellation")
            }
        }
        for cancel in [false, true] {
            let (mut world, actor, mut request) = prepared();
            let recipe = Arc::get_mut(&mut request).unwrap();
            recipe.product = 2351;
            recipe.consumed = vec![(440, 1)];
            recipe.xp = 12.5;
            recipe.impurity = true;
            let data = world.data_mut(actor).unwrap();
            data.inventory = Inventory::new(28, StackType::Normal, false);
            for _ in 0..2 {
                data.inventory
                    .insert(
                        &crate::inventory::Catalog::default(),
                        crate::inventory::Insert::new(440, 1),
                    )
                    .unwrap();
            }
            let mut random = Draw(VecDeque::from([0, 1]));
            world.start_smithing(actor, request, 2).unwrap();
            for _ in 0..5 {
                world.tick_with_random(&mut random).unwrap();
            }
            assert_eq!(world.data(actor).unwrap().stats.fine_xp(13), 0);
            assert_eq!(
                crate::smithing_make::total(
                    &world.data(actor).unwrap().inventory,
                    &world.object_types,
                    440
                ),
                1
            );
            let events = world.drain_events();
            assert!(events.iter().any(|e| matches!(
                e,
                WorldEvent::LocationMessage {
                    message: "The ore is too impure to make a bar.",
                    ..
                }
            )));
            assert!(!events.iter().any(|e| matches!(
                e,
                WorldEvent::SmithingProductMade { .. } | WorldEvent::SkillAdvanced { .. }
            )));
            if cancel {
                world
                    .submit(
                        actor,
                        Action::MoveClick {
                            destination: CoordGrid::new(3202, 3200, 0).unwrap(),
                            control: false,
                        },
                    )
                    .unwrap();
            }
            for _ in 0..12 {
                world.tick_with_random(&mut random).unwrap();
            }
            let data = world.data(actor).unwrap();
            assert_eq!(data.stats.fine_xp(13), if cancel { 0 } else { 125 });
            assert_eq!(
                crate::smithing_make::total(&data.inventory, &world.object_types, 2351),
                if cancel { 0 } else { 1 }
            );
            assert_eq!(
                crate::smithing_make::total(&data.inventory, &world.object_types, 440),
                if cancel { 1 } else { 0 }
            );
            assert_eq!(random.0.len(), usize::from(cancel));
            assert!(!world.player(actor).unwrap().queues.contains(900));
        }
    }
    #[test]
    fn invalid_advancement_rejects_queued_make_without_consumption_or_credit() {
        let (mut world, actor, request) = prepared();
        world.smithing.as_mut().unwrap().xp_rate = f64::INFINITY;
        let before = world.data(actor).unwrap().clone();
        world.start_smithing(actor, request.clone(), 2).unwrap();
        for _ in 0..10 {
            world.tick().unwrap();
        }
        let data = world.data(actor).unwrap();
        assert_eq!(data.inventory, before.inventory);
        assert_eq!(data.stats, before.stats);
        assert!(!world.player(actor).unwrap().queues.contains(900));
        let events = world.drain_events();
        assert!(events.iter().any(|event| matches!(event,
            WorldEvent::ActionRejected { actor: a, error: ActionError::InvalidStat } if *a == actor
        )));
        assert!(!events.iter().any(|event| matches!(
            event,
            WorldEvent::SmithingProductMade { .. } | WorldEvent::SkillAdvanced { .. }
        )));
        // The existing fatal-action path requests a save and logout. Its save
        // snapshot must contain the untouched ingredients, not a free product.
        let saved_data = events
            .iter()
            .find_map(|event| match event {
                WorldEvent::Save { actor: a, data } if *a == actor => Some(data.clone()),
                _ => None,
            })
            .expect("failed make must request a save");
        assert_eq!(saved_data.inventory, before.inventory);
        assert_eq!(saved_data.stats, before.stats);
        world.saved(actor).unwrap();
        world.tick().unwrap();
        world.queue_login(2, saved_data).unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        // Retrying the restored snapshot after correcting content credits once.
        world.smithing.as_mut().unwrap().xp_rate = 1.0;
        world.start_smithing(actor, request, 1).unwrap();
        for _ in 0..6 {
            world.tick().unwrap();
        }
        let data = world.data(actor).unwrap();
        assert_eq!(data.stats.fine_xp(13), 62);
        assert_eq!(
            crate::smithing_make::total(&data.inventory, &world.object_types, 2349),
            1
        );
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
