//! Disposable schemas only. These do not run the paused chemistry gameplay campaign.
use super::*;
use gielinor_world::tower_run::{Reward, Run, Service};

fn snapshot(a: &Accounts, name: &str) -> Save {
    let c = a.load(name, 1).unwrap().unwrap();
    Save {
        account_id: c.account_id,
        character_id: c.character_id,
        tower_diary: c.tower_diary,
        chemistry: c.chemistry,
        tower_run: c.tower_run,
        tower_floors: c.tower_floors,
        tower_arrival: c.tower_arrival,
        display_name: c.display_name,
        known_device: c.known_device,
        modlevel: c.modlevel,
        coords: c.coords,
        persistent_varps_json: c.varps_json,
        last_login: c.last_login,
        run_energy: c.run_energy,
        xp_rate_hundreds: c.xp_rate_hundreds,
        stats: c.stats,
        inventories: c.inventories,
    }
}
fn victory() -> Run {
    let mut run = Run::default();
    assert!(run.begin());
    for service in [Service::Altar, Service::Kitchen, Service::Forge] {
        assert!(run.service(service));
    }
    assert!(run.defeat_imp());
    assert!(run.defeat_warden());
    run
}
fn database() -> Accounts {
    let mut a = Accounts::from_database(Database::memory().unwrap());
    a.initialize().unwrap();
    a.create("consolidation", "fixture-hash-only", 1).unwrap();
    a
}

#[test]
fn fresh_and_existing_forge_v12_keep_chemistry_and_add_empty_tower_run() {
    let mut a = database();
    let mut s = snapshot(&a, "consolidation");
    assert_eq!(s.tower_run, Run::default());
    s.chemistry = gielinor_world::chemistry::State::restore(2, 1).unwrap();
    s.run_energy = 4321;
    s.persistent_varps_json = "{\"281\":42}".into();
    s.inventories = BTreeMap::from([(
        93,
        BTreeMap::from([(
            4,
            Item {
                id: 995,
                count: 71,
                vars: 0,
            },
        )]),
    )]);
    a.save(&s).unwrap();
    a.db.execute("DROP TABLE tower_floor_completion", &[]).unwrap();
    a.db.execute("DROP TABLE tower_run", &[]).unwrap();
    a.db.execute("DELETE FROM _rust_migrations WHERE version>=13", &[])
        .unwrap();
    a.initialize().unwrap();
    a.initialize().unwrap();
    let loaded = snapshot(&a, "consolidation");
    assert_eq!(loaded.chemistry, s.chemistry);
    assert_eq!(loaded.tower_run, Run::default());
    assert_eq!(loaded.inventories, s.inventories);
    assert_eq!(loaded.run_energy, 4321);
    assert_eq!(loaded.persistent_varps_json, s.persistent_varps_json);
    assert_eq!(
        a.db.query("SELECT MAX(version) FROM _rust_migrations", &[], 1)
            .unwrap()[0][0],
        int(14)
    );
}

#[test]
fn experimental_tower_v12_is_refused_without_renumbering_or_mutation() {
    let mut a = database();
    let mut s = snapshot(&a, "consolidation");
    s.tower_run = victory();
    a.save(&s).unwrap();
    // Reconstruct the actual divergent V12 shape using this disposable memory DB.
    a.db.execute("DROP TABLE chemistry_state", &[]).unwrap();
    a.db.execute("DELETE FROM _rust_migrations WHERE version>=13", &[])
        .unwrap();
    let before = a.db.query("SELECT * FROM tower_run", &[], 4).unwrap();
    let error = a.initialize().unwrap_err();
    assert!(error.message.contains("Incompatible V12"));
    assert_eq!(
        a.db.query("SELECT * FROM tower_run", &[], 4).unwrap(),
        before
    );
    assert_eq!(
        a.db.query("SELECT MAX(version) FROM _rust_migrations", &[], 1)
            .unwrap()[0][0],
        int(12)
    );
    assert!(a.db.query("SELECT * FROM chemistry_state", &[], 1).is_err());
}

