//! Conversion between persisted accounts and authoritative world state.
use crate::authentication::{java_name_hash, Authenticated};
use gielinor_cache::game::Definitions;
use gielinor_protocol::session::Accepted;
use gielinor_storage::{
    accounts::{Save, Stat},
    varps,
};
use gielinor_world::{
    inventory::Inventory, simulation::PlayerData, stats::Stats, variables::Variables, CoordGrid,
};
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Policy {
    pub world: i32,
    pub inventory: u32,
    pub equipment: u32,
    pub bank: u32,
    pub hitpoints: u8,
    pub spawn: CoordGrid,
    pub base_xp_rate_hundreds: i32,
    pub auto_display_names: bool,
    pub dev_modlevel: Option<String>,
    /// Internal mod-level name to protocol staff level and player-moderator flag.
    pub modlevels: BTreeMap<String, (u8, bool)>,
}
pub struct Character {
    pub identity: Accepted,
    pub account_id: i64,
    pub character_id: i64,
    pub known_device: Option<i64>,
    pub modlevel: Option<String>,
    pub last_login: String,
    pub new_account: bool,
}
fn inventory(defs: &Definitions, id: u32) -> Result<Inventory, &'static str> {
    defs.inventories
        .get(&id)
        .map(|ty| ty.create())
        .ok_or("Missing required inventory type")
}
impl Character {
    pub fn load(
        auth: Authenticated,
        defs: &Definitions,
        policy: &Policy,
        now: String,
    ) -> Result<(Self, PlayerData), u8> {
        let a = auth.account;
        if a.world_id.is_some_and(|id| id != policy.world) {
            return Err(5);
        }
        if a.account_id < 1 || a.character_id < 1 {
            return Err(13);
        }
        if policy.inventory == policy.equipment
            || policy.inventory == policy.bank
            || policy.equipment == policy.bank
        {
            return Err(13);
        }
        let mut stats = Stats::default();
        for (&id, stat) in &a.stats {
            stats.set_fine_xp(id, stat.fine_xp).map_err(|_| 13u8)?;
            stats.set_base(id, stat.base);
            stats.set_current(id, stat.visible);
        }
        if auth.new_account {
            let hp = defs.stats.get(&(policy.hitpoints as u32)).ok_or(13u8)?;
            if (stats.base(policy.hitpoints) as u8 as i32) < hp.min_level as i32 {
                let xp = gielinor_world::skills::xp_from_level(hp.min_level).ok_or(13u8)?;
                stats.set_xp(policy.hitpoints, xp).map_err(|_| 13u8)?;
                stats.set_base(policy.hitpoints, hp.min_level as i8);
                stats.set_current(policy.hitpoints, hp.min_level as i8);
            }
        }
        let mut name = a.display_name.unwrap_or_default();
        if policy.auto_display_names && name.trim().is_empty() {
            name = display_name(&a.username);
        }
        let coords = if auth.new_account {
            policy.spawn
        } else {
            a.coords
        };
        let mut data = PlayerData::new(a.account_id as u64, name, coords, stats);
        data.character_id = a.character_id as u64;
        data.tower_arrival = a.tower_arrival;
        data.variables = Variables::restore(varps::decode(&a.varps_json).map_err(|_| 13u8)?);
        data.energy = a.run_energy;
        data.xp_rate_hundreds = if auth.new_account {
            policy.base_xp_rate_hundreds
        } else {
            a.xp_rate_hundreds
        };
        data.inventory = inventory(defs, policy.inventory).map_err(|_| 13u8)?;
        data.equipment = inventory(defs, policy.equipment).map_err(|_| 13u8)?;
        data.bank = inventory(defs, policy.bank).map_err(|_| 13u8)?;
        for (id, slots) in a.inventories {
            let ty = u32::try_from(id).map_err(|_| 13u8)?;
            let target = if ty == policy.inventory {
                &mut data.inventory
            } else if ty == policy.equipment {
                &mut data.equipment
            } else if ty == policy.bank {
                &mut data.bank
            } else {
                data.other_inventories
                    .insert(id, inventory(defs, ty).map_err(|_| 13u8)?);
                data.other_inventories.get_mut(&id).unwrap()
            };
            for (slot, item) in slots {
                target.restore_slot(slot, Some(item)).map_err(|_| 13u8)?;
            }
            target.clear_modified();
        }
        let modlevel = policy.dev_modlevel.clone().or(a.modlevel);
        let (staff_level, player_mod) = modlevel
            .as_ref()
            .and_then(|name| policy.modlevels.get(name))
            .copied()
            .unwrap_or((0, false));
        let identity = Accepted {
            slot: 0,
            staff_level,
            player_mod,
            member: a.members,
            account_hash: ((a.account_id as u64) << 32) | a.realm as i64 as u64,
            user_id: a.character_id as u64,
            user_hash: java_name_hash(&a.username),
            authenticator_code: auth.authenticator_code,
        };
        Ok((
            Self {
                identity,
                account_id: a.account_id,
                character_id: a.character_id,
                known_device: auth
                    .authenticator_code
                    .map(|code| code as i32 as i64)
                    .or(a.known_device),
                modlevel,
                last_login: now,
                new_account: auth.new_account,
            },
            data,
        ))
    }
    pub fn save(
        &self,
        data: &PlayerData,
        defs: &Definitions,
        policy: &Policy,
    ) -> Result<Save, &'static str> {
        if data.account_id != self.account_id as u64
            || data.character_id != self.character_id as u64
        {
            return Err("Character/account mismatch");
        }
        let types = defs
            .varps
            .iter()
            .map(|(&id, ty)| {
                u16::try_from(id)
                    .map(|id| (id, *ty))
                    .map_err(|_| "Varp id outside u16")
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut inventories = BTreeMap::new();
        let primary = [
            (policy.inventory as i32, &data.inventory),
            (policy.equipment as i32, &data.equipment),
            (policy.bank as i32, &data.bank),
        ];
        for (id, inventory) in primary
            .into_iter()
            .chain(data.other_inventories.iter().map(|(&id, inv)| (id, inv)))
        {
            let ty = defs
                .inventories
                .get(&(id as u32))
                .ok_or("Missing saved inventory type")?;
            if ty.scope != 1 {
                continue;
            }
            let slots = inventory
                .slots()
                .iter()
                .enumerate()
                .filter_map(|(slot, item)| item.map(|item| (slot, item)))
                .collect();
            inventories.insert(id, slots);
        }
        let stats = defs
            .stats
            .keys()
            .map(|&id| {
                let id = u8::try_from(id).map_err(|_| "Stat id outside u8")?;
                Ok((
                    id,
                    Stat {
                        fine_xp: data.stats.fine_xp(id),
                        base: data.stats.base(id),
                        visible: data.stats.current(id),
                    },
                ))
            })
            .collect::<Result<_, &'static str>>()?;
        Ok(Save {
            tower_arrival: data.tower_arrival,
            account_id: self.account_id,
            character_id: self.character_id,
            display_name: Some(data.display_name.clone()),
            known_device: self.known_device,
            modlevel: self.modlevel.clone(),
            coords: data.coords,
            persistent_varps_json: varps::encode(&data.variables.persistent(&types)),
            last_login: Some(self.last_login.clone()),
            run_energy: data.energy,
            xp_rate_hundreds: data.xp_rate_hundreds,
            stats,
            inventories,
        })
    }
}
fn display_name(username: &str) -> String {
    username
        .trim()
        .split(' ')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            let first = chars.next().unwrap();
            let title = if first == 'ß' {
                "Ss".into()
            } else if first.is_lowercase() {
                first.to_uppercase().collect::<String>()
            } else {
                first.to_string()
            };
            title + chars.as_str()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_cache::{
        definitions::{InventoryType, StatType},
        variables,
    };
    use gielinor_storage::{accounts::Accounts, sqlite::Database};
    use gielinor_world::inventory::Item;
    fn definitions() -> Definitions {
        Definitions {
            inventories: [93, 94, 95, 99, 100]
                .into_iter()
                .map(|id| {
                    let d =
                        InventoryType::decode(&[2, 0, 28, 200, if id == 100 { 0 } else { 1 }, 0])
                            .unwrap();
                    (id, d)
                })
                .collect(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::from([
                (7, variables::varp(7, &[0]).unwrap()),
                (8, variables::varp(8, &[203, 0, 0]).unwrap()),
            ]),
            stats: BTreeMap::from([(3, StatType::decode(&[4, 10, 0]).unwrap())]),
        }
    }
    #[test]
    fn new_account_minimum_hp_compares_unsigned_stat_bytes() {
        let defs = definitions();
        let policy = Policy {
            world: 1,
            inventory: 93,
            equipment: 94,
            bank: 95,
            hitpoints: 3,
            spawn: CoordGrid::new(3094, 3107, 0).unwrap(),
            base_xp_rate_hundreds: 100,
            auto_display_names: true,
            dev_modlevel: None,
            modlevels: BTreeMap::new(),
        };
        let mut accounts = Accounts::from_database(Database::memory().unwrap());
        accounts.initialize().unwrap();
        accounts.create("fresh hp", "test-hash", 1).unwrap();
        for level in [1u8, 9, 10, 127, 128, 200, 255] {
            let mut account = accounts.load("fresh hp", 1).unwrap().unwrap();
            account.stats.insert(
                3,
                Stat {
                    base: level as i8,
                    visible: 7,
                    fine_xp: 123450,
                },
            );
            let (_, data) = Character::load(
                Authenticated {
                    account,
                    new_account: true,
                    authenticator_code: None,
                },
                &defs,
                &policy,
                "2026-10-12T01:02:03".into(),
            )
            .unwrap();
            if level < 10 {
                assert_eq!(data.stats.base(3), 10);
                assert_eq!(data.stats.current(3), 10);
                assert_eq!(data.stats.fine_xp(3), 11540);
            } else {
                assert_eq!(data.stats.base(3) as u8, level);
                assert_eq!(data.stats.current(3), 7);
                assert_eq!(data.stats.fine_xp(3), 123450);
            }
        }
    }
    #[test]
    fn account_world_save_roundtrip_preserves_permanent_extra_inventories() {
        let defs = definitions();
        let policy = Policy {
            world: 1,
            inventory: 93,
            equipment: 94,
            bank: 95,
            hitpoints: 3,
            spawn: CoordGrid::new(3094, 3107, 0).unwrap(),
            base_xp_rate_hundreds: 500,
            auto_display_names: true,
            dev_modlevel: Some("owner".into()),
            modlevels: BTreeMap::from([("owner".into(), (2, true))]),
        };
        let mut accounts = Accounts::from_database(Database::memory().unwrap());
        accounts.initialize().unwrap();
        accounts.create("test player", "test-hash", 1).unwrap();
        let mut account = accounts.load("test player", 1).unwrap().unwrap();
        account.varps_json = "{\"7\":11,\"8\":12}".into();
        let item = Item {
            id: 995,
            count: 17,
            vars: 23,
        };
        account.inventories.insert(99, BTreeMap::from([(4, item)]));
        account.inventories.insert(100, BTreeMap::from([(0, item)]));
        let (character, mut data) = Character::load(
            Authenticated {
                account,
                new_account: true,
                authenticator_code: Some(u32::MAX),
            },
            &defs,
            &policy,
            "2026-10-12T01:02:03".into(),
        )
        .unwrap();
        assert_eq!(data.display_name, "Test Player");
        assert_eq!(data.coords, policy.spawn);
        assert_eq!(data.stats.base(3), 10);
        assert_eq!(data.xp_rate_hundreds, 500);
        assert_eq!(character.identity.staff_level, 2);
        assert_eq!(character.known_device, Some(-1));
        data.coords = CoordGrid::new(3095, 3107, 0).unwrap();
        let save = character.save(&data, &defs, &policy).unwrap();
        assert_eq!(save.persistent_varps_json, "{\"7\":11}");
        assert_eq!(save.inventories[&99][&4], item);
        assert!(!save.inventories.contains_key(&100));
        accounts.save(&save).unwrap();
        let account = accounts.load("test player", 1).unwrap().unwrap();
        let (_, returning) = Character::load(
            Authenticated {
                account,
                new_account: false,
                authenticator_code: None,
            },
            &defs,
            &policy,
            "2026-10-12T02:00:00".into(),
        )
        .unwrap();
        assert_eq!(returning.coords, data.coords);
        assert_eq!(returning.other_inventories[&99].get(4), Some(item));
        assert_eq!(returning.variables.get(8), 0);
    }
}
