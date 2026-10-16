use crate::sqlite::{Database, Error, Value};
use gielinor_world::{inventory::Item, CoordGrid};
use std::collections::BTreeMap;
const MIGRATIONS: [&str; 12] = [
    include_str!("../migrations/V1__init.sql"),
    include_str!("../migrations/V2__add_run_energy_to_characters.sql"),
    include_str!("../migrations/V3__add_realm_config_columns.sql"),
    include_str!("../migrations/V4__add_xp_rate_columns.sql"),
    include_str!("../migrations/V5__add_realms_broadcast_column.sql"),
    include_str!("../migrations/V6__replace_accounts_mod_group_column.sql"),
    include_str!("../migrations/V7__tutorial_island_spawn.sql"),
    include_str!("../migrations/V8__dev_realm_tutorial_spawn.sql"),
    include_str!("../migrations/V9__tower_arrival.sql"),
    include_str!("../migrations/V10__tower_diary.sql"),
    include_str!("../migrations/V11__tower_bones_offered.sql"),
    include_str!("../migrations/V12__tower_run.sql"),
];
fn int(v: i64) -> Value {
    Value::Integer(v)
}
fn text(v: &str) -> Value {
    Value::Text(v.into())
}
fn optional(v: &Option<String>) -> Value {
    v.as_ref().map_or(Value::Null, |v| text(v))
}
fn integer(v: &Value) -> Result<i64, Error> {
    if let Value::Integer(v) = v {
        Ok(*v)
    } else {
        Err(Error::from("Expected database integer".to_string()))
    }
}
fn string(v: &Value) -> Result<String, Error> {
    if let Value::Text(v) = v {
        Ok(v.clone())
    } else {
        Err(Error::from("Expected database text".to_string()))
    }
}
fn opt_string(v: &Value) -> Result<Option<String>, Error> {
    if *v == Value::Null {
        Ok(None)
    } else {
        string(v).map(Some)
    }
}
fn opt_integer(v: &Value) -> Result<Option<i64>, Error> {
    if *v == Value::Null {
        Ok(None)
    } else {
        integer(v).map(Some)
    }
}
fn i32_value(v: &Value) -> Result<i32, Error> {
    i32::try_from(integer(v)?).map_err(|_| Error::from("Database integer outside i32".to_string()))
}

