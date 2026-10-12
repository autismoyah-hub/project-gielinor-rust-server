//! Native constructed-floor movement and banker opening. No teleport or packet injection.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, world_action,
};
use std::time::{Duration, Instant};
fn verify_scene(
    scene: &gielinor_client::scene::Scene,
    expected: &str,
    output: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Zone deltas apply only to the observed plane. Upper-floor cached objects
    // are deliberately excluded until a player visits that plane.
    let mut actual = scene
        .objects
        .iter()
        .filter(|o| o.plane == 0)
        .map(|o| {
            [
                scene.base_x + o.x as i32 - 10368,
                scene.base_y + o.y as i32 - 64,
                0,
                o.id as i32,
                o.shape as i32,
                o.rotation as i32,
            ]
        })
        .collect::<Vec<_>>();
    actual.sort();
    let mut reference = std::fs::read_to_string(expected)?
        .lines()
        .map(|line| {
            line.split('\t')
                .map(str::parse::<i32>)
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|row| row.len() == 6 && row[2] == 0)
        .map(|row| <[i32; 6]>::try_from(row).unwrap())
        .collect::<Vec<_>>();
    reference.sort();
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    for row in &actual {
        writeln!(
            file,
            "{}",
            row.iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join("\t")
        )?;
    }
    if actual != reference {
        return Err(format!(
            "native plane-0 location mismatch: {} actual versus {} Kotlin; inspect recorded scene",
            actual.len(),
            reference.len()
        )
        .into());
    }
    println!(
        "Native plane-0 location state matches all {} Kotlin startup locations",
        actual.len()
    );
    Ok(())
}
fn bank_item(client: &mut ClientState, deposit: bool) -> Result<bool, Box<dyn std::error::Error>> {
    let components = if deposit {
        vec![(15 << 16) | 3]
    } else {
        vec![(12 << 16) | 13, (12 << 16) | 12]
    };
    let id = client
        .interfaces
        .runtime
        .iter()
        .find(|(_, r)| components.contains(&r.logical_id) && r.child_index >= 0 && r.item == 995)
        .map(|(&id, _)| id);
    // Container packets precede the interface transmit scripts that draw items.
    // Keep polling/ticking until the native widget exists; never inject a packet.
    let Some(id) = id else {
        return Ok(false);
    };
    // Explicit Deposit-1 / Withdraw-1; independent of selected default quantity.
    let operation = if deposit { 3 } else { 2 };
    if !client.interface_operation(id, operation, None)? {
        return Err("native bank dispatcher rejected action; no protocol fallback".into());
    }
    println!(
        "Native bank {}-1 coin",
        if deposit { "deposit" } else { "withdraw" }
    );
    Ok(true)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 8 && args.len() != 9 {
        return Err("expected CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY KOTLIN_LOCATION_TSV NEW_SCENE_TSV".into());
    }
    let scenario = match args.get(8).map(String::as_str) {
        None | Some("bank") => "bank",
        Some("vials") => "vials",
        Some("altar") => "altar",
        _ => return Err("unknown floor scenario".into()),
    };
    let mut client = ClientState::load(DiskCache::open(&args[1], 64 << 20)?)?;
    let mut live = Connection::connect(&mut client, &args[2], &args[3], &args[4], &args[5])?;
    let start = Instant::now();
    let mut last_tick = Instant::now();
    let mut progress = Instant::now();
    let waypoints = [[10416, 120], [10408, 120], [10416, 120], [10424, 120]];
    let mut next = 0;
    let mut sent = false;
    let mut banker = false;
    let mut bank_stage = 0;
    let mut scene_verified = false;
    let mut vial_before = 0;
    let mut herb_xp_before = 0;
    let mut prayer_xp_before = 0;
    let mut altar_sent = false;
    while start.elapsed() < Duration::from_secs(120) {
        live.poll(&mut client)?;
        if progress.elapsed() > Duration::from_secs(10) {
            println!(
                "Floor readiness: {}s world={} busy={} queued={} waypoint={next} bank_stage={bank_stage} coins={}/{} position={:?}",
                start.elapsed().as_secs(),
                live.world.is_some(),
                live.scripts_busy(),
                client.interfaces.scripts.len(),
                client.inventories.total(93,995),
                client.inventories.total(95,995),
                client.data.position
            );
            progress = Instant::now();
        }
        if last_tick.elapsed() >= Duration::from_millis(20) {
            if !live.scripts_busy() && client.interfaces.scripts.is_empty() {
                if let Err(error) = client.tick_interfaces(false) {
                    live.report(error.to_string());
                }
            }
            while let Some(error) = client.script_failures.pop_front() {
                live.report(error.to_string());
            }
            client.cycle = client.cycle.wrapping_add(1);
            client.input.begin_cycle();
            last_tick = Instant::now();
        }
        if live.world.is_some()
            && !live.scripts_busy()
            && client.interfaces.scripts.is_empty()
            && client.server_tick > 0
        {
            let p = client.data.position;
            if p[0] != 0 || !(10368..10496).contains(&p[1]) || !(64..192).contains(&p[2]) {
                return Err(format!("floor fixture is outside constructed region: {p:?}").into());
            }
            if !scene_verified {
                verify_scene(&live.world.as_ref().unwrap().0, &args[6], &args[7])?;
                scene_verified = true;
            }
            if scenario == "altar" {
                if !altar_sent {
                    if client.data.boosted[5] != 0 || client.data.base[5] <= 0 {
                        return Err(
                            "altar scenario requires an explicitly drained Prayer fixture".into(),
                        );
                    }
                    prayer_xp_before = client.data.experience[5];
                    client.game_messages.clear();
                    client
                        .outbound
                        .push_back(world_action::object_first(409, 10428, 116, 0, false)?);
                    println!(
                        "Native Pray-at altar; Prayer before={}, base={}",
                        client.data.boosted[5], client.data.base[5]
                    );
                    altar_sent = true;
                } else if client
                    .game_messages
                    .iter()
                    .any(|message| message.text == "You recharge your Prayer points.")
                    && client.data.boosted[5] == client.data.base[5]
                {
                    if client.data.experience[5] != prayer_xp_before {
                        return Err("altar unexpectedly changed Prayer XP".into());
                    }
                    println!(
                        "Server confirmed restored Prayer {}, unchanged XP and exact altar message",
                        client.data.boosted[5]
                    );
                    live.session.close();
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            if bank_stage >= 4 {
                let current = client.inventories.total(93, 227);
                match bank_stage {
                    4 | 6 => {
                        client
                            .outbound
                            .push_back(world_action::object_first(57691, 10421, 122, 0, false)?);
                        println!("Native Search vial crate; held before request={current}");
                        bank_stage += 1;
                    }
                    5 if current == vial_before + 5 => {
                        println!("Server confirmed five water-filled vials");
                        bank_stage = 6;
                    }
                    7 if current == vial_before + 10 => {
                        if client.data.experience[15] != herb_xp_before {
                            return Err("vial search unexpectedly changed Herblore XP".into());
                        }
                        println!("Server confirmed repeated vial Search granted ten vials total without Herblore XP");
                        live.session.close();
                        return Ok(());
                    }
                    _ => {}
                }
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }
            if next < waypoints.len() {
                let target = waypoints[next];
                if !sent {
                    client.outbound.push_back(world_action::walk(
                        target[0] as u16,
                        target[1] as u16,
                        0,
                    )?);
                    println!("Native floor walk {:?} -> {target:?}", [p[1], p[2]]);
                    sent = true;
                } else if [p[1], p[2]] == target {
                    println!("Server confirmed floor waypoint {next}: {target:?}");
                    next += 1;
                    sent = false;
                }
            } else if !banker {
                let target = client
                    .npc_state
                    .npcs
                    .iter()
                    .filter(|(_, npc)| {
                        client.npcs.get(&(npc.id as u32)).is_some_and(|definition| {
                            definition.operations.iter().any(|op| {
                                op.index == 2
                                    && op.sub.is_none()
                                    && op.text.eq_ignore_ascii_case("Bank")
                            })
                        })
                    })
                    .min_by_key(|(_, npc)| {
                        (npc.position[0] - p[1]).abs() + (npc.position[1] - p[2]).abs()
                    });
                if let Some((&index, npc)) = target {
                    println!("Native Bank operation: NPC {} index {index}", npc.id);
                    client
                        .outbound
                        .push_back(world_action::npc(3, index, 0, false)?);
                    banker = true;
                }
            } else if [12, 15].into_iter().all(|group| {
                client
                    .interfaces
                    .children
                    .values()
                    .any(|sub| sub.group == group)
            }) {
                let coins = client.inventories.total(93, 995);
                let bank_coins = client.inventories.total(95, 995);
                match bank_stage {
                    0 if coins == 999 && bank_coins == 1 => {
                        // Resume an interrupted previous run by restoring its
                        // deposit through the same native withdrawal operation.
                        if bank_item(&mut client, false)? {
                            bank_stage = 3;
                        }
                    }
                    0 => {
                        if coins != 1000 || bank_coins != 0 {
                            return Err("bank scenario needs the recorded 1000-coin fixture".into());
                        }
                        if bank_item(&mut client, true)? {
                            println!("Native constructed-floor movement and bank interface opening confirmed at {:?}",client.data.position);
                            bank_stage = 1;
                        }
                    }
                    1 if coins == 999 && bank_coins == 1 => {
                        if bank_item(&mut client, false)? {
                            println!("Server confirmed one coin deposited");
                            bank_stage = 2;
                        }
                    }
                    2 if coins == 1000 && bank_coins == 0 => {
                        if client.inventories.total(93, 379) != 2
                            || client.inventories.total(94, 1277) != 1
                        {
                            return Err("bank transaction changed unrelated fixture items".into());
                        }
                        println!("Server confirmed withdrawal and conservation of coins, food and worn sword");
                        if scenario == "vials" {
                            if (0..28)
                                .filter(|&slot| client.inventories.item(93, slot).id < 0)
                                .count()
                                < 10
                            {
                                return Err("vial fixture requires ten free slots".into());
                            }
                            vial_before = client.inventories.total(93, 227);
                            herb_xp_before = client.data.experience[15];
                            bank_stage = 4;
                        } else {
                            live.session.close();
                            return Ok(());
                        }
                    }
                    3 if coins == 1000 && bank_coins == 0 => {
                        println!("Previous deposit recovered through native withdrawal; starting a fresh round trip");
                        bank_stage = 0;
                    }
                    _ => {}
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    live.session.close();
    let widgets = client
        .interfaces
        .runtime
        .iter()
        .filter(|(_, r)| r.logical_id >> 16 == 12 && r.item >= 0)
        .map(|(id, r)| {
            format!(
                "id={id} logical={} child={} item={}",
                r.logical_id, r.child_index, r.item
            )
        })
        .take(30)
        .collect::<Vec<_>>();
    println!("Native bank item widgets at timeout: {widgets:?}");
    Err(format!(
        "floor scenario timed out at waypoint {next}, banker={banker}, bank_stage={bank_stage}"
    )
    .into())
}
