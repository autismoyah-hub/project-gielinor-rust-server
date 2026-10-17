use super::*;
use crate::skill_multi::{Dialog, OptionItem};
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_potion_upgrades_names(
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
        let mut recipes = crate::content::stamina_upgrades(names.group("obj"), &self.definitions)
            .map_err(io::Error::other)?;
        let animation =
            u16::try_from(names.id("seq", "human_herbing_vial")?).map_err(io::Error::other)?;
        let ordinary=self.world.finished_potions.as_ref().ok_or_else(||io::Error::other("Configure ordinary chemistry first"))?;
        let amulet=ordinary.chemistry.values().next().ok_or_else(||io::Error::other("Missing validated ordinary amulet"))?.amulet;
        let extended=crate::content::extended_venom_upgrades(names.group("obj"),&self.definitions).map_err(io::Error::other)?;
        let divine=crate::content::divine_upgrades(names.group("obj"),&self.definitions).map_err(io::Error::other)?;
        let antivenom=crate::content::antivenom_upgrades(names.group("obj"),&self.definitions).map_err(io::Error::other)?;
        let antifire=crate::content::extended_antifire_upgrades(names.group("obj"),&self.definitions).map_err(io::Error::other)?;
        let chemistry=recipes.windows(2).chain(extended.windows(2)).chain(antivenom.windows(2)).chain(antifire.windows(2)).chain(divine.chunks_exact(4).flat_map(|family|family.windows(2))).map(|pair|(pair[0].output,gielinor_world::chemistry_modifier::Binding{amulet,extra_output:pair[1].output})).collect();
        recipes.extend(extended);
        recipes.extend(divine);
        recipes.extend(antivenom);
        recipes.extend(antifire);
        self.world.potion_upgrades = Some(gielinor_world::potion_upgrades::Config {
            chemistry,
            goggles:crate::content::finished_potion_goggles(names.group("obj"), &self.definitions).map_err(io::Error::other)?,
            recipes: recipes.into_iter().map(|r| ((r.base, r.secondary), r)).collect(),
            quest_progress: quest.quest_progress,
            quest_complete: quest.quest_complete,
            animation,
        });
        Ok(())
    }
    pub(super) fn open_potion_upgrade(
        &mut self,
        actor: ActorId,
        request: gielinor_world::potion_upgrades::Request,
    ) -> io::Result<()> {
        let recipe = self
            .world
            .potion_upgrades
            .as_ref()
            .and_then(|c| c.recipes.get(&(request.base, request.secondary)))
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
            .potion_upgrade = Some(request);
        Ok(())
    }
}
