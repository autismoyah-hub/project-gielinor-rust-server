use super::*;
use crate::skill_multi::{Dialog, OptionItem};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_combat_mixing_names(
        &mut self,
        names: &crate::startup::Names,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure finished potions before admission",
            ));
        }
        if names.id("stat", "herblore")? != 15 {
            return Err(io::Error::other("Unexpected target Herblore stat binding"));
        }
        let quest =
            self.world.herb_cleaning.as_ref().ok_or_else(|| {
                io::Error::other("Configure validated Herblore quest binding first")
            })?;
        let recipes = crate::content::combat_mixing_recipes(names.group("obj"), &self.definitions)
            .map_err(io::Error::other)?;
        let animation =
            u16::try_from(names.id("seq", "human_herbing_vial")?).map_err(io::Error::other)?;
        self.world.combat_mixing = Some(gielinor_world::combat_mixing::Config {
            goggles:crate::content::finished_potion_goggles(names.group("obj"), &self.definitions).map_err(io::Error::other)?,
            recipes,
            quest_progress: quest.quest_progress,
            quest_complete: quest.quest_complete,
            animation,
        });
        Ok(())
    }
    pub(super) fn open_combat_mixing(
        &mut self,
        actor: ActorId,
        request: gielinor_world::combat_mixing::Request,
    ) -> io::Result<()> {
        let recipe = self
            .world
            .combat_mixing
            .as_ref()
            .and_then(|c| c.recipes.get(request.recipe_index))
            .ok_or_else(|| io::Error::other("Finished potion recipe missing"))?;
        let label = self
            .definitions
            .items
            .get(&(recipe.output as u32))
            .ok_or_else(|| io::Error::other("Finished potion output missing"))?
            .name
            .clone();
        let presentation = self
            .smelting
            .as_ref()
            .ok_or_else(|| io::Error::other("Shared skill dialog missing"))?
            .presentation;
        let dialog = Dialog {
            title: "What would you like to make?".into(),
            options: vec![OptionItem {
                object: recipe.output,
                label,
            }],
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
            .combat_mixing = Some(request);
        Ok(())
    }
}
