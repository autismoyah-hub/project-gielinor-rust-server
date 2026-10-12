#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockExhausted;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MapClock(pub i32);
impl MapClock {
    pub fn tick(&mut self) -> Result<(), ClockExhausted> {
        self.0 = self.0.checked_add(1).ok_or(ClockExhausted)?;
        Ok(())
    }
    pub fn deadline(self, delay: i32) -> i32 {
        ((self.0 as i64 + delay as i64).min(i32::MAX as i64)) as i32
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Start,
    WorldMain,
    NpcPreTick,
    PlayerShuffle,
    PlayerInput,
    PlayerRoute,
    NpcMain,
    ControllerMain,
    PlayerMain,
    PlayerLogout,
    PlayerLogin,
    Late,
    WorldPostTick,
    PlayerPostTick,
    End,
}
pub const PHASES: [Phase; 15] = [
    Phase::Start,
    Phase::WorldMain,
    Phase::NpcPreTick,
    Phase::PlayerShuffle,
    Phase::PlayerInput,
    Phase::PlayerRoute,
    Phase::NpcMain,
    Phase::ControllerMain,
    Phase::PlayerMain,
    Phase::PlayerLogout,
    Phase::PlayerLogin,
    Phase::Late,
    Phase::WorldPostTick,
    Phase::PlayerPostTick,
    Phase::End,
];
#[derive(Debug, PartialEq, Eq)]
pub enum CycleError<E> {
    ClockExhausted,
    Handler(E),
}
pub fn run_cycle<E>(
    clock: &mut MapClock,
    mut process: impl FnMut(Phase, i32) -> Result<(), E>,
) -> Result<(), CycleError<E>> {
    for phase in PHASES {
        if phase == Phase::Late {
            clock.tick().map_err(|_| CycleError::ClockExhausted)?;
        }
        process(phase, clock.0).map_err(CycleError::Handler)?;
    }
    Ok(())
}
#[derive(Default)]
pub struct Scheduler {
    excess_nanos: u64,
}
impl Scheduler {
    /// GameService's 600 ms scheduling policy, including the exact-overrun boundary.
    pub fn delay_millis(&mut self, elapsed_nanos: u64) -> u64 {
        let elapsed = elapsed_nanos as u128 + self.excess_nanos as u128;
        let millis = elapsed / 1_000_000;
        self.excess_nanos = (elapsed % 1_000_000) as u64;
        if millis > 600 {
            (600 - millis % 600) as u64
        } else {
            (600 - millis) as u64
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clock_boundary_and_cycle_order() {
        let mut clock = MapClock(10);
        let mut events = Vec::new();
        run_cycle(&mut clock, |phase, tick| {
            events.push((phase, tick));
            Ok::<_, ()>(())
        })
        .unwrap();
        assert_eq!(events.iter().map(|x| x.0).collect::<Vec<_>>(), PHASES);
        assert_eq!(events[10], (Phase::PlayerLogin, 10));
        assert_eq!(events[11], (Phase::Late, 11));
        assert_eq!(MapClock(i32::MAX).tick(), Err(ClockExhausted));
        assert_eq!(MapClock(i32::MAX - 1).deadline(100), i32::MAX);
    }
    #[test]
    fn overrun_does_not_create_catchup_bursts() {
        let mut s = Scheduler::default();
        assert_eq!(s.delay_millis(600_000_000), 0);
        assert_eq!(s.delay_millis(601_000_000), 599);
        assert_eq!(s.delay_millis(1_200_000_000), 600);
        assert_eq!(s.delay_millis(500_000), 600);
        assert_eq!(s.delay_millis(500_000), 599);
    }
}
