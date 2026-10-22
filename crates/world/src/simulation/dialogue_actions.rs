use super::*;
use crate::dialogue::{Conversation, Effect, Event, Page, Pending, Speaker};
impl World {
    pub(super) fn start_lecture_tutor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_lecture_tutor::Config,
    ) -> Result<(), ActionError> {
        let step = crate::tutorial_steps::Step::from_var(
            self.player(actor)?.data.variables.get(config.step_varp),
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: crate::tutorial_lecture_tutor::begin(config, step),
            },
        )
    }
    pub(super) fn start_magic_tutor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_magic_tutor::Config,
    ) -> Result<(), ActionError> {
        use crate::tutorial_magic_tutor::{self as magic, Stage};
        let data = &self.player(actor)?.data;
        let stage = magic::begin(crate::tutorial_steps::Step::from_var(
            data.variables.get(config.step_varp),
        ));
        if stage == Stage::Waiting && !magic::plenty(config, &data.inventory, &self.object_types) {
            let inv = &mut self
                .players
                .get_mut(actor.slot as usize)
                .unwrap()
                .data
                .inventory;
            magic::replenish(config, inv, &self.object_types);
            return Ok(());
        }
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: magic::conversation(config, stage),
            },
        )
    }
    fn magic_tutor_action(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_magic_tutor::Config,
        action: crate::tutorial_magic_tutor::Action,
    ) -> Result<(), ActionError> {
        use crate::tutorial_magic_tutor::{self as magic, Action};
        if matches!(action, Action::Runes | Action::RunesAndStep) {
            let inv = &mut self
                .players
                .get_mut(actor.slot as usize)
                .unwrap()
                .data
                .inventory;
            magic::replenish(config, inv, &self.object_types);
        }
        let step = match action {
            Action::Step(step) => Some(step),
            Action::RunesAndStep => Some(crate::tutorial_steps::Step::MagicRunes),
            Action::Leave => {
                if let Some(leave) = self.tutorial_leave {
                    self.give_tutorial_leave_kit(actor, leave)?;
                }
                None
            }
            _ => None,
        };
        if let Some(step) = step {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::TutorialStep {
                    npc,
                    step,
                    open_tab: matches!(
                        step,
                        crate::tutorial_steps::Step::MagicTab
                            | crate::tutorial_steps::Step::HomeTele
                    )
                    .then_some(6),
                },
            });
        }
        Ok(())
    }
    pub fn dialogue_token(&self, actor: ActorId) -> Result<Option<u64>, ActionError> {
        Ok(self
            .player(actor)?
            .dialogue
            .as_ref()
            .map(|dialogue| dialogue.token).or_else(||self.location_choice_token(actor)))
    }
    pub fn cancel_dialogue(&mut self, actor: ActorId) -> Result<bool, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let choice=p.pending_location_action.is_some_and(|pending|matches!(pending.action,location_swap_actions::Continuation::Choice{..}|location_swap_actions::Continuation::Message{..}));
        if choice {p.pending_location_action=None;}
        let cancelled = p.dialogue.take().is_some() || choice;
        if cancelled {
            p.protection.suspended = false;
        }
        Ok(cancelled)
    }
    pub(super) fn start_person(&mut self, actor: ActorId, npc: NpcId) -> Result<(), ActionError> {
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: Conversation::person(),
            },
        )
    }
    pub(super) fn start_tutorial_guide(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        step_varp: u16,
    ) -> Result<(), ActionError> {
        let step = crate::tutorial_steps::Step::from_var(
            self.player(actor)?.data.variables.get(step_varp),
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: Conversation::tutorial_guide(step),
            },
        )
    }
    pub(super) fn start_survival_tutor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_survival::Config,
    ) -> Result<(), ActionError> {
        let step = crate::tutorial_steps::Step::from_var(
            self.player(actor)?.data.variables.get(config.step_varp),
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: crate::tutorial_survival::begin(config, step),
            },
        )
    }
    pub(super) fn start_master_chef(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_chef::Config,
    ) -> Result<(), ActionError> {
        let step = crate::tutorial_steps::Step::from_var(
            self.player(actor)?.data.variables.get(config.step_varp),
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: crate::tutorial_chef::begin(config, step),
            },
        )
    }
    pub(super) fn start_quest_tutor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        step_varp: u16,
    ) -> Result<(), ActionError> {
        let step = crate::tutorial_steps::Step::from_var(
            self.player(actor)?.data.variables.get(step_varp),
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: crate::tutorial_quest::begin(step),
            },
        )
    }
    pub(super) fn start_mining_tutor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_mining_tutor::Config,
    ) -> Result<(), ActionError> {
        let step = crate::tutorial_steps::Step::from_var(
            self.player(actor)?.data.variables.get(config.step_varp),
        );
        if step >= crate::tutorial_steps::Step::MineGate {
            let inv = &mut self
                .players
                .get_mut(actor.slot as usize)
                .unwrap()
                .data
                .inventory;
            crate::tutorial_mining_tutor::prepare_recap(config, inv, &self.object_types);
        }
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation: crate::tutorial_mining_tutor::begin(config, step),
            },
        )
    }
    pub(super) fn start_combat_tutor(
        &mut self,
        actor: ActorId,
        npc: NpcId,
        config: crate::tutorial_combat_tutor::Config,
    ) -> Result<(), ActionError> {
        let data = &self.player(actor)?.data;
        let step = crate::tutorial_steps::Step::from_var(data.variables.get(config.step_varp));
        let conversation = crate::tutorial_combat_tutor::begin(
            config,
            step,
            &data.inventory,
            &data.equipment,
            &self.object_types,
        );
        self.show_dialogue(
            actor,
            Pending {
                token: 0,
                npc,
                conversation,
            },
        )
    }
    pub(super) fn show_dialogue(&mut self, actor: ActorId, mut pending: Pending) -> Result<(), ActionError> {
        let Some(page) = pending.conversation.pages.front().cloned() else {
            return Ok(());
        };
        if matches!(
            page,
            Page::Chat {
                speaker: Speaker::Npc,
                ..
            } | Page::NamedChat {
                speaker: Speaker::Npc,
                ..
            }
        ) {
            if let Some(npc) = self.npc_mut(pending.npc) {
                if npc.attacking_player.is_none() && npc.escape_player.is_none() {
                    npc.movement.abort();
                    npc.movement.interacting = false;
                    npc.face_player(Some(actor));
                    npc.dialogue_face = Some(actor);
                }
            }
        }
        let p = self
            .players
            .get_mut(actor.slot as usize)
            .ok_or(ActionError::StaleActor)?;
        p.dialog_generation = p
            .dialog_generation
            .checked_add(1)
            .ok_or(ActionError::QueueFull)?;
        pending.token = p.dialog_generation;
        self.events.push(WorldEvent::Dialogue {
            actor,
            event: Event::Page {
                token: pending.token,
                npc: pending.npc,
                page,
            },
        });
        p.dialogue = Some(pending);
        p.protection.suspended = true;
        Ok(())
    }
    pub(super) fn process_dialogue_face(&mut self, id: NpcId) -> bool {
        let Some(npc) = self.npc(id) else {
            return false;
        };
        let Some(actor) = npc.dialogue_face else {
            return false;
        };
        let a = npc.movement.coords;
        let size = npc.movement.size;
        let close = self.player(actor).is_ok_and(|p| {
            let b = p.movement.coords;
            a.level() == b.level()
                && a.x() <= b.x() + p.movement.size
                && b.x() <= a.x() + size
                && a.z() <= b.z() + p.movement.size
                && b.z() <= a.z() + size
        });
        let npc = self.npc_mut(id).unwrap();
        npc.movement.abort();
        if !close {
            npc.face_player(None);
            npc.movement.interacting = false;
        }
        true
    }
    pub(super) fn resume_dialogue(
        &mut self,
        actor: ActorId,
        token: u64,
        choice: i32,
        random: &mut impl RandomSource,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        self.validate_banker_note_dialogue(actor.slot as usize);
        let p = self.player(actor)?;
        if self.location_choice_token(actor).is_some(){return self.resume_location_choice(actor,token,choice);}
        if p.dialogue.as_ref().is_none_or(|d| d.token != token) {
            return Ok(());
        }
        if p.protection.delayed() {
            return Err(ActionError::Delayed);
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let mut pending = p.dialogue.take().unwrap();
        p.protection.suspended = false;
        let Some(npc) = self.npc(pending.npc) else {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
            return Ok(());
        };
        if !npc.visible || npc.stats[3] <= 0 {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
            return Ok(());
        }
        match pending.conversation.pages.front() {
            Some(Page::Choice(_) | Page::TitledChoice { .. }) => {
                let conversation = match pending.conversation.after {
                    Effect::Banknote{ctx,confirm:true}=>self.banker_note_choice(actor,pending.npc,ctx,choice)?,
                    Effect::SkillcapeVendor {ctx,stage} => self.skillcape_vendor_choice(actor,pending.npc,ctx,stage,choice)?,
                    Effect::LectureTutor {
                        config,
                        stage: crate::tutorial_lecture_tutor::Stage::RecapChoice,
                    } => crate::tutorial_lecture_tutor::choice(config, choice),
                    Effect::MagicTutor { config, stage } => {
                        if let Some((next, action)) =
                            crate::tutorial_magic_tutor::choice(stage, choice)
                        {
                            self.magic_tutor_action(actor, pending.npc, config, action)?;
                            Some(crate::tutorial_magic_tutor::conversation(config, next))
                        } else {
                            None
                        }
                    }
                    Effect::CombatTutor {
                        config,
                        stage: crate::tutorial_combat_tutor::Stage::RecapChoice,
                    } => crate::tutorial_combat_tutor::choice(config, choice),
                    Effect::MiningTutor {
                        config,
                        stage: crate::tutorial_mining_tutor::Stage::RecapChoice,
                    } => crate::tutorial_mining_tutor::choice(config, choice),
                    Effect::Quest(crate::tutorial_quest::Stage::RecapChoice) => {
                        crate::tutorial_quest::choice(choice)
                    }
                    Effect::Chef {
                        config,
                        stage: crate::tutorial_chef::Stage::RecapChoice,
                    } => crate::tutorial_chef::choice(config, choice),
                    Effect::Survival {
                        config,
                        stage: crate::tutorial_survival::Stage::RecapChoice,
                    } => crate::tutorial_survival::choice(config, choice),
                    _ => Conversation::choice(choice),
                };
                let Some(conversation) = conversation else {
                    self.players.get_mut(actor.slot as usize).unwrap().dialogue = Some(pending);
                    self.players
                        .get_mut(actor.slot as usize)
                        .unwrap()
                        .protection
                        .suspended = true;
                    return Ok(());
                };
                pending.conversation = conversation;
            }
            Some(Page::Object{..}) if (0..=1).contains(&choice) => {
                pending.conversation.pages.pop_front();
            }
            Some(Page::Chat { .. } | Page::NamedChat { .. }) if choice == -1 => {
                pending.conversation.pages.pop_front();
            }
            _ => {
                self.players.get_mut(actor.slot as usize).unwrap().dialogue = Some(pending);
                self.players
                    .get_mut(actor.slot as usize)
                    .unwrap()
                    .protection
                    .suspended = true;
                return Ok(());
            }
        }
        if pending.conversation.pages.is_empty() {
            match pending.conversation.after {
                Effect::Banknote{..}=>{},
                Effect::SkillcapeVendor {ctx,stage} => {pending.conversation=self.finish_skillcape_vendor(actor,pending.npc,ctx,stage)?;},
                Effect::LectureTutor { config, stage } => {
                    let (step, open_tab) = crate::tutorial_lecture_tutor::finish(config, stage);
                    self.events.push(WorldEvent::Dialogue {
                        actor,
                        event: Event::TutorialStep {
                            npc: pending.npc,
                            step,
                            open_tab,
                        },
                    });
                }
                Effect::MagicTutor { config, stage } => {
                    let (next, action) = crate::tutorial_magic_tutor::continued(stage);
                    self.magic_tutor_action(actor, pending.npc, config, action)?;
                    pending.conversation = crate::tutorial_magic_tutor::conversation(config, next);
                }
                Effect::CombatTutor { config, stage } => {
                    let p = self.players.get_mut(actor.slot as usize).unwrap();
                    let current = crate::tutorial_steps::Step::from_var(
                        p.data.variables.get(config.step_varp),
                    );
                    let (next, messages) = crate::tutorial_combat_tutor::finish(
                        config,
                        stage,
                        current,
                        &mut p.data.inventory,
                        &p.data.equipment,
                        &self.object_types,
                    );
                    for message in messages {
                        self.events.push(WorldEvent::NpcInteraction {
                            actor,
                            event: crate::npc_interaction::Event::Message(message),
                        });
                    }
                    if let Some((step, open_tab)) = next {
                        self.events.push(WorldEvent::Dialogue {
                            actor,
                            event: Event::TutorialStep {
                                npc: pending.npc,
                                step,
                                open_tab,
                            },
                        });
                    }
                }
                Effect::MiningTutor { config, stage } => {
                    let p = self.players.get_mut(actor.slot as usize).unwrap();
                    let current = crate::tutorial_steps::Step::from_var(
                        p.data.variables.get(config.step_varp),
                    );
                    let (step, message) = crate::tutorial_mining_tutor::finish(
                        config,
                        stage,
                        current,
                        &mut p.data.inventory,
                        &self.object_types,
                    );
                    if let Some(message) = message {
                        self.events.push(WorldEvent::NpcInteraction {
                            actor,
                            event: crate::npc_interaction::Event::Message(message),
                        });
                    }
                    if let Some(step) = step {
                        self.events.push(WorldEvent::Dialogue {
                            actor,
                            event: Event::TutorialStep {
                                npc: pending.npc,
                                step,
                                open_tab: None,
                            },
                        });
                    }
                }
                Effect::Quest(stage) => {
                    use crate::{tutorial_quest::Stage, tutorial_steps::Step};
                    let (step, open_tab) = match stage {
                        Stage::First => (Step::QuestList, Some(2)),
                        Stage::Finish => (Step::QuestLadder, None),
                        Stage::RecapChoice => unreachable!("Choice page is still pending"),
                    };
                    self.events.push(WorldEvent::Dialogue {
                        actor,
                        event: Event::TutorialStep {
                            npc: pending.npc,
                            step,
                            open_tab,
                        },
                    });
                }
                Effect::Chef { config, stage } => {
                    use crate::{tutorial_chef::Stage, tutorial_steps::Step};
                    let p = self.players.get_mut(actor.slot as usize).unwrap();
                    let next = match stage {
                        Stage::Ingredients => {
                            for id in [config.flour, config.water] {
                                // Original checks each ingredient independently at final Continue.
                                // A failed insertion does not cancel the other ingredient or progress.
                                if !p
                                    .data
                                    .inventory
                                    .slots()
                                    .iter()
                                    .flatten()
                                    .any(|item| item.id == id && item.count > 0)
                                {
                                    let _ = p.data.inventory.insert(
                                        &self.object_types,
                                        crate::inventory::Insert::new(id, 1),
                                    );
                                }
                            }
                            Some(Step::ChefDough)
                        }
                        Stage::RecapFinish => {
                            (Step::from_var(p.data.variables.get(config.step_varp))
                                >= Step::ChefExit)
                                .then_some(Step::ChefExit)
                        }
                        Stage::RecapChoice => unreachable!("Choice page is still pending"),
                    };
                    if let Some(step) = next {
                        self.events.push(WorldEvent::Dialogue {
                            actor,
                            event: Event::TutorialStep {
                                npc: pending.npc,
                                step,
                                open_tab: None,
                            },
                        });
                    }
                }
                Effect::Survival { config, stage } => {
                    use crate::{tutorial_steps::Step, tutorial_survival::Stage};
                    let p = self.players.get_mut(actor.slot as usize).unwrap();
                    let (items, next, inventory_tab): (&[i32], _, _) = match stage {
                        Stage::First => (&[config.net], Some(Step::SurvivalInv), true),
                        Stage::Tools => (
                            &[config.axe, config.tinderbox],
                            Some(Step::SurvivalWc),
                            false,
                        ),
                        Stage::RecapFinish => (
                            &[],
                            (Step::from_var(p.data.variables.get(config.step_varp))
                                >= Step::SurvivalGate)
                                .then_some(Step::SurvivalGate),
                            false,
                        ),
                        Stage::RecapChoice => unreachable!("Choice page is still pending"),
                    };
                    for &id in items {
                        // invAdd results are ignored in the original. Each tool
                        // commits separately; one free slot may grant only axe.
                        let _ = p
                            .data
                            .inventory
                            .insert(&self.object_types, crate::inventory::Insert::new(id, 1));
                    }
                    if let Some(step) = next {
                        self.events.push(WorldEvent::Dialogue {
                            actor,
                            event: Event::Survival {
                                npc: pending.npc,
                                step,
                                inventory_tab,
                            },
                        });
                    }
                }
                Effect::RollPersonResponse => {
                    let rolled = random.below(128) == 0;
                    let p = self.player(actor)?;
                    let flyer = rolled
                        && p.data.inventory.free_space() > 0
                        && self.person_flyer.is_some_and(|id| {
                            !p.data
                                .inventory
                                .slots()
                                .iter()
                                .flatten()
                                .any(|item| item.id == id)
                        });
                    let roll = if flyer { 0 } else { random.below(22) };
                    pending.conversation = Conversation::response(flyer, roll).unwrap();
                }
                Effect::GiveFlyer => {
                    if let Some(id) = self.person_flyer {
                        let p = self.players.get_mut(actor.slot as usize).unwrap();
                        // Kotlin invAdd is non-strict: a now-full inventory cannot duplicate an item.
                        let mut insert = crate::inventory::Insert::new(id, 1);
                        insert.strict = false;
                        let _ = p.data.inventory.insert(&self.object_types, insert);
                    }
                }
                Effect::Retaliate => {
                    let clock = self.clock.0;
                    if let Some(npc) = self.npc_mut(pending.npc) {
                        npc.aggressive_player = Some(actor);
                        npc.last_combat = npc.last_combat.max(clock);
                        npc.retaliate_due.push_back(clock.wrapping_add(1));
                    }
                }
                Effect::None => {}
                Effect::TutorialGuide {
                    step,
                    open_settings,
                } => {
                    self.events.push(WorldEvent::Dialogue {
                        actor,
                        event: Event::TutorialGuide {
                            npc: pending.npc,
                            step,
                            open_settings,
                        },
                    });
                }
            }
        }
        if pending.conversation.pages.is_empty() {
            self.events.push(WorldEvent::Dialogue {
                actor,
                event: Event::Closed,
            });
            Ok(())
        } else {
            self.show_dialogue(actor, pending)
        }
    }
}
