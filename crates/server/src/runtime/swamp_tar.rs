use super::*;
impl<B:AccountBackend> Runtime<B> {
    pub fn configure_swamp_tar_names(&mut self,names:&crate::startup::Names)->io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {return Err(io::Error::other("Configure tar before admission"));}
        if names.id("stat","herblore")?!=15 {return Err(io::Error::other("Unexpected Herblore binding"));}
        let quest=self.world.herb_cleaning.as_ref().ok_or_else(||io::Error::other("Configure validated Herblore quest first"))?;
        let recipes=crate::content::swamp_tar(names.group("obj"),&self.definitions).map_err(io::Error::other)?;
        self.world.swamp_tar=Some(gielinor_world::swamp_tar::Config{
            recipes:recipes.into_iter().map(|r|((r.herb,r.tar),r)).collect(),
            quest_progress:quest.quest_progress,quest_complete:quest.quest_complete,
            animation:u16::try_from(names.id("seq","human_herbing_vial")?).map_err(io::Error::other)?,
        });
        Ok(())
    }
}
