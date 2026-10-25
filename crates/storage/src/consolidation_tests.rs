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
    a.db.execute("DROP TABLE tower_attempt_transitions", &[]).unwrap();
    a.db.execute("DROP TABLE tower_attempts", &[]).unwrap();
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
    a.db.execute("DROP TABLE tower_attempt_transitions", &[]).unwrap();
    a.db.execute("DROP TABLE tower_attempts", &[]).unwrap();
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
