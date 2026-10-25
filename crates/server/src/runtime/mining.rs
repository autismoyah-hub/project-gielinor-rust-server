use super::*;
impl<B: AccountBackend> Runtime<B> {
    /// Registered ordinary/staged rocks with standard and selected uncharged special picks.
    /// Charged/quest-bound picks, guild modifiers, gems and specialised activities are separate.
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
                    members: false,
                    fast_one_in: None,
                    level,
                    animation: names.short("seq", &format!("human_mining_{metal}_pickaxe"))?,
                    interval,
                },
            );
        }
        for (id,pick) in crate::content::special_pickaxes(names.group("obj"),names.group("seq"),names.group("stat"),&self.definitions).map_err(io::Error::other)? {
            if picks.insert(id,pick).is_some() {return Err(io::Error::other("Conflicting special pickaxe"));}
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
                    members: false,
                    restore: None,
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
        // OpenRune rollGemPreTable-eligible rocks: ordinary, limestone, lead and clay.
        let mut pre_roll_rocks = std::collections::BTreeSet::new();
        for (id, rock) in crate::content::ordinary_mining_rocks(
            names.group("obj"),
            names.group("loc"),
            names.group("stat"),
            &self.definitions,
        )
        .map_err(io::Error::other)?
        {
            pre_roll_rocks.insert(id);
            if config.rocks.insert(id, rock).is_some() {
                return Err(io::Error::other("Conflicting ordinary mining rock"));
            }
        }
        for (id, rock) in crate::content::limestone_mining(
            names.group("obj"),
            names.group("loc"),
            names.group("stat"),
            &self.definitions,
        )
        .map_err(io::Error::other)?
        {
            pre_roll_rocks.insert(id);
            if config.rocks.insert(id, rock).is_some() {
                return Err(io::Error::other("Conflicting limestone mining stage"));
            }
        }
        for (id, rock) in crate::content::lead_mining(
            names.group("obj"),
            names.group("loc"),
            names.group("stat"),
            &self.definitions,
        )
        .map_err(io::Error::other)?
        {
            pre_roll_rocks.insert(id);
            if config.rocks.insert(id, rock).is_some() {
                return Err(io::Error::other("Conflicting lead mining target"));
            }
        }
        // Clay is pre-roll eligible (OpenRune, Wiki Clay rocks); the infinite essence rock is not.
        let (clay_essence, essence) = crate::content::clay_essence_mining::clay_essence_mining(
            names.group("obj"),
            names.group("loc"),
            &self.definitions,
        )
        .map_err(io::Error::other)?;
        pre_roll_rocks.extend(crate::content::clay_essence_mining::pre_roll_rocks(&clay_essence));
        for (id, rock) in clay_essence {
            if config.rocks.insert(id, rock).is_some() {
                return Err(io::Error::other("Conflicting clay or essence rock"));
            }
        }
        let (gem_rocks, gems) = crate::content::mining_gems(
            names.group("obj"),
            names.group("loc"),
            &self.definitions,
            pre_roll_rocks,
        )
        .map_err(io::Error::other)?;
        for (id, rock) in gem_rocks {
            if config.rocks.insert(id, rock).is_some() {
                return Err(io::Error::other("Conflicting gem rock"));
            }
        }
        config.validate().map_err(io::Error::other)?;
        gems.validate(&config.rocks).map_err(io::Error::other)?;
        self.world.mining_gems = Some(gems);
        essence.validate(&config.rocks).map_err(io::Error::other)?;
        self.world.mining_essence = Some(essence);
        let outfit = crate::content::prospector_outfit(
            names.group("obj"),
            names.group("stat"),
            &self.definitions,
        )
        .map_err(io::Error::other)?;
        if self.world.gathering_outfits.insert(14, outfit).is_some() {
            return Err(io::Error::other("Conflicting Mining outfit modifier"));
        }
        self.world.mining = Some(config);
        Ok(())
    }
}
