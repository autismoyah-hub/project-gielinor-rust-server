//! Native prayer-tab/Continue scenario against the unchanged pinned client.
//! A client varbit can be predicted by scripts; logs deliberately call it an
//! observation. Server acceptance needs the paired server/TCP evidence.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    script_runtime::Source,
};
use std::time::{Duration, Instant};
fn point(client: &mut ClientState, component: u32) -> Result<[i32; 2], Box<dyn std::error::Error>> {
    let layout = client.interfaces.layout(client.canvas)?;
    let p = layout
        .widgets
        .get(&component)
        .filter(|p| !p.hidden && p.geometry.size.iter().all(|v| *v > 0))
        .ok_or("native component is not visible")?;
    Ok([
        p.absolute[0] + p.geometry.size[0] / 2,
        p.absolute[1] + p.geometry.size[1] / 2,
    ])
}
fn button(client: &mut ClientState, component: u32) -> Result<(), Box<dyn std::error::Error>> {
    point(client, component)?;
    let before = client.outbound.len();
    if !client.interface_operation(component, 1, None)? {
        return Err("native prayer dispatcher rejected action".into());
    }
    let sent: Vec<_> = client
        .outbound
        .iter()
        .skip(before)
        .map(|p| p.opcode)
        .collect();
    println!("Native prayer operation: component={component}, outbound={sent:?}");
    if !sent.contains(&47) {
        return Err("native prayer operation did not emit IF_BUTTONX47".into());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("expected CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY".into());
    }
    let mut client = ClientState::load(DiskCache::open(&args[1], 64 << 20)?)?;
    let skin = client.scripts.source.varbit(4104)?;
    let mut live = Connection::connect(&mut client, &args[2], &args[3], &args[4], &args[5])?;
    let start = Instant::now();
    let mut tick = Instant::now();
    let mut report = Instant::now();
    let mut selected = Instant::now();
    let mut actions: Option<Instant> = None;
    let mut stage = 0;
    while start.elapsed() < Duration::from_secs(150)
        && actions.is_none_or(|a| a.elapsed() < Duration::from_secs(30))
    {
        live.poll(&mut client)?;
        if tick.elapsed() >= Duration::from_millis(20) {
            if !live.scripts_busy() && client.interfaces.scripts.is_empty() {
                client.tick_interfaces(false)?;
            }
            while let Some(error) = client.script_failures.pop_front() {
                live.report(error.to_string());
            }
            client.cycle = client.cycle.wrapping_add(1);
            client.input.begin_cycle();
            tick = Instant::now();
        }
        let active = skin.get(client.scripts.variables.player(skin.base));
        if report.elapsed() > Duration::from_secs(10) {
            println!("Prayer readiness: elapsed={} stage={stage} world={} scripts_busy={} tick={} active={active}", start.elapsed().as_secs(), live.world.is_some(), live.scripts_busy(), client.server_tick);
            report = Instant::now();
        }
        if live.world.is_none()
            || client.server_tick == 0
            || live.scripts_busy()
            || !client.interfaces.scripts.is_empty()
        {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        if actions.is_none() {
            if client.interfaces.root != Some(548)
                || client.data.base[5] >= 4
                || client.data.boosted[5] == 0
                || active != 0
            {
                return Err("prayer scenario requires fixed frame, Prayer below4 with points, and inactive Thick Skin".into());
            }
            actions = Some(Instant::now());
            println!(
                "Prayer actions ready after {} seconds",
                start.elapsed().as_secs()
            );
        }
        match stage {
            0 => {
                client.input.pointer = point(&mut client, (548 << 16) | 68)?;
                client.input.button(0, true);
                client.input.button(0, false);
                client.tick_interfaces(false)?;
                client.input.begin_cycle();
                println!("Native pointer selected Prayer tab");
                selected = Instant::now();
                stage = 1;
            }
            1 if selected.elapsed() > Duration::from_millis(500) => {
                button(&mut client, (541 << 16) | 9)?;
                stage = 2;
            }
            2 if active == 1 => {
                println!("Client observed Thick Skin active: bit4104=1");
                selected = Instant::now();
                stage = 3;
            }
            3 if selected.elapsed() > Duration::from_millis(500) => {
                button(&mut client, (541 << 16) | 9)?;
                stage = 4;
            }
            4 if active == 0 => {
                println!("Client observed Thick Skin inactive: bit4104=0");
                selected = Instant::now();
                stage = 5;
            }
            5 if selected.elapsed() > Duration::from_millis(500) => {
                button(&mut client, (541 << 16) | 10)?;
                stage = 6;
            }
            6 if client.interfaces.children.values().any(|s| s.group == 229) => {
                let text = &client
                    .interfaces
                    .widgets
                    .get(&((229 << 16) | 3))
                    .ok_or("requirement text missing")?
                    .text;
                if !text.contains("Prayer") || !text.contains("level of 4") {
                    return Err(format!("unexpected prayer requirement: {text}").into());
                }
                println!("Observed server prayer requirement notice: {text}");
                let position = point(&mut client, (229 << 16) | 4)?;
                let entries = client.interface_menu(position)?;
                if client.default_menu_opens(&entries)
                    || client.begin_widget_drag(position, &entries)?
                {
                    return Err("Continue requires unsupported context/drag action".into());
                }
                let before = client.outbound.len();
                client.menu_action(entries.last().ok_or("Continue menu empty")?.action)?;
                let sent: Vec<_> = client
                    .outbound
                    .iter()
                    .skip(before)
                    .map(|p| p.opcode)
                    .collect();
                println!("Native prayer Continue: outbound={sent:?}");
                if !sent.contains(&115) {
                    return Err("native Continue did not emit115".into());
                }
                stage = 7;
            }
            7 if !client.interfaces.children.values().any(|s| s.group == 229) => {
                println!("Prayer scenario observed activation, deactivation, requirement notice and native Continue closure");
                live.session.close();
                return Ok(());
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err(format!("prayer scenario deadline at stage {stage}").into())
}
