use super::*;
use crate::{
    tutorial_leave::Config,
    tutorial_magic_ui::{self, Home},
};
impl World {
    /// The UI host admits the overlay button; this is ProtectedAccessLauncher's gate.
    pub fn tutorial_home(&mut self, actor: ActorId, config: Config) -> Result<(), ActionError> {
        let p = self.player(actor)?;
        if p.saving || p.pending_logout {
            return Err(ActionError::LoggingOut);
        }
        if p.protection.access_protected(p.has_modal) {
            return Err(ActionError::Protected);
        }
        match tutorial_magic_ui::home(
            config.progression.current(&p.data.variables),
            p.data.variables.get(config.progression.completed.id),
        ) {
            Home::None => return Ok(()),
            Home::Deny(message) => {
                self.events
                    .push(WorldEvent::LocationMessage { actor, message });
                return Ok(());
            }
            Home::Leave => (),
        }
        self.give_tutorial_leave_kit(actor, config)
    }
    /// Called within an already admitted protected dialogue or Home callback.
    pub(super) fn give_tutorial_leave_kit(
        &mut self,
        actor: ActorId,
        config: Config,
    ) -> Result<(), ActionError> {
        self.player(actor)?;
        let p = self.players.get_mut(actor.slot as usize).unwrap();
        config.grant(&mut p.data.inventory, &mut p.data.bank, &self.object_types);
        let destination = Config::destination();
        if p.movement
            .teleport(&mut self.collision, destination, self.clock.0, false)
            .is_err()
        {
            // ProtectedAccess.teleport returns after reporting the invalid map; the
            // original caller still finishes progression and does not roll back the kit.
            self.events.push(WorldEvent::ClearMapFlag(actor));
            self.events.push(WorldEvent::EngineMessage {
                actor,
                message: "Invalid teleport!",
            });
        } else {
            p.data.coords = destination;
        }
        let effects = config
            .progression
            .finish_and_leave(&mut p.data.variables, p.protection.processed_clock > 0);
        self.events
            .push(WorldEvent::TutorialLeaveProgress { actor, effects });
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Insert, Item},
        stats::Stats,
        variables::{Transmit, Varp},
    };
    fn setup(allocated: bool) -> (World, ActorId, Config) {
        let mut world = World::default();
        let start = CoordGrid::new(1686, 6115, 0).unwrap();
        world.collision.allocate(start.x(), start.z(), 0);
        if allocated {
            let to = Config::destination();
            world.collision.allocate(to.x(), to.z(), to.level());
        }
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        let config = Config {
            progression: crate::tutorial_progression::Config {
                step: varp(1),
                completed: varp(2),
            },
            items: std::array::from_fn(|i| i as i32 + 100),
            coins: 995,
        };
        world.object_types.stackable.extend(109..=114);
        world.object_types.stackable.insert(995);
        let mut data = PlayerData::new(1, "Leaving".into(), start, Stats::default());
        data.variables.set(config.progression.step, 51, false);
        data.inventory
            .insert(&world.object_types, Insert::new(42, 1))
            .unwrap();
        data.equipment
            .restore_slot(
                3,
                Some(Item {
                    id: 1205,
                    count: 1,
                    vars: 9,
                }),
            )
            .unwrap();
        world.queue_login(1, data).unwrap();
        world.tick().unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        world.drain_events();
        (world, actor, config)
    }
    #[test]
    fn home_teleport_grants_once_preserves_worn_items_and_completes_both_variables() {
        let (mut world, actor, config) = setup(true);
        world.tutorial_home(actor, config).unwrap();
        let data = world.data(actor).unwrap();
        assert_eq!(data.coords, Config::destination());
        assert_eq!(data.inventory.get(0).unwrap().id, 100);
        assert_eq!(
            data.equipment.get(3),
            Some(Item {
                id: 1205,
                count: 1,
                vars: 9
            })
        );
        assert_eq!((data.variables.get(1), data.variables.get(2)), (53, 1));
        assert_eq!(
            data.bank.get(0),
            Some(Item {
                id: 995,
                count: 25,
                vars: 0
            })
        );
        let image = data.inventory.clone();
        world.drain_events();
        world.tutorial_home(actor, config).unwrap();
        assert_eq!(world.data(actor).unwrap().inventory, image);
        assert!(world.drain_events().is_empty());
    }
    #[test]
    fn missing_destination_reports_error_but_matches_reference_completion_without_rollback() {
        let (mut world, actor, config) = setup(false);
        let start = world.data(actor).unwrap().coords;
        world.tutorial_home(actor, config).unwrap();
        let data = world.data(actor).unwrap();
        assert_eq!(data.coords, start);
        assert_eq!((data.variables.get(1), data.variables.get(2)), (53, 1));
        assert_eq!(data.inventory.get(0).unwrap().id, 100);
        let events = world.drain_events();
        assert!(matches!(
            events.as_slice(),
            [
                WorldEvent::ClearMapFlag(_),
                WorldEvent::EngineMessage {
                    message: "Invalid teleport!",
                    ..
                },
                WorldEvent::TutorialLeaveProgress { .. }
            ]
        ));
    }
    #[test]
    fn protected_launcher_does_not_clear_inventory_or_run_early_home_denial() {
        let (mut world, actor, config) = setup(true);
        let before = world.data(actor).unwrap().clone();
        world.delay_player(actor, 3).unwrap();
        assert_eq!(
            world.tutorial_home(actor, config),
            Err(ActionError::Protected)
        );
        assert_eq!(world.data(actor).unwrap().inventory, before.inventory);
        assert!(world.drain_events().is_empty());
    }
}
