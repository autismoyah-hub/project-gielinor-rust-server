use super::*;
use crate::locations::Location;
impl World {
    pub(super) fn process_woodcutting(
        &mut self,
        slot: usize,
        loc: Location,
        action: LocationAction,
        random: &mut impl RandomSource,
    ) {
        let config = self.woodcutting.as_ref().unwrap();
        let tree = config.trees[&loc.id].clone();
        let skill = config.skill;
        let full_sound = config.full_sound;
        let fall_sound = config.fall_sound;
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
        let clock = self.clock.0;
        if config.validate().is_err() {
            p.location_interaction = None;
            p.movement.interacting = false;
            p.movement.abort();
            self.events.push(WorldEvent::ActionRejected {
                actor,
                error: ActionError::InvalidItem,
            });
            return;
        }
        let revision = p.location_revision.unwrap_or(0);
        let mut trace = crate::woodcutting::Trace {
            tick: clock,
            location: loc,
            revision,
            action_delay: p.action_delay,
            result: "waiting",
            roll: None,
            threshold: None,
            product: tree.product,
            product_delta: 0,
            fine_xp_delta: 0,
            level: p.data.stats.current(skill) as i32,
            required_level: tree.level,
            tool: None,
            free_slots: p.data.inventory.free_space(),
            product_before: p
                .data
                .inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == tree.product)
                .map(|i| i64::from(i.count))
                .sum(),
            fine_xp_before: p.data.stats.fine_xp(skill),
            base_xp: tree.xp,
            xp_rate_hundreds: p.data.xp_rate_hundreds,
            depletion_roll: None,
            active_ticks: None,
            respawn_delay: None,
        };
        let mut denial = None;
        if self.locations.revision(loc) != Some(revision) {
            denial = Some("stale_target");
        }
        if let LocationAction::HeldUse { slot, object } = action {
            if !config.axes.contains_key(&object)
                || usize::try_from(slot)
                    .ok()
                    .and_then(|s| p.data.inventory.get(s))
                    .is_none_or(|i| i.id != object)
            {
                denial = Some("stale_tool");
            }
        }
        let level = p.data.stats.current(skill) as i32;
        let axe = config.axe_for_tree(&p.data.inventory, &p.data.equipment, level, &tree);
        trace.tool = axe.map(|(id, _)| id);
        if denial.is_none() && tree.members && !p.member {
            denial = Some("members");
            self.events.push(WorldEvent::GameMessage {actor,text:"You need to be a member to chop down this tree.".into()});
        }
        if denial.is_none() && level < tree.level {
            denial = Some("level");
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: format!(
                    "You need a Woodcutting level of {} to chop down this tree.",
                    tree.level
                ),
            });
        }
        if denial.is_none() && p.data.inventory.free_space() == 0 {
            denial = Some("capacity");
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: format!(
                    "Your inventory is too full to hold any more {}.",
                    tree.product_name.to_lowercase()
                ),
            });
            self.events.push(WorldEvent::CombatSound {
                actor,
                sound: full_sound,
            });
        }
        // Start on the first eligible reached interaction, including the initial
        // scheduled swing; resource success is not required to start the timer.
        let timed_depletion = if denial.is_none() && axe.is_some() {
            tree.chopping_ticks.map(|required| {
                let ticks = self.active_trees.swing(loc, revision, clock);
                trace.active_ticks = Some(ticks);
                ticks >= required
            })
        } else {
            None
        };
        if denial.is_none()
            && matches!(action, LocationAction::Operation(1))
            && p.action_delay < clock
        {
            p.action_delay = clock.wrapping_add(3);
            p.woodcutting_animation_at = clock.wrapping_add(3);
            trace.action_delay = p.action_delay;
            trace.result = "scheduled";
            p.movement.abort();
            self.events.push(WorldEvent::Woodcutting { actor, trace });
            return;
        }
        if denial.is_none() && axe.is_none() {
            denial = Some("tool");
            for text in [
                "You need an axe to chop down this tree.",
                "You do not have an axe which you have the woodcutting level to use.",
            ] {
                self.events.push(WorldEvent::GameMessage {
                    actor,
                    text: text.into(),
                });
            }
        }
        if let Some(reason) = denial {
            p.location_interaction = None;
            p.movement.interacting = false;
            p.movement.abort();
            trace.result = reason;
            self.events.push(WorldEvent::Woodcutting { actor, trace });
            return;
        }
        let (axe_id, axe) = axe.unwrap();
        if matches!(action, LocationAction::Operation(1)) {
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: axe.animation,
            });
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "You swing your axe at the tree.".into(),
            });
        }
        if p.woodcutting_animation_at <= clock {
            p.woodcutting_animation_at = clock.wrapping_add(4);
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: axe.animation,
            });
        }
        p.movement.abort();
        p.location_interaction = Some((loc, LocationAction::Operation(3)));
        if p.action_delay < clock {
            p.action_delay = clock.wrapping_add(3);
        } else if p.action_delay == clock {
            let Some(&(low, high)) = tree.rates.get(&axe_id) else {
                p.location_interaction = None;
                p.movement.interacting = false;
                self.events.push(WorldEvent::ActionRejected {
                    actor,
                    error: ActionError::InvalidItem,
                });
                return;
            };
            let threshold = crate::skills::success_rate(low, high, level.clamp(1, 99), 99)
                .clamp(0.0, 1.0)
                * 256.0;
            let roll = random.below(256);
            trace.roll = Some(roll);
            trace.threshold = Some(threshold);
            trace.result = "failed_roll";
            if (roll as f64) < threshold {
                let Ok(reward) = crate::gathering::reward_with_outfit(
                    &p.data,
                    &self.object_types,
                    skill,
                    tree.product,
                    tree.xp,
                    p.member,
                    self.gathering_outfits.get(&skill),
                ) else {
                    p.location_interaction = None;
                    p.movement.interacting = false;
                    self.events.push(WorldEvent::ActionRejected {
                        actor,
                        error: ActionError::InvalidItem,
                    });
                    trace.result = "transaction_rejected";
                    self.events.push(WorldEvent::Woodcutting { actor, trace });
                    return;
                };
                let depleted = timed_depletion.unwrap_or_else(|| {
                    let roll = random.below(255) + 1;
                    trace.depletion_roll = Some(roll);
                    roll > u32::from(tree.deplete_chance)
                });
                let replacement = if depleted {
                    p.location_interaction = None;
                    p.movement.interacting = false;
                    let duration = if tree.respawn[0] == tree.respawn[1] {
                        tree.respawn[0]
                    } else {
                        tree.respawn[0]
                            + random.below((tree.respawn[1] - tree.respawn[0] + 1) as u32) as i32
                    };
                    let stump =
                        Location::new(loc.coords, tree.stump, loc.shape, loc.angle).unwrap();
                    trace.action_delay = p.action_delay;
                    if let Err(error) = self.change_location(stump, true) {
                        trace.result = "transaction_rejected";
                        self.events.push(WorldEvent::LocationError(error));
                        self.events.push(WorldEvent::Woodcutting { actor, trace });
                        return;
                    }
                    let revision = self.locations.revision(stump).unwrap();
                    self.location_timers
                        .replace(loc, stump, revision, self.clock, duration);
                    Some(duration)
                } else {
                    None
                };
                let p = self.players.get_mut(slot).unwrap();
                let (advancement, delta) = reward.commit(&mut p.data);
                trace.fine_xp_delta = delta;
                trace.product_delta = 1;
                trace.result = "success";
                if self.tutorial_rules_completed_varp.is_some()
                    && p.stat_events
                        .advance(&crate::tutorial_rules::bindings(), skill, advancement)
                        .is_err()
                {
                    p.pending_logout = true;
                }
                self.events.push(WorldEvent::GameMessage {
                    actor,
                    text: format!("You get some {}.", tree.product_name.to_lowercase()),
                });
                self.events.push(WorldEvent::SkillAdvanced {
                    actor,
                    skill,
                    advancement,
                });
                trace.action_delay = p.action_delay;
                trace.respawn_delay = replacement;
                self.events.push(WorldEvent::Woodcutting { actor, trace });
                if depleted {
                    self.events.push(WorldEvent::PlayerCombatAnimation {
                        actor,
                        sequence: u16::MAX,
                    });
                    self.events.push(WorldEvent::CombatSound {
                        actor,
                        sound: fall_sound,
                    });
                    return;
                }
                return;
            }
        }
        trace.action_delay = p.action_delay;
        self.events.push(WorldEvent::Woodcutting { actor, trace });
    }
}
