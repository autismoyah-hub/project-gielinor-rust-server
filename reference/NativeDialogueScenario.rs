//! Owned scenario driver around the unchanged pinned native client library.
//! A failed native Continue click is reported separately from an explicit protocol probe.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, protocol::Packet,
    world_action, writer::Writer,
};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 7 {
        return Err("expected CACHE ADDRESS USER PASSWORD_FILE PUBLIC_KEY ui|protocol".into());
    }
    let probe = match args[6].as_str() {
        "ui" => false,
        "protocol" => true,
        _ => return Err("unknown scenario mode".into()),
    };
    let mut client = ClientState::load(DiskCache::open(&args[1], 64 << 20)?)?;
    let mut live = Connection::connect(&mut client, &args[2], &args[3], &args[4], &args[5])?;
    let start = Instant::now();
    let mut interface_tick = Instant::now();
    let mut target = None;
    let mut approach_sent = false;
    let mut approach_confirmed = false;
    let mut greeting = None;
    let mut ui_continue = false;
    let mut reply = false;
    let mut progress = Instant::now();
    while start.elapsed() < Duration::from_secs(120) {
        // Connection.poll consumes packets before draining scripts. If this poll
        // began with scripts, leave the drained queue empty for the next poll;
        // refilling timer listeners immediately can starve packet dispatch.
        let packets_ready = !live.scripts_busy() && client.interfaces.scripts.is_empty();
        live.poll(&mut client)?;
        if progress.elapsed() >= Duration::from_secs(15) {
            println!("Scenario progress: title={:?}, packets={}, queued_scripts={}, world={}",live.title.as_ref().map(|title| title.screen),live.packets,client.interfaces.scripts.len(),live.world.is_some());
            let mut people = client.npc_state.npcs.iter()
                .filter(|(_,npc)| (3106..=3108).contains(&npc.id))
                .map(|(slot,npc)| (*slot,npc.id,npc.position)).collect::<Vec<_>>();
            people.sort();
            println!("Dialogue visibility: position={:?}; people={people:?}",client.data.position);
            progress = Instant::now();
        }
        // Pre-login interface listeners can queue scripts which block deferred
        // ServerSeed processing; scripts drain only after session establishment.
        if packets_ready && live.title.is_none() && interface_tick.elapsed() >= Duration::from_millis(20) {
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
        if live.scripts_busy() || !client.interfaces.scripts.is_empty() {
            std::thread::sleep(Duration::from_millis(5));
            continue;
        }
        if !approach_sent && live.world.is_some() && start.elapsed() > Duration::from_secs(10) {
            client.outbound.push_back(world_action::walk(3230, 3218, 0)?);
            approach_sent = true;
            println!("Dialogue approach walk submitted: [3230, 3218]");
        }
        if approach_sent && !approach_confirmed && client.data.position == [0,3230,3218] {
            approach_confirmed = true;
            println!("Server confirmed dialogue approach: [3230, 3218]");
        }
        if target.is_none() && approach_confirmed {
            let p = client.data.position;
            if let Some((&index, npc)) = client
                .npc_state
                .npcs
                .iter()
                // man2 spawns east at 3235,3219 (wander radius five). Stay
                // east of the recovery pad so this dialogue case cannot silently
                // become the separate Tower-arrival scenario while approaching.
                .filter(|(_, npc)| npc.id == 3107 && npc.position[0] >= 3229)
                .min_by_key(|(index, npc)| {
                    ((npc.position[0] - p[1]).abs() + (npc.position[1] - p[2]).abs(), npc.position, **index)
                })
            {
                println!(
                    "Scenario initial position {:?}; Talk-to man definition {}, slot {index}",
                    p, npc.id
                );
                client
                    .outbound
                    .push_back(world_action::npc_first(index, 0, false));
                target = Some(index);
            }
        }
        if greeting.is_none()
            && client
                .interfaces
                .children
                .values()
                .any(|sub| sub.group == 217)
        {
            let Some(widget) = client.interfaces.widgets.get(&((217 << 16) | 6)) else {
                continue;
            };
            if widget.text != "Hello, how's it going?" {
                continue;
            }
            // Let the native script/layout pass process the opened interface.
            greeting = Some(Instant::now());
            println!("Native client confirmed generic-person greeting and player chat interface");
        }
        if greeting.is_some_and(|at| at.elapsed() > Duration::from_secs(1)) && !ui_continue {
            let button = (217 << 16) | 5;
            let layout = client.interfaces.layout(client.canvas)?;
            // Observe the full native parent chain, including sub-interface mounts.
            // Do not unhide widgets or bypass native pointer dispatch.
            let mut ancestor = Some(button);
            for _ in 0..24 {
                let Some(id) = ancestor else { break };
                let Some(widget) = client.interfaces.widgets.get(&id) else {
                    break;
                };
                println!(
                    "Continue ancestor {id} ({}:{}): parent={:?} hidden={} placement={:?}",
                    id >> 16,
                    id & 65535,
                    widget.parent,
                    widget.hidden,
                    layout.widgets.get(&id)
                );
                ancestor = widget.parent.or_else(|| {
                    client
                        .interfaces
                        .children
                        .iter()
                        .find(|(_, child)| child.group as u32 == id >> 16)
                        .map(|(&parent, _)| parent)
                });
            }
            for component in 0..=6 {
                let id = (217 << 16) | component;
                if let Some(widget) = client.interfaces.widgets.get(&id) {
                    println!("Dialogue widget {id}: parent={:?}, hidden={}, raw_size={:?}, size_alignment={:?}, placement={:?}",widget.parent,widget.hidden,widget.raw_size,widget.size_alignment,layout.widgets.get(&id));
                }
            }
            let point = layout
                .widgets
                .get(&button)
                .filter(|p| !p.hidden && p.geometry.size.iter().all(|v| *v > 0))
                .map(|p| {
                    [
                        p.absolute[0] + p.geometry.size[0] / 2,
                        p.absolute[1] + p.geometry.size[1] / 2,
                    ]
                });
            let before = client.outbound.len();
            let handled = if let Some(point) = point {
                // Match viewport.rs's ordinary pointer/menu path. The old IF3
                // operation-only helper does not dispatch legacy Continue.
                let entries = client.interface_menu(point)?;
                println!("Native Continue menu options: {:?}", entries.iter().map(|entry| entry.option.as_str()).collect::<Vec<_>>());
                if client.default_menu_opens(&entries) {
                    return Err("Continue requires an open context menu; no direct fallback".into());
                }
                if client.begin_widget_drag(point, &entries)? {
                    return Err("Continue unexpectedly started widget drag".into());
                }
                if let Some(entry) = entries.last() {
                    client.menu_action(entry.action)?;
                    true
                } else {
                    false
                }
            } else {
                println!("UI_FINDING: Continue has no visible layout; server/client attribution pending matched reference run");
                false
            };
            let sent = client
                .outbound
                .iter()
                .skip(before)
                .map(|p| p.opcode)
                .collect::<Vec<_>>();
            println!("Native Continue click at {point:?}: handled={handled}, outbound={sent:?}");
            ui_continue = true;
            if !sent.contains(&115) {
                println!("UI_FINDING: native Continue did not emit RESUME_PAUSEBUTTON (115)");
                if !probe {
                    live.session.close();
                    return Err("native UI continuation failed; no injected fallback used".into());
                }
                println!("EXPLICIT_PROTOCOL_PROBE: injecting Continue for separate server verification; not a native UI pass");
                let mut writer = Writer::new();
                writer.u32_middle(button);
                writer.u16(u16::MAX);
                client.outbound.push_back(Packet {
                    opcode: 115,
                    payload: writer.into_vec(),
                });
            }
        }
        if ui_continue
            && client
                .interfaces
                .children
                .values()
                .any(|sub| sub.group == 231)
        {
            if let Some(widget) = client.interfaces.widgets.get(&((231 << 16) | 6)) {
                if !widget.text.is_empty() {
                    println!("Server NPC response: {}", widget.text);
                    reply = true;
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    live.session.close();
    if !reply {
        return Err("no NPC response confirmed within bounded scenario".into());
    }
    println!(
        "Scenario passed; protocol_probe={probe}; final position {:?}",
        client.data.position
    );
    Ok(())
}
