use super::*;
impl World {
    /// Complete deferred login effects once the authenticated observer identity
    /// exists. The host renders the current marks before emitting the chat.
    pub fn finish_tower_diary_login(
        &mut self,
        actor: ActorId,
    ) -> Result<Option<crate::tower_diary::Sync>, ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        if p.tower_diary_login.is_some_and(|(sync, _)| sync.quiet_imp)
            && (p.observer_id.is_none() || self.tower_tanning.is_none())
        {
            return Err(ActionError::InvalidItem);
        }
        let Some((sync, coords)) = p.tower_diary_login.take() else {
            return Ok(None);
        };
        if sync.quiet_imp {
            // Cinder hustle and ash-tile state are not active in this host yet.
            if let (Some(config), Some(observer)) = (&self.tower_tanning, p.observer_id) {
                if let Err(error) = self.objects.add(
                    crate::ground::Object {
                        coords,
                        item: config.bones,
                        count: 1,
                        scope: crate::ground::Scope::Private,
                        created: self.clock.0,
                        receiver: Some(observer),
                        owner: Some(observer),
                    },
                    &self.object_types,
                    self.clock,
                    p.loot_drop_duration.unwrap_or(200),
                    100,
                ) {
                    self.events.push(WorldEvent::GroundError(error));
                }
            }
        }
        Ok(Some(sync))
    }
    pub(super) fn publish_tower_diary_sync(
        &mut self,
        actor: ActorId,
        sync: crate::tower_diary::Sync,
        bones: u16,
    ) {
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        p.data.tower_diary = Some(sync.marks.into());
        // Preserve the separate arrival recovery projection after this action.
        p.data.tower_arrival.diary_progress |= p.data.tower_diary.unwrap().has_progress();
        self.events.push(WorldEvent::TowerDiary {
            actor,
            marks: sync.marks,
        });
        if sync.quiet_imp {
            // The cinder-hustle/ash-tile systems are not active in this host yet.
            // Their future transient state must also be cleared on this transition.
            if let Some(observer) = p.observer_id {
                if let Err(error) = self.objects.add(
                    crate::ground::Object {
                        coords: p.movement.coords,
                        item: bones,
                        count: 1,
                        scope: crate::ground::Scope::Private,
                        created: self.clock.0,
                        receiver: Some(observer),
                        owner: Some(observer),
                    },
                    &self.object_types,
                    self.clock,
                    p.loot_drop_duration.unwrap_or(200),
                    100,
                ) {
                    self.events.push(WorldEvent::GroundError(error));
                }
            }
        }
        if let Some(message) = sync.message {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: message.into(),
            });
        }
    }
    /// Reached onOpNpc1/onOpNpc3 callbacks from FloorOneScript.
    pub(super) fn tan_tower_hides(
        &mut self,
        actor: ActorId,
        target: NpcId,
        talk: bool,
    ) -> Result<(), ActionError> {
        let config = self
            .tower_tanning
            .as_ref()
            .ok_or(ActionError::InvalidItem)?;
        let p = self.player(actor)?;
        if !config.contains(p.movement.coords)
            || talk
                && self
                    .npc(target)
                    .is_none_or(|n| !config.contains(n.movement.coords))
        {
            return Ok(());
        }
        // Do not consume items while the original session marks are unknown.
        let progress = p.data.tower_diary.ok_or(ActionError::UnknownTowerDiary)?;
        if talk {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "I'm a tanner. Bring cowhides. I don't keep a shop.".into(),
            });
        }
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        let outcome = config
            .inventory
            .tan(&mut p.data.inventory, &self.object_types);
        if !outcome.found_hides {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "Bring cowhides. The tanner tans; he does not keep a shop.".into(),
            });
            return Ok(());
        }
        if outcome.no_space {
            self.events.push(WorldEvent::GameMessage {
                actor,
                text: "Not enough space for the leather.".into(),
            });
        }
        let sync = progress
            .marks(
                config.skills.map(|id| p.data.stats.base(id) as i32),
                p.data.tower_arrival.staff_taken,
            )
            .add(1, outcome.converted)
            .session_sync();
        let bones = config.bones;
        self.publish_tower_diary_sync(actor, sync, bones);
        self.events.push(WorldEvent::GameMessage {
            actor,
            text: format!(
                "The tanner tans {} cowhide(s) into leather. Hides {}/{}.",
                outcome.converted,
                sync.marks.counts[0],
                crate::tower_diary::CAPS[0]
            ),
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        tower_diary::Progress,
        tower_tanning::{Config, HandlerConfig},
    };
    fn configured() -> World {
        let mut w = World::default();
        w.collision.allocate(3200, 3200, 0);
        w.tower_tanning = Some(HandlerConfig {
            origin: CoordGrid::new(10368, 64, 0).unwrap(),
            inventory: Config {
                hides: [1739].into(),
                leather: 1741,
            },
            skills: [12, 13, 11],
            bones: 526,
        });
        w
    }
    fn player() -> PlayerData {
        PlayerData::new(
            1,
            "Diary".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        )
    }
    #[test]
    fn login_syncs_legacy_marks_before_arrival_and_keeps_unknown_absent() {
        for known in [false, true] {
            let mut w = configured();
            let mut data = player();
            if known {
                data.tower_diary = Some(Progress {
                    counts: [7, 0, 0, 0, 0, 0],
                    ..Default::default()
                });
            }
            data.inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: 1739,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            w.queue_login(1, data).unwrap();
            w.tick().unwrap();
            let actor = w.actors().next().unwrap();
            assert_eq!(w.data(actor).unwrap().tower_diary.is_some(), known);
            assert_eq!(w.data(actor).unwrap().tower_arrival.diary_progress, known);
            let sync = w.finish_tower_diary_login(actor).unwrap();
            assert_eq!(sync.is_some(), known);
            if let Some(sync) = sync {
                assert!(sync.marks.schema11);
                assert_eq!((sync.marks.counts[0], sync.marks.bounty), (5, 1));
                assert_eq!(sync.message, Some("The chest holds a bronze axe."));
            }
            assert!(w.finish_tower_diary_login(actor).unwrap().is_none());
        }
    }
    #[test]
    fn login_bones_wait_for_identity_and_effects_can_only_finish_once() {
        let mut w = configured();
        let mut data = player();
        let mut marks = Progress {
            schema11: true,
            pending: [true, false],
            ..Default::default()
        };
        marks.done[..9].fill(true);
        data.tower_diary = Some(marks);
        let coords = data.coords;
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        assert_eq!(
            w.finish_tower_diary_login(actor),
            Err(ActionError::InvalidItem)
        );
        assert_eq!(w.objects.count(), 0);
        w.set_observer_id(actor, 700).unwrap();
        let sync = w.finish_tower_diary_login(actor).unwrap().unwrap();
        assert!(sync.quiet_imp);
        assert!(w.finish_tower_diary_login(actor).unwrap().is_none());
        let bones = w.objects.at(coords).next().unwrap().snapshot();
        assert_eq!((bones.item, bones.owner, bones.count), (526, Some(700), 1));
        assert_eq!(w.objects.count(), 1);
    }
}
