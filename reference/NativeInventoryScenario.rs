//! Native interface dispatch with authoritative inventory/equipment confirmation.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection};
use std::time::{Duration, Instant};
fn select_tab(client: &mut ClientState, tab: u32) -> Result<(), Box<dyn std::error::Error>> {
    if client.interfaces.root != Some(548) {
        return Err("scenario requires fixed gameframe".into());
    }
    let layout = client.interfaces.layout(client.canvas)?;
    let id = (548 << 16) | (63 + tab);
    let p = layout
        .widgets
        .get(&id)
        .filter(|p| !p.hidden && p.geometry.size.iter().all(|v| *v > 0))
        .ok_or("native side-tab button not visible")?;
    client.input.pointer = [
        p.absolute[0] + p.geometry.size[0] / 2,
        p.absolute[1] + p.geometry.size[1] / 2,
    ];
    client.input.button(0, true);
    client.input.button(0, false);
    client.tick_interfaces(false)?;
    client.input.begin_cycle();
    println!(
        "Native pointer selected side tab {tab} at {:?}",
        client.input.pointer
    );
    Ok(())
}
fn dispatch(
    client: &mut ClientState,
    logical: u32,
    child: i32,
    operation: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = client
        .interfaces
        .runtime
        .iter()
        .find(|(_, runtime)| runtime.logical_id == logical && runtime.child_index == child)
        .map(|(&id, _)| id)
        .ok_or_else(|| {
            if let Some(widget)=client.interfaces.widgets.get(&logical) {
                println!("Inventory widget definition: {widget:?}");
                println!("Inventory widget runtime: {:?}",client.interfaces.runtime.get(&logical));
                println!("Inventory widget layout: {:?}",client.interfaces.layout(client.canvas).ok().and_then(|layout|layout.widgets.get(&logical).cloned()));
                println!("Attached inventory parent: {:?}",client.interfaces.children.iter().filter(|(_,sub)|sub.group==149).collect::<Vec<_>>());
            }
            let found=client.interfaces.runtime.iter().filter(|(_,r)|r.logical_id>>16==logical>>16)
                .map(|(id,r)|format!("{id}: logical={}, child={}, item={}",r.logical_id,r.child_index,r.item)).take(35).collect::<Vec<_>>();
            format!("native widget instance absent: requested {logical}/{child}; group widgets {found:?}")
        })?;
    let before = client.outbound.len();
    if !client.interface_operation(id, operation, None)? {
        return Err("native interface dispatcher rejected action; no protocol fallback".into());
    }
    let sent = client
        .outbound
        .iter()
        .skip(before)
        .map(|packet| packet.opcode)
        .collect::<Vec<_>>();
    println!("Native interface operation {operation}: component={logical}, child={child}, outbound={sent:?}");
    if !sent.contains(&47) {
        return Err("native interface operation did not enqueue IF_BUTTONX (47)".into());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 7 {
        return Err(
            "expected CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY transact|verify|bury".into(),
        );
    }
    let verify = match args[6].as_str() {
        "transact" => false,
        "verify" => true,
        "bury" => false,
        _ => return Err("invalid inventory scenario".into()),
    };
    let bury = args[6] == "bury";
    let mut client = ClientState::load(DiskCache::open(&args[1], 64 << 20)?)?;
    let mut live = Connection::connect(&mut client, &args[2], &args[3], &args[4], &args[5])?;
    let start = Instant::now();
    let mut interface_tick = Instant::now();
    let mut stage = 0;
    let mut selected_at = Instant::now();
    let mut progress_at = Instant::now();
    let mut action_started: Option<Instant> = None;
    // Kotlin's bounded PoW/login can consume most of a 45-second total window.
    // Keep a finite wall-clock limit and expose readiness rather than treating
    // authentication/world-load delay as an inventory transaction failure.
    // Reference runs consumed 81 and 105 seconds before actions were ready.
    // Bound startup plus actions to150s, and the action sequence itself to30s.
    // This keeps the native driver below the Rust server's300-tick lifetime.
    while start.elapsed() < Duration::from_secs(150)
        && action_started.is_none_or(|at| at.elapsed() < Duration::from_secs(30))
    {
        live.poll(&mut client)?;
        if progress_at.elapsed() >= Duration::from_secs(10) {
            println!("Scenario readiness: elapsed={}s stage={stage} world={} scripts_busy={} queued_scripts={} tick={} position={:?} inventory_sword={} worn_sword={}",
                start.elapsed().as_secs(),live.world.is_some(),live.scripts_busy(),client.interfaces.scripts.len(),client.server_tick,client.data.position,
                client.inventories.total(93,1277),client.inventories.total(94,1277));
            progress_at = Instant::now();
        }
        if interface_tick.elapsed() >= Duration::from_millis(20) {
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
            interface_tick = Instant::now();
        }
        if live.world.is_none() || start.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        let sword = client.inventories.item(93, 0).id == 1277;
        let worn = client.inventories.item(94, 3).id == 1277;
        let visible_weapon = client
            .player_state
            .as_ref()
            .and_then(|state| state.players.get(state.local as usize))
            .and_then(Option::as_ref)
            .and_then(|player| player.appearance.as_ref())
            .map(|update| update.appearance.equipment[3]);
        // Inventory and equipment arrive in distinct packets. Do not reject an
        // intermediate TCP fragment between the two authoritative updates.
        let conserved = client.inventories.total(93, 995) == 1000
            && client.inventories.total(93, 379) == 2
            && client.inventories.total(93, 1277) + client.inventories.total(94, 1277) == 1;
        // Appearance and containers may span different reads of the same tick.
        // An inventory-only confirmation does not prove the visible equipment.
        let appearance_matches = visible_weapon.is_some_and(|value| (value == 1277 + 2048) == worn);
        if !conserved || !appearance_matches || client.server_tick == 0 {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        if verify {
            if !worn || sword {
                return Err("native login did not restore equipped sword".into());
            }
            println!(
                "Native login restored worn sword and its appearance, 1000 coins and two lobsters; position {:?}",
                client.data.position
            );
            live.session.close();
            return Ok(());
        }
        if live.scripts_busy() || !client.interfaces.scripts.is_empty() {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        if action_started.is_none() {
            println!(
                "Inventory actions ready after {} seconds",
                start.elapsed().as_secs()
            );
            action_started = Some(Instant::now());
        }
        if bury {
            match stage {
                0 => {
                    if client.data.experience[5] != 0
                        || client.inventories.total(93, 526) != 1
                        || client.inventories.total(93, 530) != 1
                    {
                        return Err("burial scenario requires the explicit zero-XP, one-regular/one-bat-bone fixture".into());
                    }
                    select_tab(&mut client, 3)?;
                    selected_at = Instant::now();
                    stage = 1;
                }
                1 | 3 if selected_at.elapsed() > Duration::from_millis(500) => {
                    let bone = if stage == 1 { 526 } else { 530 };
                    let slot = (0..28)
                        .find(|&slot| client.inventories.item(93, slot).id == bone)
                        .ok_or("bone disappeared before native Bury")?;
                    dispatch(&mut client, 149 << 16, slot as i32, 2)?;
                    println!(
                        "Native Bury item {bone}, Prayer XP before={}",
                        client.data.experience[5]
                    );
                    stage += 1;
                }
                2 if client.inventories.total(93, 526) == 0 && client.data.experience[5] == 22 => {
                    println!("Regular bone consumed; native Prayer XP=22 (server fine XP checked after logout)");
                    selected_at = Instant::now();
                    stage = 3;
                }
                4 if client.inventories.total(93, 530) == 0 && client.data.experience[5] == 49 => {
                    if !client
                        .game_messages
                        .iter()
                        .any(|m| m.text == "You bury the bones.")
                    {
                        return Err("burial message missing".into());
                    }
                    println!(
                        "Both native burials confirmed; Prayer XP=49, coins/food/sword conserved"
                    );
                    live.session.close();
                    return Ok(());
                }
                _ => (),
            }
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        match stage {
            0 => {
                if !sword || worn {
                    return Err("transact requires the offline unequipped fixture".into());
                }
                select_tab(&mut client, 3)?;
                selected_at = Instant::now();
                stage = 1;
            }
            1 if selected_at.elapsed() > Duration::from_millis(500) && !live.scripts_busy() => {
                dispatch(&mut client, 149 << 16, 0, 3)?;
                stage = 2;
            }
            2 if worn && !sword => {
                println!("Server confirmed equip and appearance: worn slot 3=1277, inventory slot 0 empty");
                select_tab(&mut client, 4)?;
                selected_at = Instant::now();
                stage = 3;
            }
            3 if selected_at.elapsed() > Duration::from_millis(500) && !live.scripts_busy() => {
                dispatch(&mut client, (387 << 16) | 18, -1, 1)?;
                stage = 4;
            }
            4 if sword && !worn => {
                println!("Server confirmed unequip and item conservation");
                select_tab(&mut client, 3)?;
                selected_at = Instant::now();
                stage = 5;
            }
            5 if selected_at.elapsed() > Duration::from_millis(500) && !live.scripts_busy() => {
                dispatch(&mut client, 149 << 16, 0, 3)?;
                stage = 6;
            }
            6 if worn && !sword => {
                println!("Server confirmed final equipped state; next login must restore it");
                live.session.close();
                return Ok(());
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    live.session.close();
    Err(format!("inventory scenario timed out at stage {stage}").into())
}
