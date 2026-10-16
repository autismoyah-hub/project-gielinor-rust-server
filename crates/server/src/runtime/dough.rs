use super::*;
use crate::skill_multi::{Dialog, OptionItem};

impl<B: AccountBackend> Runtime<B> {
    pub fn configure_dough_names(&mut self, names: &crate::startup::Names) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure ordinary dough before admission",
            ));
        }
        let presentation = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Configure shared skill dialog before dough"))?
            .presentation;
        let mut config = crate::content::dough(
            names.group("obj"),
            &self.definitions,
            u8::try_from(names.id("stat", "cooking")?).map_err(io::Error::other)?,
        )
        .map_err(io::Error::other)?;
        config.tutorial = self.tutorial.as_ref().map(|t| t.progression);
        self.world.dough = Some(config);
        self.dough_presentation = Some(presentation);
        Ok(())
    }
    pub(super) fn open_dough(
        &mut self,
        actor: ActorId,
        request: gielinor_world::dough::Request,
    ) -> io::Result<()> {
        let config = self
            .world
            .dough
            .as_ref()
            .ok_or_else(|| io::Error::other("Dough configuration missing"))?;
        let presentation = self
            .dough_presentation
            .ok_or_else(|| io::Error::other("Dough presentation missing"))?;
        let labels = ["Bread dough", "Pastry dough", "Pizza base"];
        let dialog = Dialog {
            title: "What would you like to make?".into(),
            options: config
                .outputs
                .iter()
                .zip(labels)
                .map(|(output, label)| OptionItem {
                    object: output.id,
                    label: label.into(),
                })
                .collect(),
            // Target original SkillMultiVerb.Make; not OpenRune's string value.
            verb: 22,
            max_count: 28,
            selected_count: 1,
        };
        self.open_skill_multi(actor, presentation, dialog)?;
        self.active
            .get_mut(&actor)
            .unwrap()
            .skill_multi
            .as_mut()
            .unwrap()
            .dough = Some(request);
        Ok(())
    }
}
