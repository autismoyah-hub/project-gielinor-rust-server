use crate::sqlite::{Database, Error, Value};
use gielinor_world::{inventory::Item, CoordGrid};
use std::collections::BTreeMap;
const MIGRATIONS: [&str; 8] = [
    include_str!("../migrations/V1__init.sql"),
    include_str!("../migrations/V2__add_run_energy_to_characters.sql"),
    include_str!("../migrations/V3__add_realm_config_columns.sql"),
    include_str!("../migrations/V4__add_xp_rate_columns.sql"),
    include_str!("../migrations/V5__add_realms_broadcast_column.sql"),
    include_str!("../migrations/V6__replace_accounts_mod_group_column.sql"),
    include_str!("../migrations/V7__tutorial_island_spawn.sql"),
    include_str!("../migrations/V8__dev_realm_tutorial_spawn.sql"),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stat {
    pub visible: i8,
    pub base: i8,
    pub fine_xp: i32,
}
#[derive(Clone, Debug)]
pub struct Save {
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
                8,
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
        Ok(Some(Account {
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
    fn schema_and_character_roundtrip_are_atomic() {
        let mut a = Accounts::from_database(Database::memory().unwrap());
        a.initialize().unwrap();
        a.initialize().unwrap();
        let (account_id, character_id) = a.create("Parity_Actor", "fixture-hash-only", 1).unwrap();
        let mut s = Save {
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
        assert_eq!(loaded.run_energy, 1234);
        assert!(loaded.last_logout.is_some());
        s.run_energy = 999;
        s.inventories.get_mut(&93).unwrap().insert(
            1,
            Item {
                id: 1,
                count: 0,
                vars: 0,
            },
        );
        assert!(a.save(&s).is_err());
        assert_eq!(a.load("parity_actor", 1).unwrap().unwrap().run_energy, 1234);
    }
}
