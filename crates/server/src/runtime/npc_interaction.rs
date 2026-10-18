use super::*;
use gielinor_world::{
    npc_interaction::{Event as NpcEvent, Handler},
    simulation::{Action, NpcId},
    ui::Component,
};
impl<B: AccountBackend> Runtime<B> {
    /// Trusted cache component-to-spell registration, before admitting sessions.
    pub fn configure_elemental_magic(
        &mut self,
        combat: gielinor_world::spell_combat::Config,
        spells: gielinor_world::elemental_spell::Catalog,
        buttons: BTreeMap<u32, i32>,
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure NPC spell buttons before admission",
            ));
        }
        for (&component, &spell) in &buttons {
            if !self.components.contains_key(&component) || !spells.spells.contains_key(&spell) {
                return Err(io::Error::other(format!(
                    "Unresolved NPC spell button {component}/{spell}"
                )));
            }
        }
        self.world.spell_combat = Some(combat);
        self.world.elemental_spells = Some(spells);
        self.npc_spell_buttons = buttons;
        Ok(())
    }
    pub(super) fn npc_spell_target(
        &mut self,
        actor: ActorId,
        request: gielinor_protocol::npc_input::Target,
    ) -> io::Result<()> {
        let err = |e| io::Error::other(format!("{e:?}"));
        let delayed = self.world.protection(actor).map_err(err)?.delayed();
        let target = self
            .world
            .npc_at_index(request.index)
            .filter(|&id| self.world.npc(id).is_some_and(|n| !n.delayed));
        let enabled = self.components.get(&request.source).is_some_and(|c| {
            self.active[&actor].ui.npc_target(
                Component(request.source),
                request.source_slot,
                c.events,
            )
        });
        if delayed || target.is_none() || !enabled {
            if let Some(connection) = self.connection(actor) {
                let packet = gielinor_protocol::game::clear_map_flag();
                self.transport
                    .send(connection, packet.opcode, &packet.payload)?;
            }
            self.events
                .push(Event::World(WorldEvent::ClearMapFlag(actor)));
        }
        if delayed {
            return Ok(());
        }
        // Permissions must be captured before ifClose removes the modal.
        self.close_dialogs_and_modals(actor)?;
        self.world.cancel_dialogue(actor).map_err(err)?;
        self.world
            .cancel_location_target_action(actor)
            .map_err(err)?;
        if target.is_none() || !enabled {
            return Ok(());
        }
        let Some(&spell) = self.npc_spell_buttons.get(&request.source) else {
            // Non-spell target handlers (including item-on-NPC) are not mapped here.
            if let Some(connection) = self.connection(actor) {
                self.events.push(Event::PacketRejected {
                    connection,
                    actor,
                    opcode: 86,
                    reason: "Unimplemented NPC target component".into(),
                });
            }
            return Ok(());
        };
        self.world
            .submit(
                actor,
                Action::CastNpc {
                    target: target.unwrap(),
                    spell,
                    control: request.control,
                },
            )
            .map_err(err)
    }
    /// Uses the same floor/session coordinates as the configured tanner.
    pub fn configure_tower_chest(
        &mut self,
        location: u32,
        objects: &gielinor_cache::symbols::Symbols,
    ) -> io::Result<()> {
        use gielinor_world::tower_chest::{name_matches, Config, HandlerConfig};
        if !self.connections.is_empty() || !self.definitions.locations.contains_key(&location) {
            return Err(io::Error::other(
                "Configure a known tower chest before admission",
            ));
        }
        let common = self
            .world
            .tower_tanning
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure Tower session data first"))?;
        let mut ids = [0; 12];
        for (slot, name) in [
            "bronze_axe",
            "needle",
            "thread",
            "bronze_pickaxe",
            "tinderbox",
            "net",
            "knife",
            "hammer",
            "3dose1attack",
            "bronze_arrow",
            "coins",
            "beer",
        ]
        .into_iter()
        .enumerate()
        {
            let id = *objects
                .get(name)
                .ok_or_else(|| io::Error::other(format!("Missing chest object {name}")))?;
            if !self.definitions.items.contains_key(&(id as u32)) {
                return Err(io::Error::other(format!("Missing chest definition {name}")));
            }
            ids[slot] = id;
        }
        let merge_aliases = std::array::from_fn(|i| {
            let name = ["Thread", "Bronze arrow", "Coins"][i];
            self.definitions
                .items
                .iter()
                .filter_map(|(&id, ty)| {
                    (ty.cert.template == 0 && name_matches(&ty.name, name)).then_some(id as i32)
                })
                .collect()
        });
        self.world.tower_chest = Some(HandlerConfig {
            location,
            origin: common.origin,
            skills: common.skills,
            bones: common.bones,
            inventory: Config { ids, merge_aliases },
        });
        Ok(())
    }
    pub fn configure_tower_offering(
        &mut self,
        location: u32,
        animation: u16,
        accepted: [i32; 2],
        xp_parameter: u32,
        default_xp: Option<i32>,
    ) -> io::Result<()> {
        use gielinor_world::tower_offering::{self, Bones, HandlerConfig};
        if !self.connections.is_empty()
            || !self.definitions.locations.contains_key(&location)
            || self.world.prayer.is_none()
            || self.world.tower_staff.is_none()
            || accepted
                .iter()
                .any(|id| !self.definitions.items.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other(
                "Configure resolved Tower offering dependencies before admission",
            ));
        }
        let common = self
            .world
            .tower_tanning
            .as_ref()
            .ok_or_else(|| io::Error::other("Tower diary configuration missing"))?;
        let xp: BTreeMap<_, _> = self
            .definitions
            .items
            .iter()
            .filter(|(_, ty)| tower_offering::is_bones(&ty.name))
            .map(|(&id, ty)| {
                let fine = match ty.params.get(&xp_parameter) {
                    Some(gielinor_cache::definitions::Param::Integer(value)) => Some(*value),
                    _ => default_xp,
                };
                (id as i32, fine.map(|xp| xp as f64 / 10.0))
            })
            .collect();
        self.world.tower_offering = Some(HandlerConfig {
            location,
            animation,
            accepted,
            origin: common.origin,
            bones: Bones {
                named: xp.keys().copied().collect(),
            },
            xp,
            skills: common.skills,
            drop_bones: common.bones,
        });
        Ok(())
    }
    pub fn configure_tower_staff(
        &mut self,
        location: u32,
        staff: i32,
        talisman: i32,
    ) -> io::Result<()> {
        use gielinor_world::{
            tower_chest::name_matches,
            tower_staff::{staff_name, Config, HandlerConfig},
        };
        if !self.connections.is_empty() || !self.definitions.locations.contains_key(&location) {
            return Err(io::Error::other(
                "Configure resolved staff crate before admission",
            ));
        }
        let origin = self
            .world
            .tower_tanning
            .as_ref()
            .ok_or_else(|| io::Error::other("Tower origin missing"))?
            .origin;
        let staff_type = self
            .definitions
            .items
            .get(&(staff as u32))
            .ok_or_else(|| io::Error::other("Staff definition missing"))?;
        let talisman_type = self
            .definitions
            .items
            .get(&(talisman as u32))
            .ok_or_else(|| io::Error::other("Talisman definition missing"))?;
        if !staff_type.name.is_ascii() || !talisman_type.name.is_ascii() {
            return Err(io::Error::other(
                "Non-ASCII canonical staff kit names require additional case mapping",
            ));
        }
        let staff_names = self
            .definitions
            .items
            .iter()
            .filter_map(|(&id, ty)| staff_name(&ty.name).then_some(id as i32))
            .collect();
        let owned = |canonical: i32, packed: &str, alias: &str| {
            self.definitions
                .items
                .iter()
                .filter_map(|(&id, ty)| {
                    (id as i32 == canonical
                        || name_matches(&ty.name, packed)
                        || ty.cert.template == 0 && name_matches(&ty.name, alias))
                    .then_some(id as i32)
                })
                .collect()
        };
        self.world.tower_staff = Some(HandlerConfig {
            location,
            origin,
            inventory: Config {
                staff,
                talisman,
                staff_names,
                staff_owned: owned(staff, &staff_type.name, "Staff of air"),
                talisman_owned: owned(talisman, &talisman_type.name, "Mind talisman"),
            },
        });
        Ok(())
    }
    pub fn configure_tower_tanner(
        &mut self,
        origin: gielinor_world::CoordGrid,
        npc: u16,
        leather: i32,
        bones: u16,
        skills: [u8; 3],
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure tower tanner before admission"));
        }
        if !self.definitions.npcs.contains_key(&(npc as u32))
            || !self.definitions.items.contains_key(&(leather as u32))
            || !self.definitions.items.contains_key(&(bones as u32))
            || skills
                .iter()
                .any(|id| !self.definitions.stats.contains_key(&(*id as u32)))
        {
            return Err(io::Error::other("Unresolved tower tanner definitions"));
        }
        let hides = self
            .definitions
            .items
            .iter()
            .filter_map(|(&id, item)| {
                gielinor_world::tower_tanning::is_cowhide(
                    id as i32,
                    &item.name,
                    item.cert.template != 0,
                )
                .then_some(id as i32)
            })
            .collect();
        self.world.tower_tanning = Some(gielinor_world::tower_tanning::HandlerConfig {
            origin,
            inventory: gielinor_world::tower_tanning::Config { hides, leather },
            skills,
            bones,
        });
        for op in [1, 3] {
            self.world
                .npc_handlers
                .insert((npc, op), Handler::TowerTanner);
        }
        Ok(())
    }
    pub fn configure_bankers(
        &mut self,
        groups: &[i32],
        parents: (Component, Component),
    ) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure bankers before admission"));
        }
        for (&id, ty) in &self.definitions.npcs {
            if groups.contains(&ty.content_group) {
                if let Ok(id) = u16::try_from(id) {
                    self.world.npc_handlers.insert((id, 3), Handler::Bank);
                }
            }
        }
        self.npc_bank_parents = Some(parents);
        Ok(())
    }
    pub(super) fn npc_button(
        &mut self,
        actor: ActorId,
        op: gielinor_protocol::npc_input::Operation,
    ) -> io::Result<()> {
        if op.suboperation != 0 {
            return Ok(());
        }
        let delayed = self
            .world
            .protection(actor)
            .map_err(|e| io::Error::other(format!("{e:?}")))?
            .delayed();
        let target = self.world.npc_at_index(op.index);
        if !delayed
            && (target.is_none()
                || target.is_some_and(|t| self.world.can_operate_npc(actor, t, op.operation)))
        {
            self.close_dialogs_and_modals(actor)?;
        }
        self.world
            .submit(
                actor,
                Action::OperateNpc {
                    target: target.unwrap_or(NpcId {
                        slot: op.index,
                        generation: 0,
                    }),
                    operation: op.operation,
                    control: op.control,
                },
            )
            .map_err(|e| io::Error::other(format!("{e:?}")))
    }
    pub(super) fn npc_event(&mut self, actor: ActorId, event: NpcEvent) -> io::Result<()> {
        match event {
            NpcEvent::Tanning(target) => self.open_tanning(actor,target)?,
            NpcEvent::Face(target) => {
                if let Some(active) = self.active.get_mut(&actor) {
                    active.face_npc = Some(target);
                    active.pending_face = None;
                    active.face_changed = true;
                }
            }
            NpcEvent::Bank => {
                if let Some((main, side)) = self.npc_bank_parents {
                    self.open_bank(actor, main, side)?;
                }
            }
            NpcEvent::Message(text) => {
                if let Some(connection) = self.connection(actor) {
                    let packet = gielinor_protocol::chat::game(0, None, text)
                        .map_err(|e| io::Error::other(format!("{e:?}")))?;
                    self.transport
                        .send(connection, packet.opcode, &packet.payload)?;
                }
            }
        }
        Ok(())
    }
}
