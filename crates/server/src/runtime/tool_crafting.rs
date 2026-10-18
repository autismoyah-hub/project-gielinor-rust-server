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
                    label: if r.output_count == 1 { format!("{} (level {})",r.name,r.level) } else { format!("{} x{} (level {})",r.name,r.output_count,r.level) },
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

impl<B: AccountBackend> Runtime<B> {
    pub(super) fn open_assembly_production(
        &mut self,
        actor: ActorId,
        request: gielinor_world::assembly_production::Request,
    ) -> io::Result<()> {
        let presentation = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Shared assembly dialog is not configured"))?
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
        let output = request.recipe.recipe.output;
        let name = &self
            .definitions
            .items
            .get(&(output as u32))
            .ok_or_else(|| io::Error::other("Missing assembly output definition"))?
            .name;
        let dialog = Dialog {
            title: "What would you like to make?".into(),
            options: vec![OptionItem {
                object: output,
                label: format!("{} (level {})", name, request.recipe.recipe.level),
            }],
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
            .assembly_production = Some(request);
        Ok(())
    }
}
