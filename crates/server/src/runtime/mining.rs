use super::*;
impl<B: AccountBackend> Runtime<B> {
    /// Ordinary copper/tin with Wiki standard metal pickaxe intervals.
    /// Special picks, guild modifiers, gems and specialised rocks are excluded.
    pub fn configure_mining(&mut self, names: &crate::startup::Names) -> io::Result<()> {
        use gielinor_world::mining::{Config, Pickaxe, Rock};
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure mining before admission"));
        }
        let mut picks = BTreeMap::new();
        for (metal, level, interval) in [
            ("bronze", 1, 8),
            ("iron", 1, 7),
            ("steel", 6, 6),
            ("black", 11, 5),
            ("mithril", 21, 5),
            ("adamant", 31, 4),
            ("rune", 41, 3),
        ] {
            picks.insert(
                names.id("obj", &format!("{metal}_pickaxe"))? as i32,
                Pickaxe {
                    level,
                    animation: names.short("seq", &format!("human_mining_{metal}_pickaxe"))?,
                    interval,
                },
            );
        }
        let mut config = Config {
            skill: u8::try_from(names.id("stat", "mining")?).map_err(io::Error::other)?,
            picks,
            rocks: Default::default(),
            full_sound: names.short("synth", "pillory_wrong")?,
        };
        let depleted = names.id("loc", "rocks1")?;
        let copper = names.id("loc", "copperrock1")?;
        let tin = names.id("loc", "tinrock1")?;
        let copper_ore = names.id("obj", "copper_ore")? as i32;
        let tin_ore = names.id("obj", "tin_ore")? as i32;
        for (id, product) in [(copper, copper_ore), (tin, tin_ore)] {
            let loc = self
                .definitions
                .locations
                .get(&id)
                .ok_or_else(|| io::Error::other("Missing mining rock definition"))?;
            let item = self
                .definitions
                .items
                .get(&(product as u32))
                .ok_or_else(|| io::Error::other("Missing ore definition"))?;
            if !loc.ops[0]
                .as_deref()
                .is_some_and(|op| op.eq_ignore_ascii_case("Mine"))
                || !self.definitions.locations.contains_key(&depleted)
                || item.cert.template != 0
                || item.stackable
            {
                return Err(io::Error::other("Unattested mining rock or ore definition"));
            }
            config.rocks.insert(
                id,
                Rock {
                    level: 1,
                    product,
                    product_name: item.name.clone(),
                    xp: 17.5,
                    depleted,
                    respawn: 4,
                    rates: (100, 350),
                },
            );
        }
        for &id in config.picks.keys() {
            if self
                .definitions
                .items
                .get(&(id as u32))
                .is_none_or(|item| item.cert.template != 0)
            {
                return Err(io::Error::other(
                    "Missing or noted mining pickaxe definition",
                ));
            }
        }
        config.validate().map_err(io::Error::other)?;
        self.world.mining = Some(config);
        Ok(())
    }
}
