//! Real bounded-worker/SQLite acceptance on a NEW disposable directory only.
//! Synthetic local credentials only; no client gameplay or production database.
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
// Identity RSA is confined to an in-memory synthetic protocol fixture; no socket.
fn login(password:&str)->gielinor_protocol::login::Login {
    let mut auth=vec![1];auth.extend([0;24]);auth.push(2);auth.extend([0;4]);auth.push(0);auth.extend(password.as_bytes());auth.push(0);
    let mut body=b"worker_fixture\0".to_vec();body.push(0);body.extend(765u16.to_be_bytes());body.extend(503u16.to_be_bytes());body.extend([0;24]);body.push(0);body.extend([0;5]);
    body.extend([0;18]);body.extend([0;8]);body.extend([0;3]);body.extend([0;4]);body.extend([0;18]);body.extend([0;4]);body.extend([0;5]);body.extend([0;92]);
    gielinor_protocol::xtea::encrypt(&mut body,[0;4]);
    let mut payload=vec![0,0,0,239,0,0,0,1,0,0,0,0,1,0,0];payload.extend((auth.len() as u16).to_be_bytes());payload.extend(auth);payload.extend(body);
    let key=gielinor_protocol::rsa::PrivateKey::from_be_bytes(&[1],&[255;256]).unwrap();
    gielinor_protocol::login::decode(&payload,&key).expect("Valid synthetic decoded authentication fixture")
}
fn authenticate(worker:&Worker,connection:u64,password:&str,expected:Option<u8>)->Result<(),Box<dyn std::error::Error>> {
    worker.submit(Request::Authenticate {connection,login:login(password),unix_seconds:1792483200,local_datetime:"2026-10-20T08:00:00".into()}).map_err(|_|"Authentication queue rejected")?;
    let until=Instant::now()+Duration::from_secs(15);
    loop {
        match worker.poll() {
            Ok(Completion::Authenticated {connection:received,result})=>{
                assert_eq!(received,connection);
                return match (expected,result) {
                    (None,Ok(auth))=>{assert!(!auth.new_account);Ok(())},
                    (Some(expected),Err(actual)) if expected==actual=>Ok(()),
                    (_,Err(actual))=>Err(format!("Unexpected authentication rejection {actual}").into()),
                    (_,Ok(_))=>Err("Unexpected authentication admission".into()),
                };
            }
            Ok(_)=>return Err("Unexpected save completion".into()),
            Err(TryRecvError::Disconnected)=>return Err("Worker disconnected".into()),
            Err(TryRecvError::Empty)=>{}
        }
        if Instant::now()>=until {return Err("Authentication timeout".into());}
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
        let hasher=gielinor_storage::password::PasswordHasher::load(Path::new(&args[2])).map_err(|e|format!("{e:?}"))?;
        let encoded=hasher.hash(b"synthetic-checkpoint-only").map_err(|e|format!("{e:?}"))?;
        a.create("worker_fixture", &encoded, 1)
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
        last_login: Some("2026-10-20T08:00:00".into()),
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
    // A bare never-saved registration remains incomplete, even with valid password.
    authenticate(&worker,1,"synthetic-checkpoint-only",Some(13))?;
    authenticate(&worker,2,"wrong-synthetic-password",Some(3))?;
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
    // Simulate a worker/server crash after the durable first checkpoint, before
    // any logout. Reopen the actual SQLite/Argon2 owner and authenticate normally.
    assert!(worker.shutdown()?.is_empty());
    let worker=Worker::start(Settings {database:database.clone(),argon2_library:args[2].clone().into(),authentication:authentication::Config {realm:1,require_registration:false,dev_mode:false,ignore_passwords:false},queue_capacity:2})?;
    authenticate(&worker,3,"synthetic-checkpoint-only",None)?;
    authenticate(&worker,4,"wrong-synthetic-password",Some(3))?;
    assert!(read(&database)?.last_logout.is_none());
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
    authenticate(&worker,5,"synthetic-checkpoint-only",None)?;
    assert!(worker.shutdown()?.is_empty());
    // A complete close/reopen preserves the same acknowledged content.
    assert_eq!(read(&database)?.tower_run, snapshot.tower_run);
    let receipt="{\"status\":\"PASS\",\"checkpoint_auth_recovery\":true,\"incomplete_registration_rejected\":true,\"wrong_password_rejected\":true,\"checkpoint_requests\":2,\"logout_requests\":1,\"checkpoint_preserved_last_logout\":true,\"ordinary_logout_stamped\":true,\"run\":1,\"coins\":10,\"duplicate_snapshot_conserved_inventory\":true,\"scope\":\"Real account worker and SQLite transaction; synthetic disposable state, no earned gameplay\"}\n";
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
