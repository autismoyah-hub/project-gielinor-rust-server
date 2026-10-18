use super::*;
impl<B: AccountBackend> Runtime<B> {
    /// Wiki oak profile with bronze axe. Regular trees are opt-in while GR-10's
    /// conflicting respawn evidence remains unresolved; the caller must bind a
    /// documented fixture policy rather than silently selecting a default.
    pub fn configure_woodcutting(
        &mut self,
        names: &crate::startup::Names,
        regular_respawn: Option<[i32; 2]>,
    ) -> io::Result<()> {
        use gielinor_world::woodcutting::{Axe, Config, Tree};
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure gathering before admission"));
        }
        let axe = names.id("obj", "bronze_axe")? as i32;
        let mut config = Config {
            skill: u8::try_from(names.id("stat", "woodcutting")?).map_err(io::Error::other)?,
            axes: [(
                axe,
                Axe {
                    level: 1,
                    animation: names.short("seq", "human_woodcutting_bronze_axe")?,
                },
            )]
            .into(),
            trees: Default::default(),
            full_sound: names.short("synth", "pillory_wrong")?,
            fall_sound: names.short("synth", "tree_fall_sound")?,
        };
        let oak = names.id("loc", "oaktree")?;
        let oak_stump = names.id("loc", "oaktree_stump")?;
        let oak_logs = names.id("obj", "oak_logs")? as i32;
        // GR-09/GR-10 Wiki policy: 45 active ticks, next successful log,
        // 14 respawn ticks. Local PG's 15-tick respawn is not adopted here.
        config.trees.insert(
            oak,
            Tree {
                level: 15,
                product: oak_logs,
                product_name: "Oak logs".into(),
                xp: 37.5,
                stump: oak_stump,
                deplete_chance: 0,
                chopping_ticks: Some(45),
                respawn: [14, 14],
                rates: [(axe, (32, 100))].into(),
            },
        );
        if let Some(respawn) = regular_respawn {
            config.trees.insert(
                names.id("loc", "tree")?,
                Tree {
                    level: 1,
                    product: names.id("obj", "logs")? as i32,
                    product_name: "Logs".into(),
                    xp: 25.0,
                    stump: names.id("loc", "treestump2")?,
                    deplete_chance: 0,
                    chopping_ticks: None,
                    respawn,
                    rates: [(axe, (64, 200))].into(),
                },
            );
        }
        for (id, tree) in crate::content::timed_trees(
            names.group("obj"),
            names.group("loc"),
            names.group("stat"),
            &self.definitions,
        )
        .map_err(io::Error::other)?
        {
            if config.trees.insert(id, tree).is_some() {
                return Err(io::Error::other("Conflicting ordinary timed tree"));
            }
        }
        let regular_logs = names.id("obj", "logs")? as i32;
        for tool in crate::content::woodcutting_axes(
            names.group("obj"),
            names.group("seq"),
            names.group("stat"),
            &self.definitions,
        )
        .map_err(io::Error::other)?
        {
            if let Some(old) = config.axes.insert(tool.id, tool.axe) {
                if old.level != tool.axe.level || old.animation != tool.axe.animation {
                    return Err(io::Error::other("Conflicting ordinary woodcutting axe"));
                }
            }
            for (&id, tree) in &mut config.trees {
                let rates = if id == oak {
                    tool.oak_rates
                } else if tree.product == regular_logs && tree.level == 1 {
                    tool.regular_rates
                } else if let Some(&rates) = tree.rates.get(&tool.id) {
                    rates
                } else {
                    return Err(io::Error::other("Unattested ordinary axe tree rates"));
                };
                tree.rates.insert(tool.id, rates);
            }
        }
        config.validate().map_err(io::Error::other)?;
        for (&id, profile) in &mut config.trees {
            let tree = self
                .definitions
                .locations
                .get(&id)
                .ok_or_else(|| io::Error::other("Missing gathering tree definition"))?;
            if !tree.ops[0]
                .as_deref()
                .is_some_and(|op| op.eq_ignore_ascii_case("Chop down"))
                || !self.definitions.locations.contains_key(&profile.stump)
            {
                return Err(io::Error::other("Unattested tree operation or stump"));
            }
            let product = self
                .definitions
                .items
                .get(&(profile.product as u32))
                .ok_or_else(|| io::Error::other("Missing gathering product definition"))?;
            let tool = self
                .definitions
                .items
                .get(&(axe as u32))
                .ok_or_else(|| io::Error::other("Missing gathering axe definition"))?;
            if product.cert.template != 0 || tool.cert.template != 0 || product.stackable {
                return Err(io::Error::other(
                    "Gathering profile requires unnoted, nonstackable logs",
                ));
            }
            profile.product_name = product.name.clone();
        }
        self.world.woodcutting = Some(config);
        Ok(())
    }
}
