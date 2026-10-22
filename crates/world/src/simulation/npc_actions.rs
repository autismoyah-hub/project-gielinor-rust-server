use super::*;
use crate::{
    npc_interaction::{Event, Handler, Pending},
    raycast::{self, Bounds},
    reach::Rectangle,
};
impl World {
    pub fn npc_at_index(&self, index: u16) -> Option<NpcId> {
        self.npcs.get(index as usize).map(|e| e.id)
    }
    /// Checks performed before ifClose by OpNpcHandler; preserve existing work on invalid ops.
    pub fn can_operate_npc(&self, actor: ActorId, target: NpcId, op: u8) -> bool {
        let Ok(p) = self.player(actor) else {
            return false;
        };
        let Some(npc) = self.npc(target) else {
            return false;
        };
        !(op==3 && self.pickpocket.as_ref().is_some_and(|c|c.targets.contains_key(&npc.type_id)) && (p.movement.rooted||p.pickpocket_stunned()))
            && !p.protection.delayed()
            && !npc.delayed
            && self
                .npc_options
                .has_operation(npc.visible_type(), &p.data.variables, op)
    }
    pub(super) fn clear_npc_route(&mut self, slot: usize) {
        if let Some(p) = self.players.get_mut(slot) {
            p.npc_interaction = None;
            p.movement.interacting = false;
            p.movement.abort();
            self.events.push(WorldEvent::ClearMapFlag(p.id));
        }
    }
    pub(super) fn request_npc(
        &mut self,
        actor: ActorId,
        target: NpcId,
        operation: u8,
        control: bool,
    ) -> Result<(), ActionError> {
        self.request_npc_inner(actor, target, operation, control, None)
    }
    pub(super) fn request_npc_inner(
        &mut self,
        actor: ActorId,
        target: NpcId,
        operation: u8,
        control: bool,
        spell: Option<i32>,
    ) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.protection.delayed() {
            self.events.push(WorldEvent::ClearMapFlag(actor));
            return Ok(());
        }
        let Some(npc) = self.npc(target).filter(|n| spell.is_none() || !n.delayed) else {
            let p = self.players.get_mut(actor.slot as usize).unwrap();
            p.begin_held_action();
            p.location_interaction = None;
            p.object_interaction = None;
            p.npc_interaction = None;
            p.bank_count = None;
            p.movement.interacting = false;
            p.queues.clear_weak();
            // clearPendingAction cancels the interaction, but does not abort a walking route.
            self.events.push(WorldEvent::ClearMapFlag(actor));
            return Ok(());
        };
        if spell.is_none() && !self.can_operate_npc(actor, target, operation) {
            return Ok(());
        }
        let visible = self
            .npc_options
            .morph(npc.visible_type(), &p.data.variables)
            .unwrap_or(npc.visible_type());
        let handler = self
            .npc_handlers
            .get(&(visible, operation))
            .or_else(|| self.npc_handlers.get(&(npc.type_id, operation)))
            .copied();
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.begin_held_action();
        p.location_interaction = None;
        p.object_interaction = None;
        p.bank_count = None;
        p.npc_interaction = Some(Pending {
            target,
            operation,
            handler: if spell.is_some() {
                Some(Handler::Combat)
            } else {
                handler
            },
            spell,
        });
        p.movement.interacting = true;
        p.movement.abort();
        // abort clears temporary speed, so apply the request after clearing the old route.
        p.movement.temporary_speed = if control {
            Some(if p.active_speed == Speed::Run || p.data.energy < 100 {
                Speed::Walk
            } else {
                Speed::Run
            })
        } else {
            None
        };
        self.events.push(WorldEvent::NpcInteraction {
            actor,
            event: Event::Face(target),
        });
        Ok(())
    }
    /// Returns true when an early interaction consumed this cycle's movement.
    pub(super) fn process_npc_interaction(
        &mut self,
        slot: usize,
        early: bool,
        random: &mut impl RandomSource,
    ) -> bool {
        let Some(p) = self.players.get(slot) else {
            return false;
        };
        let Some(pending) = p.npc_interaction else {
            return false;
        };
        if p.saving || p.protection.access_protected(p.has_modal) {
            return false;
        }
        let actor = p.id;
        if let Some(Handler::HeldItemReply{raw,slot:selected,item})=pending.handler {
            if !self.npc_item_reply_valid(actor,pending.target,raw,selected,item){self.clear_npc_route(slot);return true;}
        }
        if let Some(Handler::Shear{raw,selected})=pending.handler {
            if !self.sheep_admission(actor,pending.target,raw,selected){self.clear_npc_route(slot);return true;}
        }
        if let Some(Handler::Banknote{raw,visible,..})=pending.handler {
            if p.pending_logout || p.death_queued || p.data.stats.current(3) as u8==0
                || !self.banker_note_identity(actor,pending.target,raw,visible) {
                self.clear_npc_route(slot);return true;
            }
        }
        if pending.handler == Some(Handler::Combat) && p.data.stats.current(3) as u8 == 0 {
            self.clear_npc_route(slot);
            return true;
        }
        if pending.handler == Some(Handler::Combat) && pending.spell.is_none() {
            if let Some((message, cancel)) = self.npc_combat_denial(actor, pending.target) {
                if let Some(message) = message {
                    self.events.push(WorldEvent::NpcInteraction {
                        actor,
                        event: Event::Message(message),
                    });
                }
                if cancel {
                    self.clear_npc_route(slot);
                }
                return true;
            }
        }
        let p = self.players.get(slot).unwrap();
        let Some(npc) = self
            .npc(pending.target)
            .filter(|n| n.visible && !n.delayed && n.stats[3] > 0)
        else {
            self.clear_npc_route(slot);
            return true;
        };
        if npc.movement.coords.level() != p.movement.coords.level() {
            self.clear_npc_route(slot);
            return true;
        }
        let rect = Rectangle {
            coords: npc.movement.coords,
            width: npc.movement.size,
            length: npc.movement.size,
            angle: 0,
            blocked: 0,
            exclusive: true,
        };
        let touches = rect.reached(&self.collision, p.movement.coords, p.movement.size);
        let source = p.movement.coords;
        let target = npc.movement.coords;
        let size = npc.movement.size;
        let overlap = source.x() < target.x() + size
            && source.x() + p.movement.size > target.x()
            && source.z() < target.z() + size
            && source.z() + p.movement.size > target.z();
        let distance=if matches!(pending.handler,Some(Handler::Banknote{..})){3}else{2};
        let within = source.x() >= target.x() - distance
            && source.x() <= target.x() + size - 1 + distance
            && source.z() >= target.z() - distance
            && source.z() <= target.z() + size - 1 + distance;
        let bank_approach = matches!(pending.handler,Some(Handler::Bank|Handler::Banknote{..}))
            && !overlap
            && within
            && raycast::clear(
                &self.collision,
                Bounds {
                    coords: source,
                    width: 1,
                    length: 1,
                },
                Bounds {
                    coords: target,
                    width: size,
                    length: size,
                },
                true,
                0x100000,
            );
        let actor = p.id;
        let combat_range = if pending.spell.is_some() {
            10
        } else if pending.handler == Some(Handler::Combat) {
            self.attack_profile(actor).map(|p| p.range).unwrap_or(1)
        } else {
            0
        };
        let combat_approach = combat_range > 1
            && !overlap
            && source.x() >= target.x() - combat_range
            && source.x() <= target.x() + size - 1 + combat_range
            && source.z() >= target.z() - combat_range
            && source.z() <= target.z() + size - 1 + combat_range
            && raycast::clear(
                &self.collision,
                Bounds {
                    coords: source,
                    width: 1,
                    length: 1,
                },
                Bounds {
                    coords: target,
                    width: size,
                    length: size,
                },
                true,
                0,
            );
        if pending.handler == Some(Handler::Combat) && (touches || combat_approach) {
            self.players.get_mut(slot).unwrap().movement.abort();
            let result = if let Some(spell) = pending.spell {
                self.cast_manual_npc(actor, pending.target, spell, random)
            } else {
                self.resolve_combat_attack(actor)
                    .and_then(|(attack, selected_spell)| {
                        if matches!(attack, crate::attack_profile::Attack::Melee { .. }) {
                            self.attack_npc_melee(actor, pending.target, attack, random)
                        } else if matches!(attack, crate::attack_profile::Attack::Ranged { .. }) {
                            self.attack_npc_ranged(actor, pending.target, attack, random)
                        } else if let crate::attack_profile::Attack::Spell { defensive } = attack {
                            let key = selected_spell.ok_or(ActionError::InvalidItem)?;
                            let spell = self
                                .combat_wear
                                .as_ref()
                                .and_then(|wear| wear.autocast.spells.get(&key))
                                .ok_or(ActionError::InvalidItem)?
                                .item;
                            self.attack_npc_elemental(
                                actor,
                                pending.target,
                                spell,
                                defensive,
                                random,
                            )
                        } else {
                            Err(ActionError::UnsupportedCombat)
                        }
                    })
            };
            if let Err(error) = result {
                self.clear_npc_route(slot);
                self.events
                    .push(WorldEvent::ActionRejected { actor, error });
            }
            return true;
        }
        // Authored fish occupy blocked water. A qualified net action may finish
        // from the existing three-tile shore range; collision stays unchanged.
        let cinderworks_fishing = pending.handler == Some(Handler::CinderworksFishing)
            && self.cinderworks.as_ref().is_some_and(|c|crate::tower_layout::contains(c.origin,source)
                && c.work.is_some_and(|w|npc.type_id==w.fishing_spot&&crate::tower_layout::contains(c.origin,npc.spawn)));
        let fishing_approach=cinderworks_fishing&&source.chebyshev_distance(target)<=3;
        if touches || bank_approach || fishing_approach {
            if let Some(Handler::HeldItemReply{raw,slot:selected,item})=pending.handler {
                self.clear_npc_route(slot);self.finish_npc_item_reply(actor,pending.target,raw,selected,item);return true;
            }
            if let Some(Handler::Shear{raw,selected})=pending.handler {
                self.clear_npc_route(slot);self.start_shearing(slot,pending.target,raw,selected);return true;
            }
            if let Some(Handler::Banknote{slot:selected,note,raw,visible})=pending.handler {
                self.clear_npc_route(slot);
                if let Err(error)=self.start_banker_note(actor,pending.target,selected,note,raw,visible) {
                    self.events.push(WorldEvent::ActionRejected{actor,error});
                }
                return true;
            }
            if pending.handler==Some(Handler::Pickpocket) {self.clear_npc_route(slot);self.start_pickpocket(slot,pending.target);return true;}
            if pending.handler == Some(Handler::TowerTanner) {
                self.clear_npc_route(slot);
                if let Err(error) =
                    self.tan_tower_hides(actor, pending.target, pending.operation == 1)
                {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::LectureTutor(config)) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_lecture_tutor(actor, pending.target, config) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::MagicTutor(config)) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_magic_tutor(actor, pending.target, config) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if pending.handler==Some(Handler::CinderworksFishing) {
                self.clear_npc_route(slot);
                if cinderworks_fishing {self.start_cinderworks_fishing(slot,pending.target);}
                return true;
            }
            if pending.handler == Some(Handler::TutorialFishing) {
                self.clear_npc_route(slot);
                self.process_tutorial_fishing(slot);
                return true;
            }
            if let Some(Handler::CombatTutor(config)) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_combat_tutor(actor, pending.target, config) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::MiningTutor(config)) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_mining_tutor(actor, pending.target, config) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::QuestTutor { step_varp }) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_quest_tutor(actor, pending.target, step_varp) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::MasterChef(config)) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_master_chef(actor, pending.target, config) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::SurvivalTutor(config)) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_survival_tutor(actor, pending.target, config) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::TutorialGuide { step_varp }) = pending.handler {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_tutorial_guide(actor, pending.target, step_varp) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            if let Some(Handler::SkillcapeVendor(config))=pending.handler {
                self.clear_npc_route(slot);
                if let Err(error)=self.start_skillcape_vendor(actor,pending.target,config){self.events.push(WorldEvent::ActionRejected{actor,error});}
                return true;
            }
            if pending.handler == Some(Handler::Person) {
                self.clear_npc_route(slot);
                if let Err(error) = self.start_person(actor, pending.target) {
                    self.events
                        .push(WorldEvent::ActionRejected { actor, error });
                }
                return true;
            }
            let p = self.players.get_mut(slot).unwrap();
            p.npc_interaction = None;
            p.movement.interacting = false;
            p.movement.abort();
            let event = match pending.handler {
                Some(Handler::Bank) => Event::Bank,
                Some(Handler::Banknote{..}) => unreachable!("Banknote dialogue handled above"),
                Some(Handler::Shear{..}) => unreachable!("Shearing handled above"),
                Some(Handler::HeldItemReply{..}) => unreachable!("Held refusal handled above"),
                Some(Handler::Tanning) => Event::Tanning(pending.target),
                Some(Handler::Pickpocket) => unreachable!("Pickpocket handled above"),
                Some(Handler::TowerTanner) => unreachable!("Tanning handled above"),
                Some(Handler::Combat) => unreachable!("Combat handled above"),
                Some(Handler::SkillcapeVendor(_)) => unreachable!("Vendor dialogue handled above"),
                Some(Handler::Person) => unreachable!("Dialogue handled above"),
                Some(Handler::TutorialFishing | Handler::CinderworksFishing) => unreachable!("Fishing handled above"),
                Some(
                    Handler::TutorialGuide { .. }
                    | Handler::SurvivalTutor(_)
                    | Handler::MasterChef(_)
                    | Handler::MiningTutor(_)
                    | Handler::CombatTutor(_)
                    | Handler::MagicTutor(_)
                    | Handler::LectureTutor(_)
                    | Handler::QuestTutor { .. },
                ) => {
                    unreachable!("Tutorial dialogue handled above")
                }
                None => Event::Message("Nothing interesting happens."),
            };
            self.events
                .push(WorldEvent::NpcInteraction { actor: p.id, event });
            self.events.push(WorldEvent::ClearMapFlag(p.id));
            return true;
        }
        if early {
            if p.movement.waypoint_count() <= 1 {
                let route = crate::routing::rectangle_route(
                    &self.collision,
                    source,
                    rect,
                    p.movement.size,
                    true,
                    25,
                    Strategy::Normal,
                );
                let p = self.players.get_mut(slot).unwrap();
                p.movement.route(route.waypoints, p.active_speed);
            }
        } else if p.movement.destination().is_none() && p.movement.last_movement < self.clock.0 {
            let actor = p.id;
            self.clear_npc_route(slot);
            self.events.push(WorldEvent::NpcInteraction {
                actor,
                event: Event::Message("I can't reach that!"),
            });
        }
        false
    }
}
