use gielinor_cache::game::GameCache;
use gielinor_protocol::rsa::PrivateKey;
use gielinor_server::{
    account_worker, authentication, bootstrap, characters, js5,
    runtime::{self, AccountBackend},
    startup,
    transport::{Limits, Transport},
};
use gielinor_world::CoordGrid;
use std::{
    collections::BTreeMap,
    fs, io,
    net::SocketAddr,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{TryRecvError, TrySendError},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

static STOP: AtomicBool = AtomicBool::new(false);
unsafe extern "system" fn interrupt(signal: u32) -> i32 {
    if signal == 0 || signal == 1 {
        STOP.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}
#[repr(C)]
#[derive(Default)]
struct LocalTime {
    year: u16,
    month: u16,
    weekday: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    millis: u16,
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetLocalTime(time: *mut LocalTime);
    fn SetConsoleCtrlHandler(
        handler: Option<unsafe extern "system" fn(u32) -> i32>,
        add: i32,
    ) -> i32;
}
fn local_time() -> String {
    let mut t = LocalTime::default();
    unsafe {
        GetLocalTime(&mut t);
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        t.year, t.month, t.day, t.hour, t.minute, t.second
    )
}
struct Backend(Option<account_worker::Worker>);
impl AccountBackend for Backend {
    fn submit(
        &self,
        request: account_worker::Request,
    ) -> Result<(), TrySendError<account_worker::Request>> {
        match &self.0 {
            Some(worker) => worker.submit(request),
            None => Err(TrySendError::Disconnected(request)),
        }
    }
    fn poll(&self) -> Result<account_worker::Completion, TryRecvError> {
        match &self.0 {
            Some(worker) => worker.poll(),
            None => Err(TryRecvError::Empty),
        }
    }
}
fn fail(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}
fn seed() -> io::Result<u64> {
    let mut bytes = [0; 8];
    gielinor_storage::password::secure_random(&mut bytes)
        .map_err(|e| fail(format!("Entropy: {e:?}")))?;
    Ok(u64::from_be_bytes(bytes))
}
fn report(events: Vec<runtime::Event>) {
    for event in events {
        match event {
            runtime::Event::World(gielinor_world::simulation::WorldEvent::BankCount(trace)) => {
                println!("Bank count: {}", trace.json());
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::NpcMeleeXp(trace)) => {
                println!("NPC melee XP: {}", trace.json());
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::NpcImpact(trace)) => {
                println!("NPC impact: {}", trace.json());
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::NpcDrop(trace)) => {
                println!("NPC drop: {}", trace.json());
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::AssemblyMade {actor,trace}) => {
                println!("Assembly transition: {}",trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::HerbCleaned {actor,trace}) => {
                println!("Herb cleaning transition: {}",trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::UnfinishedPotionMade {actor,trace}) => {
                println!("Unfinished potion transition: {}", trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::FinishedPotionMade {actor,trace}) => {
                println!("Finished potion transition: {}", trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::DoughMade {actor,trace}) => {
                println!("Dough transition: {}",trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::Grinding {actor,trace}) => {
                println!("Grinding transition: {}",trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::WaterFilled {actor,trace}) => {
                println!("Water transition: {}",trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::Milking {actor,trace}) => {
                println!("Milk transition: {}",trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::FoodTransition { actor, trace }) => {
                println!("Food transition: {}", trace.json(actor));
            }
            runtime::Event::World(gielinor_world::simulation::WorldEvent::LocationRestoration(trace)) => {
                println!("Location restoration: {}", trace.json());
            }
            runtime::Event::SaveFailed { actor, reason } => {
                eprintln!("Save failed for {actor:?}; retry pending: {reason}")
            }
            runtime::Event::SaveCompleted { actor, generation, observed_tick } => println!(
                "Save acknowledgement: {{\"actor_slot\":{},\"actor_generation\":{},\"save_generation\":{},\"observed_tick\":{},\"save_completed\":true}}",
                actor.slot, actor.generation, generation, observed_tick),
            runtime::Event::SynchronizationFailed { actor, reason } => {
                eprintln!("Synchronization failed for {actor:?}: {reason}")
            }
            runtime::Event::PlayerReady { actor, .. } => println!("Player ready: {actor:?}"),
            runtime::Event::World(gielinor_world::simulation::WorldEvent::Woodcutting {
                actor,
                trace,
            }) if trace.result != "waiting" => println!(
                "Gathering: {}",
                trace.json(actor, gielinor_world::gathering::Activity::Woodcutting)
            ),
            runtime::Event::World(gielinor_world::simulation::WorldEvent::Mining {
                actor,
                trace,
            }) if trace.result != "waiting" => println!(
                "Gathering: {}",
                trace.json(actor, gielinor_world::gathering::Activity::Mining)
            ),
            _ => (),
        }
    }
}
pub fn run() -> io::Result<()> {
    let mut values = BTreeMap::new();
    let mut directories = Vec::new();
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--help" {
            println!("Gielinor Rust development server (partial implementation)\n--game-cache PATH --js5-cache PATH --symbols PATH [--symbols OVERLAY]\n--check validates cache/content without a socket or account database.\nTo serve: --rsa-exponent RAW_BE_FILE --rsa-modulus RAW_BE_FILE --argon2 DLL\n[--state-name isolated] [--bind 127.0.0.1:0] [--ticks N] [--spawn X,Z,LEVEL] [--stop-file PATH] [--elemental-source ISOLATED_FIXTURE_DIR]\nCtrl+C or creation of the stop file requests logout and drains pending saves. Port 43594 is forbidden.");
            return Ok(());
        }
        if arg == "--check" {
            check = true;
            continue;
        }
        if ![
            "--game-cache",
            "--js5-cache",
            "--symbols",
            "--rsa-exponent",
            "--rsa-modulus",
            "--argon2",
            "--state-name",
            "--bind",
            "--ticks",
            "--spawn",
            "--stop-file",
            "--elemental-source",
        ]
        .contains(&arg.as_str())
        {
            return Err(fail(format!("Unknown option {arg}")));
        }
        let value = args
            .next()
            .ok_or_else(|| fail(format!("Missing value for {arg}")))?;
        if arg == "--symbols" {
            directories.push(PathBuf::from(value));
        } else if values.insert(arg.clone(), value).is_some() {
            return Err(fail(format!("Duplicate option {arg}")));
        }
    }
    let required = |name: &str| {
        values
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| fail(format!("Required option {name}")))
    };
    if directories.is_empty() {
        return Err(fail("At least one --symbols directory is required"));
    }
    let address: SocketAddr = values
        .get("--bind")
        .map_or("127.0.0.1:0", String::as_str)
        .parse()
        .map_err(io::Error::other)?;
    if !address.ip().is_loopback() || address.port() == 43594 {
        return Err(fail("Only loopback ports other than 43594 are allowed"));
    }
    let ticks: Option<u64> = values
        .get("--ticks")
        .map(|s| s.parse())
        .transpose()
        .map_err(io::Error::other)?;
    let stop_file = values.get("--stop-file").map(PathBuf::from);
    let coords = values
        .get("--spawn")
        .map_or("3222,3218,0", String::as_str)
        .split(',')
        .map(str::parse::<i32>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
    if coords.len() != 3 {
        return Err(fail("Spawn requires X,Z,LEVEL"));
    }
    let spawn = CoordGrid::new(coords[0], coords[1], coords[2])
        .map_err(|e| fail(format!("Spawn: {e:?}")))?;
    let names =
        startup::Names::load(&directories.iter().map(PathBuf::as_path).collect::<Vec<_>>())?;
    let mut cache = GameCache::open(Path::new(required("--game-cache")?), 0xff_ffff)?;
    let definitions = cache.definitions_all()?;
    let mut maps = Vec::new();
    for x in (spawn.mx() - 1).max(0)..=(spawn.mx() + 1).min(255) {
        for z in (spawn.mz() - 1).max(0)..=(spawn.mz() + 1).min(255) {
            if let Some(map) = cache.map_square(((x << 8) | z) as u16)? {
                maps.push(map);
            }
        }
    }
    if !maps
        .iter()
        .any(|m| m.id == ((spawn.mx() << 8) | spawn.mz()) as u16)
    {
        return Err(fail("Spawn map is absent"));
    }
    // Floor 1 copies zones outside the login map window. Resolve all template
    // sources before preparing the world, including the old dungeon square.
    let source_ids = gielinor_world::tower_layout::template()
        .values()
        .map(|zone| ((zone.x / 8) << 8) | (zone.z / 8))
        .collect::<std::collections::BTreeSet<_>>();
    for id in source_ids {
        if !maps.iter().any(|map| map.id == id) {
            maps.push(
                cache
                    .map_square(id)?
                    .ok_or_else(|| fail(format!("Missing tower source map {id}")))?,
            );
        }
    }
    // Template collision must exclude normal-world NPC occupancy. Those actors
    // remain in their source squares; the tower population is created separately.
    let mut source_collision = gielinor_world::collision::CollisionMap::default();
    for map in &maps {
        map.terrain_collision(&mut source_collision);
    }
    let mut source_locations = Vec::new();
    for map in &maps {
        source_locations.extend(map.location_collision(&definitions, &mut source_collision)?);
    }
    let tower_origin = CoordGrid::new(10368, 64, 0).map_err(|e| fail(format!("{e:?}")))?;
    let tower = gielinor_server::tower_map::prepare(
        tower_origin,
        &source_collision,
        &source_locations,
        &definitions,
        names.group("loc"),
    )
    .map_err(fail)?;
    let tower_population =
        gielinor_server::tower_map::npcs(tower_origin, &definitions, names.group("npc"))
            .map_err(fail)?;
    let loaded = bootstrap::build(&definitions, &maps).map_err(fail)?;
    println!(
        "Loaded {} map squares, {} NPCs, {} locations",
        loaded.squares.len(),
        loaded.npcs.len(),
        loaded.locations.len()
    );
    let provider = js5::Worker::start(
        PathBuf::from(required("--js5-cache")?),
        0xff_ffff,
        32 * 1024 * 1024,
    )?;
    let key = if check {
        PrivateKey::from_be_bytes(&[1], &[255, 255])
    } else {
        PrivateKey::from_be_bytes(
            &fs::read(required("--rsa-exponent")?)?,
            &fs::read(required("--rsa-modulus")?)?,
        )
    }
    .map_err(|_| fail("Invalid RSA private key"))?;
    let mut transport = Transport::prepare(key, provider.checksums, Limits::default())?;
    transport.configure_js5(Box::new(provider))?;
    let policy = characters::Policy {
        world: 1,
        inventory: names.id("inv", "inv")?,
        equipment: names.id("inv", "worn")?,
        bank: names.id("inv", "bank")?,
        hitpoints: u8::try_from(names.id("stat", "hitpoints")?).map_err(io::Error::other)?,
        spawn,
        base_xp_rate_hundreds: 100,
        auto_display_names: true,
        dev_modlevel: None,
        modlevels: BTreeMap::new(),
    };
    let mut runtime =
        runtime::Runtime::new(loaded.world, transport, definitions, policy, Backend(None));
    startup::configure(&mut runtime, &mut cache, &names)?;
    if let Some(source) = values.get("--elemental-source") {
        let source = PathBuf::from(source);
        if !source.is_dir() {
            return Err(fail("Elemental source fixture directory does not exist"));
        }
        let requirements =
            gielinor_server::elemental_requirements::load(&source, &source.join("obj.sym"))?;
        for failure in &requirements.failures {
            eprintln!("Elemental source diagnostic: {failure}");
        }
        startup::configure_elemental_spells(&mut runtime, &mut cache, &names, &requirements, 1.0)?;
        println!(
            "Configured {} elemental spells from isolated source fixture {}",
            runtime
                .world
                .elemental_spells
                .as_ref()
                .unwrap()
                .spells
                .len(),
            source.display()
        );
    }
    // No account worker or listening socket exists yet. A failure discards this
    // unpublished runtime; successful publication preserves configured NPC hooks.
    let (world, constructed) = gielinor_server::tower_map::publish(
        std::mem::take(&mut runtime.world),
        tower_origin,
        tower,
        tower_population,
    )
    .map_err(fail)?;
    runtime.world = world;
    runtime.constructed = constructed;
    runtime.world.tower_arrival =
        runtime
            .floor_progression
            .map(|progression| gielinor_world::tower_arrival::Config {
                origin: tower_origin,
                progression,
            });
    runtime.configure_tower_tanner(
        tower_origin,
        names.short("npc", "threshold_tanner")?,
        names.id("obj", "leather")? as i32,
        names.short("obj", "bones")?,
        ["crafting", "smithing", "firemaking"]
            .map(|name| u8::try_from(names.id("stat", name)?).map_err(io::Error::other))
            .into_iter()
            .collect::<io::Result<Vec<_>>>()?
            .try_into()
            .unwrap(),
    )?;
    runtime.configure_tower_chest(names.id("loc", "threshold_chest")?, names.group("obj"))?;
    runtime.configure_tower_staff(
        names.id("loc", "crate_old")?,
        names.id("obj", "staff_of_air")? as i32,
        names.id("obj", "mind_talisman")? as i32,
    )?;
    runtime.configure_tower_restore(
        names.short("timer", "tower_ash")?,
        u8::try_from(names.id("stat", "agility")?).map_err(io::Error::other)?,
        names.group("obj"),
    )?;
    runtime.configure_tower_combat(
        [
            names.short("npc", "ash_warden")?,
            names.short("npc", "ash_imp")?,
        ],
        names.short("npc", "chicken")?,
    )?;
    runtime.configure_woodcutting(&names, None)?;
    runtime.configure_mining(&names)?;
    let offering_xp = names.id("param", "skill_xp")?;
    let offering_default = cache
        .parameters()?
        .get(&offering_xp)
        .and_then(|p| p.default_int);
    runtime.configure_tower_offering(
        names.id("loc", "altar")?,
        names.short("seq", "human_bone_sacrifice")?,
        [
            names.id("obj", "bones")? as i32,
            names.id("obj", "big_bones")? as i32,
        ],
        offering_xp,
        offering_default,
    )?;
    let outside = CoordGrid::new(3225, 3218, 0).map_err(|e| fail(format!("{e:?}")))?;
    let config = gielinor_world::tower_cinderworks::Config {
        origin:tower_origin, outside,
        plaque:names.id("loc", "plaque")?,
        portal:names.id("loc", "threshold_portal")?,
        chest:names.id("loc", "threshold_chest")?,
        tools:["bronze_axe", "bronze_pickaxe", "net", "hammer", "tinderbox", "knife"]
            .into_iter().map(|name| names.id("obj", name).map(|id| id as i32)).collect::<io::Result<Vec<_>>>()?,
        scimitar:names.id("obj", "bronze_scimitar")? as i32,
        dagger:names.id("obj", "bronze_dagger")? as i32,
        coins:names.id("obj", "coins")? as i32,
        beer:names.id("obj", "beer")? as i32,
        imp:names.short("npc", "ash_imp")?,
        warden:names.short("npc", "ash_warden")?,
        quench:names.id("loc", "barrel")?,
        quench_tiles:vec![tower_origin.translate(53,71,0).unwrap(),tower_origin.translate(60,71,0).unwrap()],
        warning_graphic:names.short("spotanim", "smokepuff")?,
        strike_graphic:names.short("spotanim", "zamorak_flame")?,
        death_animation:names.short("seq", "human_death")?,
        work:Some(gielinor_world::tower_cinderworks::WorkConfig {
            fishing_spot:names.short("npc", "0_48_48_newbiefishing")?, net:names.id("obj", "net")? as i32,
            raw:names.id("obj", "raw_shrimp")? as i32,cooked:names.id("obj", "shrimp")? as i32,
            cooking_loc:names.id("loc", "newbierange")?,fish_skill:names.id("stat", "fishing")? as u8,
            cook_skill:names.id("stat", "cooking")? as u8,fish_animation:names.short("seq", "human_smallnet")?,
            cook_animation:names.short("seq", "human_cooking")?,queue:names.id("queue", "smithing_make")? as i32,
        }),
    };
    let portal = gielinor_world::locations::Location::new(outside, config.portal, 10, 0).map_err(fail)?;
    if !runtime.world.npc_death.as_ref().is_some_and(|death| [config.imp,config.warden].iter().all(|id| death.definitions.get(id).is_some_and(|definition| definition.animation_ticks>0))) {
        return Err(fail("Cinderworks guardians require cache-resolved death animations before admission"));
    }
    runtime.world.change_location(portal, true).map_err(fail)?;
    runtime.world.npc_handlers.insert((config.work.unwrap().fishing_spot,1),gielinor_world::npc_interaction::Handler::CinderworksFishing);
    runtime.configure_cinderworks(config)?;
    if check {
        println!(
            "Implemented content configuration validated. Full gameplay parity remains incomplete."
        );
        return Ok(());
    }
    let state = values
        .get("--state-name")
        .map_or("isolated", String::as_str);
    if state.is_empty()
        || state.len() > 64
        || !state
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(fail("Invalid isolated state name"));
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let parent = root.join("work");
    fs::create_dir_all(&parent)?;
    if !parent.canonicalize()?.starts_with(&root) {
        return Err(fail("State parent escapes server workspace"));
    }
    let parent = parent.join("state");
    fs::create_dir_all(&parent)?;
    if !parent.canonicalize()?.starts_with(&root) {
        return Err(fail("State parent escapes server workspace"));
    }
    let state = parent.join(state);
    fs::create_dir_all(&state)?;
    if !state.canonicalize()?.starts_with(&root) {
        return Err(fail("State directory escapes server workspace"));
    }
    let _lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .share_mode(0)
        .open(state.join("server.lock"))?;
    // No account database is opened until every content resolver has succeeded.
    let worker = account_worker::Worker::start(account_worker::Settings {
        database: state.join("accounts.sqlite"),
        argon2_library: PathBuf::from(required("--argon2")?),
        authentication: authentication::Config {
            realm: 1,
            require_registration: false,
            dev_mode: false,
            ignore_passwords: false,
        },
        queue_capacity: 64,
    })
    .map_err(fail)?;
    runtime.configure_realm_messages(worker.login_messages.clone())?;
    runtime.configure_backend(Backend(Some(worker)))?;
    runtime.transport.queue_until_tick()?;
    runtime.transport.listen(address)?;
    if unsafe { SetConsoleCtrlHandler(Some(interrupt), 1) } == 0 {
        return Err(io::Error::last_os_error());
    }
    println!(
        "Listening on {} (revision 239.1; partial gameplay implementation)",
        runtime.transport.address()?
    );
    let mut scheduler = gielinor_world::tick::Scheduler::default();
    let mut next = Instant::now();
    let mut completed = 0u64;
    let mut stopping = false;
    loop {
        if !stopping
            && (STOP.load(Ordering::Relaxed)
                || ticks.is_some_and(|limit| completed >= limit)
                || (Instant::now() >= next && stop_file.as_ref().is_some_and(|path| path.exists())))
        {
            runtime.begin_shutdown();
            stopping = true;
        }
        let unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_secs();
        report(runtime.pump(unix, &local_time(), seed)?);
        if stopping && runtime.is_drained() {
            break;
        }
        let now = Instant::now();
        if now >= next {
            runtime.world.unix_millis = u64::try_from(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(io::Error::other)?
                    .as_millis(),
            )
            .map_err(io::Error::other)?;
            report(runtime.tick()?);
            completed += 1;
            let elapsed = now.elapsed().as_nanos().min(u64::MAX as u128) as u64;
            next = Instant::now() + Duration::from_millis(scheduler.delay_millis(elapsed));
        }
        std::thread::sleep(
            next.saturating_duration_since(Instant::now())
                .min(Duration::from_millis(10)),
        );
    }
    unsafe {
        SetConsoleCtrlHandler(Some(interrupt), 0);
    }
    println!("Shutdown completed; accounts drained.");
    Ok(())
}
