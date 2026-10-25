use super::*;
/// Wiki Firemaking rev 15367339 log table, normal through yew, bound to rev-239
/// symbols. Jatoba logs (level 40) has no rev-239 symbol and is not bound.
/// OpenRune's firemaking_logs rows truncate mahogany/yew XP to 157/202.
const LOGS: [(&str, &str, u8, u32); 9] = [
    ("logs", "Logs", 1, 400),
    ("achey_tree_logs", "Achey tree logs", 1, 400),
    ("oak_logs", "Oak logs", 15, 600),
    ("willow_logs", "Willow logs", 30, 900),
    ("teak_logs", "Teak logs", 35, 1050),
    ("arctic_pine_log", "Arctic pine logs", 42, 1250),
    ("maple_logs", "Maple logs", 45, 1350),
    ("mahogany_logs", "Mahogany logs", 50, 1575),
    ("yew_logs", "Yew logs", 60, 2025),
];
impl<B: AccountBackend> Runtime<B> {
    /// Ordinary firemaking (OpenRune BurnLogEvents): tinderbox on logs in either
    /// direction and "Light" on ground logs. Fires are the ordinary `loc.fire`
    /// that manual fire cooking accepts.
    pub fn configure_firemaking(&mut self, names: &crate::startup::Names) -> io::Result<()> {
        use gielinor_world::firemaking::{Config, Log};
        if !self.connections.is_empty() {
            return Err(io::Error::other("Configure firemaking before admission"));
        }
        let item = |symbol: &str, name: &str| -> io::Result<(i32, &gielinor_cache::definitions::ItemType)> {
            let id = names.id("obj", symbol)?;
            let def = self
                .definitions
                .items
                .get(&id)
                .filter(|d| d.name == name && d.cert.template == 0 && !d.stackable)
                .ok_or_else(|| io::Error::other(format!("Unattested firemaking item {symbol}")))?;
            Ok((id as i32, def))
        };
        let (tinderbox, _) = item("tinderbox", "Tinderbox")?;
        let (ashes, _) = item("ashes", "Ashes")?;
        let mut logs = std::collections::BTreeMap::new();
        for (symbol, name, level, xp_tenths) in LOGS {
            let (id, def) = item(symbol, name)?;
            // The cache's ground "Light" option is what the client sends as OPOBJ4.
            if def.ground_ops[3].as_deref() != Some("Light") {
                return Err(io::Error::other(format!("{name} lacks the ground Light option")));
            }
            logs.insert(
                id,
                Log {
                    level,
                    xp_tenths,
                    members: def.members,
                    name: name.to_lowercase(),
                },
            );
        }
        let fire = names.id("loc", "fire")?;
        if self.definitions.locations.get(&fire).is_none_or(|l| l.name != "Fire") {
            return Err(io::Error::other("Unattested ordinary fire"));
        }
        let skill = u8::try_from(names.id("stat", "firemaking")?).map_err(io::Error::other)?;
        let max_level = self
            .definitions
            .stats
            .get(&u32::from(skill))
            .filter(|s| s.name == "Firemaking")
            .ok_or_else(|| io::Error::other("Missing Firemaking stat"))?
            .max_level;
        let config = Config {
            tinderbox,
            logs,
            fire,
            ashes,
            animation: names.short("seq", "human_createfire")?,
            skill,
            max_level,
            // OpenRune skillSuccess(64, 512): Wiki 65/256 at level 1, 513/256 at 99.
            success: (64, 512),
            // Fire rev 15370980: "60-119 secs randomly" (OpenRune 100..200 ticks).
            fire_ticks: [100, 198],
            // Ordinary ground-item lifetime, as for player drops.
            object_ticks: 200,
        };
        for &log in config.logs.keys() {
            self.world
                .held_use_handlers
                .register(
                    gielinor_world::held_use::Scope::Types,
                    tinderbox,
                    log,
                    gielinor_world::held_use::Handler::Firemaking { log },
                )
                .map_err(|e| io::Error::other(format!("Firemaking registration: {e:?}")))?;
        }
        for id in config.logs.keys().copied().chain([tinderbox]) {
            let content = self.definitions.items[&(id as u32)].content_group;
            self.world
                .held_use_types
                .insert(id, gielinor_world::held_use::Type { id, content });
        }
        self.world.firemaking = Some(config);
        Ok(())
    }
}
