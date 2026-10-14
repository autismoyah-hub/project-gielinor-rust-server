use super::*;
impl<B: AccountBackend> Runtime<B> {
    pub fn configure_tower_restore(
        &mut self,
        timer: u16,
        agility: u8,
        objects: &gielinor_cache::symbols::Symbols,
    ) -> io::Result<()> {
        use gielinor_world::{tower_chest::name_matches, tower_food, tower_kit, tower_restore};
        if !self.connections.is_empty() {
            return Err(io::Error::other(
                "Configure Tower login restore before admission",
            ));
        }
        let common = self
            .world
            .tower_tanning
            .as_ref()
            .ok_or_else(|| io::Error::other("Tower diary missing"))?;
        let staff = self
            .world
            .tower_staff
            .as_ref()
            .ok_or_else(|| io::Error::other("Tower staff missing"))?;
        let named = |name: &str, certificates: bool| -> std::collections::BTreeSet<i32> {
            self.definitions
                .items
                .iter()
                .filter_map(|(&id, ty)| {
                    ((certificates || ty.cert.template == 0) && name_matches(&ty.name, name))
                        .then_some(id as i32)
                })
                .collect()
        };
        let item = |symbol: &str,
                    name: &str,
                    type_match: bool,
                    count: i32,
                    waiting: &'static str,
                    restored: &'static str|
         -> io::Result<tower_restore::Item> {
            let id = *objects
                .get(symbol)
                .ok_or_else(|| io::Error::other(format!("Missing restore object {symbol}")))?;
            let ty =
                self.definitions.items.get(&(id as u32)).ok_or_else(|| {
                    io::Error::other(format!("Missing restore definition {symbol}"))
                })?;
            let mut owned = named(name, false);
            if type_match {
                if !ty.name.is_ascii() {
                    return Err(io::Error::other(
                        "Unsupported non-ASCII canonical restore name",
                    ));
                }
                owned.extend(named(&ty.name, true));
                owned.insert(id);
            }
            Ok(tower_restore::Item {
                id,
                count,
                owned,
                waiting,
                restored,
            })
        };
        let rewards = tower_restore::Config {
            axe: item(
                "bronze_axe",
                "Bronze axe",
                false,
                1,
                "Your bronze axe is waiting. Drop something.",
                "You still have the bronze axe.",
            )?,
            needle: item(
                "needle",
                "Needle",
                true,
                1,
                "Your needle is waiting. Drop something.",
                "",
            )?,
            thread: item(
                "thread",
                "Thread",
                true,
                4,
                "Your thread is waiting. Drop something.",
                "",
            )?,
            pickaxe: item(
                "bronze_pickaxe",
                "Bronze pickaxe",
                false,
                1,
                "Your bronze pickaxe is waiting. Drop something.",
                "You still have the bronze pickaxe.",
            )?,
            tinderbox: item(
                "tinderbox",
                "Tinderbox",
                false,
                1,
                "Your tinderbox is waiting. Drop something.",
                "You still have the tinderbox.",
            )?,
            scimitar: item(
                "bronze_scimitar",
                "Bronze scimitar",
                false,
                1,
                "Your bronze scimitar is waiting. Drop something.",
                "You still have the bronze scimitar.",
            )?,
            net: item(
                "net",
                "Small fishing net",
                true,
                1,
                "Your small fishing net is waiting. Drop something.",
                "You still have the small fishing net.",
            )?,
            knife: item(
                "knife",
                "Knife",
                false,
                1,
                "Your knife is waiting. Drop something.",
                "You still have the knife.",
            )?,
            hammer: item(
                "hammer",
                "Hammer",
                false,
                1,
                "Your hammer is waiting. Drop something.",
                "You still have the hammer.",
            )?,
        };
        let leather = *objects
            .get("leather")
            .ok_or_else(|| io::Error::other("Missing leather symbol"))?;
        let mut leather_types = named("Leather", false);
        if self
            .definitions
            .items
            .get(&(leather as u32))
            .is_some_and(|ty| ty.cert.template == 0)
        {
            leather_types.insert(leather);
        }
        let mind = *objects
            .get("mindrune")
            .ok_or_else(|| io::Error::other("Missing mind rune symbol"))?;
        let mind_type = self
            .definitions
            .items
            .get(&(mind as u32))
            .ok_or_else(|| io::Error::other("Missing mind rune definition"))?;
        let mut mind_runes = named("Mind rune", false);
        if mind_type.cert.template == 0 {
            mind_runes.insert(mind);
        }
        self.world.tower_restore = Some(tower_restore::HandlerConfig {
            timer,
            agility,
            mind_runes,
            origin: common.origin,
            skills: common.skills,
            bones: common.bones,
            kit: tower_kit::Config {
                named: tower_kit::NAMES.map(|n| named(n, false)),
            },
            leather: leather_types,
            staff: staff.inventory.clone(),
            rewards,
            food: tower_food::Config {
                cooked: self
                    .definitions
                    .items
                    .iter()
                    .filter_map(|(&id, ty)| tower_food::cooked_name(&ty.name).then_some(id as i32))
                    .collect(),
                bank_shrimp: named("Shrimps", true),
                space: tower_food::SPACE_NAMES.map(|n| named(n, false)),
            },
        });
        Ok(())
    }
}