#[test]
fn claim_and_chemistry_roll_back_together_on_late_database_failure() {
    let mut a = database();
    let mut saved = snapshot(&a, "consolidation");
    saved.tower_run = victory();
    saved.chemistry = gielinor_world::chemistry::State::restore(3, 1).unwrap();
    saved.inventories = BTreeMap::from([(
        93,
        BTreeMap::from([(
            0,
            Item {
                id: 995,
                count: 23,
                vars: 0,
            },
        )]),
    )]);
    a.save(&saved).unwrap();
    let mut claim = saved.clone();
    assert!(claim.tower_run.paid(saved.tower_run.id));
    claim.chemistry.break_amulet();
    claim.run_energy = 9000;
    claim
        .inventories
        .get_mut(&93)
        .unwrap()
        .get_mut(&0)
        .unwrap()
        .count += 500;
    claim.inventories.get_mut(&93).unwrap().insert(
        1,
        Item {
            id: 996,
            count: 1,
            vars: 0,
        },
    );
    a.db.migration_script("CREATE TRIGGER reject_claim BEFORE INSERT ON inventory_objs WHEN NEW.obj=996 BEGIN SELECT RAISE(ABORT,'disposable claim failure'); END;").unwrap();
    assert!(a.save(&claim).is_err());
    let after = snapshot(&a, "consolidation");
    assert_eq!(after.tower_run, saved.tower_run);
    assert_eq!(after.chemistry, saved.chemistry);
    assert_eq!(after.inventories, saved.inventories);
    assert_eq!(after.run_energy, saved.run_energy);
    a.db.execute("DROP TRIGGER reject_claim", &[]).unwrap();
    a.save(&claim).unwrap();
    a.save(&claim).unwrap();
    let mut relog = snapshot(&a, "consolidation");
    assert_eq!(relog.inventories, claim.inventories);
    assert_eq!(relog.tower_run.reward(), None);
    assert!(!relog.tower_run.paid(saved.tower_run.id));
    assert!(relog.tower_run.begin());
    for service in [Service::Forge, Service::Altar, Service::Kitchen] {
        assert!(relog.tower_run.service(service));
    }
    assert!(relog.tower_run.defeat_imp());
    assert!(relog.tower_run.defeat_warden());
    assert_eq!(relog.tower_run.reward(), Some(Reward::Replay));
}

#[test]
fn checkpoint_commits_content_without_creating_or_overwriting_logout_metadata() {
    let mut a=database();
    let mut s=snapshot(&a,"consolidation");
    assert!(a.load("consolidation",1).unwrap().unwrap().last_logout.is_none());
    assert!(s.tower_run.begin());
    a.save_checkpoint(&s).unwrap();
    let c=a.load("consolidation",1).unwrap().unwrap();
    assert!(c.last_logout.is_none());assert_eq!(c.tower_run,s.tower_run);
    a.db.execute("UPDATE characters SET last_logout='2001-02-03 04:05:06' WHERE id=?",&[int(s.character_id)]).unwrap();
    assert!(s.tower_run.service(Service::Forge));
    a.save_checkpoint(&s).unwrap();
    let c=a.load("consolidation",1).unwrap().unwrap();
    assert_eq!(c.last_logout.as_deref(),Some("2001-02-03 04:05:06"));
    assert_eq!(c.tower_run,s.tower_run);
    a.save(&s).unwrap();
    assert_ne!(a.load("consolidation",1).unwrap().unwrap().last_logout.as_deref(),Some("2001-02-03 04:05:06"));
}

#[test]
fn v13_backfill_preserves_pending_rewards_and_replay_unlocks_without_grants() {
    use gielinor_world::tower_progression::FloorId;
    for replay in [false, true] {
        let mut a = database();
        let mut saved = snapshot(&a, "consolidation");
        saved.tower_run = victory();
        if replay {
            assert!(saved.tower_run.paid(saved.tower_run.id));
            assert!(saved.tower_run.begin());
        }
        a.save(&saved).unwrap();
        let original = snapshot(&a, "consolidation");
        a.db.execute("DROP TABLE tower_floor_completion", &[]).unwrap();
        a.db.execute("DELETE FROM _rust_migrations WHERE version=14", &[]).unwrap();
        a.initialize().unwrap();
        a.initialize().unwrap();
        let loaded = snapshot(&a, "consolidation");
        assert!(loaded.tower_floors.completed(FloorId::CINDERWORKS));
        assert_eq!(loaded.tower_run, original.tower_run);
        assert_eq!(loaded.inventories, original.inventories);
        assert_eq!(loaded.stats, original.stats);
        assert_eq!(loaded.run_energy, original.run_energy);
        assert_eq!(loaded.chemistry, original.chemistry);
        assert_eq!(loaded.tower_floors.floors().count(), 1);
    }
}

