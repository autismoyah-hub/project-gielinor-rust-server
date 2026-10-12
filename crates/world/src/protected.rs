//! Player protected-access predicates use the last processed clock, not input time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub current_clock: i32,
    pub processed_clock: i32,
    pub delay_until: i32,
    pub pending_shutdown: bool,
    pub suspended: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            current_clock: -1,
            processed_clock: -1,
            delay_until: i32::MIN,
            pending_shutdown: false,
            suspended: false,
        }
    }
}
impl State {
    /// PathingEntity.delay uses JVM Int addition, including overflow.
    pub fn delay(&mut self, cycles: i32) {
        self.delay_until = self.current_clock.wrapping_add(cycles);
    }
    pub fn delayed(self) -> bool {
        self.delay_until > self.processed_clock && !self.pending_shutdown
    }
    pub fn busy(self, has_modal: bool) -> bool {
        self.delayed() || has_modal
    }
    pub fn access_protected(self, has_modal: bool) -> bool {
        (self.busy(has_modal) || self.suspended) && !self.pending_shutdown
    }
    pub fn modal_button_protected(self) -> bool {
        self.delayed() || self.suspended
    }
    pub fn can_process_movement(
        has_modal: bool,
        pending_queue: bool,
        interacting: bool,
        moved_previous_cycle: bool,
    ) -> bool {
        (interacting && moved_previous_cycle) || !has_modal || !pending_queue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_on_expiry_tick_cannot_cancel_before_main_process() {
        let mut s = State {
            current_clock: 50,
            processed_clock: 49,
            delay_until: 50,
            suspended: true,
            ..State::default()
        };
        assert!(s.delayed());
        s.processed_clock = 50;
        assert!(!s.delayed());
        assert!(s.access_protected(false));
        s.suspended = false;
        assert!(!s.access_protected(false));
    }
    #[test]
    fn shutdown_bypasses_access_but_not_suspended_modal_button() {
        let s = State {
            delay_until: 100,
            pending_shutdown: true,
            suspended: true,
            ..State::default()
        };
        assert!(!s.delayed());
        assert!(s.busy(true));
        assert!(!s.access_protected(true));
        assert!(s.modal_button_protected());
    }
}
