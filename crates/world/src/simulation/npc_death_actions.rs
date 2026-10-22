use super::*;
use crate::npc_death::State;
impl World {
    /// Death coroutines resume before reveal, regeneration, queues and interactions.
    pub(super) fn process_npc_death(
        &mut self,
        id: NpcId,
        random: &mut impl crate::npc::RandomSource,
    ) -> bool {
        let Some(entry) = self.npcs.get(id.slot as usize).filter(|e| e.id == id) else {
            return true;
        };
        if !entry.death_queued {
            return false;
        }
        let Some(config) = &self.npc_death else {
            return true;
        };
        let Some(definition) = config.definitions.get(&entry.npc.type_id).copied() else {
            return true;
        };
        let bones = config.bones;
        let chicken = config.chickens.get(&entry.npc.type_id).copied();
        let extras = self.npc_guaranteed_extras.get(&entry.npc.type_id).cloned();
        let suppress_drops=self.npc_no_death_drops.contains(&entry.npc.type_id);
        let initial_hero=entry.npc.hero_points.winner();
        let initial_type=entry.npc.type_id;
        let clock = self.clock.0;
        if entry.death_state.is_none() {
            self.duck_death_reactions(id);
            if let Some((config, hero)) = self.tutorial_combat.zip(initial_hero) {
                let type_id = initial_type;
                // Match findHero: use the top character UUID; never fall back to a
                // lower-ranked online player when the winner has logged out.
                if let Some((_, player)) = self
                    .players
                    .iter_mut()
                    .find(|(_, p)| p.data.character_id == hero)
                {
                    let actor = player.id;
                    for effect in config.death(
                        type_id,
                        &mut player.data.variables,
                        player.protection.processed_clock > 0,
                    ) {
                        match effect {
                            crate::tutorial_combat::Effect::Message(message) => self
                                .events
                                .push(WorldEvent::LocationMessage { actor, message }),
                            crate::tutorial_combat::Effect::Progress(effects) => self
                                .events
                                .push(WorldEvent::TutorialCombatProgress { actor, effects }),
                        }
                    }
                }
            }
            let entry = self.npcs.get_mut(id.slot as usize).unwrap();
            let npc = &mut entry.npc;
            let arrival = i32::from(npc.movement.last_movement == clock.wrapping_sub(1));
            entry.death_state = Some(State::Arrival {
                at: clock.wrapping_add(arrival),
                drop_coords: npc.movement.coords,
            });
            npc.movement.abort();
            npc.movement.interacting = false;
            npc.attacking_player = None;
            npc.escape_player = None;
            npc.face_player(None);
            npc.delayed = true;
        }
        match self
            .npcs
            .get(id.slot as usize)
            .unwrap()
            .death_state
            .unwrap()
        {
            State::Arrival { at, drop_coords } if clock >= at => {
                let entry = self.npcs.get_mut(id.slot as usize).unwrap();
                let source = entry.npc.aggressive_player;
                let uid = crate::combat_state::npc_uid(id.slot, entry.npc.type_id);
                entry.death_state = Some(State::Animation {
                    until: clock.wrapping_add(definition.animation_ticks),
                    drop_coords,
                });
                if let Some(actor) = source {
                    if let Some(p) = self
                        .players
                        .get_mut(actor.slot as usize)
                        .filter(|p| p.id == actor)
                    {
                        if p.aggressive_npc == Some(id) {
                            p.last_npc_combat = 0;
                        }
                        if let Some(config) = self.combat_state {
                            if let Some(update) = config.clear_on_death(
                                &mut p.data.variables,
                                uid,
                                p.protection.processed_clock > 0,
                            ) {
                                self.events.push(WorldEvent::CombatVariables {
                                    actor,
                                    updates: vec![update],
                                });
                            }
                        }
                        if let Some(sound) = definition.sound {
                            self.events.push(WorldEvent::CombatSound { actor, sound });
                        }
                    }
                }
                self.events.push(WorldEvent::NpcCombatAnimation {
                    actor: id,
                    sequence: definition.animation,
                });
                if definition.animation_ticks <= 0 {
                    self.despawn_npc(id);
                    self.events.push(WorldEvent::NpcCombatError {
                        actor: id,
                        error: ActionError::InvalidItem,
                    });
                }
            }
            State::Animation { until, drop_coords } if clock >= until => {
                let entry = self.npcs.get_mut(id.slot as usize).unwrap();
                let npc = &mut entry.npc;
                let npc_type = npc.type_id;
                let hero = npc.hero_points.winner();
                let respawns = npc.respawns && npc.respawn_rate >= 0;
                npc.movement.occupancy(&mut self.collision, false);
                npc.visible = false;
                entry.death_state = Some(State::Respawn {
                    at: clock.wrapping_add(npc.respawn_rate),
                });
                let mut trace = chicken.map(|_| crate::npc_death::DropPolicy::Chicken).or_else(||extras.as_ref().map(|e|e.policy)).map(|policy| crate::npc_death::DropTrace {
                    policy,
                    tick: clock,
                    npc: id,
                    npc_type,
                    coords: drop_coords,
                    owner: None,
                    draw: None,
                    products: Vec::new(),
                    handles: Vec::new(),
                    result: crate::npc_death::DropResult::NoEligibleOwner,
                });
                if let Some((owner, observer, duration)) = hero.filter(|_|!suppress_drops).and_then(|uuid| {
                    self.players
                        .iter()
                        .find(|(_, p)| p.data.character_id == uuid)
                        .and_then(|(_, p)| {
                            p.observer_id.map(|observer| {
                                (p.id, observer, p.loot_drop_duration.unwrap_or(200))
                            })
                        })
                }) {
                    let mut products = vec![(bones, 1)];
                    if let Some(extras) = &extras { products.extend_from_slice(&extras.products); }
                    if let Some(trace) = &mut trace { trace.owner=Some(owner); }
                    if let Some(profile) = chicken {
                        products.push((profile.raw, 1));
                        let draw = random.below(128);
                        if let Some(trace) = &mut trace {
                            trace.owner = Some(owner);
                            trace.draw = Some(draw);
                        }
                        let feathers = crate::npc_death::Chicken::feather_count(draw)
                            .expect("RandomSource must return a value below its exclusive bound");
                        if feathers > 0 {
                            products.push((profile.feather, feathers));
                        }
                    }
                    if let Some(trace) = &mut trace {
                        trace.products = products.clone();
                    }
                    let drops: Vec<_> = products
                        .into_iter()
                        .map(|(item, count)| crate::ground::Object {
                            coords: drop_coords,
                            item,
                            count,
                            scope: crate::ground::Scope::Private,
                            created: clock,
                            receiver: Some(observer),
                            owner: Some(observer),
                        })
                        .collect();
                    // Respawn state is already committed: invalid batches are
                    // reported once and never retried as another death reward.
                    match self.objects.add_batch(
                        &drops,
                        &self.object_types,
                        self.clock,
                        duration,
                        100,
                    ) {
                        Ok(handles) => {
                            if let Some(trace) = &mut trace {
                                trace.result = crate::npc_death::DropResult::Committed;
                                trace.handles = handles
                                    .iter()
                                    .map(|group| group.iter().map(|h| h.id()).collect())
                                    .collect();
                            }
                        }
                        Err(error) => {
                            if let Some(trace) = &mut trace {
                                trace.result = crate::npc_death::DropResult::Rejected;
                            }
                            self.events.push(WorldEvent::GroundError(error));
                        }
                    }
                }
                if let Some(trace) = trace {
                    self.events.push(WorldEvent::NpcDrop(trace));
                }
                if !respawns {
                    self.npcs.remove(id.slot as usize);
                }
            }
            State::Respawn { at } if clock >= at => {
                self.generation = self
                    .generation
                    .checked_add(1)
                    .expect("actor generation exhausted");
                let entry = self.npcs.get_mut(id.slot as usize).unwrap();
                entry.id.generation = self.generation;
                entry.death_queued = false;
                entry.death_state = None;
                entry.duck_jobs.clear();
                entry.hits = Default::default();
                entry.allocated_cycle = clock;
                entry.allocated_pending = true;
                let npc = &mut entry.npc;
                npc.movement.coords = npc.spawn;
                npc.movement.last_processed = CoordGrid::NULL;
                npc.movement.pending_telejump = true;
                npc.movement.abort();
                npc.stats = npc.base_stats;
                npc.hero_points.clear();
                npc.retaliate_due.clear();
                npc.aggressive_player = None;
                npc.face_player = None;
                npc.dialogue_face = None;
                npc.face_changed = false;
                npc.last_combat = -1000;
                npc.last_attack = -1000;
                npc.combat_action_delay = 0;
                npc.flat_armour = 0;
                npc.escape_idle = 0;
                npc.visible = true;
                npc.delayed = false;
                npc.movement.occupancy(&mut self.collision, true);
                self.events.push(WorldEvent::NpcCombatAnimation {
                    actor: entry.id,
                    sequence: u16::MAX,
                });
                if let Some(binding) = self.npc_bindings.get(&npc.type_id) {
                    binding.apply(npc);
                    if let Some(sequence) = binding.spawn_animation {
                        self.events.push(WorldEvent::NpcCombatAnimation {
                            actor: entry.id,
                            sequence,
                        });
                    }
                }
                let respawned = entry.id;
                self.update_npc_hunt_zone(respawned, true);
            }
            _ => (),
        }
        true
    }
}
