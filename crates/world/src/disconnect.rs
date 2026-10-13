//! Kotlin PlayerLogoutProcess.processClientDisconnect reconnect grace and
//! inclusive combat-prevention boundary. This does not authenticate reconnects.
#[derive(Default, Debug, Clone, Copy)]
pub struct Grace {
    disconnected: bool,
    cycles: u16,
    prevented: u16,
    queued: bool,
}
impl Grace {
    pub fn disconnected(&self) -> bool {
        self.disconnected
    }
    pub fn disconnect(&mut self) {
        // Repeated transport close notifications must not restart the grace.
        self.disconnected = true;
    }
    pub fn reconnect(&mut self) {
        *self = Self::default();
    }
    pub fn tick(&mut self, clock: i32, prevent_until: i32) -> bool {
        if !self.disconnected || self.queued {
            return false;
        }
        if self.cycles < 16 {
            self.cycles += 1;
            return false;
        }
        let bypass = self.prevented >= 100;
        self.prevented += 1;
        if clock <= prevent_until && !bypass {
            return false;
        }
        self.queued = true;
        true
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_sixteen_logout_phases_preserve_actor() {
        let mut g = Grace::default();
        g.disconnect();
        for tick in 0..16 {
            assert!(!g.tick(tick, -1));
        }
        assert!(g.tick(16, -1));
        assert!(!g.tick(17, -1));
    }
    #[test]
    fn repeated_disconnect_does_not_extend_grace() {
        let mut g = Grace::default();
        for tick in 0..16 {
            g.disconnect();
            assert!(!g.tick(tick, -1));
        }
        assert!(g.tick(16, -1));
    }
    #[test]
    fn reconnect_cancels_pending_logout() {
        let mut g = Grace::default();
        g.disconnect();
        for tick in 0..16 {
            assert!(!g.tick(tick, -1));
        }
        g.reconnect();
        assert!(!g.tick(16, -1));
        g.disconnect();
        assert!(!g.tick(17, -1));
    }
    #[test]
    fn combat_boundary_is_inclusive_and_hard_cap_is_one_hundred() {
        let mut g = Grace::default();
        g.disconnect();
        for tick in 0..116 {
            assert!(!g.tick(tick, i32::MAX));
        }
        assert!(g.tick(116, i32::MAX));
        let mut g = Grace::default();
        g.disconnect();
        for tick in 0..17 {
            assert!(!g.tick(tick, 16));
        }
        assert!(g.tick(17, 16));
    }
}
