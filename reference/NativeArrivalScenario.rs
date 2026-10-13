//! Deterministic first Tower arrival from the canonical fresh Lumbridge fixture.
//! Uses the unchanged client's normal walk encoder; no teleport or fixture mutation.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, world_action};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 6 {
        return Err("expected CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY".into());
    }
    let mut client = ClientState::load(DiskCache::open(&args[1], 64 << 20)?)?;
    let mut live = Connection::connect(&mut client, &args[2], &args[3], &args[4], &args[5])?;
    let start = Instant::now();
    let mut sent = false;
    let mut arrived = false;
    let mut settled = None;
    let mut last_position = None;
    let mut interface_tick = Instant::now();
    let mut progress = Instant::now();
    while start.elapsed() < Duration::from_secs(120) {
        let packets_ready = !live.scripts_busy() && client.interfaces.scripts.is_empty();
        live.poll(&mut client)?;
        if progress.elapsed() >= Duration::from_secs(15) {
            println!("Arrival readiness: seconds={} packets={} busy={} queued={} tick={} walk_sent={sent} world={}",start.elapsed().as_secs(),live.packets,live.scripts_busy(),client.interfaces.scripts.len(),client.server_tick,live.world.is_some());
            progress = Instant::now();
        }
        if live.title.is_none() && last_position != Some(client.data.position) {
            println!("Arrival position={:?}", client.data.position);
            last_position = Some(client.data.position);
        }
        if live.world.is_some() && !live.scripts_busy() && client.interfaces.scripts.is_empty() {
            let fixture_ready = client.data.position == [0, 3224, 3218]
                && client.inventories.total(93, 1277) == 1
                && client.inventories.total(93, 995) == 1000
                && client.inventories.total(93, 379) == 2;
            if !sent && fixture_ready {
                client.outbound.push_back(world_action::walk(3222, 3218, 0)?);
                println!("Arrival walk submitted: [3222, 3218]");
                sent = true;
            }
            if sent && client.data.position == [0, 10424, 120] {
                if !arrived {
                    println!("Server confirmed first Tower arrival: [10424, 120]");
                    arrived = true;
                }
                // A received empty container is evidence; an absent container is not.
                let empty = [93, 94].into_iter().all(|id| {
                    client.inventories.containers.get(&id)
                        .is_some_and(|items| items.iter().all(|item| item.id < 0 || item.count == 0))
                });
                if empty {
                    let at = settled.get_or_insert_with(Instant::now);
                    if at.elapsed() >= Duration::from_secs(2) {
                        println!("Server confirmed first-arrival inventory and equipment cleared");
                        println!("Scenario passed; arrival first; final position {:?}", client.data.position);
                        live.session.close();
                        return Ok(());
                    }
                } else {
                    settled = None;
                }
            }
        }
        // Drive native interface cycles, but leave a script-free poll between
        // timer refills so deferred network events cannot starve.
        if packets_ready && !live.scripts_busy() && client.interfaces.scripts.is_empty()
            && interface_tick.elapsed() >= Duration::from_millis(20) {
            if let Err(error) = client.tick_interfaces(false) {live.report(error.to_string());}
            while let Some(error) = client.script_failures.pop_front() {live.report(error.to_string());}
            client.cycle = client.cycle.wrapping_add(1);
            client.input.begin_cycle();
            interface_tick = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    live.session.close();
    Err(format!("arrival deadline: walk_sent={sent}, arrived={arrived}, final_position={:?}, sword={}, coins={}, food={}",client.data.position,client.inventories.total(93,1277),client.inventories.total(93,995),client.inventories.total(93,379)).into())
}
