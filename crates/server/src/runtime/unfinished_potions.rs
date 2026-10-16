use super::*;
use crate::skill_multi::{Dialog, OptionItem};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_unfinished_potions_names(
        &mut self,
        names: &crate::startup::Names,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure unfinished potions before admission",
            ));
        }
        if names.id("stat", "herblore")? != 15 {
            return Err(io::Error::other("Unexpected target Herblore stat binding"));
        }
        let quest =
            self.world.herb_cleaning.as_ref().ok_or_else(|| {
                io::Error::other("Configure validated Herblore quest binding first")
            })?;
        let recipes = crate::content::unfinished_potions(names.group("obj"), &self.definitions)
            .map_err(io::Error::other)?;
        let animation =
            u16::try_from(names.id("seq", "human_herbing_vial")?).map_err(io::Error::other)?;
        self.world.unfinished_potions = Some(gielinor_world::unfinished_potions::Config {
            recipes: recipes.into_iter().map(|r| ((r.herb, r.vessel), r)).collect(),
            quest_progress: quest.quest_progress,
            quest_complete: quest.quest_complete,
            animation,
        });
        Ok(())
    }
    pub(super) fn open_unfinished_potion(
        &mut self,
        actor: ActorId,
        request: gielinor_world::unfinished_potions::Request,
    ) -> io::Result<()> {
        let recipe = self
            .world
            .unfinished_potions
            .as_ref()
            .and_then(|c| c.recipes.get(&(request.herb, request.vessel)))
            .ok_or_else(|| io::Error::other("Unfinished potion recipe missing"))?;
        let label = self
            .definitions
            .items
            .get(&(recipe.output as u32))
            .ok_or_else(|| io::Error::other("Unfinished potion output missing"))?
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
            .unfinished_potion = Some(request);
        Ok(())
    }
}