pub struct Account {
    pub tower_diary: Option<gielinor_world::tower_diary::Progress>,
    pub tower_run: gielinor_world::tower_run::Run,
    pub tower_arrival: gielinor_world::tower_arrival::State,
    pub account_id: i64,
    pub character_id: i64,
    pub realm: i32,
    pub username: String,
    pub display_name: Option<String>,
    pub password_hash: String,
    pub members: bool,
    pub modlevel: Option<String>,
    pub twofa_enabled: bool,
    pub twofa_secret: Option<String>,
    pub twofa_last_verified: Option<String>,
    pub known_device: Option<i64>,
    pub last_login: Option<String>,
    pub last_logout: Option<String>,
    pub world_id: Option<i32>,
    pub muted_until: Option<String>,
    pub banned_until: Option<String>,
    pub coords: CoordGrid,
    pub varps_json: String,
    pub run_energy: i32,
    pub xp_rate_hundreds: i32,
    pub stats: BTreeMap<u8, Stat>,
    pub inventories: BTreeMap<i32, BTreeMap<usize, Item>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginMessages {
    pub welcome: Option<String>,
    pub broadcast: Option<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stat {
    pub visible: i8,
    pub base: i8,
    pub fine_xp: i32,
}
#[derive(Clone, Debug)]
pub struct Save {
    pub tower_diary: Option<gielinor_world::tower_diary::Progress>,
    pub tower_run: gielinor_world::tower_run::Run,
    pub tower_arrival: gielinor_world::tower_arrival::State,
    pub account_id: i64,
    pub character_id: i64,
    pub display_name: Option<String>,
    pub known_device: Option<i64>,
    pub modlevel: Option<String>,
    pub coords: CoordGrid,
    pub persistent_varps_json: String,
    pub last_login: Option<String>,
    pub run_energy: i32,
    pub xp_rate_hundreds: i32,
    pub stats: BTreeMap<u8, Stat>,
    pub inventories: BTreeMap<i32, BTreeMap<usize, Item>>,
}
pub struct Accounts {
    db: Database,
}
impl Accounts {
    /// RealmConfigLoader preserves nullable welcome text and suppresses only
    /// blank broadcasts. Read this on the database owner's startup thread.
    pub fn login_messages(&self, realm: i32) -> Result<LoginMessages, Error> {
        let rows = self.db.query(
            "SELECT login_message,login_broadcast FROM realms WHERE id=?",
            &[int(realm as i64)],
            1,
        )?;
        let row = rows
            .first()
            .ok_or_else(|| Error::from("Unknown login realm".to_owned()))?;
        let broadcast = opt_string(&row[1])?.filter(|value| {
            !value.chars().all(|c| {
                // Kotlin Char.isWhitespace is Java isWhitespace OR isSpaceChar:
                // includes ASCII record separators, excludes Unicode NEXT LINE.
                ('\u{1c}'..='\u{1f}').contains(&c) || (c.is_whitespace() && c != '\u{85}')
            })
        });
        Ok(LoginMessages {
            welcome: opt_string(&row[0])?,
            broadcast,
        })
    }
    pub fn from_database(db: Database) -> Self {
        Self { db }
    }
    /// Initialize only a new, isolated database; never silently take over a Kotlin database.
    pub fn initialize(&mut self) -> Result<(), Error> {
        let own = self.db.query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='_rust_migrations'",
            &[],
            1,
        )?;
        if own.is_empty() {
            let tables = self.db.query(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
                &[],
                1024,
            )?;
            if !tables.is_empty() {
                return Err(Error::from(
                    "Refusing to migrate a database not created by this server".to_string(),
                ));
            }
        }
        self.db.transaction(|db| {
            db.execute(
                "CREATE TABLE IF NOT EXISTS _rust_migrations(version INTEGER PRIMARY KEY)",
                &[],
            )?;
            let applied = db.query(
                "SELECT version FROM _rust_migrations ORDER BY version",
                &[],
                MIGRATIONS.len(),
            )?;
            for (i, row) in applied.iter().enumerate() {
                if integer(&row[0])? != i as i64 + 1 {
                    return Err(Error::from(
                        "Non-contiguous schema migration history".to_string(),
                    ));
                }
            }
            for (index, migration) in MIGRATIONS.iter().enumerate().skip(applied.len()) {
                db.migration_script(migration)?;
                db.execute(
                    "INSERT INTO _rust_migrations VALUES(?)",
                    &[int(index as i64 + 1)],
                )?;
            }
            Ok(())
        })
    }
    pub fn create(
        &mut self,
        username: &str,
        password_hash: &str,
        realm: i32,
    ) -> Result<(i64, i64), Error> {
        let username = username.to_lowercase();
        self.db.transaction(|db|{
            db.execute("INSERT INTO accounts(login_username,password_hash) VALUES(?,?) ON CONFLICT(login_username) DO NOTHING",&[text(&username),text(password_hash)])?;
            let rows=db.query("SELECT id FROM accounts WHERE login_username=?",&[text(&username)],1)?;
            let account=integer(&rows.first().ok_or_else(||Error::from("Account creation failed".to_string()))?[0])?;
            db.execute("INSERT INTO characters(account_id,realm_id) VALUES(?,?)",&[int(account),int(realm as i64)])?;Ok((account,db.last_insert_id()))
        })
    }
    pub fn load(&self, username: &str, realm: i32) -> Result<Option<Account>, Error> {
        let rows=self.db.query("SELECT a.id,c.id,a.login_username,a.display_name,a.password_hash,a.members,a.modlevel,a.twofa_enabled,a.twofa_secret,a.known_device,c.last_login,c.last_logout,c.muted_until,c.banned_until,c.x,c.z,c.level,c.varps,c.run_energy,c.xp_rate_in_hundreds,a.twofa_last_verified,c.world_id FROM accounts a JOIN characters c ON c.account_id=a.id WHERE a.login_username=? AND c.realm_id=?",&[text(&username.to_lowercase()),int(realm as i64)],1)?;
        let Some(r) = rows.first() else {
            return Ok(None);
        };
        let character_id = integer(&r[1])?;
        let mut stats = BTreeMap::new();
        for row in self.db.query(
            "SELECT stat_id,vis_level,base_level,fine_xp FROM stats WHERE character_id=?",
            &[int(character_id)],
            256,
        )? {
            let id = u8::try_from(integer(&row[0])?)
                .map_err(|_| Error::from("Invalid persisted stat id".to_string()))?;
            let fine_xp = i32_value(&row[3])?;
            if !(0..=2_000_000_000).contains(&fine_xp) {
                return Err(Error::from("Invalid persisted fine XP".to_string()));
            }
            stats.insert(
                id,
                Stat {
                    visible: i32_value(&row[1])? as i8,
                    base: i32_value(&row[2])? as i8,
                    fine_xp,
                },
            );
        }
        let mut inventories: BTreeMap<i32, BTreeMap<usize, Item>> = BTreeMap::new();
        for row in self.db.query("SELECT i.inv_type,o.slot,o.obj,o.count,o.vars FROM inventories i JOIN inventory_objs o ON o.inventories_id=i.id WHERE i.character_id=? ORDER BY i.inv_type,o.slot",&[int(character_id)],65536)? {
            let slot=usize::try_from(integer(&row[1])?).map_err(|_|Error::from("Negative inventory slot".to_string()))?;if slot>=65536{return Err(Error::from("Persisted inventory slot exceeds limit".to_string()))}
            inventories.entry(i32_value(&row[0])?).or_default().insert(slot,Item{id:i32_value(&row[2])?,count:i32_value(&row[3])?,vars:i32_value(&row[4])?});
        }
        let tower = self.db.query(
            "SELECT kit_given,staff_taken,diary_progress,bones_offered FROM tower_arrival WHERE character_id=?",
            &[int(character_id)],
            1,
        )?;
        let tower_arrival = if let Some(row) = tower.first() {
            gielinor_world::tower_arrival::State {
                kit_given: integer(&row[0])? != 0,
                staff_taken: integer(&row[1])? != 0,
                diary_progress: integer(&row[2])? != 0,
                bones_offered: match &row[3] {
                    Value::Null => None,
                    Value::Integer(0) => Some(false),
                    Value::Integer(1) => Some(true),
                    _ => return Err(Error::from("Invalid bones-offered flag".to_string())),
                },
            }
        } else {
            Default::default()
        };
        let tower_diary = crate::tower_diary::load(&self.db, character_id)?;
        Ok(Some(Account {
            tower_run: crate::tower_run::load(&self.db, character_id)?,
            tower_diary,
            tower_arrival,
            account_id: integer(&r[0])?,
            character_id,
            realm,
            username: string(&r[2])?,
            display_name: opt_string(&r[3])?,
            password_hash: string(&r[4])?,
            members: integer(&r[5])? != 0,
            modlevel: opt_string(&r[6])?,
            twofa_enabled: integer(&r[7])? != 0,
            twofa_secret: opt_string(&r[8])?,
            twofa_last_verified: opt_string(&r[20])?,
            known_device: opt_integer(&r[9])?,
            last_login: opt_string(&r[10])?,
            last_logout: opt_string(&r[11])?,
            world_id: opt_integer(&r[21])?
                .map(|v| {
                    i32::try_from(v).map_err(|_| Error::from("World id outside i32".to_string()))
                })
                .transpose()?,
            muted_until: opt_string(&r[12])?,
            banned_until: opt_string(&r[13])?,
            coords: CoordGrid::new(i32_value(&r[14])?, i32_value(&r[15])?, i32_value(&r[16])?)
                .map_err(|_| Error::from("Invalid saved coordinate".to_string()))?,
            varps_json: string(&r[17])?,
            run_energy: i32_value(&r[18])?,
            xp_rate_hundreds: i32_value(&r[19])?,
            stats,
            inventories,
        }))
    }
    pub fn save(&mut self, s: &Save) -> Result<(), Error> {
        if s.coords == CoordGrid::NULL || !(0..=10000).contains(&s.run_energy) {
            return Err(Error::from("Invalid character save state".to_string()));
        }
        self.db.transaction(|db|{
            if db.query("SELECT id FROM characters WHERE id=? AND account_id=?",&[int(s.character_id),int(s.account_id)],1)?.is_empty(){return Err(Error::from("Character/account mismatch".to_string()))}
            db.execute("UPDATE characters SET x=?,z=?,level=?,varps=?,last_login=?,run_energy=?,xp_rate_in_hundreds=?,last_logout=CURRENT_TIMESTAMP WHERE id=?",&[int(s.coords.x() as i64),int(s.coords.z() as i64),int(s.coords.level() as i64),text(&s.persistent_varps_json),optional(&s.last_login),int(s.run_energy as i64),int(s.xp_rate_hundreds as i64),int(s.character_id)])?;
            db.execute("UPDATE accounts SET display_name=?,known_device=?,modlevel=? WHERE id=?",&[optional(&s.display_name),s.known_device.map_or(Value::Null,int),optional(&s.modlevel),int(s.account_id)])?;
            db.execute("INSERT INTO tower_arrival(character_id,kit_given,staff_taken,diary_progress,bones_offered) VALUES(?,?,?,?,?) ON CONFLICT(character_id) DO UPDATE SET kit_given=excluded.kit_given,staff_taken=excluded.staff_taken,diary_progress=excluded.diary_progress,bones_offered=COALESCE(excluded.bones_offered,tower_arrival.bones_offered)", &[int(s.character_id),int(s.tower_arrival.kit_given as i64),int(s.tower_arrival.staff_taken as i64),int(s.tower_arrival.diary_progress as i64),s.tower_arrival.bones_offered.map_or(Value::Null,|v|int(v as i64))])?;
            if let Some(progress) = s.tower_diary { crate::tower_diary::save(db, s.character_id, progress)?; }
            crate::tower_run::save(db, s.character_id, s.tower_run)?;
            for (id,stat) in &s.stats {if !(0..=2_000_000_000).contains(&stat.fine_xp){return Err(Error::from("Invalid fine XP".to_string()))}db.execute("INSERT INTO stats(character_id,stat_id,vis_level,base_level,fine_xp) VALUES(?,?,?,?,?) ON CONFLICT(character_id,stat_id) DO UPDATE SET vis_level=excluded.vis_level,base_level=excluded.base_level,fine_xp=excluded.fine_xp,updated_at=CASE WHEN stats.fine_xp!=excluded.fine_xp THEN CURRENT_TIMESTAMP ELSE stats.updated_at END",&[int(s.character_id),int(*id as i64),int(stat.visible as i64),int(stat.base as i64),int(stat.fine_xp as i64)])?;}
            db.execute("DELETE FROM inventories WHERE character_id=?",&[int(s.character_id)])?;
            for (kind,items) in &s.inventories {if items.is_empty(){continue}db.execute("INSERT INTO inventories(character_id,inv_type) VALUES(?,?)",&[int(s.character_id),int(*kind as i64)])?;let inv=db.last_insert_id();for (slot,item) in items {if *slot>=65536||item.id<0||item.count<1{return Err(Error::from("Invalid inventory item".to_string()))}db.execute("INSERT INTO inventory_objs VALUES(?,?,?,?,?)",&[int(inv),int(*slot as i64),int(item.id as i64),int(item.count as i64),int(item.vars as i64)])?;}}
            Ok(())
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn realm_messages_preserve_null_and_empty_welcome_but_filter_kotlin_blank_broadcast() {
        let mut accounts = Accounts::from_database(Database::memory().unwrap());
        accounts.initialize().unwrap();
        assert_eq!(
            accounts.login_messages(1).unwrap(),
            LoginMessages {
                welcome: Some("Welcome to RS Mod.".into()),
                broadcast: None
            }
        );
        for (welcome, broadcast, expected) in [
            (Value::Null, Value::Null, None),
            (text(""), text(" \t\r\n\u{a0}\u{1c}"), None),
            (text("Custom realm"), text("\u{85}"), Some("\u{85}")),
            (
                text("Custom realm"),
                text(" Realm event "),
                Some(" Realm event "),
            ),
        ] {
            accounts
                .db
                .execute(
                    "UPDATE realms SET login_message=?,login_broadcast=? WHERE id=1",
                    &[welcome.clone(), broadcast],
                )
                .unwrap();
            let messages = accounts.login_messages(1).unwrap();
            assert_eq!(messages.welcome, opt_string(&welcome).unwrap());
            assert_eq!(messages.broadcast.as_deref(), expected);
        }
        assert!(accounts.login_messages(999).is_err());
    }
    #[test]
    fn wiki_shop_buy_and_sell_survive_database_close_and_reopen() {
        use gielinor_world::{
            inventory::{Catalog, Inventory, StackType},
            shop::{Profile, Quantity, Shop, Stock},
        };
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gielinor-wiki-shop-{}-{stamp}.sqlite",
            std::process::id()
        ));
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap(),
        );
        let mut accounts = Accounts::from_database(Database::open(&path, false).unwrap());
        accounts.initialize().unwrap();
        let (account_id, character_id) =
            accounts.create("WikiShop", "fixture-hash-only", 1).unwrap();
        let mut save = Save {
            tower_run: Default::default(),
            tower_diary: None,
            tower_arrival: Default::default(),
            account_id,
            character_id,
            display_name: Some("Wiki Shop".into()),
            known_device: None,
            modlevel: None,
            coords: CoordGrid::new(3200, 3200, 0).unwrap(),
            persistent_varps_json: "{}".into(),
            last_login: None,
            run_energy: 10000,
            xp_rate_hundreds: 100,
            stats: BTreeMap::new(),
            inventories: BTreeMap::new(),
        };
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        let mut shop = Shop::new(Profile {
            currency: 995,
            capacity: 2,
            stock: vec![Stock {
                item: 100,
                count: 2,
                interval: 100,
            }],
            values: [(100, 10)].into(),
            player_pays_percent: 100.0,
            shop_pays_percent: 40.0,
            change_percent: 10.0,
            nonstock_interval: 100,
        })
        .unwrap();
        let mut held = Inventory::new(28, StackType::Normal, false);
        held.restore_slot(
            0,
            Some(Item {
                id: 995,
                count: 100,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            shop.buy(&mut held, &catalog, 0, 100, Quantity::One)
                .unwrap()
                .currency,
            10
        );
        for (round, expected_coins, expected_goods) in [(0, 90, 1), (1, 95, 0)] {
            if round == 1 {
                assert_eq!(
                    shop.sell(&mut held, &catalog, 1, 100, Quantity::One)
                        .unwrap()
                        .currency,
                    5
                );
            }
            save.inventories.insert(
                93,
                held.slots()
                    .iter()
                    .enumerate()
                    .filter_map(|(slot, item)| item.map(|item| (slot, item)))
                    .collect(),
            );
            accounts.save(&save).unwrap();
            drop(accounts);
            accounts = Accounts::from_database(Database::open(&path, false).unwrap());
            let loaded = accounts.load("wikishop", 1).unwrap().unwrap();
            assert_eq!(loaded.inventories, save.inventories);
            held = Inventory::new(28, StackType::Normal, false);
            for (&slot, &item) in &loaded.inventories[&93] {
                held.restore_slot(slot, Some(item)).unwrap();
            }
            let total = |id| {
                held.slots()
                    .iter()
                    .flatten()
                    .filter(|i| i.id == id)
                    .map(|i| i.count)
                    .sum::<i32>()
            };
            assert_eq!((total(995), total(100)), (expected_coins, expected_goods));
            // Loading character state must never replay an exchange.
            assert_eq!(
                accounts.load("wikishop", 1).unwrap().unwrap().inventories,
                loaded.inventories
            );
        }
        // Shop stock stayed in the same world here. This does not choose a shop
        // stock restart/persistence policy or attest native UI/worker delivery.
        assert_eq!(shop.inventory().get(0).unwrap().count, 2);
        drop(accounts);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn wiki_fractional_awards_survive_database_close_and_reopen() {
        use gielinor_world::{skills, stats::Stats};
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "gielinor-wiki-xp-{}-{stamp}.sqlite",
            std::process::id()
        ));
        // Reserve an owned path without ever truncating an existing fixture.
        drop(
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap(),
        );
        let mut accounts = Accounts::from_database(Database::open(&path, false).unwrap());
        accounts.initialize().unwrap();
        let (account_id, character_id) = accounts.create("WikiXp", "fixture-hash-only", 1).unwrap();
        let mut save = Save {
            tower_run: Default::default(),
            tower_diary: None,
            tower_arrival: Default::default(),
            account_id,
            character_id,
            display_name: Some("Wiki XP".into()),
            known_device: None,
            modlevel: None,
            coords: CoordGrid::new(3200, 3200, 0).unwrap(),
            persistent_varps_json: "{}".into(),
            last_login: None,
            run_energy: 10000,
            xp_rate_hundreds: 100,
            stats: BTreeMap::new(),
            inventories: BTreeMap::new(),
        };
        for rate in [100, 500] {
            let mut stats = Stats::default();
            save.xp_rate_hundreds = rate;
            for awards in 1..=3 {
                for (skill, xp, fine) in
                    [(5, 4.5, 45), (13, 6.2, 62), (14, 17.5, 175), (8, 37.5, 375)]
                {
                    skills::advance(&mut stats, skill, xp, rate as f64 / 100.0, 99).unwrap();
                    assert_eq!(stats.fine_xp(skill), fine * (rate / 100) * awards);
                    save.stats.insert(
                        skill,
                        Stat {
                            visible: stats.current(skill),
                            base: stats.base(skill),
                            fine_xp: stats.fine_xp(skill),
                        },
                    );
                }
                accounts.save(&save).unwrap();
                drop(accounts);
                accounts = Accounts::from_database(Database::open(&path, false).unwrap());
                let loaded = accounts.load("wikixp", 1).unwrap().unwrap();
                assert_eq!(loaded.xp_rate_hundreds, rate);
                assert_eq!(loaded.stats, save.stats);
                stats = Stats::default();
                for (&skill, stat) in &loaded.stats {
                    stats.set_current(skill, stat.visible);
                    stats.set_base(skill, stat.base);
                    stats.set_fine_xp(skill, stat.fine_xp).unwrap();
                }
            }
        }
        drop(accounts);
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn schema_and_character_roundtrip_are_atomic() {
        let mut a = Accounts::from_database(Database::memory().unwrap());
        a.initialize().unwrap();
        a.initialize().unwrap();
        let (account_id, character_id) = a.create("Parity_Actor", "fixture-hash-only", 1).unwrap();
        let mut run = gielinor_world::tower_run::Run::default();
        assert!(run.begin());
        for service in [gielinor_world::tower_run::Service::Forge, gielinor_world::tower_run::Service::Kitchen, gielinor_world::tower_run::Service::Altar] {
            assert!(run.service(service));
        }
        assert!(run.defeat_imp());
        assert!(run.defeat_warden());
        let mut s = Save {
            tower_run: run,
            tower_diary: Some(gielinor_world::tower_diary::Progress {
                counts: [5, 4, 2, 1, 0, 15],
                actions: [true, false, true, false, true, false, true],
                done: [
                    true, true, false, false, false, false, false, false, false, false, false,
                ],
                pending: [true, false],
                bounty: 3,
                schema11: true,
                imp_bones: true,
            }),
            tower_arrival: gielinor_world::tower_arrival::State {
                kit_given: true,
                staff_taken: false,
                diary_progress: true,
                bones_offered: Some(true),
            },
            account_id,
            character_id,
            display_name: Some("Parity Actor".into()),
            known_device: None,
            modlevel: None,
            coords: CoordGrid::new(1686, 6115, 0).unwrap(),
            persistent_varps_json: "{\"281\":42}".into(),
            last_login: Some("2026-10-11 12:00:00".into()),
            run_energy: 1234,
            xp_rate_hundreds: 15000,
            stats: BTreeMap::from([(
                0,
                Stat {
                    visible: 50,
                    base: 45,
                    fine_xp: 61512,
                },
            )]),
            inventories: BTreeMap::from([(
                93,
                BTreeMap::from([(
                    4,
                    Item {
                        id: 995,
                        count: 100,
                        vars: 0,
                    },
                )]),
            )]),
        };
        a.save(&s).unwrap();
        let loaded = a.load("PARITY_ACTOR", 1).unwrap().unwrap();
        assert_eq!(loaded.coords, s.coords);
        assert_eq!(loaded.inventories, s.inventories);
        assert_eq!(loaded.stats, s.stats);
        assert_eq!(loaded.tower_arrival, s.tower_arrival);
        assert_eq!(loaded.tower_diary, s.tower_diary);
        assert_eq!(loaded.tower_run, run);
        assert_eq!(loaded.run_energy, 1234);
        assert!(loaded.last_logout.is_some());
        let known_diary = s.tower_diary;
        s.tower_diary = None;
        s.tower_arrival.bones_offered = None;
        a.save(&s).unwrap();
        assert_eq!(
            a.load("parity_actor", 1).unwrap().unwrap().tower_diary,
            known_diary
        );
        assert_eq!(
            a.load("parity_actor", 1)
                .unwrap()
                .unwrap()
                .tower_arrival
                .bones_offered,
            Some(true)
        );
        s.tower_diary = known_diary;
        s.tower_arrival.bones_offered = Some(false);
        s.run_energy = 999;
        s.tower_arrival.kit_given = false;
        s.tower_diary.as_mut().unwrap().counts[0] = 1;
        assert!(s.tower_run.paid(run.id));
        s.inventories.get_mut(&93).unwrap().insert(
            1,
            Item {
                id: 1,
                count: 0,
                vars: 0,
            },
        );
        assert!(a.save(&s).is_err());
        // A failed inventory save must roll back the reward payment marker too.
        assert_eq!(a.load("parity_actor", 1).unwrap().unwrap().tower_run, run);
        assert_eq!(
            a.load("parity_actor", 1)
                .unwrap()
                .unwrap()
                .tower_arrival
                .bones_offered,
            Some(true)
        );
        assert_eq!(a.load("parity_actor", 1).unwrap().unwrap().run_energy, 1234);
        assert_eq!(
            a.load("parity_actor", 1)
                .unwrap()
                .unwrap()
                .tower_diary
                .unwrap()
                .counts[0],
            5
        );
        assert!(
            a.load("parity_actor", 1)
                .unwrap()
                .unwrap()
                .tower_arrival
                .kit_given
        );
    }

    #[test]
    fn owned_version_eight_upgrades_without_resetting_character_state() {
        let mut a = Accounts::from_database(Database::memory().unwrap());
        a.initialize().unwrap();
        let (_, id) = a.create("Before_Tower", "fixture-hash-only", 1).unwrap();
        a.db.execute(
            "UPDATE characters SET x=3224,z=3218,level=0,run_energy=4321 WHERE id=?",
            &[int(id)],
        )
        .unwrap();
        // Reconstruct the preceding owned schema entirely in this memory database.
        a.db.execute("DROP TABLE tower_run", &[]).unwrap();
        a.db.execute("DROP TABLE tower_diary", &[]).unwrap();
        a.db.execute("DROP TABLE tower_arrival", &[]).unwrap();
        a.db.execute("DELETE FROM _rust_migrations WHERE version>=9", &[])
            .unwrap();
        a.initialize().unwrap();
        let loaded = a.load("before_tower", 1).unwrap().unwrap();
        assert_eq!(loaded.coords, CoordGrid::new(3224, 3218, 0).unwrap());
        assert_eq!(loaded.run_energy, 4321);
        assert_eq!(loaded.tower_arrival, Default::default());
        assert_eq!(loaded.tower_diary, None);
        a.initialize().unwrap();
    }
}
