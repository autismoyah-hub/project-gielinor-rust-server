use super::*;
impl World {
    pub(super) fn process_tower_arrival(&mut self, slot: usize) {
        let Some(config) = self.tower_arrival else {
            return;
        };
        let Some(p) = self.players.get_mut(slot) else {
            return;
        };
        let age = self.clock.0.wrapping_sub(p.login_cycle);
        // EngineLoginReady catches tutorial completion after two cycles;
        // the ordinary ash timer repeats every eight cycles after login.
        if p.saving
            || !(age == 0 || age == 2 || age > 0 && age % 8 == 0)
            || !crate::tower_arrival::recovery_pad(p.movement.coords)
        {
            return;
        }
        let packed = p
            .data
            .inventory
            .slots()
            .iter()
            .chain(p.data.equipment.slots())
            .any(Option::is_some);
        let Ok(arrival) = p.data.tower_arrival.arrival(config.origin, packed) else {
            return;
        };
        if let Err(error) =
            p.movement
                .teleport(&mut self.collision, arrival.destination, self.clock.0, true)
        {
            self.events.push(WorldEvent::LocationError(error));
            return;
        }
        p.data.coords = arrival.destination;
        p.npc_interaction = None;
        p.location_interaction = None;
        p.object_interaction = None;
        let worn: Vec<_> = p
            .data
            .equipment
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(slot, item)| item.is_some().then_some(slot))
            .collect();
        let hitpoints_message = if arrival.first {
            config.progression.ensure_starter_hitpoints(&mut p.data)
        } else {
            None
        };
        crate::tower_arrival::apply(&mut p.data, arrival);
        let actor = p.id;
        if arrival.strip {
            for slot in worn {
                self.worn_changed(actor, slot);
            }
        }
        self.events.push(WorldEvent::ClearMapFlag(actor));
        self.events.push(WorldEvent::TowerArrival {
            actor,
            arrival,
            hitpoints_message,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup(allocated: bool) -> (World, ActorId) {
        setup_at(allocated, 3222)
    }
    fn setup_at(allocated: bool, x: i32) -> (World, ActorId) {
        let mut w = World::default();
        w.tower_arrival = Some(crate::tower_arrival::Config {
            origin: CoordGrid::new(10368, 64, 0).unwrap(),
            progression: crate::floor_progression::Policy {
                attack: 0,
                defence: 1,
                strength: 2,
                hitpoints: 3,
                runecrafting: 20,
                herblore: 15,
            },
        });
        w.collision.allocate(3222, 3218, 0);
        w.collision.allocate(3224, 3218, 0);
        if allocated {
            w.collision.allocate(10424, 120, 0);
        }
        let mut data = PlayerData::new(
            1,
            "Arrival".into(),
            CoordGrid::new(x, 3218, 0).unwrap(),
            Stats::default(),
        );
        data.inventory
            .restore_slot(
                0,
                Some(crate::inventory::Item {
                    id: 995,
                    count: 1000,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        (w, actor)
    }
    #[test]
    fn arrival_updates_authoritative_state_and_synchronization_before_save() {
        let (mut w, actor) = setup(true);
        let data = w.data(actor).unwrap();
        assert_eq!(data.coords, CoordGrid::new(10424, 120, 0).unwrap());
        assert!(data.tower_arrival.kit_given);
        assert!(data.inventory.get(0).is_none());
        assert!(w.drain_events().iter().any(|event| matches!(event,WorldEvent::SynchronizePlayers {players,..} if players.iter().any(|(id,m)| *id==actor && m.telejump))));
        w.request_logout(actor).unwrap();
        assert!(w.drain_events().iter().any(|event| matches!(event,WorldEvent::Save {data,..} if data.tower_arrival.kit_given && data.inventory.get(0).is_none())));
    }
    #[test]
    fn missing_destination_never_strips_inventory_or_marks_kit() {
        let (mut w, actor) = setup(false);
        for _ in 0..10 {
            w.tick().unwrap();
        }
        let data = w.data(actor).unwrap();
        assert_eq!(data.coords, CoordGrid::new(3222, 3218, 0).unwrap());
        assert!(!data.tower_arrival.kit_given);
        assert_eq!(data.inventory.get(0).unwrap().count, 1000);
    }
    #[test]
    fn walking_onto_pad_after_login_is_caught_by_periodic_check() {
        let (mut w, actor) = setup_at(true, 3224);
        while w.clock.0 < 3 {
            w.tick().unwrap();
        }
        w.submit(
            actor,
            Action::Walk {
                destination: CoordGrid::new(3222, 3218, 0).unwrap(),
                run: false,
            },
        )
        .unwrap();
        while w.clock.0 < 8 {
            w.tick().unwrap();
            assert!(!w.data(actor).unwrap().tower_arrival.kit_given);
        }
        assert_eq!(
            w.data(actor).unwrap().coords,
            CoordGrid::new(3222, 3218, 0).unwrap()
        );
        w.tick().unwrap();
        assert_eq!(
            w.data(actor).unwrap().coords,
            CoordGrid::new(10424, 120, 0).unwrap()
        );
        assert!(w.data(actor).unwrap().tower_arrival.kit_given);
    }
}
