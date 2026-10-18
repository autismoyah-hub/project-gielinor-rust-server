use super::*;
use crate::skill_multi::{Dialog, OptionItem};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_standard_tanners(&mut self, names: &crate::startup::Names) -> io::Result<()> {
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure tanning before admission"));
        }
        let profiles = crate::content::standard_tanners(
            names.group("obj"),
            names.group("npc"),
            &self.definitions,
        )
        .map_err(io::Error::other)?;
        for &kind in profiles.keys() {
            if self.world.npc_handlers.contains_key(&(kind, 3))
                || self.world.tanning.contains_key(&kind)
            {
                return Err(io::Error::other("Conflicting standard tanner"));
            }
        }
        for (kind, recipes) in profiles {
            self.world
                .npc_handlers
                .insert((kind, 3), gielinor_world::npc_interaction::Handler::Tanning);
            self.world.tanning.insert(kind, recipes);
        }
        Ok(())
    }
    pub(super) fn open_tanning(
        &mut self,
        actor: ActorId,
        target: gielinor_world::simulation::NpcId,
    ) -> io::Result<()> {
        let request = match self.world.tanning_request(actor, target) {
            Ok(r) => r,
            Err(error) => {
                self.events
                    .push(Event::World(WorldEvent::ActionRejected { actor, error }));
                return Ok(());
            }
        };
        let presentation = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Missing shared tanning presentation"))?
            .presentation;
        let options = request
            .recipes()
            .map(|r| OptionItem {
                object: r.output,
                label: format!(
                    "{} ({} coins)",
                    self.definitions.items[&(r.output as u32)].name,
                    r.cost
                ),
            })
            .collect();
        self.open_skill_multi(
            actor,
            presentation,
            Dialog {
                title: "What would you like to tan?".into(),
                options,
                verb: 22,
                max_count: 28,
                selected_count: 1,
            },
        )?;
        self.active
            .get_mut(&actor)
            .unwrap()
            .skill_multi
            .as_mut()
            .unwrap()
            .tanning = Some(request);
        Ok(())
    }
}
