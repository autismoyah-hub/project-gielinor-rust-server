//! NPC aggression: NpcPlayerHuntProcessor (NpcPreTick) finds a player target, and the
//! player part of NpcHuntProcessor (NpcMain) advances the hunt clock and consumes it.
//! Non-player hunt kinds (npc/obj/scenery) are not consumed here.
use super::*;
use crate::hunt::{Kind, Mode, Vis};
use std::cmp::Reverse;

/// Standard skill ids: attack, defence, strength, hitpoints, ranged, magic, prayer.
const COMBAT_SKILLS: [u8; 7] = [0, 1, 2, 3, 4, 6, 5];

impl World {
    /// Hunt modes by id, and hunt modes for NPC types whose cache type has none.
    pub fn configure_hunt(&mut self, modes: BTreeMap<u16, Mode>, defaults: BTreeMap<u16, u16>) {
        self.hunt_modes = modes;
        self.hunt_defaults = defaults;
        for (_, entry) in self.npcs.iter_mut() {
            apply_hunt_default(&self.hunt_defaults, &mut entry.npc);
        }
    }
    pub(super) fn process_npc_player_hunts(&mut self, random: &mut impl RandomSource) {
        if self.hunt_modes.is_empty() {
            return;
        }
        let ids: Vec<_> = self.npcs.iter().map(|(_, entry)| entry.id).collect();
        for id in ids {
            let Some(entry) = self.npcs.get(id.slot as usize).filter(|e| e.id == id) else {
                continue;
            };
            let npc = &entry.npc;
            if !npc.visible || npc.delayed || entry.death_queued || npc.stats[3] <= 0 {
                continue;
            }
            let Some(mode) = npc.hunt_mode.and_then(|m| self.hunt_modes.get(&m)) else {
                continue;
            };
            if npc.hunt_range == 0 || (!npc.viewed && !npc.headless) {
                continue;
            }
            if npc.hunt_clock < mode.rate - 1 {
                continue;
            }
            if !mode.player_hunt_supported() || npc_has_interaction(npc) {
                continue;
            }
            if let Some(target) = self.hunt_player_target(npc, mode, random) {
                self.npcs.get_mut(id.slot as usize).unwrap().npc.hunt_player = Some(target);
            }
        }
    }
    /// Hunt.findPlayers order: zones by descending x then z, players in zone-registry
    /// insertion order. Each passing player replaces the choice with chance 1/(count+1).
    fn hunt_player_target(
        &self,
        npc: &Npc,
        mode: &Mode,
        random: &mut impl RandomSource,
    ) -> Option<ActorId> {
        let center = npc.movement.coords;
        let range = npc.hunt_range;
        let origin = npc_search::zone(center);
        let zone_distance = (range + 7) / 8;
        let mut candidates: Vec<_> = self
            .players
            .iter()
            .filter(|(_, p)| {
                p.hunt_zone[0] == origin[0]
                    && (p.hunt_zone[1] - origin[1]).abs() <= zone_distance
                    && (p.hunt_zone[2] - origin[2]).abs() <= zone_distance
            })
            .map(|(_, p)| ((Reverse(p.hunt_zone[1]), Reverse(p.hunt_zone[2]), p.hunt_order), p))
            .collect();
        candidates.sort_by_key(|(key, _)| *key);
        let clock = self.clock.0;
        let mut target = None;
        let mut count = 0;
        for (_, p) in candidates {
            let coords = p.movement.coords;
            if p.saving || p.movement.hidden || coords.level() != center.level() {
                continue;
            }
            let (dx, dz) = (coords.x() - center.x(), coords.z() - center.z());
            if dx.abs().max(dz.abs()) > range {
                continue;
            }
            if mode.vis != Vis::Off {
                let bounds = |coords| crate::raycast::Bounds {
                    coords,
                    width: 1,
                    length: 1,
                };
                let sight = mode.vis == Vis::LineOfSight;
                if !crate::raycast::clear(&self.collision, bounds(center), bounds(coords), sight, 0)
                {
                    continue;
                }
            }
            if mode.not_busy && p.protection.busy(p.has_modal) {
                continue;
            }
            // No Wilderness on the ordinary world yet: the check always applies.
            if mode.not_too_strong
                && crate::hunt::too_strong(player_combat_level(&p.data), npc.combat_level)
            {
                continue;
            }
            let multi = self
                .multiway_area
                .is_some_and(|id| self.areas.at(coords).contains(&id));
            if !multi {
                if mode
                    .not_combat
                    .is_some_and(|varp| p.data.variables.get(varp).wrapping_add(8) > clock)
                {
                    continue;
                }
                if mode.not_combat_self.is_some() && npc.last_combat.wrapping_add(8) > clock {
                    continue;
                }
            }
            if mode
                .vars
                .iter()
                .flatten()
                .any(|c| !c.operator.evaluate(p.data.variables.get(c.varp), c.required))
            {
                continue;
            }
            count += 1;
            if crate::hunt::replaces(count, random) {
                target = Some(p.id);
            }
        }
        target
    }
    /// NpcHuntProcessor for player hunts: advance the hunt clock, then attack the
    /// found player and reset the hunt (range 0 unless the mode keeps hunting).
    pub(super) fn process_npc_hunt(&mut self, id: NpcId) {
        let Some(entry) = self.npcs.get_mut(id.slot as usize).filter(|e| e.id == id) else {
            return;
        };
        let npc = &mut entry.npc;
        let Some(mode) = npc.hunt_mode.and_then(|m| self.hunt_modes.get(&m)) else {
            return;
        };
        if !npc.visible || npc.delayed || npc.stats[3] <= 0 {
            return;
        }
        let near = npc.viewed || npc.headless;
        if mode.kind == Kind::Player || !(mode.pause_when_nobody_near && !near) {
            npc.hunt_clock = npc.hunt_clock.saturating_add(1);
        }
        if mode.kind != Kind::Player {
            return;
        }
        let Some(target) = npc.hunt_player else {
            return;
        };
        if !self
            .players
            .get(target.slot as usize)
            .is_some_and(|p| p.id == target && !p.saving)
        {
            return;
        }
        if mode.attacks_players() {
            npc.attacking_player = Some(target);
            npc.escape_player = None;
            npc.movement.interacting = true;
            npc.face_player(Some(target));
        }
        let keep_hunting = mode.keep_hunting;
        npc.reset_hunt();
        if !keep_hunting {
            npc.hunt_range = 0;
        }
    }
    /// PlayerRegistry.change during PlayerPostTick, and the add at login.
    pub(super) fn update_player_hunt_zone(&mut self, slot: usize, force: bool) {
        let Some(player) = self.players.get_mut(slot) else {
            return;
        };
        let next = npc_search::zone(player.movement.coords);
        if !force && next == player.hunt_zone {
            return;
        }
        self.player_hunt_order = self
            .player_hunt_order
            .checked_add(1)
            .expect("player zone order exhausted");
        player.hunt_zone = next;
        player.hunt_order = self.player_hunt_order;
    }
}
pub(super) fn apply_hunt_default(defaults: &BTreeMap<u16, u16>, npc: &mut Npc) {
    if npc.default_hunt_mode.is_some() {
        return;
    }
    if let Some(&mode) = defaults.get(&npc.type_id) {
        npc.default_hunt_mode = Some(mode);
        npc.hunt_mode = npc.hunt_mode.or(Some(mode));
    }
}
fn npc_has_interaction(npc: &Npc) -> bool {
    npc.attacking_player.is_some()
        || npc.escape_player.is_some()
        || npc.dialogue_face.is_some()
        || npc.movement.interacting
}
fn player_combat_level(data: &PlayerData) -> i32 {
    let [a, d, s, h, r, m, p] = COMBAT_SKILLS.map(|id| data.stats.base(id) as u8 as i32);
    crate::skills::combat_level(a, d, s, h, r, m, p)
}