#[test]
fn permanent_conquest_and_possessions_roll_back_together_and_stale_saves_cannot_revoke() {
    use gielinor_world::tower_progression::FloorId;
    let mut a = database();
    let fresh = snapshot(&a, "consolidation");
    let mut earned = fresh.clone();
    earned.tower_run = victory();
    earned.inventories = BTreeMap::from([(93, BTreeMap::from([(0, Item { id: 995, count: 19, vars: 0 })]))]);
    a.db.migration_script("CREATE TRIGGER reject_floor_inventory BEFORE INSERT ON inventory_objs BEGIN SELECT RAISE(ABORT,'floor rollback'); END;").unwrap();
    assert!(a.save_checkpoint(&earned).is_err());
    let failed = snapshot(&a, "consolidation");
    assert!(!failed.tower_floors.completed(FloorId::CINDERWORKS));
    assert_eq!(failed.tower_run, fresh.tower_run);
    assert_eq!(failed.inventories, fresh.inventories);
    a.db.execute("DROP TRIGGER reject_floor_inventory", &[]).unwrap();
    a.save_checkpoint(&earned).unwrap();
    let first = snapshot(&a, "consolidation");
    assert!(first.tower_floors.completed(FloorId::CINDERWORKS));
    assert_eq!(first.inventories, earned.inventories);
    a.save_checkpoint(&earned).unwrap();
    // Simulate a snapshot missing only the new permanent ledger, not an old inventory.
    earned.tower_run.paid(earned.tower_run.id);
    earned.tower_run.begin();
    assert_eq!(earned.tower_floors.floors().count(), 0);
    a.save_checkpoint(&earned).unwrap();
    let replay = snapshot(&a, "consolidation");
    assert!(replay.tower_floors.completed(FloorId::CINDERWORKS));
    assert_eq!(replay.inventories, first.inventories);
    a.create("separate character", "fixture-hash-only", 1).unwrap();
    assert_eq!(a.load("separate character", 1).unwrap().unwrap().tower_floors.floors().count(), 0);
}

#[test]
fn two_floor_synthetic_contract_survives_actual_database_close_and_reopen() {
    use gielinor_world::tower_progression::{Definition, FloorId, Objectives};
    let path = std::env::temp_dir().join(format!("tower-floors-{}-{}.sqlite", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let mut a = Accounts::from_database(Database::open(&path, false).unwrap());
    a.initialize().unwrap();
    a.create("consolidation", "fixture-hash-only", 1).unwrap();
    let mut saved = snapshot(&a, "consolidation");
    saved.tower_run = victory();
    saved.tower_floors.reconcile_cinderworks(saved.tower_run);
    saved.stats.insert(0, Stat { visible: 7, base: 7, fine_xp: 65001 });
    saved.inventories = BTreeMap::from([
        (93, BTreeMap::from([(0, Item { id: 995, count: 83, vars: 0 })])),
        (94, BTreeMap::from([(3, Item { id: 1205, count: 1, vars: 0 })])),
    ]);
    let second = FloorId::new(2).unwrap();
    let definition = Definition { id: second, prerequisites: &[FloorId::CINDERWORKS], required: 1, optional: 2 };
    let mut objectives = Objectives::new(definition).unwrap();
    objectives.credit(definition, 1);
    saved.tower_floors.complete(definition, &objectives).unwrap();
    a.save_checkpoint(&saved).unwrap();
    drop(a);
    let mut reopened = Accounts::from_database(Database::open(&path, false).unwrap());
    reopened.initialize().unwrap();
    let loaded = snapshot(&reopened, "consolidation");
    assert_eq!(loaded.tower_floors, saved.tower_floors);
    assert_eq!(loaded.tower_run, saved.tower_run);
    assert_eq!(loaded.inventories, saved.inventories);
    assert_eq!(loaded.stats, saved.stats);
    assert!(loaded.tower_floors.completed(second));
    let mut stale_ledger = loaded.clone();
    stale_ledger.tower_floors = Default::default();
    reopened.save_checkpoint(&stale_ledger).unwrap();
    let preserved = snapshot(&reopened, "consolidation");
    assert!(preserved.tower_floors.completed(second));
    assert_eq!(preserved.stats, saved.stats);
    assert_eq!(preserved.inventories, saved.inventories);
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}
