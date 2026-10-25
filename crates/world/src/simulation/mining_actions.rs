use super::*;
use crate::locations::Location;
#[cfg(test)]
#[path="coal_bag_mining_tests.rs"]
mod coal_bag_tests;
#[cfg(test)]
#[path="mining_gems_tests.rs"]
mod gem_tests;
#[cfg(test)]
#[path="clay_essence_mining_tests.rs"]
mod clay_essence_tests;
impl World {
    pub(super) fn process_mining(
        &mut self,
        slot: usize,
        loc: Location,
        random: &mut impl RandomSource,
    ) {
        let config = self.mining.as_ref().unwrap();
        let rock = config.rocks[&loc.id].clone();
        let skill = config.skill;
        let sound = config.full_sound;
        let continue_stage = rock.restore.is_some() && config.rocks.contains_key(&rock.depleted);
        let gems = self.mining_gems.clone();
        let essence = self.mining_essence.clone().filter(|e| e.rocks.contains(&loc.id));
        let p = self.players.get_mut(slot).unwrap();
        let actor = p.id;
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
        let clock = self.clock.0;
        let revision = p.location_revision.unwrap_or(0);
        let level = p.data.stats.current(skill) as i32;
        let pick = config.pick_for_member(&p.data.inventory, &p.data.equipment, level, p.member);
        let (ore, ore_name) = match &essence {
            Some(e) => {
                let (item, name) = e.product(&rock, p.member, level);
                (item, name.to_string())
            }
            None => (rock.product, rock.product_name.clone()),
        };
        let mut trace = crate::gathering::Trace {
            tick: clock,
            location: loc,
            revision,
            action_delay: p.action_delay,
            result: "waiting",
            roll: None,
            threshold: None,
            product: ore,
            product_delta: 0,
            stored_delta: 0,
            fine_xp_delta: 0,
            level,
            required_level: rock.level,
            tool: pick.map(|(id, _)| id),
            free_slots: p.data.inventory.free_space(),
            product_before: p
                .data
                .inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == ore)
                .map(|i| i64::from(i.count))
                .sum(),
            fine_xp_before: p.data.stats.fine_xp(skill),
            base_xp: rock.xp,
            xp_rate_hundreds: p.data.xp_rate_hundreds,
            depletion_roll: None,
            active_ticks: None,
            respawn_delay: None,
        };
        let denial = if self.locations.revision(loc) != Some(revision) {
            Some(("stale_target", None))
        } else if rock.members && !p.member {
            Some(("members", Some("You need to be a member to mine this rock.".into())))
        } else if pick.is_none() {
            Some(("tool", Some("You need a pickaxe to mine this rock.".into())))
        } else if level < rock.level {
            Some((
                "level",
                Some(format!(
                    "You need a Mining level of {} to mine this rock.",
                    rock.level
                )),
            ))
        } else if trace.free_slots == 0 {
            Some((
                "capacity",
                Some(format!(
                    "Your inventory is too full to hold any more {}.",
                    ore_name.to_lowercase()
                )),
            ))
        } else {
            None
        };
        if let Some((reason, message)) = denial {
            if let Some(text) = message {
                self.events.push(WorldEvent::GameMessage { actor, text });
            }
            if reason == "tool" {
                self.events.push(WorldEvent::GameMessage {
                    actor,
                    text: "You do not have a pickaxe which you have the Mining level to use."
                        .into(),
                });
            }
            if reason == "capacity" {
                self.events.push(WorldEvent::CombatSound { actor, sound });
            }
            p.location_interaction = None;
            p.movement.interacting = false;
            p.movement.abort();
            trace.result = reason;
            self.events.push(WorldEvent::Mining { actor, trace });
            return;
        }
        let (_, pick) = pick.unwrap();
        p.movement.abort();
        if p.action_delay < clock {
            // Rust currently uses zero for a never-armed shared action clock.
            if !(p.action_delay > 0 && p.action_delay == clock - 1) {
                self.events.push(WorldEvent::GameMessage {
                    actor,
                    text: "You swing your pick at the rock.".into(),
                });
            }
            p.action_delay = self.clock.deadline(pick.action_delay(random) - 1);
            self.events.push(WorldEvent::PlayerCombatAnimation {
                actor,
                sequence: pick.animation,
            });
            trace.result = "scheduled";
        } else if p.action_delay == clock {
            let glory = gems.as_ref().is_some_and(|g| g.glory(&p.data.equipment));
            let gem_rock = gems.as_ref().filter(|g| g.gem_rocks.contains(&loc.id));
            // rollGemPreTable runs before the ore roll on eligible rocks.
            if let Some(g) = gems.as_ref().filter(|g| g.pre_roll_rocks.contains(&loc.id)) {
                let draw = random.below(crate::mining::Gems::pre_roll_total(glory));
                trace.roll = Some(draw);
                match g.pre_roll(draw) {
                    crate::mining::PreRoll::KeepOre => {}
                    crate::mining::PreRoll::SkipOre => {
                        trace.result = "gem_skip_ore";
                        trace.action_delay = p.action_delay;
                        self.events.push(WorldEvent::Mining { actor, trace });
                        return;
                    }
                    crate::mining::PreRoll::Gem(item) => {
                        // A gem find grants no experience, keeps the rock and keeps mining.
                        match crate::gathering::reward(&p.data, &self.object_types, skill, item, 0.0) {
                            Err(error) => {
                                trace.result = "transaction_rejected";
                                p.location_interaction = None;
                                p.movement.interacting = false;
                                self.events.push(WorldEvent::ActionRejected { actor, error });
                            }
                            Ok(reward) => {
                                reward.commit(&mut p.data);
                                trace.result = "gem_found";
                                trace.product = item;
                                trace.product_delta = 1;
                                self.events.push(WorldEvent::GameMessage {
                                    actor,
                                    text: format!(
                                        "You manage to mine some {}.",
                                        g.names[&item].to_lowercase()
                                    ),
                                });
                            }
                        }
                        trace.action_delay = p.action_delay;
                        self.events.push(WorldEvent::Mining { actor, trace });
                        return;
                    }
                }
                trace.roll = None;
            }
            // A charged glory triples a gem rock's success rates.
            let rates = if gem_rock.is_some() && glory {
                (rock.rates.0 * 3, rock.rates.1 * 3)
            } else {
                rock.rates
            };
            let threshold =
                crate::skills::success_rate(rates.0, rates.1, level.clamp(1, 99), 99)
                    .clamp(0.0, 1.0)
                    * 256.0;
            let roll = random.below(256);
            trace.roll = Some(roll);
            trace.threshold = Some(threshold);
            trace.result = "failed_roll";
            if (roll as f64) < threshold {
                let (product, product_name) = match gem_rock {
                    Some(g) => {
                        let gem = g
                            .gem_rock(random.below(crate::mining::GEM_ROCK_TOTAL))
                            .expect("RandomSource must return a value below its exclusive bound");
                        (gem, g.names[&gem].clone())
                    }
                    None => (ore, ore_name.clone()),
                };
                trace.product = product;
                match crate::gathering::mining_reward(
                    &p.data,
                    &self.object_types,
                    skill,
                    product,
                    rock.xp,
                    p.member,
                    self.gathering_outfits.get(&skill),
                    self.coal_bag,
                ) {
                    Err(error) => {
                        trace.result = "transaction_rejected";
                        p.location_interaction = None;
                        p.movement.interacting = false;
                        self.events
                            .push(WorldEvent::ActionRejected { actor, error });
                    }
                    Ok(reward) if rock.infinite() => {
                        // OpenRune isInfinite: no depletion; opLoc3 keeps mining the same rock.
                        let stored = reward.stored_delta;
                        let (advancement, delta) = reward.commit(&mut p.data);
                        trace.result = "success";
                        trace.product_delta = 1 - stored;
                        trace.stored_delta = stored;
                        trace.fine_xp_delta = delta;
                        if self.tutorial_rules_completed_varp.is_some()
                            && p.stat_events
                                .advance(&crate::tutorial_rules::bindings(), skill, advancement)
                                .is_err()
                        {
                            p.pending_logout = true;
                        }
                        self.events.push(WorldEvent::SkillAdvanced {
                            actor,
                            skill,
                            advancement,
                        });
                        self.events.push(WorldEvent::GameMessage {
                            actor,
                            text: format!(
                                "You manage to mine some {}.",
                                product_name.to_lowercase()
                            ),
                        });
                        trace.action_delay = p.action_delay;
                        self.events.push(WorldEvent::Mining { actor, trace });
                        return;
                    }
                    Ok(reward) => {
                        p.location_interaction = None;
                        p.movement.interacting = false;
                        trace.depletion_roll = Some(random.below(255) + 1);
                        let depleted =
                            Location::new(loc.coords, rock.depleted, loc.shape, loc.angle).unwrap();
                        if let Err(error) = self.change_location(depleted, true) {
                            trace.result = "transaction_rejected";
                            self.events.push(WorldEvent::LocationError(error));
                            self.events.push(WorldEvent::Mining { actor, trace });
                            return;
                        }
                        let revision = self.locations.revision(depleted).unwrap();
                        let restored=Location::new(loc.coords,rock.restore.unwrap_or(loc.id),loc.shape,loc.angle).unwrap();
                        self.location_timers.replace(
                            restored,
                            depleted,
                            revision,
                            self.clock,
                            rock.respawn,
                        );
                        let p = self.players.get_mut(slot).unwrap();
                        let stored=reward.stored_delta;
                        let (advancement, delta) = reward.commit(&mut p.data);
                        trace.respawn_delay = Some(rock.respawn);
                        trace.result = "success";
                        trace.product_delta = 1-stored;
                        trace.stored_delta=stored;
                        trace.fine_xp_delta = delta;
                        if self.tutorial_rules_completed_varp.is_some()
                            && p.stat_events
                                .advance(&crate::tutorial_rules::bindings(), skill, advancement)
                                .is_err()
                        {
                            p.pending_logout = true;
                        }
                        self.events.push(WorldEvent::SkillAdvanced {
                            actor,
                            skill,
                            advancement,
                        });
                        self.events.push(WorldEvent::GameMessage {
                            actor,
                            text: format!(
                                "You manage to mine some {}.",
                                product_name.to_lowercase()
                            ),
                        });
                        if continue_stage {
                            p.location_revision=Some(revision);
                            p.location_interaction=Some((depleted,LocationAction::Operation(1)));
                            p.movement.interacting=true;
                        } else {
                            p.location_interaction = None;
                            p.movement.interacting = false;
                            self.events.push(WorldEvent::PlayerCombatAnimation {
                                actor,
                                sequence: u16::MAX,
                            });
                        }
                        self.events.push(WorldEvent::Mining { actor, trace });
                        return;
                    }
                }
            }
        }
        trace.action_delay = p.action_delay;
        self.events.push(WorldEvent::Mining { actor, trace });
    }
}
