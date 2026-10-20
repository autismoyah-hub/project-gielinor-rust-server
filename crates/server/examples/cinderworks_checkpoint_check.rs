//! Real bounded-worker/SQLite acceptance on a NEW disposable directory only.
//! No client, credentials, progression earnings or production database involved.
use gielinor_server::{
    account_worker::{Completion, Request, Settings, Worker},
    authentication,
};
use gielinor_storage::{
    accounts::{Accounts, Save},
    sqlite::Database,
};
use gielinor_world::{inventory::Item, tower_run::Service};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::mpsc::TryRecvError,
    time::{Duration, Instant},
};

fn read(path: &Path) -> Result<gielinor_storage::accounts::Account, Box<dyn std::error::Error>> {
    let a = Accounts::from_database(Database::open(path, true).map_err(|e| e.message)?);
    Ok(a.load("worker_fixture", 1)
        .map_err(|e| e.message)?
        .ok_or("Missing fixture character")?)
}
fn acknowledged(worker: &Worker, token: u64) -> Result<(), Box<dyn std::error::Error>> {
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        match worker.poll() {
            Ok(Completion::Saved {
                token: received,
                result,
            }) => {
                if received != token {
                    return Err("Worker token mismatch".into());
                }
                result?;
                return Ok(());
            }
            Ok(_) => return Err("Unexpected authentication completion".into()),
            Err(TryRecvError::Disconnected) => return Err("Worker disconnected".into()),
            Err(TryRecvError::Empty) => {}
        }
        if Instant::now() >= until {
            return Err("Worker acknowledgment timeout".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: cinderworks_checkpoint_check NEW_OUTPUT_DIRECTORY ARGON2_DLL".into());
    }
    let directory = Path::new(&args[1]);
    std::fs::create_dir(directory)?;
    let database = directory.join("accounts.sqlite");
    {
        let mut a =
            Accounts::from_database(Database::open(&database, false).map_err(|e| e.message)?);
        a.initialize().map_err(|e| e.message)?;
        a.create("worker_fixture", "not-an-authentication-fixture", 1)
            .map_err(|e| e.message)?;
    }
    let c = read(&database)?;
    assert!(c.last_logout.is_none());
    let mut run = c.tower_run;
    assert!(run.begin());
    assert!(run.service(Service::Forge));
    let snapshot = Save {
        account_id: c.account_id,
        character_id: c.character_id,
        tower_diary: c.tower_diary,
        chemistry: c.chemistry,
        tower_run: run,
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
        inventories: BTreeMap::from([(
            93,
            BTreeMap::from([(
                0,
                Item {
                    id: 995,
                    count: 10,
                    vars: 0,
                },
            )]),
        )]),
    };
    let worker = Worker::start(Settings {
        database: database.clone(),
        argon2_library: args[2].clone().into(),
        authentication: authentication::Config {
            realm: 1,
            require_registration: false,
            dev_mode: false,
            ignore_passwords: false,
        },
        queue_capacity: 2,
    })?;
    for token in [91, 92] {
        worker
            .submit(Request::Checkpoint {
                token,
                data: snapshot.clone(),
            })
            .map_err(|_| "Checkpoint rejected")?;
        acknowledged(&worker, token)?;
        let c = read(&database)?;
        assert!(c.last_logout.is_none());
        assert_eq!(c.tower_run, snapshot.tower_run);
        assert_eq!(c.inventories, snapshot.inventories);
    }
    worker
        .submit(Request::Save {
            token: 93,
            data: snapshot.clone(),
        })
        .map_err(|_| "Logout save rejected")?;
    acknowledged(&worker, 93)?;
    let c = read(&database)?;
    assert!(c.last_logout.is_some());
    assert_eq!(c.tower_run, snapshot.tower_run);
    assert_eq!(c.inventories, snapshot.inventories);
    assert!(worker.shutdown()?.is_empty());
    // A complete close/reopen preserves the same acknowledged content.
    assert_eq!(read(&database)?.tower_run, snapshot.tower_run);
    let receipt="{\"status\":\"PASS\",\"checkpoint_requests\":2,\"logout_requests\":1,\"checkpoint_preserved_last_logout\":true,\"ordinary_logout_stamped\":true,\"run\":1,\"coins\":10,\"duplicate_snapshot_conserved_inventory\":true,\"scope\":\"Real account worker and SQLite transaction; synthetic disposable state, no earned gameplay\"}\n";
    std::fs::write(directory.join("receipt.json"), receipt)?;
    print!("{receipt}");
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Checkpoint diagnostic: {error}");
        std::process::exit(1);
    }
}
