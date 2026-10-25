//! MapClockScript and SpecialEnergyScript. Wall time is supplied by the host.
use crate::{
    bank_actions::Binding,
    player_timers::Timers,
    variables::{self, Variables, Varp},
};

#[derive(Clone, Debug)]
pub struct Config {
    pub clock_timer: u16,
    pub special_timer: u16,
    pub milliseconds: Binding,
    pub seconds: Binding,
    pub map_clock: Varp,
    pub playtime: Varp,
    pub energy: Varp,
    pub new_account: Binding,
}
impl Config {
    pub fn prepare_account(
        &self,
        vars: &mut Variables,
        new_account: bool,
    ) -> Result<(), variables::Error> {
        vars.set_bit(
            self.new_account.base,
            self.new_account.bit,
            i32::from(new_account),
            false,
        )?;
        Ok(())
    }
    pub fn login(
        &self,
        vars: &mut Variables,
        timers: &mut Timers,
        cycle: i32,
    ) -> Result<(), variables::Error> {
        if vars.bit(self.new_account.bit)? != 0 {
            vars.set(self.energy, 1000, false);
        }
        timers.schedule(self.clock_timer, cycle, 1).unwrap();
        timers.schedule(self.special_timer, cycle, 50).unwrap();
        Ok(())
    }
    pub fn tick(
        &self,
        timer: u16,
        vars: &mut Variables,
        cycle: i32,
        unix_millis: u64,
    ) -> Result<Vec<variables::Update>, variables::Error> {
        let mut updates = Vec::new();
        if timer == self.clock_timer {
            // Preserve Kotlin's two immediate base-varp writes, in source order.
            for (binding, value) in [
                (self.milliseconds, (unix_millis % 1000) as i32),
                (self.seconds, ((unix_millis / 1000) % 60) as i32),
            ] {
                updates.extend(vars.set_bit(binding.base, binding.bit, value, true)?);
            }
            updates.extend(vars.set(self.map_clock, cycle, true));
            let playtime = vars.get(self.playtime.id).wrapping_add(1);
            updates.extend(vars.set(self.playtime, playtime, true));
        } else if timer == self.special_timer {
            let previous = vars.get(self.energy.id);
            let increased = previous.wrapping_add(100).min(1000);
            if increased > previous {
                updates.extend(vars.set(self.energy, increased, true));
            }
        }
        Ok(updates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::{Transmit, Varbit};
    fn config() -> Config {
        let varp = |id| Varp {
            id,
            permanent: true,
            transmit: Transmit::Different,
            bit_protect: false,
            client_code: -1,
        };
        Config {
            clock_timer: 1,
            special_timer: 6,
            milliseconds: Binding {
                base: varp(3077),
                bit: Varbit {
                    base: 3077,
                    lsb: 0,
                    msb: 9,
                },
            },
            seconds: Binding {
                base: varp(3077),
                bit: Varbit {
                    base: 3077,
                    lsb: 10,
                    msb: 15,
                },
            },
            map_clock: varp(3079),
            playtime: varp(65515),
            energy: varp(300),
            new_account: Binding {
                base: varp(100),
                bit: Varbit {
                    base: 100,
                    lsb: 0,
                    msb: 0,
                },
            },
        }
    }
    #[test]
    fn admitted_world_dispatches_timers_and_retains_their_values_in_save() {
        use crate::{
            simulation::{PlayerData, World, WorldEvent},
            stats::Stats,
            CoordGrid,
        };
        let mut world = World::default();
        world.clock.0 = 100;
        world.session_vars = Some(config());
        world.unix_millis = 59_123;
        world.collision.allocate(3224, 3218, 0);
        world
            .queue_login(
                1,
                PlayerData::new(
                    1,
                    "Clock".into(),
                    CoordGrid::new(3224, 3218, 0).unwrap(),
                    Stats::default(),
                ),
            )
            .unwrap();
        world.tick().unwrap();
        let actor = world.actors().next().unwrap();
        assert!(!world.data(actor).unwrap().variables.contains(65515));
        world.drain_events();
        for _ in 0..51 {
            world.tick().unwrap();
        }
        assert_eq!(world.data(actor).unwrap().variables.get(300), 200);
        assert_eq!(world.data(actor).unwrap().variables.get(65515), 51);
        assert!(world.drain_events().iter().any(|e| matches!(e, WorldEvent::SessionVariables { updates, .. } if updates.iter().any(|u| u.id == 300 && u.value == 100))));
        world.request_logout(actor).unwrap();
        assert!(world.drain_events().iter().any(|e| matches!(e, WorldEvent::Save { data, .. } if data.variables.get(65515) == 51 && data.variables.get(3079) == 151 && data.variables.get(300) == 200)));
    }
    #[test]
    fn clock_preserves_bits_write_order_rollover_and_elapsed_playtime() {
        let c = config();
        let mut vars = Variables::default();
        vars.set(c.milliseconds.base, 1 << 20, false);
        vars.set(c.playtime, i32::MAX, false);
        let updates = c.tick(1, &mut vars, 81, 59_999).unwrap();
        assert_eq!(
            updates.iter().map(|u| u.id).collect::<Vec<_>>(),
            [3077, 3077, 3079, 65515]
        );
        assert_eq!(updates[0].value, (1 << 20) | 999);
        assert_eq!(vars.bit(c.seconds.bit), Ok(59));
        assert_eq!(vars.get(c.playtime.id), i32::MIN);
        c.tick(1, &mut vars, 82, 60_000).unwrap();
        assert_eq!(vars.get(3077), 1 << 20);
        assert_eq!(vars.get(c.map_clock.id), 82);
    }
    #[test]
    fn existing_accounts_regenerate_after_fifty_cycles_and_new_accounts_start_full() {
        let c = config();
        let mut vars = Variables::default();
        let mut timers = Timers::default();
        c.login(&mut vars, &mut timers, 10).unwrap();
        assert!(!vars.contains(c.energy.id));
        for cycle in 11..60 {
            timers.process(cycle, |_, id| {
                c.tick(id, &mut vars, cycle, 0).unwrap();
                true
            });
        }
        assert_eq!(vars.get(c.energy.id), 0);
        timers.process(60, |_, id| {
            c.tick(id, &mut vars, 60, 0).unwrap();
            true
        });
        assert_eq!(vars.get(c.energy.id), 100);
        vars.set(c.energy, 950, false);
        c.tick(6, &mut vars, 110, 0).unwrap();
        assert_eq!(vars.get(c.energy.id), 1000);
        assert!(c.tick(6, &mut vars, 160, 0).unwrap().is_empty());
        vars.set(c.energy, i32::MAX, false);
        assert!(c.tick(6, &mut vars, 210, 0).unwrap().is_empty());
        vars.set_bit(c.new_account.base, c.new_account.bit, 1, false)
            .unwrap();
        c.login(&mut vars, &mut timers, 220).unwrap();
        assert_eq!(vars.get(c.energy.id), 1000);
    }
    #[test]
    fn authenticated_account_status_overrides_any_loaded_flag_before_login_hooks() {
        let c = config();
        let mut vars = Variables::default();
        let mut timers = Timers::default();
        vars.set(c.new_account.base, 0b1011, false);
        c.prepare_account(&mut vars, false).unwrap();
        assert_eq!(vars.get(c.new_account.base.id), 0b1010);
        c.login(&mut vars, &mut timers, -1).unwrap();
        assert!(!vars.contains(c.energy.id));
        c.prepare_account(&mut vars, true).unwrap();
        c.login(&mut vars, &mut timers, -1).unwrap();
        assert_eq!(vars.get(c.energy.id), 1000);
    }

    #[test]
    fn retained_energy_999_caps_and_late_relogin_restarts_fifty_cycle_timer() {
        let c = config();
        let definitions = [(c.energy.id, c.energy)].into();
        let mut vars = Variables::default();
        vars.set(c.energy, 999, false);
        // The save uses persistent varps; loading an existing account must not
        // promote its retained 999 to 1000 or preserve an old elapsed timer.
        let saved = vars.persistent(&definitions);
        let mut loaded = Variables::default();
        for (id, value) in saved { loaded.set(definitions[&id], value, false); }
        let mut timers = Timers::default();
        c.prepare_account(&mut loaded, false).unwrap();
        c.login(&mut loaded, &mut timers, 900).unwrap();
        assert_eq!(loaded.get(300), 999);
        for cycle in 901..950 {
            timers.process(cycle, |_, id| { c.tick(id, &mut loaded, cycle, 0).unwrap(); true });
        }
        assert_eq!(loaded.get(300), 999);
        let mut updates = vec![];
        timers.process(950, |_, id| { updates.extend(c.tick(id, &mut loaded, 950, 0).unwrap()); true });
        assert_eq!(loaded.get(300), 1000);
        assert!(updates.iter().any(|u| u.id == 300 && u.value == 1000));
        assert!(c.tick(c.special_timer, &mut loaded, 1000, 0).unwrap().is_empty());
    }
}
