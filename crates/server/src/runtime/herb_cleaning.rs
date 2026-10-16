use super::*;

impl<B: AccountBackend> Runtime<B> {
    pub fn configure_herb_cleaning_names(
        &mut self,
        names: &crate::startup::Names,
    ) -> io::Result<()> {
        if !self.active.is_empty() || !self.connections.is_empty() {
            return Err(io::Error::other("Configure herb cleaning before admission"));
        }
        let skill = u8::try_from(names.id("stat", "herblore")?).map_err(io::Error::other)?;
        // Read permission only: no quest state or rewards are written here.
        self.world.herb_cleaning = Some(
            crate::content::herb_cleaning_config(
                names.group("obj"),
                names.group("varp"),
                &self.definitions,
                skill,
            )
            .map_err(io::Error::other)?,
        );
        Ok(())
    }
}
