use super::*;
use crate::tutorial_ui::Effect;
use gielinor_protocol::client_script::Argument;
use gielinor_world::{
    simulation::NpcId,
    tutorial_steps::Step,
    ui::{operation_mask, Component},
};

impl<B: AccountBackend> Runtime<B> {
    pub fn configure_tutorial_bank(&mut self, bank: u32, poll: u32, coins: i32) -> io::Result<()> {
        if !self.active.is_empty()
            || !self.connections.is_empty()
            || self.npc_bank_parents.is_none()
            || self.bank_presentation.is_none()
            || bank == poll
            || [bank, poll]
                .iter()
                .any(|id| !self.world.location_types.contains_key(id))
            || coins < 0
            || !self.definitions.items.contains_key(&(coins as u32))
        {
            return Err(io::Error::other(
                "Configure valid bank presentation and tutorial booth objects before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before booths"))?;
        self.world.tutorial_bank = Some(gielinor_world::tutorial_bank::Config {
            progression: tutorial.progression,
            bank,
            poll,
            coins,
        });
        Ok(())
    }
    pub(super) fn tutorial_bank_event(
        &mut self,
        actor: ActorId,
        booth: gielinor_world::tutorial_bank::Booth,
        next: Option<Step>,
    ) -> io::Result<()> {
        use gielinor_world::tutorial_bank::{self, Booth};
        let messages: &[&str] = match booth {
            Booth::Bank => {
                let (main, side) = self
                    .npc_bank_parents
                    .ok_or_else(|| io::Error::other("Missing tutorial bank presentation"))?;
                self.open_bank(actor, main, side)?;
                &[tutorial_bank::BANK_MESSAGE]
            }
            Booth::Poll => &tutorial_bank::POLL_MESSAGES,
        };
        for message in messages {
            if let Some(connection) = self.connection(actor) {
                let packet = gielinor_protocol::chat::game(0, None, message)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?;
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
        }
        if let Some(step) = next {
            self.set_tutorial_step(actor, step, None)?;
        }
        Ok(())
    }
    pub fn configure_lecture_tutors(&mut self) -> io::Result<()> {
        use gielinor_world::tutorial_lecture_tutor::{Config, Kind};
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial instructors before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before instructors"))?;
        for (name, kind) in [
            ("tut2_prayer_tutor", Kind::Prayer),
            ("tut2_bank_tutor", Kind::Account),
        ] {
            self.world.npc_handlers.insert(
                (tutorial.focus.npcs[name], 1),
                gielinor_world::npc_interaction::Handler::LectureTutor(Config {
                    kind,
                    step_varp: tutorial.progression.step.id,
                }),
            );
        }
        Ok(())
    }
    pub fn configure_tutorial_leave(&mut self, items: [i32; 18], coins: i32) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial leave before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before leave"))?;
        if items
            .into_iter()
            .chain([coins])
            .any(|id| id < 0 || !self.definitions.items.contains_key(&(id as u32)))
        {
            return Err(io::Error::other("Missing tutorial leave object"));
        }
        self.tutorial_leave = Some(gielinor_world::tutorial_leave::Config {
            progression: tutorial.progression,
            items,
            coins,
        });
        self.world.tutorial_leave = self.tutorial_leave;
        Ok(())
    }
    pub fn configure_magic_tutor(&mut self, air: i32, mind: i32) -> io::Result<()> {
        if !self.active.is_empty()
            || !self.connections.is_empty()
            || self.tutorial_leave.is_none()
            || [air, mind]
                .iter()
                .any(|id| *id < 0 || !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure valid Magic Instructor items and leave kit before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before Magic Instructor"))?;
        self.world.npc_handlers.insert(
            (tutorial.focus.npcs["tut2_magic_tutor"], 1),
            gielinor_world::npc_interaction::Handler::MagicTutor(
                gielinor_world::tutorial_magic_tutor::Config {
                    step_varp: tutorial.progression.step.id,
                    air,
                    mind,
                },
            ),
        );
        Ok(())
    }
    pub fn configure_tutorial_fire(
        &mut self,
        tinderbox: i32,
        logs: i32,
        location: u32,
        animation: u16,
        skill: u8,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other("Configure tutorial fire before admission"));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before fire"))?;
        if [tinderbox, logs]
            .into_iter()
            .any(|id| !self.definitions.items.contains_key(&(id as u32)))
            || !self.definitions.locations.contains_key(&location)
        {
            return Err(io::Error::other("Missing tutorial fire definitions"));
        }
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing Firemaking stat"))?
            .max_level;
        let config = gielinor_world::tutorial_fire::Config {
            progression: tutorial.progression,
            logs,
            location,
            animation,
            skill,
            max_level,
            global_xp_rate: 1.0,
        };
        self.world
            .held_use_handlers
            .register(
                gielinor_world::held_use::Scope::Types,
                tinderbox,
                logs,
                gielinor_world::held_use::Handler::TutorialFire(config),
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        for id in [tinderbox, logs] {
            let ty = &self.definitions.items[&(id as u32)];
            self.world.held_use_types.insert(
                id,
                gielinor_world::held_use::Type {
                    id,
                    content: ty.content_group,
                },
            );
        }
        Ok(())
    }
    pub fn configure_tutorial_mining(
        &mut self,
        rocks: [u32; 2],
        ores: [i32; 2],
        pickaxe: i32,
        animation: u16,
        skill: u8,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial mining before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before mining"))?;
        if rocks
            .into_iter()
            .any(|id| !self.definitions.locations.contains_key(&id))
            || ores
                .into_iter()
                .chain([pickaxe])
                .any(|id| !self.definitions.items.contains_key(&(id as u32)))
        {
            return Err(io::Error::other("Missing tutorial mining definitions"));
        }
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing Mining stat"))?
            .max_level;
        self.world.tutorial_mining = Some(gielinor_world::tutorial_mining::Config {
            progression: tutorial.progression,
            tin_rock: rocks[0],
            copper_rock: rocks[1],
            tin: ores[0],
            copper: ores[1],
            pickaxe,
            animation,
            skill,
            max_level,
            global_xp_rate: 1.0,
        });
        Ok(())
    }
    pub fn configure_tutorial_shrimp(
        &mut self,
        location: u32,
        raw: i32,
        cooked: i32,
        animation: u16,
        skill: u8,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial shrimp before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before shrimp"))?;
        if !self.definitions.items.contains_key(&(raw as u32))
            || !self.definitions.items.contains_key(&(cooked as u32))
            || !self.definitions.locations.contains_key(&location)
        {
            return Err(io::Error::other("Missing tutorial shrimp definitions"));
        }
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing Cooking stat"))?
            .max_level;
        self.world.tutorial_shrimp = Some(gielinor_world::tutorial_shrimp::Config {
            progression: tutorial.progression,
            location,
            raw,
            cooked,
            animation,
            skill,
            max_level,
            global_xp_rate: 1.0,
        });
        Ok(())
    }
    pub fn configure_tutorial_woodcutting(
        &mut self,
        trees: [u32; 2],
        oak: u32,
        logs: i32,
        animation: u16,
        skill: u8,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial woodcutting before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before woodcutting"))?;
        if !self.definitions.items.contains_key(&(logs as u32))
            || trees
                .into_iter()
                .chain([oak])
                .any(|id| !self.definitions.locations.contains_key(&id))
        {
            return Err(io::Error::other("Missing tutorial woodcutting definitions"));
        }
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing Woodcutting stat"))?
            .max_level;
        self.world.tutorial_woodcutting = Some(gielinor_world::tutorial_woodcutting::Config {
            progression: tutorial.progression,
            trees,
            oak,
            logs,
            animation,
            skill,
            max_level,
            global_xp_rate: 1.0,
        });
        Ok(())
    }
    pub fn configure_tutorial_fishing(
        &mut self,
        npc: u16,
        shrimp: i32,
        animation: u16,
        skill: u8,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial fishing before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before fishing"))?;
        if !self.definitions.npcs.contains_key(&(npc as u32))
            || !self.definitions.items.contains_key(&(shrimp as u32))
        {
            return Err(io::Error::other("Missing tutorial fishing definitions"));
        }
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing Fishing stat"))?
            .max_level;
        self.world.tutorial_fishing = Some(gielinor_world::tutorial_fishing::Config {
            progression: tutorial.progression,
            shrimp,
            animation,
            skill,
            max_level,
            global_xp_rate: 1.0,
        });
        self.world.npc_handlers.insert(
            (npc, 1),
            gielinor_world::npc_interaction::Handler::TutorialFishing,
        );
        Ok(())
    }
    pub fn configure_tutorial_bread(
        &mut self,
        location: u32,
        dough: i32,
        bread: i32,
        animation: u16,
        skill: u8,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial bread before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before bread"))?;
        if !self.definitions.items.contains_key(&(dough as u32))
            || !self.definitions.items.contains_key(&(bread as u32))
            || !self.definitions.locations.contains_key(&location)
        {
            return Err(io::Error::other("Missing tutorial bread definitions"));
        }
        let max_level = self
            .definitions
            .stats
            .get(&(skill as u32))
            .ok_or_else(|| io::Error::other("Missing tutorial Cooking stat"))?
            .max_level;
        self.world.tutorial_bread = Some(gielinor_world::tutorial_bread::Config {
            progression: tutorial.progression,
            location,
            dough,
            bread,
            animation,
            skill,
            max_level,
            global_xp_rate: 1.0,
        });
        Ok(())
    }
    pub fn configure_tutorial_dough(
        &mut self,
        flour: i32,
        water: i32,
        pot: i32,
        bucket: i32,
        dough: i32,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial dough before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before dough"))?;
        for id in [flour, water, pot, bucket, dough] {
            if !self.definitions.items.contains_key(&(id as u32)) {
                return Err(io::Error::other(format!(
                    "Missing tutorial dough item {id}"
                )));
            }
        }
        let config = gielinor_world::tutorial_dough::Config {
            progression: tutorial.progression,
            flour,
            water,
            pot,
            bucket,
            dough,
        };
        self.world
            .held_use_handlers
            .register(
                gielinor_world::held_use::Scope::Types,
                flour,
                water,
                gielinor_world::held_use::Handler::TutorialDough(config),
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        self.world.held_use_types = self
            .definitions
            .items
            .iter()
            .map(|(&id, ty)| {
                (
                    id as i32,
                    gielinor_world::held_use::Type {
                        id: id as i32,
                        content: ty.content_group,
                    },
                )
            })
            .collect();
        Ok(())
    }
    pub fn configure_mining_tutor(&mut self, pickaxe: i32, hammer: i32) -> io::Result<()> {
        if !self.active.is_empty()
            || !self.connections.is_empty()
            || [pickaxe, hammer]
                .iter()
                .any(|id| !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure valid Mining Instructor items before admission",
            ));
        }
        let config = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before Mining Instructor"))?;
        self.world.npc_handlers.insert(
            (config.focus.npcs["tut2_mining_tutor"], 1),
            gielinor_world::npc_interaction::Handler::MiningTutor(
                gielinor_world::tutorial_mining_tutor::Config {
                    step_varp: config.progression.step.id,
                    pickaxe,
                    hammer,
                },
            ),
        );
        Ok(())
    }
    pub fn configure_combat_tutor(
        &mut self,
        dagger: i32,
        sword: i32,
        shield: i32,
        bow: i32,
        arrows: i32,
    ) -> io::Result<()> {
        if !self.active.is_empty()
            || !self.connections.is_empty()
            || [dagger, sword, shield, bow, arrows]
                .iter()
                .any(|id| !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure valid Combat Instructor items before admission",
            ));
        }
        let config = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before Combat Instructor"))?;
        self.world.npc_handlers.insert(
            (config.focus.npcs["tut2_combat_tutor"], 1),
            gielinor_world::npc_interaction::Handler::CombatTutor(
                gielinor_world::tutorial_combat_tutor::Config {
                    step_varp: config.progression.step.id,
                    dagger,
                    sword,
                    shield,
                    bow,
                    arrows,
                },
            ),
        );
        Ok(())
    }
    pub fn configure_tutorial_combat(&mut self, rat: u16, chicken: u16) -> io::Result<()> {
        if !self.active.is_empty()
            || !self.connections.is_empty()
            || [rat, chicken]
                .iter()
                .any(|id| !self.definitions.npcs.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure valid tutorial combat NPCs before admission",
            ));
        }
        let tutorial = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before combat progression"))?;
        self.world.tutorial_combat = Some(gielinor_world::tutorial_combat::Config {
            progression: tutorial.progression,
            rat,
            chicken,
        });
        Ok(())
    }
    pub fn configure_quest_tutor(&mut self) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other("Configure Quest Tutor before admission"));
        }
        let config = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before Quest Tutor"))?;
        self.world.npc_handlers.insert(
            (config.focus.npcs["tut2_quest_tutor"], 1),
            gielinor_world::npc_interaction::Handler::QuestTutor {
                step_varp: config.progression.step.id,
            },
        );
        Ok(())
    }
    pub fn configure_master_chef(&mut self, flour: i32, water: i32) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other("Configure Master Chef before admission"));
        }
        let config = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before Master Chef"))?;
        self.world.npc_handlers.insert(
            (config.focus.npcs["tut2_nav_tutor"], 1),
            gielinor_world::npc_interaction::Handler::MasterChef(
                gielinor_world::tutorial_chef::Config {
                    step_varp: config.progression.step.id,
                    flour,
                    water,
                },
            ),
        );
        Ok(())
    }
    pub fn configure_survival_tutor(
        &mut self,
        net: i32,
        axe: i32,
        tinderbox: i32,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure Survival Tutor before admission",
            ));
        }
        let config = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before Survival Tutor"))?;
        self.world.npc_handlers.insert(
            (config.focus.npcs["tut2_survival_tutor"], 1),
            gielinor_world::npc_interaction::Handler::SurvivalTutor(
                gielinor_world::tutorial_survival::Config {
                    step_varp: config.progression.step.id,
                    net,
                    axe,
                    tinderbox,
                },
            ),
        );
        Ok(())
    }
    pub fn configure_tutorial_gates(
        &mut self,
        gates: std::collections::BTreeMap<u32, gielinor_world::tutorial_gates::Gate>,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure tutorial gates before admission",
            ));
        }
        let config = self
            .tutorial
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure tutorial before gates"))?;
        let hint_npcs = Step::ALL
            .into_iter()
            .filter_map(|step| {
                gielinor_world::tutorial_focus::hint_npc(step)
                    .map(|name| (step, config.focus.npcs[name]))
            })
            .collect();
        self.world.tutorial_gates = Some(gielinor_world::tutorial_gates::Config {
            progression: config.progression,
            gates,
            hint_npcs,
        });
        Ok(())
    }
    pub(super) fn tutorial_gate_focus(
        &mut self,
        actor: ActorId,
        step: Step,
        hint: Option<NpcId>,
    ) -> io::Result<()> {
        let config = self
            .tutorial
            .clone()
            .ok_or_else(|| io::Error::other("Tutorial is not configured"))?;
        // Even None is a resolved departure-position result. Do not search at
        // the arrival position after a ladder moves the actor between maps.
        self.apply_tutorial_effects(actor, config.focus.apply(step, hint.map(|npc| npc.slot))?)
    }
    pub(super) fn tutorial_overlay_button(
        &mut self,
        actor: ActorId,
        component: u32,
    ) -> io::Result<()> {
        let Some(config) = self.tutorial.clone() else {
            return Ok(());
        };
        if component == config.focus.home_teleport {
            if let Some(leave) = self.tutorial_leave {
                match self.world.tutorial_home(actor, leave) {
                    Ok(())
                    | Err(
                        gielinor_world::simulation::ActionError::Protected
                        | gielinor_world::simulation::ActionError::LoggingOut,
                    ) => (),
                    Err(error) => return Err(io::Error::other(format!("{error:?}"))),
                }
            }
            return Ok(());
        }
        if component == config.focus.wind_strike {
            let vars = &self
                .world
                .data(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .variables;
            if gielinor_world::tutorial_magic_ui::wind(
                config.progression.current(vars),
                vars.get(config.progression.completed.id),
            ) {
                // Refresh focus even when already at MagicCast; the reference does not
                // require protected access and does not cast or invent a target here.
                self.set_tutorial_step(actor, Step::MagicCast, None)?;
                let packet = gielinor_protocol::chat::game(
                    0,
                    None,
                    gielinor_world::tutorial_magic_ui::WIND_MESSAGE,
                )
                .map_err(|e| io::Error::other(format!("{e:?}")))?;
                if let Some(connection) = self.connection(actor) {
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
            return Ok(());
        }
        let Some(&panel) = config.panels.get(&component) else {
            return Ok(());
        };
        let vars = &self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables;
        // The reference handlers test current progress, without an isActive gate.
        match gielinor_world::tutorial_panels::opened(panel, config.progression.current(vars)) {
            gielinor_world::tutorial_panels::Action::None => Ok(()),
            gielinor_world::tutorial_panels::Action::Set(step) => {
                self.set_tutorial_step(actor, step, None)
            }
            gielinor_world::tutorial_panels::Action::PointBackToGuide => {
                let coords = self
                    .world
                    .observe(actor)
                    .map_err(|e| io::Error::other(format!("{e:?}")))?
                    .coords;
                let npc = self
                    .world
                    .find_npc(coords, config.focus.npcs["tut2_basics_tutor"], 64)
                    .map_err(io::Error::other)?;
                if let Some(npc) = npc {
                    self.apply_tutorial_effects(
                        actor,
                        [Effect::Packet(
                            gielinor_protocol::hint_arrow::Arrow::Npc(npc.slot).encode(),
                        )],
                    )?;
                }
                self.run_client_script(actor, 1974, &[Argument::String("<col=0000ff>Settings menu</col><br>Good. Now speak to the Gielinor Guide again to continue.".into())])
            }
        }
    }
    fn tutorial_variables_transmit(&self, actor: ActorId) -> io::Result<bool> {
        // World events are consumed after PlayerMain. Until the first view is
        // committed this hook still represents Kotlin's EngineLoginReady phase.
        let viewed = self
            .active
            .get(&actor)
            .is_some_and(|a| a.observer.is_some());
        Ok(viewed
            && self
                .world
                .protection(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .processed_clock
                > 0)
    }
    pub fn configure_tutorial(
        &mut self,
        config: crate::tutorial_character::Config,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other("Configure tutorial before admission"));
        }
        self.world.npc_handlers.insert(
            (config.focus.npcs["tut2_basics_tutor"], 1),
            gielinor_world::npc_interaction::Handler::TutorialGuide {
                step_varp: config.progression.step.id,
            },
        );
        self.tutorial = Some(std::sync::Arc::new(config));
        Ok(())
    }
    fn tutorial_focus(
        &mut self,
        actor: ActorId,
        step: Step,
        preferred: Option<NpcId>,
    ) -> io::Result<()> {
        let config = self
            .tutorial
            .clone()
            .ok_or_else(|| io::Error::other("Tutorial is not configured"))?;
        let hint = if let Some(name) = gielinor_world::tutorial_focus::hint_npc(step) {
            let type_id = config.focus.npcs[name];
            let matching = preferred.filter(|id| {
                self.world
                    .npc(*id)
                    .is_some_and(|npc| npc.type_id == type_id)
            });
            let found = match matching {
                Some(id) => Some(id),
                None => self
                    .world
                    .find_npc(
                        self.world
                            .observe(actor)
                            .map_err(|e| io::Error::other(format!("{e:?}")))?
                            .coords,
                        type_id,
                        64,
                    )
                    .map_err(io::Error::other)?,
            };
            found.map(|id| id.slot)
        } else {
            None
        };
        self.apply_tutorial_effects(actor, config.focus.apply(step, hint)?)
    }
    pub fn set_tutorial_step(
        &mut self,
        actor: ActorId,
        step: Step,
        preferred: Option<NpcId>,
    ) -> io::Result<()> {
        let config = self
            .tutorial
            .clone()
            .ok_or_else(|| io::Error::other("Tutorial is not configured"))?;
        let logged = self.tutorial_variables_transmit(actor)?;
        let effects = config.progression.set(
            &mut self
                .world
                .data_mut(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .variables,
            step,
            preferred,
            logged,
        );
        self.tutorial_progress_effects(actor, &effects)
    }
    pub(super) fn tutorial_progress_effects(
        &mut self,
        actor: ActorId,
        effects: &[gielinor_world::tutorial_progression::Effect],
    ) -> io::Result<()> {
        let config = self
            .tutorial
            .clone()
            .ok_or_else(|| io::Error::other("Tutorial is not configured"))?;
        for effect in effects {
            use gielinor_world::tutorial_progression::Effect as Progress;
            match effect {
                Progress::Variable(update) => self.apply_tutorial_effects(
                    actor,
                    [Effect::Packet(gielinor_protocol::varp::update(
                        update.id,
                        update.value,
                    ))],
                )?,
                Progress::Focus { step, npc } => self.tutorial_focus(actor, *step, *npc)?,
                Progress::End => self.apply_tutorial_effects(actor, config.focus.end()?)?,
            }
        }
        Ok(())
    }
    fn tutorial_main_modal(
        &mut self,
        actor: ActorId,
        parent: Component,
        interface: u16,
    ) -> io::Result<()> {
        self.run_client_script(actor, 2524, &[Argument::Int(-1), Argument::Int(-1)])?;
        self.open_interface(actor, Some(parent), interface, false)
    }
    fn tutorial_experience(&mut self, actor: ActorId) -> io::Result<()> {
        let config = self
            .tutorial
            .clone()
            .ok_or_else(|| io::Error::other("Tutorial is not configured"))?;
        self.tutorial_main_modal(actor, config.mainmodal, config.experience)?;
        for component in config.experience_buttons {
            self.set_interface_events(actor, component, -1, -1, operation_mask(1).unwrap())?;
        }
        Ok(())
    }
    /// EngineLoginReady hook. Call only after opening the login gameframe and
    /// before constructing its first player-info appearance snapshot.
    pub fn begin_tutorial(&mut self, actor: ActorId) -> io::Result<()> {
        let Some(config) = self.tutorial.clone() else {
            return Ok(());
        };
        let vars = &self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables;
        if !config.progression.active(vars) {
            return Ok(());
        }
        let step = config.progression.current(vars);
        self.run_client_script(actor, 2644, &[])?;
        if step == Step::CharCreate {
            self.set_tutorial_step(actor, Step::CharCreate, None)?;
            let active = self
                .active
                .get_mut(&actor)
                .ok_or_else(|| io::Error::other("Invalid tutorial actor"))?;
            active.design.hidden = true;
            active.appearance_rebuild = true;
            let (body, pronoun) = (active.design.body_type, active.design.pronoun);
            self.apply_tutorial_effects(
                actor,
                [
                    Effect::VariableBit {
                        bit: config.body_bit,
                        value: body as i32,
                    },
                    Effect::VariableBit {
                        bit: config.pronoun_bit,
                        value: pronoun as i32,
                    },
                    Effect::Packet(gielinor_protocol::interfaces::hide(
                        config.focus.blocker,
                        true,
                    )),
                    Effect::Packet(gielinor_protocol::music::song(62, 0, 0, 0, 0)),
                    Effect::Packet(gielinor_protocol::interfaces::hide(
                        config.focus.blocker,
                        false,
                    )),
                ],
            )?;
            self.tutorial_main_modal(actor, config.mainmodal, config.creator)?;
            self.set_interface_events(actor, config.pronouns, 1, 3, operation_mask(1).unwrap())?;
            self.apply_tutorial_effects(actor, config.focus.guide(Step::CharCreate)?)?;
        } else {
            self.tutorial_focus(actor, step, None)?;
            if step == Step::ExpSelect {
                self.tutorial_experience(actor)?;
            }
        }
        Ok(())
    }
    fn tutorial_close(&mut self, actor: ActorId) -> io::Result<()> {
        let aborted = self
            .world
            .close_dialogs(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?;
        if aborted {
            self.active.get_mut(&actor).unwrap().ui.input_dialog_open = false;
            let connection = self.connection(actor).unwrap();
            self.transport.send(connection, 117, &[])?;
        }
        let parents: Vec<_> = self.active[&actor]
            .ui
            .modals
            .entries()
            .map(|(parent, _)| parent)
            .collect();
        for parent in parents {
            self.close_interface(actor, parent)?;
        }
        Ok(())
    }
    pub(super) fn tutorial_character_button(
        &mut self,
        actor: ActorId,
        component: u32,
    ) -> io::Result<()> {
        let Some(config) = self.tutorial.clone() else {
            return Ok(());
        };
        let vars = &self
            .world
            .data(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .variables;
        if !config.progression.active(vars) {
            return Ok(());
        }
        let step = config.progression.current(vars);
        if component == config.confirm && matches!(step, Step::CharCreate | Step::ExpSelect) {
            let active = self.active.get_mut(&actor).unwrap();
            active.design.hidden = false;
            active.appearance_rebuild = true;
            self.tutorial_close(actor)?;
            self.apply_tutorial_effects(
                actor,
                [Effect::Packet(gielinor_protocol::music::song(
                    62, 0, 0, 0, 0,
                ))],
            )?;
            self.set_tutorial_step(actor, Step::ExpSelect, None)?;
            self.tutorial_experience(actor)?;
        } else if step == Step::ExpSelect {
            if let Some(index) = config
                .experience_buttons
                .iter()
                .position(|c| c.0 == component)
            {
                self.apply_tutorial_effects(
                    actor,
                    [Effect::VariableBit {
                        bit: config.adventure_bit,
                        value: i32::from(index == 0),
                    }],
                )?;
                self.tutorial_close(actor)?;
                // Tower is an alternative onboarding route. After the normal
                // appearance and experience choices, its own guide takes over.
                let coords = self.world.data(actor).map_err(|e| io::Error::other(format!("{e:?}")))?.coords;
                let tower = self.world.tower_arrival.is_some_and(|config| gielinor_world::tower_layout::contains(config.origin, coords));
                self.set_tutorial_step(actor, if tower { Step::Completed } else { Step::GielinorTalk }, None)?;
                if tower {
                    self.close_interface(actor, config.focus.chatmodal)?;
                    self.close_interface(actor, config.focus.atmosphere)?;
                    if let Some(connection) = self.connection(actor) {
                        let packet = gielinor_protocol::chat::game(0, None, "Welcome to the Cinderworks. Read the sanctuary plaque to begin your run.")
                            .map_err(|e| io::Error::other(format!("{e:?}")))?;
                        self.transport.send(connection, packet.opcode, &packet.payload)?;
                    }
                }
            }
        }
        Ok(())
    }
    /// Consume the ordered TutorialFocusGuidance plan through the normal UI
    /// lifecycle. Reopening a panel is conditional on its current overlay state.
    pub fn apply_tutorial_effects(
        &mut self,
        actor: ActorId,
        effects: impl IntoIterator<Item = Effect>,
    ) -> io::Result<()> {
        let connection = self
            .connection(actor)
            .ok_or_else(|| io::Error::other("Disconnected tutorial actor"))?;
        for effect in effects {
            match effect {
                Effect::Packet(packet) => {
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?
                }
                Effect::Overlay { parent, interface } => {
                    self.open_interface(actor, Some(parent), interface, true)?
                }
                Effect::Panel {
                    parent,
                    interface,
                    open,
                } => {
                    let ui = &self
                        .active
                        .get(&actor)
                        .ok_or_else(|| io::Error::other("Invalid tutorial actor"))?
                        .ui;
                    let current = ui.overlays.parent(interface);
                    if open {
                        if current.is_none() {
                            self.open_interface(actor, Some(parent), interface, true)?;
                        }
                    } else if let Some(current) = current {
                        self.close_interface(actor, current)?;
                    }
                }
                Effect::VariableBit { bit, value } => {
                    let bit = *self
                        .definitions
                        .varbits
                        .get(&bit)
                        .ok_or_else(|| io::Error::other("Missing tutorial flash bit"))?;
                    let base = *self
                        .definitions
                        .varps
                        .get(&(bit.base as u32))
                        .ok_or_else(|| io::Error::other("Missing tutorial flash base"))?;
                    // EngineLoginReady precedes the player's first processed
                    // map clock. An admitted connection alone is not loggedIn.
                    let logged = self.tutorial_variables_transmit(actor)?;
                    let update = self
                        .world
                        .data_mut(actor)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?
                        .variables
                        .set_bit(base, bit, value, logged)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    if let Some(update) = update {
                        let packet = gielinor_protocol::varp::update(update.id, update.value);
                        self.transport
                            .send(connection, packet.opcode, &packet.payload)?;
                    }
                }
            }
        }
        Ok(())
    }
}
