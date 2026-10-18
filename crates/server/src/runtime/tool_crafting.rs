use super::*;
use crate::skill_multi::{Dialog, OptionItem};
impl<B: AccountBackend> Runtime<B> {
    pub(super) fn open_tool_crafting(
        &mut self,
        actor: ActorId,
        request: gielinor_world::tool_crafting::Request,
    ) -> io::Result<()> {
        let presentation = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Shared crafting dialog is not configured"))?
            .presentation;
        let available = request.available(
            &self
                .world
                .data(actor)
                .map_err(|e| io::Error::other(format!("{e:?}")))?
                .inventory,
        );
        if available == 0 {
            return Ok(());
        }
        let dialog = Dialog {
            title: "What would you like to make?".into(),
            options: request
                .recipes
                .iter()
                .map(|r| OptionItem {
                    object: r.exchange.output,
                    label: format!("{} (level {})", r.name, r.level),
                })
                .collect(),
            verb: 22,
            max_count: available as i32,
            selected_count: 1,
        };
        self.open_skill_multi(actor, presentation, dialog)?;
        self.active
            .get_mut(&actor)
            .unwrap()
            .skill_multi
            .as_mut()
            .unwrap()
            .tool_crafting = Some(request);
        Ok(())
    }
}
