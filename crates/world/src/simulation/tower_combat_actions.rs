use super::*;
impl World {
    /// StandardNpcHitModifier publishes content Modify before flat armour.
    /// Invoked after rolled-damage XP, immediately before queueing the hit.
    pub(super) fn modify_npc_combat_damage(
        &mut self,
        source: Option<ActorId>,
        target: NpcId,
        mut damage: i32,
    ) -> Result<i32, ActionError> {
        let kind = self.npc(target).ok_or(ActionError::StaleActor)?.type_id;
        if let Some(config) = self.tower_combat.as_ref() {
            let player = source.and_then(|id| self.player(id).ok());
            if config.bosses.contains(&kind) {
                let armed = player
                    .and_then(|p| p.data.equipment.get(3))
                    .is_some_and(|item| config.scimitars.contains(&item.id)
                        || self.cinderworks.as_ref().is_some_and(|c| player.is_some_and(|p| p.data.tower_run.active()) && c.forged_weapon(item.id)));
                if !armed {
                    damage = 0;
                    if let Some(p) = player {
                        self.events.push(WorldEvent::GameMessage {
                            actor: p.id,
                            text: "Wield a bronze scimitar. A bow or a staff does not count."
                                .into(),
                        });
                    }
                }
            } else if kind == config.chicken && damage > 0 {
                if let Some(p) = player {
                    let bow = p
                        .data
                        .equipment
                        .get(3)
                        .is_some_and(|i| config.shortbows.contains(&i.id));
                    let arrows = p
                        .data
                        .equipment
                        .get(13)
                        .is_some_and(|i| config.bronze_arrows.contains(&i.id));
                    if bow && arrows {
                        let actor = p.id;
                        let bones = config.bones;
                        if let Some(diary) = p.data.tower_diary {
                            let sync = diary
                                .marks(
                                    config.skills.map(|s| p.data.stats.base(s) as i32),
                                    p.data.tower_arrival.staff_taken,
                                )
                                .note(6)
                                .session_sync();
                            self.publish_tower_diary_sync(actor, sync, bones);
                        } else {
                            self.events.push(WorldEvent::ActionRejected {
                                actor,
                                error: ActionError::UnknownTowerDiary,
                            });
                        }
                    }
                }
            }
        }
        let npc = self.npc_mut(target).ok_or(ActionError::StaleActor)?;
        if npc.flat_armour > 0 {
            let blocked = npc.flat_armour.min(damage);
            npc.flat_armour -= blocked;
            damage -= blocked;
        } else if npc.flat_armour < 0 {
            let extra = npc.flat_armour.wrapping_abs();
            npc.flat_armour = 0;
            damage = damage.wrapping_add(extra);
        }
        Ok(damage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        npc::{BlockWalk, Npc, Restrict},
        tower_diary::Progress,
    };
    fn setup(kind: u16) -> (World, ActorId, NpcId) {
        let mut w = World::default();
        let c = CoordGrid::new(3200, 3200, 0).unwrap();
        w.collision.allocate(c.x(), c.z(), 0);
        w.queue_login(1, PlayerData::new(1, "Archer".into(), c, Stats::default()))
            .unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        let n = w
            .spawn_npc(
                Npc::new(
                    kind,
                    c.translate(1, 0, 0).unwrap(),
                    1,
                    Restrict::Normal,
                    BlockWalk::None,
                )
                .unwrap(),
            )
            .unwrap();
        w.tower_combat = Some(crate::tower_combat::Config {
            bosses: [1, 2].into(),
            chicken: 3,
            scimitars: [1321, 900].into(),
            shortbows: [841].into(),
            bronze_arrows: [882].into(),
            skills: [12, 13, 11],
            bones: 526,
        });
        w.drain_events();
        (w, a, n)
    }
    fn wear(w: &mut World, a: ActorId, slot: usize, id: i32) {
        w.data_mut(a)
            .unwrap()
            .equipment
            .restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
    }
    #[test]
    fn tower_gate_runs_before_armour_and_resolves_current_player_identity() {
        let (mut w, a, n) = setup(1);
        w.npc_mut(n).unwrap().flat_armour = 4;
        assert_eq!(w.modify_npc_combat_damage(Some(a), n, 7), Ok(0));
        assert_eq!(w.npc(n).unwrap().flat_armour, 4);
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e,WorldEvent::GameMessage{actor,..} if *actor==a)));
        wear(&mut w, a, 3, 900);
        assert_eq!(w.modify_npc_combat_damage(Some(a), n, 7), Ok(3));
        assert_eq!(w.npc(n).unwrap().flat_armour, 0);
        let stale = ActorId {
            generation: a.generation.wrapping_add(1),
            ..a
        };
        assert_eq!(w.modify_npc_combat_damage(Some(stale), n, 7), Ok(0));
        assert_eq!(w.modify_npc_combat_damage(None, n, 7), Ok(0));
        assert!(w.drain_events().is_empty());
        // Source flat armour follows the event even if a negative armour value
        // introduces damage after the Tower hook zeroed the hit.
        w.npc_mut(n).unwrap().flat_armour = -2;
        assert_eq!(w.modify_npc_combat_damage(None, n, 7), Ok(2));
    }
    #[test]
    fn tower_chicken_credits_positive_prearmour_damage_without_floor_or_style_gate() {
        let (mut w, a, n) = setup(3);
        wear(&mut w, a, 3, 841);
        wear(&mut w, a, 13, 882);
        let mut diary = Progress {
            schema11: true,
            ..Default::default()
        };
        diary.done[..8].fill(true);
        diary.counts[5] = 15;
        w.data_mut(a).unwrap().tower_diary = Some(diary);
        w.npc_mut(n).unwrap().flat_armour = 8;
        assert_eq!(w.modify_npc_combat_damage(Some(a), n, 0), Ok(0));
        assert!(!w.data(a).unwrap().tower_diary.unwrap().actions[6]);
        assert_eq!(w.modify_npc_combat_damage(Some(a), n, 3), Ok(0));
        assert_eq!(w.npc(n).unwrap().flat_armour, 5);
        assert!(w.data(a).unwrap().tower_diary.unwrap().actions[6]);
        assert!(w
            .drain_events()
            .iter()
            .any(|e| matches!(e,WorldEvent::TowerDiary{actor,..} if *actor==a)));
    }
    #[test]
    fn tower_chicken_preserves_unknown_and_requires_arrows_remaining_in_quiver() {
        let (mut w, a, n) = setup(3);
        wear(&mut w, a, 3, 841);
        assert_eq!(w.modify_npc_combat_damage(Some(a), n, 2), Ok(2));
        assert!(w.drain_events().is_empty());
        wear(&mut w, a, 13, 882);
        assert_eq!(w.modify_npc_combat_damage(Some(a), n, 2), Ok(2));
        assert!(w.data(a).unwrap().tower_diary.is_none());
        assert!(w.drain_events().iter().any(|e| matches!(
            e,
            WorldEvent::ActionRejected {
                error: ActionError::UnknownTowerDiary,
                ..
            }
        )));
    }
}
