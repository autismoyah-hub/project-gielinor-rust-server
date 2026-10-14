use super::*;
impl World {
    pub(super) fn process_smelting(&mut self, slot: usize, location: u32) {
        let Some(furnace) = self.furnaces.get(&location) else {
            return;
        };
        let p = self.players.get(slot).unwrap();
        if furnace.tutorial.is_some_and(|progression| {
            !progression.require_at_least(&p.data.variables, crate::tutorial_steps::Step::MineSmelt)
        }) {
            self.events.push(WorldEvent::LocationMessage {
                actor: p.id,
                message: "This is a furnace for smelting metal. You'll learn how to use it soon.",
            });
            return;
        }
        self.events
            .push(WorldEvent::SmeltingRequested { actor: p.id });
    }
}
