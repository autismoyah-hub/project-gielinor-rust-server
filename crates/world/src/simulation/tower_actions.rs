use super::*;
impl World {
    pub(super) fn process_tower_arrival(&mut self, slot: usize) {
        if self.cinderworks.is_some(){self.process_cinderworks_arrival(slot);return;}
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
            .filter_map(|(slot, item)| item.map(|item| (slot, item.id)))
            .collect();
        let hitpoints_message = if arrival.first {
            config.progression.ensure_starter_hitpoints(&mut p.data)
        } else {
            None
        };
        crate::tower_arrival::apply(&mut p.data, arrival);
        let actor = p.id;
        if arrival.strip {
            for (slot, item) in worn {
                self.worn_changed(actor, slot, item);
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
    // #124: the pad only acts when an arrival mode is configured. Off = `tower_arrival`
    // None and no Cinderworks, as `--ordinary-world` configures the server.
    fn legacy_config() -> Option<crate::tower_arrival::Config> {
        Some(crate::tower_arrival::Config {
            origin: CoordGrid::new(10368, 64, 0).unwrap(),
            progression: crate::floor_progression::Policy {
                attack: 0,
                defence: 1,
                strength: 2,
                hitpoints: 3,
                runecrafting: 20,
                herblore: 15,
            },
        })
    }
    fn coins(count: i32) -> crate::inventory::Item {
        crate::inventory::Item {
            id: 995,
            count,
            vars: 0,
        }
    }
    fn login_mode(legacy: bool, at: CoordGrid) -> (World, ActorId) {
        let mut w = World::default();
        w.tower_arrival = if legacy { legacy_config() } else { None };
        w.collision.allocate(at.x(), at.z(), at.level());
        w.collision.allocate(3222, 3218, 0);
        w.collision.allocate(3224, 3218, 0);
        w.collision.allocate(10424, 120, 0);
        w.collision.allocate(10424, 113, 0);
        let mut data = PlayerData::new(1, "Pad".into(), at, Stats::default());
        data.inventory.restore_slot(0, Some(coins(1000))).unwrap();
        data.equipment
            .restore_slot(
                3,
                Some(crate::inventory::Item {
                    id: 13,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.bank.restore_slot(100, Some(coins(777))).unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let actor = w.actors().next().unwrap();
        (w, actor)
    }
    fn tick_to(w: &mut World, clock: i32, events: &mut Vec<WorldEvent>) {
        while w.clock.0 < clock {
            w.tick().unwrap();
            events.extend(w.drain_events());
        }
    }
    fn no_arrival(events: &[WorldEvent], actor: ActorId) -> bool {
        !events.iter().any(|e| {
            matches!(e, WorldEvent::TowerArrival { actor: a, .. } if *a == actor)
                || matches!(e, WorldEvent::ClearMapFlag(a) if *a == actor)
                || matches!(e, WorldEvent::SynchronizePlayers { players, .. }
                    if players.iter().any(|(id, m)| *id == actor && m.telejump))
        })
    }
    fn possessions(w: &World, actor: ActorId) -> (CoordGrid, String) {
        let d = w.data(actor).unwrap();
        (
            d.coords,
            format!(
                "{:?}|{:?}|{:?}|{:?}|{}",
                d.inventory.slots(),
                d.equipment.slots(),
                d.bank.slots(),
                d.tower_arrival,
                d.stats.current(3)
            ),
        )
    }
    #[test]
    fn off_mode_leaves_a_player_spawned_on_the_pad_untouched() {
        let spawn = CoordGrid::new(3222, 3218, 0).unwrap();
        let (mut w, actor) = login_mode(false, spawn);
        let mut events = w.drain_events();
        let before = possessions(&w, actor);
        assert_eq!(before.0, spawn);
        for clock in [2, 8, 16, 24, 32, 33] {
            tick_to(&mut w, clock, &mut events);
            assert_eq!(possessions(&w, actor), before, "clock {clock}");
        }
        assert!(no_arrival(&events, actor));
        assert!(!w.data(actor).unwrap().tower_arrival.kit_given);
    }
    #[test]
    fn off_mode_ignores_walking_onto_the_pad() {
        let (mut w, actor) = login_mode(false, CoordGrid::new(3224, 3218, 0).unwrap());
        let mut events = w.drain_events();
        tick_to(&mut w, 3, &mut events);
        w.submit(
            actor,
            Action::Walk {
                destination: CoordGrid::new(3222, 3218, 0).unwrap(),
                run: false,
            },
        )
        .unwrap();
        tick_to(&mut w, 25, &mut events);
        let d = w.data(actor).unwrap();
        assert_eq!(d.coords, CoordGrid::new(3222, 3218, 0).unwrap());
        assert_eq!(d.inventory.get(0), Some(coins(1000)));
        assert!(!d.tower_arrival.kit_given);
        assert!(no_arrival(&events, actor));
    }
    fn lethal_on_ordinary_death(legacy: bool) -> (World, ActorId, Vec<WorldEvent>) {
        let (mut w, actor) = login_mode(legacy, CoordGrid::new(3300, 3300, 0).unwrap());
        w.ordinary_death = Some(crate::player_death::Config {
            centre: CoordGrid::new(3221, 3218, 0).unwrap(),
            animation: 836,
            skills: (0..23).collect(),
            special_attack: crate::variables::Varp {
                id: 301,
                permanent: false,
                transmit: crate::variables::Transmit::Different,
                bit_protect: false,
                client_code: 0,
            },
            stop_music: 147,
            jingle: 90,
        });
        w.regeneration = Some(crate::player_regen::Config {
            skills: (0..23).collect(),
            hitpoints: 3,
            prayer: 5,
            bracelet: 11133,
            drained_timer: 1,
            boosted_timer: 2,
            health_timer: 3,
            rapid_timer: 4,
        });
        w.collision.allocate(3221, 3218, 0);
        let hp = w.data(actor).unwrap().stats.current(3);
        assert!(hp > 0);
        w.queue_hit(
            crate::hits::Target::Player(actor),
            crate::hits::Hit {
                source: None,
                style: crate::hits::Style::Typeless,
                damage: hp as i32,
                display: None,
            },
            0,
        )
        .unwrap();
        let mut events = w.drain_events();
        // Ordinary recovery completes within ten ticks (two, then four, in the death tests).
        let start = w.clock.0;
        while w.clock.0 < start + 12
            && !events.iter().any(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. }))
        {
            w.tick().unwrap();
            events.extend(w.drain_events());
        }
        assert!(events.iter().any(|e| matches!(e, WorldEvent::OrdinaryDeathRecovered { .. })));
        (w, actor, events)
    }
    fn tower_arrival_for(events: &[WorldEvent], actor: ActorId) -> bool {
        events
            .iter()
            .any(|e| matches!(e, WorldEvent::TowerArrival { actor: a, .. } if *a == actor))
    }
    #[test]
    fn off_mode_death_respawn_on_the_pad_stays_in_the_ordinary_world() {
        let (mut w, actor, death_events) = lethal_on_ordinary_death(false);
        // The respawn teleport and its ClearMapFlag belong to ordinary death recovery
        // (player_death_actions.rs), not to a Tower arrival; only TowerArrival marks one.
        assert!(!tower_arrival_for(&death_events, actor));
        let respawn = w.data(actor).unwrap().coords;
        assert!(crate::tower_arrival::recovery_pad(respawn), "respawn {respawn:?} is on the pad");
        let after = possessions(&w, actor);
        // Seventeen further ticks always include at least two periodic pad checks.
        let mut later = Vec::new();
        for _ in 0..17 {
            w.tick().unwrap();
            later.extend(w.drain_events());
            assert_eq!(possessions(&w, actor), after, "clock {}", w.clock.0);
        }
        assert!(!tower_arrival_for(&later, actor));
        assert!(!later
            .iter()
            .any(|e| matches!(e, WorldEvent::ClearMapFlag(a) if *a == actor)));
    }
    #[test]
    fn legacy_mode_catches_a_death_respawn_on_the_pad() {
        // The same death in ordinary mode respawns on the pad (asserted there too).
        let (off, off_actor, _) = lethal_on_ordinary_death(false);
        assert!(crate::tower_arrival::recovery_pad(off.data(off_actor).unwrap().coords));
        // Ordinary death recovery runs before the PlayerMain pad check in the same tick, so a
        // respawn on a periodic tick is caught in that tick; otherwise at the next check.
        let (mut w, actor, mut events) = lethal_on_ordinary_death(true);
        let until = w.clock.0 + 9;
        tick_to(&mut w, until, &mut events);
        assert_eq!(w.data(actor).unwrap().coords.x(), 10424);
        assert!(tower_arrival_for(&events, actor));
    }
    #[test]
    fn legacy_mode_strips_once_then_only_re_teleports() {
        let (mut w, actor) = login_mode(true, CoordGrid::new(3222, 3218, 0).unwrap());
        let d = w.data(actor).unwrap();
        assert_eq!(d.coords, CoordGrid::new(10424, 120, 0).unwrap());
        assert!(d.tower_arrival.kit_given);
        assert!(d.inventory.get(0).is_none());
        assert!(d.equipment.get(3).is_none());
        assert_eq!(d.bank.get(100), Some(coins(777)));
        let slot = actor.slot as usize;
        let clock = w.clock.0;
        let pad = CoordGrid::new(3222, 3218, 0).unwrap();
        let p = w.players.get_mut(slot).unwrap();
        p.movement.teleport(&mut w.collision, pad, clock, true).unwrap();
        p.data.coords = pad;
        p.data.inventory.restore_slot(0, Some(coins(5))).unwrap();
        let mut events = Vec::new();
        tick_to(&mut w, 17, &mut events);
        let d = w.data(actor).unwrap();
        assert_eq!(d.coords, CoordGrid::new(10424, 113, 0).unwrap());
        assert_eq!(d.inventory.get(0), Some(coins(5)));
        assert_eq!(d.bank.get(100), Some(coins(777)));
    }
    #[test]
    fn tiles_outside_the_pad_are_never_moved_in_any_mode() {
        for legacy in [false, true] {
            for at in [
                CoordGrid::new(3224, 3218, 0).unwrap(),
                CoordGrid::new(3218, 3218, 0).unwrap(),
                CoordGrid::new(3222, 3215, 0).unwrap(),
                CoordGrid::new(3222, 3221, 0).unwrap(),
                CoordGrid::new(3222, 3218, 1).unwrap(),
            ] {
                assert!(!crate::tower_arrival::recovery_pad(at));
                let (mut w, actor) = login_mode(legacy, at);
                let mut events = w.drain_events();
                let before = possessions(&w, actor);
                tick_to(&mut w, 33, &mut events);
                assert_eq!(possessions(&w, actor), before, "legacy {legacy} at {at:?}");
                assert!(no_arrival(&events, actor), "legacy {legacy} at {at:?}");
            }
        }
    }
}
