//! Independent native controls and packet observation. No local gameplay mutation.
use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,live_runtime::Connection,context_menu::Action};
use std::time::{Duration,Instant};
type R<T> = Result<T,Box<dyn std::error::Error>>;
fn pump(c:&mut ClientState,l:&mut Connection,seconds:u64,ready:impl Fn(&ClientState,&Connection)->bool)->R<()> {
    let start=Instant::now();let mut cycle=Instant::now();let mut progress=Instant::now();
    loop {
        l.poll(c)?;if l.logged_out().is_some(){return Err("unexpected native disconnect".into());}
        if cycle.elapsed()>=Duration::from_millis(20){
            if !l.scripts_busy() && c.interfaces.scripts.is_empty(){c.tick_interfaces(false)?;}
            if let Some(e)=c.script_failures.pop_front(){return Err(format!("native script failure: {e}").into());}
            c.cycle=c.cycle.wrapping_add(1);c.input.begin_cycle();cycle=Instant::now();
        }
        if ready(c,l){return Ok(());}
        if progress.elapsed()>Duration::from_secs(10){println!("PG_WAIT elapsed={} tick={} hp={} busy={} scripts={}",start.elapsed().as_secs(),c.server_tick,c.data.boosted[3],l.scripts_busy(),c.interfaces.scripts.len());progress=Instant::now();}
        if start.elapsed()>Duration::from_secs(seconds){return Err(format!("observer deadline; tick={} hp={} busy={} scripts={}",c.server_tick,c.data.boosted[3],l.scripts_busy(),c.interfaces.scripts.len()).into());}
        std::thread::sleep(Duration::from_millis(5));
    }
}
fn ticks(c:&mut ClientState,l:&mut Connection,n:i32)->R<()> {let target=c.server_tick+n;pump(c,l,30,|c,_|c.server_tick>=target)}
fn ready(c:&mut ClientState,l:&mut Connection)->R<()> {pump(c,l,180,|c,l|l.world.is_some() && c.server_tick>0 && !l.scripts_busy() && c.interfaces.scripts.is_empty() && c.interfaces.root==Some(548))}
fn dispatch(c:&mut ClientState,action:Action,label:&str)->R<()> {
    let before=c.outbound.len();c.menu_action(action)?;
    for p in c.outbound.iter().skip(before){println!("PG_TX label={label:?} tick={} opcode={} payload={:02x?}",c.server_tick,p.opcode,p.payload);}
    Ok(())
}
fn tab(c:&mut ClientState,label:&str)->R<()> {
    let layout=c.interfaces.layout(c.canvas)?;
    for (&id,p) in &layout.widgets {
        if id>>16!=548 || p.hidden || p.clip[2]<=p.clip[0] || p.clip[3]<=p.clip[1]{continue;}
        let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
        if let Some(e)=c.interface_menu(point)?.into_iter().find(|e|e.option==label){dispatch(c,e.action,label)?;return Ok(());}
    }
    Err(format!("native tab unavailable: {label}").into())
}
fn item_action(c:&mut ClientState,slot:i32,worn:bool,label:&str)->R<Action> {
    let logical=if worn{(387<<16)|17}else{149<<16};
    let id=if worn{logical}else{c.interfaces.runtime.iter().find(|(_,r)|r.logical_id==logical && r.child_index==slot).map(|(&id,_)|id).ok_or("native inventory widget missing")?};
    let layout=c.interfaces.layout(c.canvas)?;let p=layout.widgets.get(&id).filter(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1]).ok_or("native item placement unavailable")?;
    let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
    c.interface_menu(point)?.into_iter().find(|e|e.option==label).map(|e|e.action).ok_or_else(||format!("native item option unavailable: {label}").into())
}
fn choice(c:&mut ClientState,text:&str)->R<Option<Action>> {
    let layout=c.interfaces.layout(c.canvas)?;
    for (&id,w) in &c.interfaces.widgets {
        if !w.text.contains(text){continue;}
        let Some(p)=layout.widgets.get(&id).filter(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1])else{continue;};
        let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
        if let Some(e)=c.interface_menu(point)?.into_iter().find(|e|matches!(e.action,Action::Continue(target) if target==id)){return Ok(Some(e.action));}
    }
    Ok(None)
}
fn wait_choice(c:&mut ClientState,l:&mut Connection,text:&str)->R<Action> {
    let start=Instant::now();loop {pump(c,l,30,|_,_|true)?;if let Some(a)=choice(c,text)?{return Ok(a);}if start.elapsed()>Duration::from_secs(30){return Err(format!("choice not visible: {text}").into());}std::thread::sleep(Duration::from_millis(20));}
}
fn sent_packet(c:&mut ClientState,a:Action,label:&str)->R<(u16,Vec<u8>)> {
    let before=c.outbound.len();dispatch(c,a,label)?;assert_eq!(c.outbound.len(),before+1);
    let p=c.outbound.back().unwrap();Ok((p.opcode,p.payload.clone()))
}
fn replay(c:&ClientState,l:&mut Connection,p:&(u16,Vec<u8>),label:&str)->R<()> {
    println!("PG_REPLAY label={label:?} tick={} opcode={} payload={:02x?}",c.server_tick,p.0,p.1);l.session.send(p.0,&p.1)?;Ok(())
}
fn invariant(c:&ClientState,xp:&[i32;25])->R<()> {
    assert_eq!(&c.data.experience,xp);assert_eq!((c.inventories.item(93,3).id,c.inventories.item(93,3).count),(21163,1));
    assert_eq!((c.inventories.item(94,2).id,c.inventories.item(94,2).count),(21163,1));assert_eq!(c.inventories.item(93,27).count,1000);Ok(())
}
fn chemistry(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;invariant(c,&xp)?;tab(c,"Inventory")?;ticks(c,l,2)?;
    let a=item_action(c,3,false,"Break")?;let held=sent_packet(c,a,"held Break open")?;
    let a=wait_choice(c,l,"Cancel")?;let mut stale=sent_packet(c,a,"held real Cancel")?;assert_eq!(stale.0,115);assert_eq!(&stale.1[4..],&[0,2]);ticks(c,l,3)?;invariant(c,&xp)?;
    stale.1[4..].copy_from_slice(&[0,1]);
    replay(c,l,&stale,"Confirm after Cancel")?;ticks(c,l,3)?;invariant(c,&xp)?;
    let mut wrong=held;assert_eq!((wrong.0,wrong.1.len()),(47,9));wrong.1[6..8].copy_from_slice(&21164u16.to_be_bytes());wrong.1[8]=4;replay(c,l,&wrong,"wrong held item claim")?;ticks(c,l,3)?;invariant(c,&xp)?;
    let a=item_action(c,3,false,"Options")?;dispatch(c,a,"held Options open")?;let a=wait_choice(c,l,"Continue when")?;let mut old_stop=sent_packet(c,a,"held real Continue")?;ticks(c,l,3)?;old_stop.1[4..].copy_from_slice(&[0,2]);
    replay(c,l,&old_stop,"held Stop after completed Continue")?;ticks(c,l,3)?;invariant(c,&xp)?;
    tab(c,"Worn Equipment")?;ticks(c,l,3)?;let a=item_action(c,2,true,"Options")?;let worn=sent_packet(c,a,"worn Options open")?;
    let a=wait_choice(c,l,"Continue when")?;let mut stale=sent_packet(c,a,"worn real Continue")?;ticks(c,l,3)?;invariant(c,&xp)?;
    let mut wrong=worn;assert_eq!((wrong.0,wrong.1.len()),(47,9));wrong.1[6..8].copy_from_slice(&21164u16.to_be_bytes());replay(c,l,&wrong,"wrong worn explicit item claim")?;ticks(c,l,3)?;invariant(c,&xp)?;
    let a=item_action(c,2,true,"Options")?;dispatch(c,a,"worn Options reopen")?;let _=wait_choice(c,l,"Stop when")?;
    l.session.send(95,&[])?;println!("PG_CLOSE_MODAL worn tick={}",c.server_tick);ticks(c,l,3)?;stale.1[4..].copy_from_slice(&[0,2]);replay(c,l,&stale,"worn Stop after modal close")?;ticks(c,l,3)?;invariant(c,&xp)?;
    println!("PG_CASE_PASS chemistry cancelled-held/closed-held/closed-worn/stale-item-claims; native items and XP unchanged; server SQL must verify charges2/stop0");Ok(())
}
fn chemistry_close(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;invariant(c,&xp)?;tab(c,"Worn Equipment")?;ticks(c,l,3)?;
    let a=item_action(c,2,true,"Options")?;dispatch(c,a,"worn Options baseline")?;
    let a=wait_choice(c,l,"Continue when")?;let mut stale=sent_packet(c,a,"worn genuine Continue")?;ticks(c,l,3)?;
    let a=item_action(c,2,true,"Options")?;dispatch(c,a,"worn Options before Close")?;let _=wait_choice(c,l,"Stop when")?;
    dispatch(c,Action::Close,"genuine native Close")?;assert!(choice(c,"Stop when")?.is_none());ticks(c,l,3)?;
    assert_eq!(stale.0,115);stale.1[4..].copy_from_slice(&[0,2]);replay(c,l,&stale,"stale Stop after genuine Close")?;ticks(c,l,3)?;invariant(c,&xp)?;
    println!("PG_PROBE_COMPLETE genuine native Close and stale response; authoritative SQL must retain charges2/stop0");Ok(())
}
fn widget_point(c:&ClientState,slot:i32)->R<(u32,[i32;2])> {
    let id=c.interfaces.runtime.iter().find(|(_,r)|r.logical_id==149<<16 && r.child_index==slot).map(|(&id,_)|id).ok_or("selected native item widget missing")?;
    let layout=c.interfaces.layout(c.canvas)?;let p=layout.widgets.get(&id).filter(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1]).ok_or("selected item not visible")?;
    Ok((id,[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2]))
}
fn use_slots(c:&mut ClientState,first:i32,second:i32)->R<()> {
    let (id,p)=widget_point(c,first)?;let a=c.interface_menu(p)?.into_iter().find(|e|matches!(e.action,Action::SelectSpell(w) if w==id)).ok_or("native Use selection absent")?.action;
    dispatch(c,a,"native selected Use")?;
    let (id,p)=widget_point(c,second)?;let a=c.interface_menu(p)?.into_iter().find(|e|matches!(e.action,Action::CastInterface(w) if w==id)).ok_or("native Use target absent")?.action;
    dispatch(c,a,"native Use target")?;Ok(())
}
fn make(c:&mut ClientState,l:&mut Connection,all:bool)->R<()> {
    let start=Instant::now();loop {
        pump(c,l,30,|_,_|true)?;let layout=c.interfaces.layout(c.canvas)?;
        if let Some(p)=layout.widgets.get(&((270<<16)|15)).filter(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1]) {
            let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
            if let Some(a)=c.interface_menu(point)?.into_iter().find(|e|match e.action{Action::Continue(id)|Action::Button{id,..}|Action::Interface{id,..}=>c.interfaces.runtime.get(&id).map_or(id,|r|r.logical_id)==(270<<16)|15,_=>false}).map(|e|e.action){
                if all {
                    let layout=c.interfaces.layout(c.canvas)?;
                    let (&id,_)=c.interfaces.widgets.iter().find(|(id,w)|w.text.trim()=="All" && c.interfaces.runtime.get(*id).map_or(**id,|r|r.logical_id)>>16==270 && layout.widgets.get(*id).is_some_and(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1])).ok_or("native All control absent")?;
                    let p=&layout.widgets[&id];let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
                    let select=c.interface_menu(point)?.into_iter().find(|e|matches!(e.action,Action::Interface{..}) && (e.option=="All" || e.option=="Select")).ok_or("native All menu absent")?.action;
                    c.input.pointer=point;c.input.button(0,true);c.input.button(0,false);c.tick_interfaces(false)?;c.input.begin_cycle();dispatch(c,select,"native All quantity")?;
                    pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
                }
                dispatch(c,a,"native Make recipe")?;return Ok(());
            }
        }
        if start.elapsed()>Duration::from_secs(30){return Err("native Make dialog unavailable".into());}
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn combat_interrupt(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let before=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();
    tab(c,"Inventory")?;ticks(c,l,3)?;
    use_slots(c,3,0)?;make(c,l,false)?;
    let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native Walk before first completion")?;ticks(c,l,7)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),before);
    println!("PG_CASE_PASS combat native movement cancellation before first commit; inventory and XP unchanged");
    use_slots(c,3,0)?;make(c,l,true)?;
    pump(c,l,30,|c,_|c.inventories.total(93,12695)>=1)?;
    let made=c.inventories.total(93,12695);assert_eq!(made,1);assert_eq!(c.data.experience[15]-xp[15],750);
    println!("PG_DISCONNECT_PARTIAL combat outputs={made} tick={} display_xp={}; actual socket close with All work remaining",c.server_tick,c.data.experience[15]);
    // main closes the real session immediately. Server SQL/trace decides whether
    // any already admitted clock committed; no fabricated zero-race assertion.
    Ok(())
}
fn chemistry_interrupt(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let before=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();
    assert_eq!(c.inventories.item(94,0).id,29974);assert_eq!(c.inventories.item(94,2).id,21163);
    tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,0,1)?;make(c,l,false)?;
    let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native chemistry Walk before completion")?;ticks(c,l,7)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),before);
    println!("PG_CASE_PASS chemistry native movement before completion leaves items and XP unchanged with goggles and amulet equipped");
    use_slots(c,0,1)?;make(c,l,true)?;pump(c,l,30,|c,_|c.inventories.total(93,175)+c.inventories.total(93,2446)>=1)?;
    let made=c.inventories.total(93,175)+c.inventories.total(93,2446);assert_eq!(made,1);
    let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native chemistry Walk after first observed output")?;
    // Received inventory can lag the authoritative production clock. Observe
    // server-reported movement, then require quiescence; do not assume the Walk
    // has already reached the server when menu_action merely queues it.
    pump(c,l,30,|c,_|c.data.position[1]==point[0] as i32 && c.data.position[2]==point[1] as i32)?;
    ready(c,l)?;let stopped=c.inventories.total(93,175)+c.inventories.total(93,2446);assert!(stopped>=made && stopped<14);
    let stopped_xp=c.data.experience;println!("PG_MOVEMENT_ACK chemistry observed_first={made} stopped_outputs={stopped} tick={} position={:?}",c.server_tick,c.data.position);
    ticks(c,l,10)?;assert_eq!(c.inventories.total(93,175)+c.inventories.total(93,2446),stopped);assert_eq!(c.data.experience,stopped_xp);assert_eq!(c.inventories.item(94,0).id,29974);
    println!("PG_CASE_PASS chemistry interrupted All stable after actual movement acknowledgement; neck/charges/secondary/RNG outcomes require server trace and SQL");Ok(())
}
fn chemistry_crash(c:&mut ClientState,l:&mut Connection)->R<()> {
    tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,0,1)?;make(c,l,true)?;
    pump(c,l,30,|c,_|c.inventories.total(93,175)+c.inventories.total(93,2446)>=1)?;
    let observed=c.inventories.total(93,175)+c.inventories.total(93,2446);assert!(observed>0 && observed<14);
    println!("PG_CRASH_READY chemistry observed_outputs={observed} tick={}; real All accepted, ingredients remain, keep real socket open for owned server termination",c.server_tick);
    let start=Instant::now();loop {
        match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}
        if start.elapsed()>Duration::from_secs(30){return Err("owned crash injection did not arrive".into());}
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn food_negative(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;tab(c,"Inventory")?;ticks(c,l,3)?;
    assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
    let a=item_action(c,3,false,"Eat")?;let stale=sent_packet(c,a,"full bag selected whole cake")?;pump(c,l,30,|c,_|c.inventories.item(93,3).id==1893)?;ticks(c,l,3)?;
    assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
    replay(c,l,&stale,"stale whole cake while matching duplicate remains")?;ticks(c,l,4)?;
    assert_eq!(c.inventories.item(93,3).id,1893);assert_eq!(c.inventories.item(93,1).id,1891);assert_eq!(c.data.experience,xp);
    let a=item_action(c,9,false,"Drink")?;dispatch(c,a,"native brew for real overheal")?;pump(c,l,30,|c,_|c.inventories.item(93,9).id==6687 && c.data.boosted[3]>c.data.base[3])?;
    // An inventory acknowledgement is not expiry of the potion's protected
    // consumption delay. Keep this functional overheal case outside that delay.
    ticks(c,l,6)?;let hp=c.data.boosted[3];assert_eq!(hp,13);let a=item_action(c,3,false,"Eat")?;dispatch(c,a,"native food at pre-existing overheal")?;
    pump(c,l,30,|c,_|c.inventories.item(93,3).id==1895)?;
    println!("PG_FOOD_OVERHEAL before={hp} after={} base={} tick={}",c.data.boosted[3],c.data.base[3],c.server_tick);
    assert_eq!(c.data.boosted[3],hp);assert_eq!(c.data.experience,xp);assert_eq!(c.inventories.item(93,1).id,1891);assert_eq!(c.inventories.item(93,27).count,1000);
    println!("PG_CASE_PASS full28 selected remainder, stale claimed input with duplicate, real potion overheal preserved by food, no XP");Ok(())
}
fn prayer_burial(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
    assert_eq!(c.inventories.item(93,3).id,526);assert_eq!(c.inventories.item(93,17).id,526);
    tab(c,"Inventory")?;ticks(c,l,3)?;let a=item_action(c,17,false,"Bury")?;let stale=sent_packet(c,a,"native selected Bury slot17")?;
    pump(c,l,30,|c,_|c.inventories.item(93,17).id==-1 && c.data.experience[5]>xp[5])?;
    assert_eq!(c.inventories.item(93,3).id,526);assert_eq!(c.inventories.item(93,27).count,1000);
    for i in 0..25{if i!=5{assert_eq!(c.data.experience[i],xp[i]);}}
    ticks(c,l,5)?;let committed=c.data.experience;replay(c,l,&stale,"stale Bury slot17 with untouched matching bone slot3")?;ticks(c,l,7)?;
    assert_eq!(c.data.experience,committed);assert_eq!(c.inventories.item(93,3).id,526);assert_eq!(c.inventories.item(93,17).id,-1);
    println!("PG_CASE_PASS native selected burial full28 slot17; earlier bone preserved; stale real claimed-slot replay no extra award; displayed Prayer XP {}->{}; exact fineXP assessed against independently loaded cache",xp[5],committed[5]);Ok(())
}
fn gems(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let initial=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();
    assert_eq!(c.data.base[12],99);tab(c,"Inventory")?;ticks(c,l,3)?;
    use_slots(c,0,4)?;let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native gem cancellation before first completion")?;ticks(c,l,7)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),initial);
    println!("PG_CASE_PASS native gem pending action cancelled by immediate Walk without consumption or XP");
    let mut fine=130344311i32;
    for (index,(raw,cut,award)) in [(1623,1607,2500),(1621,1605,3375),(1619,1603,4250),(1617,1601,5375),(1631,1615,6875),(6571,6573,8375),(19496,19493,2500)].into_iter().enumerate(){
        ready(c,l)?;let slot=4+3*index as i32;assert_eq!(c.inventories.item(93,slot).id,raw);let n=c.outbound.len();
        if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}
        let p=c.outbound.iter().skip(n).find(|p|p.opcode==27).ok_or("gem native pair packet absent")?;let stale=(p.opcode,p.payload.clone());
        fine+=award;pump(c,l,30,|c,_|c.inventories.item(93,slot).id==cut && c.data.experience[12]==fine/10)?;ticks(c,l,3)?;
        assert_eq!(c.inventories.item(93,slot+1).id,raw);assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);
        assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);let committed=c.data.experience;
        replay(c,l,&stale,"stale gem pair while matching raw and tool duplicates remain")?;ticks(c,l,4)?;assert_eq!(c.data.experience,committed);assert_eq!(c.inventories.item(93,slot+1).id,raw);
        for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_GEM_COMMITTED index={index} slot={slot} raw={raw} cut={cut} fine_award={award} display_xp={} direction_reverse={}",c.data.experience[12],index%2==1);
    }
    println!("PG_CASE_PASS seven native gem cuts, both directions, full28 selected duplicates, retained chisels,effective500percent fractional XP and stale-pair refusal; native125percent is blocked by pinned login policy");Ok(())
}
fn gem_effects(c:&mut ClientState,l:&mut Connection)->R<()> {
    // New scope: independently observed decoded animation/partial packets and
    // every profile in the opposite selected direction on a fresh full bag.
    gems(c,l)?;let mut fine=130377561i32;
    for (index,(raw,cut,award)) in [(1623,1607,2500),(1621,1605,3375),(1619,1603,4250),(1617,1601,5375),(1631,1615,6875),(6571,6573,8375),(19496,19493,2500)].into_iter().enumerate(){
        ready(c,l)?;let slot=5+3*index as i32;assert_eq!(c.inventories.item(93,slot).id,raw);
        println!("PG_GEM_EFFECT_BEGIN index={index} slot={slot} raw={raw} cut={cut} tick={}",c.server_tick);
        if index%2==0{use_slots(c,slot,0)?;}else{use_slots(c,0,slot)?;}
        fine+=award;pump(c,l,30,|c,_|c.inventories.item(93,slot).id==cut && c.data.experience[12]==fine/10)?;ticks(c,l,3)?;
        assert_eq!(c.inventories.item(93,slot-1).id,cut);assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);
        assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
        println!("PG_GEM_SECOND_COMMITTED index={index} slot={slot} cut={cut} fine_award={award} display_xp={} direction_reverse={}",c.data.experience[12],index%2==0);
    }
    println!("PG_CASE_PASS fourteen native gem cuts; every profile in both directions; actual decoded sequence/effect and first-cycle packet cadence require independent assessment");Ok(())
}
fn glass_make_action(c:&mut ClientState,l:&mut Connection,index:u32)->R<Action> {
    let start=Instant::now();let logical=(270<<16)|(15+index);
    loop {
        pump(c,l,30,|_,_|true)?;let layout=c.interfaces.layout(c.canvas)?;
        if !l.scripts_busy() && c.interfaces.scripts.is_empty(){
            if let Some(p)=layout.widgets.get(&logical).filter(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1]) {
                let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
                if let Some(a)=c.interface_menu(point)?.into_iter().find(|e|match e.action{Action::Continue(id)|Action::Button{id,..}|Action::Interface{id,..}=>c.interfaces.runtime.get(&id).map_or(id,|r|r.logical_id)==logical,_=>false}).map(|e|{println!("PG_GLASS_MENU index={index} option={:?} target={:?}",e.option,e.target);e.action}){
                    for (&id,w) in &c.interfaces.widgets {let logical=c.interfaces.runtime.get(&id).map_or(id,|r|r.logical_id);if logical>>16==270 && (!w.text.is_empty() || !w.data_text.is_empty()){println!("PG_GLASS_WIDGET id={id} logical={logical} text={:?} data={:?}",w.text,w.data_text);}}
                    println!("PG_GLASS_OPTION_READY index={index} logical={logical} tick={}",c.server_tick);return Ok(a);
                }
            }
        }
        if start.elapsed()>Duration::from_secs(30){return Err(format!("native glass choice{index} unavailable").into());}std::thread::sleep(Duration::from_millis(20));
    }
}
fn production_profiles(c:&mut ClientState,l:&mut Connection,shells:bool)->R<()> {
    let gems=[(1623,1607,20,2500),(1621,1605,27,3375),(1619,1603,34,4250),(1617,1601,43,5375),(1631,1615,55,6875),(6571,6573,67,8375),(19496,19493,89,2500)];
    let xp=c.data.experience;let mut fine=130344311;let mut expected=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();tab(c,"Inventory")?;ticks(c,l,3)?;
    let start=if shells{7}else{0};let end=if shells{16}else{7};let first_slot=4+start;
    use_slots(c,0,first_slot)?;let action=glass_make_action(c,l,0)?;let cancelled=sent_packet(c,action,"native single-choice Make before genuine Close")?;dispatch(c,Action::Close,"native production Close before first award")?;ticks(c,l,6)?;
    replay(c,l,&cancelled,"stale single-choice response after genuine Close")?;ticks(c,l,4)?;assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),expected);
    println!("PG_CASE_PASS native production Make/Close and stale continuation refused before award");
    for index in start..end {
        ready(c,l)?;let(raw,output,level,award)=if index<7{gems[index as usize]}else{(3345+2*(index-7),3327+2*(index-7),15,1625)};let slot=4+index;assert_eq!(c.inventories.item(93,slot).id,raw);
        if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}
        let action=glass_make_action(c,l,0)?;let label=plain_ui(&c.interfaces.widgets.get(&((270<<16)|15)).ok_or("single product widget absent")?.data_text);
        let name=&c.items.items.get(&(output as u32)).ok_or("native output definition absent")?.name;assert!(label.to_ascii_lowercase().contains(&name.to_ascii_lowercase()) && label.contains(&format!("level {level}")),"output{output} label={label:?} expected={name:?}");
        let packet=sent_packet(c,action,"native registered single-choice Make-one")?;assert_eq!((packet.0,packet.1.len()),(115,6));assert_eq!(&packet.1[4..],&[0,1]);fine+=award;
        pump(c,l,30,|c,_|c.inventories.item(93,slot).id==output && c.data.experience[12]==fine/10)?;expected[slot as usize]=(output,1);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),expected);
        replay(c,l,&packet,"stale completed single-choice production response")?;ticks(c,l,3)?;assert_eq!(c.data.experience[12],fine/10);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_PRODUCTION_COMMITTED index={index} slot={slot} raw={raw} output={output} fine_award={award} display_xp={} label={label:?}",c.data.experience[12]);
    }
    println!("PG_CASE_PASS registered native single-choice profiles shells={shells}; complete full-bag selected outputs/labels/fineXP and stale refusal; actual restart assessment separate");Ok(())
}
fn production_batch(c:&mut ClientState,l:&mut Connection,shells:bool)->R<()> {
    let(raw,output,award)=if shells{(3345,3327,1625)}else{(1623,1607,2500)};let xp=c.data.experience;let mut made=0;let mut fine=130344311;tab(c,"Inventory")?;ticks(c,l,3)?;
    for (index,(label,count)) in [("1",1),("5",5),("10",10),("X",3),("All",4)].into_iter().enumerate(){
        ready(c,l)?;let raws=(0..28).filter(|&s|c.inventories.item(93,s).id==raw).collect::<Vec<_>>();let slot=if index%2==0{raws[0]}else{*raws.last().ok_or("production raw absent")?};
        if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}
        let packet=glass_quantity(c,l,0,label,count)?;made+=count;fine+=count*award;
        pump(c,l,90,|c,_|c.inventories.total(93,output)==made && c.inventories.total(93,raw)==23-made && c.data.experience[12]==fine/10)?;ticks(c,l,3)?;
        assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
        replay(c,l,&packet,"stale completed native production quantity response")?;ticks(c,l,3)?;assert_eq!(c.data.experience[12],fine/10);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_PRODUCTION_BATCH_COMMITTED label={label} count={count} selected_slot={slot} output={output} total_made={made} fine_award={} display_xp={}",count*award,c.data.experience[12]);
    }
    assert_eq!(made,23);println!("PG_CASE_PASS native production quantities1/5/10/X3/All4 shells={shells};23selected ordinary products; exact once-per-raw fineXP and stale refusal; custom-X wire observations require independent assessment");Ok(())
}
fn native_focused_x(c:&mut ClientState,l:&mut Connection,index:u32,n:i32)->R<(u16,Vec<u8>)> {
    let _=glass_make_action(c,l,index)?;let layout=c.interfaces.layout(c.canvas)?;let logical=(270<<16)|11;
    let (&id,_)=c.interfaces.widgets.iter().find(|(id,w)|plain_ui(&w.text).trim()=="X" && c.interfaces.runtime.get(*id).map_or(**id,|r|r.logical_id)==logical && layout.widgets.get(*id).is_some_and(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1])).ok_or("visible native X control absent")?;
    let p=&layout.widgets[&id];let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];let action=c.interface_menu(point)?.into_iter().find(|e|matches!(e.action,Action::Interface{id,..} if c.interfaces.runtime.get(&id).map_or(id,|r|r.logical_id)==logical)).ok_or("native X menu action absent")?.action;
    println!("PG_X_FOCUS_BEGIN widget={id} action={action:?} onclick={} onrelease={} onop={} mask={}",c.interfaces.listener(id,1400).is_some(),c.interfaces.listener(id,1402).is_some(),c.interfaces.listener(id,1409).is_some(),c.interfaces.operation_mask(id));
    c.input.pointer=point;c.input.button(0,true);c.tick_interfaces(false)?;c.input.begin_cycle();pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
    std::thread::sleep(Duration::from_millis(25));c.input.button(0,false);c.tick_interfaces(false)?;c.input.begin_cycle();pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
    dispatch(c,action,"native X Other quantity after separate press/release frames")?;pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
    let start=Instant::now();loop{
        pump(c,l,30,|_,_|true)?;let layout=c.interfaces.layout(c.canvas)?;let mut prompt=false;
        if !l.scripts_busy() && c.interfaces.scripts.is_empty(){for (&wid,w) in &c.interfaces.widgets{if !layout.widgets.get(&wid).is_some_and(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1]){continue;}let text=plain_ui(&w.text).to_ascii_lowercase();if (text.contains("enter") && (text.contains("amount") || text.contains("quantity") || text.contains("number"))) || text.contains("how many"){println!("PG_X_INPUT_PROMPT widget={wid} text={:?}",w.text);prompt=true;}}}
        if prompt{break;}
        if start.elapsed()>Duration::from_secs(30){for (&wid,w) in &c.interfaces.widgets{if layout.widgets.get(&wid).is_some_and(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1]) && (!w.text.is_empty() || !w.data_text.is_empty()){let logical=c.interfaces.runtime.get(&wid).map_or(wid,|r|r.logical_id);println!("PG_X_VISIBLE_WIDGET id={wid} logical={logical} text={:?} data={:?} content={} mask={}",w.text,w.data_text,w.content,c.interfaces.operation_mask(wid));}}return Err("native custom-quantity input prompt absent after full press/release; no keyboard text injected".into());}std::thread::sleep(Duration::from_millis(20));
    }
    for character in n.to_string().encode_utf16(){c.input.character(character);c.tick_interfaces(false)?;c.input.begin_cycle();ticks(c,l,1)?;}
    c.input.key(84,true);c.tick_interfaces(false)?;c.input.key(84,false);c.input.begin_cycle();pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
    println!("PG_X_FOCUSED_TYPED count={n} prompt_observed=true separate_pointer_frames=true");let fresh=glass_make_action(c,l,index)?;let packet=sent_packet(c,fresh,"native focused custom quantity Make")?;assert_eq!((packet.0,packet.1.len()),(115,6));assert_eq!(u16::from_be_bytes([packet.1[4],packet.1[5]]) as i32,n);Ok(packet)
}
fn production_x_boundary(c:&mut ClientState,l:&mut Connection,focused:bool)->R<()> {
    let xp=c.data.experience;tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,26,0)?;let packet=if focused{native_focused_x(c,l,0,10)?}else{glass_quantity(c,l,0,"X",10)?};
    pump(c,l,90,|c,_|c.inventories.total(93,1607)==10 && c.inventories.total(93,1623)==13 && c.data.experience[12]==(130344311+25000)/10)?;
    replay(c,l,&packet,"stale multi-digit native X10response")?;ticks(c,l,5)?;assert_eq!(c.inventories.total(93,1607),10);assert_eq!(c.inventories.total(93,1623),13);assert_eq!(c.data.experience[12],(130344311+25000)/10);
    assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
    println!("PG_CASE_PASS native multi-digit X10 produces10selected sapphires/25000fineXP and retains13raws without stale repeat");Ok(())
}
fn production_partial(c:&mut ClientState,l:&mut Connection,shells:bool,crash:bool)->R<()> {
    let(raw,output,unit)=if shells{(3345,3327,1625)}else{(1623,1607,2500)};tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,26,0)?;let _=glass_quantity(c,l,0,"All",23)?;
    pump(c,l,30,|c,_|{let made=c.inventories.total(93,output);made>0 && made<23 && c.inventories.total(93,raw)==23-made && c.data.experience[12]==(130344311+made*unit)/10})?;
    let made=c.inventories.total(93,output);println!("PG_PRODUCTION_PARTIAL shells={shells} observed_products={made} remaining_raw={} output={output} tick={}; authoritative SQL owns any already-admitted race",23-made,c.server_tick);
    if !crash{println!("PG_DISCONNECT_PARTIAL production shells={shells} outputs={made}; actual socket closes with captured All work remaining");return Ok(());}
    println!("PG_CRASH_READY production unsaved All result observed with captured raw items remaining; real socket open; no logout/save requested");let start=Instant::now();loop{
        match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}
        if start.elapsed()>Duration::from_secs(30){return Err("owned production crash injection did not arrive".into());}std::thread::sleep(Duration::from_millis(10));
    }
}
fn bank_deposit_action(c:&mut ClientState,l:&mut Connection,slot:i32)->R<Action> {
    let start=Instant::now();loop{
        pump(c,l,30,|_,_|true)?;let layout=c.interfaces.layout(c.canvas)?;
        if !l.scripts_busy() && c.interfaces.scripts.is_empty(){
            let ids=c.interfaces.runtime.iter().filter_map(|(&id,r)|(r.logical_id>>16==15 && r.child_index==slot).then_some(id)).collect::<Vec<_>>();
            for id in ids {
                let Some(p)=layout.widgets.get(&id).filter(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1])else{continue;};
                let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
                if let Some(e)=c.interface_menu(point)?.into_iter().find(|e|e.option.eq_ignore_ascii_case("Deposit-All")){println!("PG_BANK_MENU slot={slot} option={:?} target={:?}",e.option,e.target);return Ok(e.action);}
            }
        }
        if start.elapsed()>Duration::from_secs(90){return Err("native bank sidebar Deposit-All unavailable".into());}std::thread::sleep(Duration::from_millis(20));
    }
}
fn pouch(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;tab(c,"Inventory")?;ticks(c,l,3)?;
    assert_eq!(c.inventories.item(93,4).count,3);assert_eq!(c.inventories.item(93,5).count,2);
    let a=item_action(c,4,false,"Open")?;dispatch(c,a,"native selected pouch Open-one")?;pump(c,l,30,|c,_|c.inventories.item(93,4).count==2 && c.inventories.item(93,27).count==1003)?;
    let a=item_action(c,4,false,"Open-all")?;let packet=sent_packet(c,a,"native selected pouch Open-all")?;pump(c,l,30,|c,_|c.inventories.item(93,4).id==-1 && c.inventories.item(93,27).count==1009)?;
    replay(c,l,&packet,"stale selected pouch slot4with duplicate slot5intact")?;ticks(c,l,5)?;assert_eq!(c.inventories.item(93,5).count,2);assert_eq!(c.inventories.item(93,27).count,1009);assert_eq!(c.data.experience,xp);
    println!("PG_CASE_PASS native pouch Open-one/Open-all selected full bag;3pouches->9coins; duplicate and XP retained; stale slot replay refused");
    let mut bank=None;
    for (&index,npc) in &c.npc_state.npcs {
        if npc.id!=3227{continue;}let def=c.npcs.get(&(npc.id as u32)).ok_or("native banker definition absent")?;
        for i in 0..5 {if npc.appearance.operations&(1<<i)==0{continue;}
            if gielinor_client::entity_ops::label(&def.operations,i,None,&c.scripts.variables,&mut c.scripts.source)?.is_some_and(|s|s.eq_ignore_ascii_case("Bank")){bank=Some(Action::Npc{index,operation:(i+1) as u8,sub:0});}
        }
    }
    dispatch(c,bank.ok_or("native cache-resolved Banker tutor Bank operation absent")?,"native cache-resolved Banker tutor Bank")?;
    let action=bank_deposit_action(c,l,27)?;let sent=gielinor_client::live_runtime::PG_SENT_PACKETS.lock().unwrap().len();dispatch(c,action,"native bankable Coins Deposit-All control")?;
    pump(c,l,60,|c,_|c.inventories.item(93,27).id==-1 && c.inventories.total(95,995)==1009)?;
    let packet={let sent_packets=gielinor_client::live_runtime::PG_SENT_PACKETS.lock().unwrap();let frames=sent_packets.iter().skip(sent).filter(|(_,op,b)|*op==47 && b.len()==9 && b[4..6]==27u16.to_be_bytes() && b[6..8]==995u16.to_be_bytes()).collect::<Vec<_>>();assert_eq!(frames.len(),1,"one actually submitted native bank deposit expected");(frames[0].1,frames[0].2.clone())};
    let before=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();let mut denied=packet.clone();assert_eq!((denied.0,denied.1.len()),(47,9));denied.1[4..6].copy_from_slice(&5u16.to_be_bytes());denied.1[6..8].copy_from_slice(&22521u16.to_be_bytes());
    replay(c,l,&denied,"actual native bank transfer retargeted to restricted pouch slot5and matching22521claim")?;ticks(c,l,6)?;
    assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),before);assert_eq!(c.inventories.total(95,995),1009);
    assert_eq!(c.inventories.item(93,5).count,2);assert_eq!(c.inventories.total(95,22521),0);assert_eq!(c.data.experience,xp);
    let refusal=c.game_messages.iter().find(|m|m.text=="A magical force prevents you from banking this item!").ok_or("specific no_bank refusal message absent; unchanged inventory alone cannot prove restriction route")?;
    println!("PG_POUCH_BANK_REFUSAL text={:?}",refusal.text);
    println!("PG_CASE_PASS native no_bank Coinpouch deposit refusal via retargeted actual native bank-transfer frame, with1009coin positive deposit control; NPCoperation cache-resolved, no rendered hit-test claim");Ok(())
}
fn plain_ui(text:&str)->String {
    let mut result=String::new();let mut tag=false;
    for ch in text.chars(){if ch=='<'{tag=true;}else if ch=='>'{tag=false;}else if !tag{result.push(ch);}}
    result
}
fn glass_quantity(c:&mut ClientState,l:&mut Connection,index:u32,label:&str,n:i32)->R<(u16,Vec<u8>)> {
    let action=glass_make_action(c,l,index)?;let layout=c.interfaces.layout(c.canvas)?;
    let child=match label{"1"=>7,"5"=>8,"10"=>9,"X"=>11,"All"=>12,_=>return Err("unknown quantity control".into())};
    let control=(270<<16)|child; // Observed native quantity controls; child14 is keyboard-hint text.
    let (&id,_)=c.interfaces.widgets.iter().find(|(id,w)|plain_ui(&w.text).trim()==label && c.interfaces.runtime.get(*id).map_or(**id,|r|r.logical_id)==control && layout.widgets.get(*id).is_some_and(|p|!p.hidden && p.clip[2]>p.clip[0] && p.clip[3]>p.clip[1])).ok_or_else(||format!("native quantity control{label} absent"))?;
    let p=&layout.widgets[&id];let point=[(p.clip[0]+p.clip[2])/2,(p.clip[1]+p.clip[3])/2];
    let menu=c.interface_menu(point)?;
    for entry in &menu{println!("PG_GLASS_QUANTITY_MENU label={label} option={:?} target={:?}",entry.option,entry.target);}
    let select=menu.into_iter().find(|e|matches!(e.action,Action::Interface{id,..} if c.interfaces.runtime.get(&id).map_or(id,|r|r.logical_id)==control)).map(|e|e.action);
    c.input.pointer=point;c.input.button(0,true);c.input.button(0,false);c.tick_interfaces(false)?;c.input.begin_cycle();
    if let Some(select)=select{dispatch(c,select,&format!("native glass quantity{label}"))?;}else{println!("PG_GLASS_QUANTITY_CLICK label={label} native_pointer={point:?}; no matching operation, actual click listener/default selection must produce requested count");}
    pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
    if label=="X" {
        for character in n.to_string().encode_utf16(){c.input.character(character);c.tick_interfaces(false)?;c.input.begin_cycle();ticks(c,l,1)?;}
        c.input.key(84,true);c.tick_interfaces(false)?;c.input.key(84,false);c.input.begin_cycle();
        println!("PG_GLASS_TYPED_X count={n} native_key84=true distinct_character_and_enter_frames=true");pump(c,l,30,|c,l|!l.scripts_busy() && c.interfaces.scripts.is_empty())?;
    }
    let p=sent_packet(c,action,&format!("native glass quantity{label} Make"))?;
    assert_eq!(p.0,115);assert_eq!(p.1.len(),6);assert_eq!(u16::from_be_bytes([p.1[4],p.1[5]]) as i32,n,"native requested count differs from actual control");Ok(p)
}
fn glass_quantities(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let mut fine=130344311i32;tab(c,"Inventory")?;ticks(c,l,3)?;
    for (index,(label,count,output,award)) in [("1",1,1919,875),("5",5,4527,950),("10",10,4525,1250),("X",3,229,1750),("All",4,6667,2125)].into_iter().enumerate(){
        ready(c,l)?;let raws=(0..28).filter(|&s|c.inventories.item(93,s).id==1775).collect::<Vec<_>>();let slot=if index%2==0{raws[0]}else{*raws.last().ok_or("glass raw absent")?};
        if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}
        let p=glass_quantity(c,l,index as u32,label,count)?;fine+=count*award;
        pump(c,l,90,|c,_|c.inventories.total(93,output)==count && c.data.experience[12]==fine/10)?;ticks(c,l,4)?;
        assert_eq!(c.inventories.item(93,0).id,1785);assert_eq!(c.inventories.item(93,1).id,1785);assert_eq!(c.inventories.item(93,27).count,1000);
        assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);let committed=c.data.experience;
        replay(c,l,&p,"stale glass batch response after completed dialog")?;ticks(c,l,4)?;assert_eq!(c.data.experience,committed);
        for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_GLASS_BATCH_COMMITTED label={label} count={count} selected_slot={slot} output={output} fine_award={} display_xp={}",count*award,c.data.experience[12]);
    }
    assert_eq!(c.inventories.total(93,1775),0);println!("PG_CASE_PASS native glass quantities1/5/10/X3/All4 consumed23raws; retained pipes;31875fineXP; exact actualResume115counts; stale batches refused");Ok(())
}
fn glass_all(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,26,0)?;let packet=glass_quantity(c,l,4,"All",23)?;
    pump(c,l,110,|c,_|c.inventories.total(93,6667)==23 && c.data.experience[12]==(130344311+48875)/10)?;ticks(c,l,5)?;
    let committed=c.data.experience;replay(c,l,&packet,"stale completed native All23 glass batch")?;ticks(c,l,6)?;assert_eq!(c.data.experience,committed);
    assert_eq!(c.inventories.total(93,1775),0);assert_eq!(c.inventories.item(93,0).id,1785);assert_eq!(c.inventories.item(93,1).id,1785);assert_eq!(c.inventories.item(93,27).count,1000);
    for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
    println!("PG_CASE_PASS native All23 emptyFishbowls; exact generated/transmitted count23; selected26first, full28 retained pipes and48875fineXP; stale batch refused. Xblocked separately, not retried here");Ok(())
}
fn glass_partial_start(c:&mut ClientState,l:&mut Connection)->R<i32> {
    let xp=c.data.experience;tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,26,0)?;let _=glass_make_action(c,l,0)?;
    for component in [(269<<16)|15,(270<<16)|99] {
        let packet=gielinor_client::interface_action::resume(component,1);println!("PG_NEGATIVE_GLASS_RESUME component={component} opcode={} payload={:02x?}",packet.opcode,packet.payload);l.session.send(packet.opcode,&packet.payload)?;
    }
    ticks(c,l,5)?;assert_eq!(c.data.experience,xp);assert_eq!(c.inventories.total(93,1775),23);assert_eq!(c.inventories.total(93,1919),0);
    dispatch(c,Action::Close,"native Close after foreign/index refusal")?;ticks(c,l,3)?;ready(c,l)?;
    use_slots(c,26,0)?;let _=glass_quantity(c,l,0,"All",23)?;
    pump(c,l,30,|c,_|{let made=c.inventories.total(93,1919);made>0 && c.inventories.total(93,1775)==23-made && c.data.experience[12]==(130344311+875*made)/10})?;
    let made=c.inventories.total(93,1919);assert!(made>0 && made<23);assert_eq!(c.inventories.item(93,0).id,1785);assert_eq!(c.inventories.item(93,1).id,1785);
    println!("PG_GLASS_PARTIAL observed_products={made} remaining_raw={} display_xp={} tick={}; SQL/trace owns any already-admitted race",23-made,c.data.experience[12],c.server_tick);Ok(made)
}
fn glass_interrupt(c:&mut ClientState,l:&mut Connection)->R<()> {
    let made=glass_partial_start(c,l)?;
    println!("PG_DISCONNECT_PARTIAL glass outputs={made}; actual socket closes immediately with captured All work remaining");Ok(())
}
fn glass_crash(c:&mut ClientState,l:&mut Connection)->R<()> {
    let made=glass_partial_start(c,l)?;println!("PG_CRASH_READY glass unsaved All products={made}; real socket remains open; no normal logout/save requested");
    let start=Instant::now();loop{
        match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}
        if start.elapsed()>Duration::from_secs(30){return Err("owned glass crash injection did not arrive".into());}std::thread::sleep(Duration::from_millis(10));
    }
}
fn glass_remaining(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let mut fine=130344311i32;tab(c,"Inventory")?;ticks(c,l,3)?;
    for (index,label,count,slot,output,award) in [(3,"X",3,26,229,1750),(4,"All",20,17,6667,2125)] {
        ready(c,l)?;
        if label=="X"{use_slots(c,slot,0)?;}else{use_slots(c,0,slot)?;}
        let packet=glass_quantity(c,l,index,label,count)?;fine+=count*award;
        pump(c,l,100,|c,_|c.inventories.total(93,output)==count && c.data.experience[12]==fine/10)?;ticks(c,l,4)?;
        let committed=c.data.experience;replay(c,l,&packet,"stale completed native remaining glass batch")?;ticks(c,l,4)?;assert_eq!(c.data.experience,committed);
        assert_eq!(c.inventories.item(93,0).id,1785);assert_eq!(c.inventories.item(93,1).id,1785);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
        for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_GLASS_REMAINING_COMMITTED label={label} count={count} selected_slot={slot} output={output} fine_award={} display_xp={}",count*award,c.data.experience[12]);
    }
    assert_eq!(c.inventories.total(93,1775),0);println!("PG_CASE_PASS remaining native glassX3/All20; genuine typed amount and server-derived All;23outputs/47750fineXP;1/5/10campaign retained separately");Ok(())
}
fn glass(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let initial=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();tab(c,"Inventory")?;ticks(c,l,3)?;
    use_slots(c,0,4)?;let a=glass_make_action(c,l,0)?;let stale=sent_packet(c,a,"native glass Make before cancellation")?;
    let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native glass pending Walk cancellation")?;ticks(c,l,7)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),initial);
    use_slots(c,4,0)?;let _=glass_make_action(c,l,0)?;dispatch(c,Action::Close,"genuine native glass Close")?;ticks(c,l,3)?;
    replay(c,l,&stale,"stale glass Make after genuine native Close")?;ticks(c,l,5)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),initial);
    println!("PG_CASE_PASS native glass queued movement and genuine Close/stale continuation preserve complete inventory and XP");
    let mut fine=130344311i32;
    for (index,(output,award)) in [(1919,875),(4527,950),(4525,1250),(229,1750),(6667,2125),(567,2625),(4542,2750),(10980,3500)].into_iter().enumerate(){
        ready(c,l)?;let slot=4+index as i32;
        if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}
        let a=glass_make_action(c,l,index as u32)?;let p=sent_packet(c,a,"native glass chosen one product")?;fine+=award;
        pump(c,l,30,|c,_|c.inventories.item(93,slot).id==output && c.data.experience[12]==fine/10)?;ticks(c,l,2)?;let committed=c.data.experience;
        replay(c,l,&p,"stale glass quantity response after completed dialog")?;ticks(c,l,3)?;assert_eq!(c.data.experience,committed);
        assert_eq!(c.inventories.item(93,0).id,1785);assert_eq!(c.inventories.item(93,1).id,1785);assert_eq!(c.inventories.item(93,26).id,1775);assert_eq!(c.inventories.item(93,27).count,1000);
        for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_GLASS_COMMITTED index={index} slot={slot} output={output} fine_award={award} display_xp={}",c.data.experience[12]);
    }
    println!("PG_CASE_PASS eight native glass choices; both use directions; full28 exact selected outputs/retained pipes;15825fineXP; stale response refusal; batch quantity controls remain separate");Ok(())
}
fn shells(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;let initial=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();
    assert_eq!(c.data.base[12],99);tab(c,"Inventory")?;ticks(c,l,3)?;
    use_slots(c,0,4)?;let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native shell cancellation before completion")?;ticks(c,l,7)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),initial);
    println!("PG_CASE_PASS native shell pending cancellation without consumption or XP");
    let mut fine=130344311i32;
    for second in 0..2 {for index in 0..9 {
        ready(c,l)?;let raw=3345+2*index;let cut=3327+2*index;let slot=4+2*index+second;let reverse=(index+second)%2==1;
        assert_eq!(c.inventories.item(93,slot).id,raw);let n=c.outbound.len();
        if reverse{use_slots(c,slot,0)?;}else{use_slots(c,0,slot)?;}
        let p=c.outbound.iter().skip(n).find(|p|p.opcode==27).ok_or("shell native pair packet absent")?;let stale=(p.opcode,p.payload.clone());
        fine+=1625;pump(c,l,30,|c,_|c.inventories.item(93,slot).id==cut && c.data.experience[12]==fine/10)?;ticks(c,l,2)?;
        assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);
        assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
        if second==0 {
            assert_eq!(c.inventories.item(93,slot+1).id,raw);let committed=c.data.experience;
            replay(c,l,&stale,"stale shell pair with matching raw and chisel duplicates retained")?;ticks(c,l,4)?;
            assert_eq!(c.data.experience,committed);assert_eq!(c.inventories.item(93,slot+1).id,raw);
        }
        for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_SHELL_COMMITTED index={index} slot={slot} raw={raw} cut={cut} fine_award=1625 display_xp={} direction_reverse={reverse}",c.data.experience[12]);
    }}
    println!("PG_CASE_PASS eighteen native shell carvings; nine exact round pointed bark pairs both directions; full28 retained chisels; exact cumulative fineXP; stale selected-pair refusal");Ok(())
}
fn gem_crash(c:&mut ClientState,l:&mut Connection)->R<()> {
    tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,0,4)?;
    pump(c,l,30,|c,_|c.inventories.item(93,4).id==1607 && c.data.experience[12]==13034681)?;
    println!("PG_CRASH_READY gem unsaved selected Sapphire result observed; real socket remains open; no normal logout/save acknowledgement requested");
    let start=Instant::now();loop {
        match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}
        if start.elapsed()>Duration::from_secs(30){return Err("owned gem crash injection did not arrive".into());}std::thread::sleep(Duration::from_millis(10));
    }
}
fn food(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[3],10);println!("PG_INITIAL food tick={} current_hp={} display_xp={}; initial HP2 and any pre-action regeneration must be checked in received packets",c.server_tick,c.data.boosted[3],c.data.experience[3]);tab(c,"Inventory")?;ticks(c,l,2)?;
    for (index,(slot,id,remainder)) in [(7,2142,-1),(3,1891,1893),(3,1893,1895),(3,1895,-1),(5,2289,2291),(5,2291,-1)].into_iter().enumerate() {
        ready(c,l)?;assert_eq!(c.inventories.item(93,slot).id,id);
        let widget=c.interfaces.runtime.iter().find(|(_,r)|r.logical_id==149<<16 && r.child_index==slot as i32).map(|(&id,_)|id).ok_or("food widget missing")?;
        println!("PG_FOOD_BEGIN index={index} tick={} slot={slot} id={id} hp={} xp={}",c.server_tick,c.data.boosted[3],c.data.experience[3]);
        let before=c.outbound.len();assert!(c.interface_operation(widget,2,None)?);
        for p in c.outbound.iter().skip(before){println!("PG_TX label=Eat index={index} opcode={} payload={:02x?}",p.opcode,p.payload);}
        pump(c,l,30,|c,_|c.inventories.item(93,slot).id==remainder)?;
        assert_eq!(c.data.experience,xp);assert_eq!(c.inventories.item(93,1).id,1891);assert_eq!(c.inventories.item(93,4).id,2289);assert_eq!(c.inventories.item(93,27).count,1000);
        println!("PG_FOOD_COMMITTED index={index} tick={} slot={slot} remainder={remainder} hp={} busy={} scripts={}",c.server_tick,c.data.boosted[3],l.scripts_busy(),c.interfaces.scripts.len());ticks(c,l,6)?;
    }
    println!("PG_CASE_PASS food six actual selected-slot native actions; HP correctness requires received-packet/server-trace assessment, not a relaxed HP oracle");Ok(())
}
fn immediate_tools(c:&mut ClientState,l:&mut Connection,kind:&str)->R<()> {
    let staff=kind=="immediate-staff";let certainty=kind=="immediate-certainty";let crash=kind=="immediate-crash";
    let (tool,raw)=if staff{(946,771)}else{(1755,3211)};let xp=c.data.experience;
    assert_eq!(c.data.base[12],if staff{99}else if certainty{40}else{12});
    let mut fine=if staff{130344311}else if certainty{372240}else{15840};
    assert_eq!(c.data.experience[12],fine/10);tab(c,"Inventory")?;ticks(c,l,3)?;
    let mut successes=0;let mut failures=0;
    for index in 0..if crash{1}else if staff{3}else{12}{
        ready(c,l)?;let slot=4+index;assert_eq!(c.inventories.item(93,slot).id,raw);
        let n=c.outbound.len();let messages=c.game_messages.len();let start_tick=c.server_tick;
        if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}
        let p=c.outbound.iter().skip(n).find(|p|p.opcode==27).ok_or("immediate native held packet absent")?;let stale=(p.opcode,p.payload.clone());
        pump(c,l,30,|c,_|c.inventories.item(93,slot).id!=raw)?;
        let out=c.inventories.item(93,slot).id;
        let award=if staff{assert_eq!(out,772);0}else if out==3420{successes+=1;300}else{assert_eq!(out,968);assert!(!certainty);failures+=1;75};fine+=award;
        pump(c,l,30,|c,_|c.data.experience[12]==fine/10)?;ticks(c,l,2)?;
        let message=if staff{"You carve the Dramen staff."}else if out==3420{"You cut the limestone brick."}else{"You fail to cut the limestone into a brick."};
        assert!(c.game_messages.iter().skip(messages).any(|m|m.text==message),"expected immediate game message missing: {message}");
        for m in c.game_messages.iter().skip(messages){println!("PG_IMMEDIATE_MESSAGE kind={kind} index={index} tick={} message={:?}",c.server_tick,m.text);}
        let before=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();let committed=c.data.experience;
        replay(c,l,&stale,"stale immediate selected held pair with duplicate raw retained")?;ticks(c,l,3)?;
        assert_eq!(c.data.experience,committed);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),before);
        assert_eq!(c.inventories.item(93,0).id,tool);assert_eq!(c.inventories.item(93,1).id,tool);assert_eq!(c.inventories.item(93,25).id,raw);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);
        for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        assert!(!c.outbound.iter().skip(n).any(|p|p.opcode==115),"immediate route unexpectedly required production response");
        println!("PG_IMMEDIATE_COMMITTED kind={kind} index={index} selected_slot={slot} raw={raw} output={out} fine_award={award} fine_total={fine} action_tick={start_tick} observation_tick={} display_xp={} current_level={} base_level={}",c.server_tick,c.data.experience[12],c.data.boosted[12],c.data.base[12]);
    }
    if crash{
        println!("PG_CRASH_READY immediate one unsaved limestone credit and fineXP observed; socket held open; no save/logout requested");let start=Instant::now();loop{
            match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}
            if start.elapsed()>Duration::from_secs(30){return Err("owned immediate crash injection absent".into());}std::thread::sleep(Duration::from_millis(10));
        }
    }
    println!("PG_CASE_PASS native immediate kind={kind} successes={successes} failures={failures}; full28 both directions stale captured pair refusal; retained tools and duplicate raw; exact rewards; natural outcome branch coverage assessed separately");Ok(())
}
fn fallible_gems(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[12],16);assert_eq!(xp[12],2746);let mut fine=27460;tab(c,"Inventory")?;ticks(c,l,3)?;
    let initial=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();
    use_slots(c,0,4)?;let a=glass_make_action(c,l,0)?;let old=sent_packet(c,a,"native fallible Make before Close")?;dispatch(c,Action::Close,"native fallible Close before award")?;ticks(c,l,3)?;replay(c,l,&old,"native stale fallible Make after Close")?;ticks(c,l,4)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),initial);
    println!("PG_CASE_PASS native fallible Make/Close and stale response preserve full inventory/XP");
    for(index,(raw,out,success_xp,fail_xp,name))in[(1625,1609,150,38,"Opal"),(1627,1611,200,50,"Jade"),(1629,1613,250,63,"Red topaz")].into_iter().enumerate(){
        let first=4+7*index as i32;
        for second in[false,true]{ready(c,l)?;let slot=if second{first}else{first+6};let before=c.outbound.len();let messages=c.game_messages.len();
            if second{use_slots(c,slot,0)?;}else{use_slots(c,0,slot)?;}
            let p=c.outbound.iter().skip(before).find(|p|p.opcode==27).ok_or("native fallible held packet missing")?;let held=(p.opcode,p.payload.clone());let count=if second{6}else{1};let make=glass_quantity(c,l,0,if second{"All"}else{"1"},count)?;
            pump(c,l,60,|c,_|c.inventories.total(93,raw)==if second{0}else{6})?;let slots=if second{(first..first+6).collect::<Vec<_>>()}else{vec![slot]};let mut good=0;let mut bad=0;
            for &s in &slots{let item=c.inventories.item(93,s);assert_eq!(item.count,1);if item.id==out{good+=1;}else{assert_eq!(item.id,1633);bad+=1;}}
            let award=(good*success_xp+bad*fail_xp)*5;fine+=award;pump(c,l,30,|c,_|c.data.experience[12]==fine/10)?;ticks(c,l,2)?;
            let expected_good=format!("You cut the {name}.");assert_eq!(c.game_messages.iter().skip(messages).filter(|m|m.text==expected_good).count(),good as usize);assert_eq!(c.game_messages.iter().skip(messages).filter(|m|m.text=="You accidentally crush the gem.").count(),bad as usize);
            for m in c.game_messages.iter().skip(messages){println!("PG_FALLIBLE_MESSAGE index={index} second={second} message={:?}",m.text);}
            let committed=c.data.experience;let inventory=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();replay(c,l,&make,"native stale completed fallible quantity response")?;replay(c,l,&held,"native stale selected fallible held pair")?;ticks(c,l,4)?;assert_eq!(c.data.experience,committed);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),inventory);
            assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
            println!("PG_FALLIBLE_BATCH index={index} second={second} raw={raw} cut={out} selected_slot={slot} count={count} good={good} crushed={bad} fine_award={award} fine_total={fine} display_xp={} current_level={} base_level={}",c.data.experience[12],c.data.boosted[12],c.data.base[12]);
        }
    }
    println!("PG_CASE_PASS native fallible three profiles both held directions; genuine1/All captured batches;21selected cut/crushed outputs; exact rewards/messages/stale refusals; distribution and customXacceptance remain separate");Ok(())
}
fn crabs(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[12],15);assert_eq!(xp[12],2411);let mut fine=24110;tab(c,"Inventory")?;ticks(c,l,3)?;
    let initial=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();
    use_slots(c,0,2)?;let a=glass_make_action(c,l,0)?;let old=sent_packet(c,a,"native crab Make before Close")?;dispatch(c,Action::Close,"native crab Close before award")?;ticks(c,l,3)?;replay(c,l,&old,"native stale fallible Make after Close")?;ticks(c,l,4)?;
    assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),initial);
    println!("PG_CASE_PASS native crab Make/Close and stale response preserve full inventory/XP");
    for(index,(raw,out,broken,name))in[(7538,7539,7541,"Crab helmet"),(7536,7537,7540,"Crab claw")].into_iter().enumerate(){
        let first=2+12*index as i32;
        for second in[false,true]{ready(c,l)?;let slot=if second{first}else{first+11};let before=c.outbound.len();let messages=c.game_messages.len();
            if second{use_slots(c,slot,0)?;}else{use_slots(c,0,slot)?;}
            let p=c.outbound.iter().skip(before).find(|p|p.opcode==27).ok_or("native crab held packet missing")?;let held=(p.opcode,p.payload.clone());let count=if second{11}else{1};let make=glass_quantity(c,l,0,if second{"All"}else{"1"},count)?;
            pump(c,l,60,|c,_|c.inventories.total(93,raw)==if second{0}else{11})?;let slots=if second{(first..first+11).collect::<Vec<_>>()}else{vec![slot]};let mut good=0;let mut bad=0;
            for &s in &slots{let item=c.inventories.item(93,s);assert_eq!(item.count,1);if item.id==out{good+=1;}else{assert_eq!(item.id,broken);bad+=1;}}
            let award=good*1625;fine+=award;pump(c,l,30,|c,_|c.data.experience[12]==fine/10)?;ticks(c,l,2)?;
            let expected_good=format!("You carve the {name}.");assert_eq!(c.game_messages.iter().skip(messages).filter(|m|m.text==expected_good).count(),good as usize);assert_eq!(c.game_messages.iter().skip(messages).filter(|m|m.text=="You accidentally break the crab material.").count(),bad as usize);
            for m in c.game_messages.iter().skip(messages){println!("PG_CRAB_MESSAGE index={index} second={second} message={:?}",m.text);}
            let committed=c.data.experience;let inventory=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();replay(c,l,&make,"native stale completed fallible quantity response")?;replay(c,l,&held,"native stale selected fallible held pair")?;ticks(c,l,4)?;assert_eq!(c.data.experience,committed);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),inventory);
            assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
            println!("PG_CRAB_BATCH index={index} second={second} raw={raw} cut={out} selected_slot={slot} count={count} good={good} crushed={bad} fine_award={award} fine_total={fine} display_xp={} current_level={} base_level={}",c.data.experience[12],c.data.boosted[12],c.data.base[12]);
        }
    }
    println!("PG_CASE_PASS native crab two profiles both held directions; genuine1/All captured batches;24selected carved/broken outputs; exact rewards/messages/stale refusals; distribution and customXacceptance remain separate");Ok(())
}
fn crab_quantities(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[12],15);assert_eq!(xp[12],2411);let mut fine=24110;tab(c,"Inventory")?;ticks(c,l,3)?;
    for(index,(slot,count,label,remaining))in[(26,5,"5",18),(8,10,"10",8)].into_iter().enumerate(){ready(c,l)?;let before=c.outbound.len();let messages=c.game_messages.len();if index==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}let p=c.outbound.iter().skip(before).find(|p|p.opcode==27).ok_or("native crab quantity held packet missing")?;let held=(p.opcode,p.payload.clone());let make=glass_quantity(c,l,0,label,count)?;pump(c,l,90,|c,_|c.inventories.total(93,7538)==remaining)?;
        let slots=if index==0{vec![26,4,5,6,7]}else{(8..18).collect::<Vec<_>>()};let mut good=0;let mut broken=0;for s in slots{let item=c.inventories.item(93,s);assert_eq!(item.count,1);if item.id==7539{good+=1;}else{assert_eq!(item.id,7541);broken+=1;}}let award=good*1625;fine+=award;pump(c,l,30,|c,_|c.data.experience[12]==fine/10)?;ticks(c,l,2)?;
        assert_eq!(c.game_messages.iter().skip(messages).filter(|m|m.text=="You carve the Crab helmet.").count(),good as usize);assert_eq!(c.game_messages.iter().skip(messages).filter(|m|m.text=="You accidentally break the crab material.").count(),broken as usize);
        let saved=c.data.experience;let inv=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();replay(c,l,&make,"native stale completed crab builtin quantity")?;replay(c,l,&held,"native stale selected crab builtin held pair")?;ticks(c,l,4)?;assert_eq!(c.data.experience,saved);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),inv);assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}
        println!("PG_CRAB_QUANTITY index={index} selected_slot={slot} requested={count} good={good} broken={broken} fine_award={award} fine_total={fine} remaining={remaining}");
    }
    println!("PG_CASE_PASS native crab builtin5then10genuine quantity frames;15mixed selected products/both held directions/full28and8raws retained; exact rewards/messages/stale refusals; customXremains separate");Ok(())
}
fn crab_partial(c:&mut ClientState,l:&mut Connection,crash:bool)->R<()> {
    assert_eq!(c.data.base[12],15);assert_eq!(c.data.experience[12],2411);tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,26,0)?;let _=glass_quantity(c,l,0,"All",23)?;
    pump(c,l,30,|c,_|{let mut fine=24110;let mut made=0;for s in 4..27{match c.inventories.item(93,s).id{7539=>{fine+=1625;made+=1;},7541=>{made+=1;},_=>{}}}made>0 && made<23 && c.data.experience[12]==fine/10})?;
    let mut fine=24110;let mut made=0;let mut good=0;let mut bad=0;for s in 4..27{match c.inventories.item(93,s).id{7539=>{fine+=1625;made+=1;good+=1;},7541=>{made+=1;bad+=1;},7538=>{},_=>return Err("unexpected native partial crab item".into())}}
    assert_eq!(c.data.experience[12],fine/10);assert!(made>0 && made<23);assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);
    println!("PG_CRAB_INTERRUPT_READY tick={} made={made} good={good} broken={bad} fine_observed={fine}; real socket disconnect during captured All, including valid zero-XP broken credits; SQL may retain another already-admitted complete result",c.server_tick);
    if crash{println!("PG_CRASH_READY native crab captured All observed unsaved prefix; only this owned disposable game server is under test");let start=Instant::now();loop{match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}if start.elapsed()>Duration::from_secs(30){return Err("owned crab recovery-test termination absent".into());}std::thread::sleep(Duration::from_millis(10));}}
    Ok(())
}
fn fallible_partial(c:&mut ClientState,l:&mut Connection)->R<()> {
    assert_eq!(c.data.base[12],16);assert_eq!(c.data.experience[12],2746);tab(c,"Inventory")?;ticks(c,l,3)?;use_slots(c,26,0)?;let _=glass_quantity(c,l,0,"All",23)?;
    pump(c,l,30,|c,_|{let mut fine=27460;let mut made=0;for s in 4..27{match c.inventories.item(93,s).id{1609=>{fine+=750;made+=1;},1633=>{fine+=190;made+=1;},_=>{}}}made>0 && made<23 && c.data.experience[12]==fine/10})?;
    let mut fine=27460;let mut made=0;let mut good=0;let mut bad=0;
    for s in 4..27{match c.inventories.item(93,s).id{1609=>{fine+=750;made+=1;good+=1;},1633=>{fine+=190;made+=1;bad+=1;},1625=>{},_=>return Err("unexpected native fallible partial output".into())}}
    assert_eq!(c.data.experience[12],fine/10);assert!(made>0 && made<23);assert_eq!(c.inventories.item(93,0).id,1755);assert_eq!(c.inventories.item(93,1).id,1755);assert_eq!(c.inventories.item(93,27).count,1000);
    println!("PG_FALLIBLE_INTERRUPT_READY tick={} made={made} good={good} crushed={bad} fine_observed={fine}; real socket disconnect during captured All; SQL may retain additional already-admitted credits before disconnect processed",c.server_tick);Ok(())
}
fn sinew(c:&mut ClientState,l:&mut Connection,crash:bool)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[12],99);tab(c,"Inventory")?;ticks(c,l,3)?;
    if !crash{let before=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();use_slots(c,0,4)?;let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native sinew pending Walk cancellation")?;ticks(c,l,6)?;assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),before);println!("PG_CASE_PASS native sinew pending movement cancels without inventory/XP mutation");}
    let mut fine=130344311;
    for(index,slot)in(if crash{vec![4]}else{vec![4,26,9,17]}).into_iter().enumerate(){ready(c,l)?;let before=c.outbound.len();let start=c.server_tick;if index%2==0{use_slots(c,0,slot)?;}else{use_slots(c,slot,0)?;}let p=c.outbound.iter().skip(before).find(|p|p.opcode==27).ok_or("native sinew pair packet missing")?;let stale=(p.opcode,p.payload.clone());fine+=750;
        pump(c,l,30,|c,_|c.inventories.item(93,slot).id==9436 && c.data.experience[12]==fine/10)?;ticks(c,l,2)?;let saved=c.data.experience;let inv=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();replay(c,l,&stale,"native stale selected sinew pair with matching tails retained")?;ticks(c,l,3)?;assert_eq!(c.data.experience,saved);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),inv);
        assert_eq!(c.inventories.item(93,0).id,946);assert_eq!(c.inventories.item(93,1).id,946);assert_eq!(c.inventories.item(93,27).count,1000);assert_eq!(c.inventories.total(93,9436),index as i32+1);assert_eq!(c.inventories.total(93,19665),22-index as i32);assert_eq!((0..28).filter(|&s|c.inventories.item(93,s).id>=0).count(),28);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}assert!(!c.outbound.iter().skip(before).any(|p|p.opcode==115));
        println!("PG_SINEW_COMMITTED index={index} slot={slot} raw=19665 output=9436 fine_award=750 fine_total={fine} action_tick={start} observation_tick={} display_xp={}",c.server_tick,c.data.experience[12]);
    }
    if crash{println!("PG_CRASH_READY native manual sinew one unsaved selected product and750fineXP; only this disposable local server is under test; socket held open");let start=Instant::now();loop{match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}if start.elapsed()>Duration::from_secs(30){return Err("owned sinew recovery-test termination absent".into());}std::thread::sleep(Duration::from_millis(10));}}
    println!("PG_CASE_PASS native manual sinew four selected credits both held directions; full28 retained knives/coins/19raws;3000fineXP; stale selected refusal; no quantity response or automatic repeats");Ok(())
}
fn battlestaves(c:&mut ClientState,l:&mut Connection,crash:bool)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[12],99);tab(c,"Inventory")?;ticks(c,l,3)?;
    if !crash{let inv=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();use_slots(c,0,22)?;let point=[u16::try_from(c.data.position[1]+1)?,u16::try_from(c.data.position[2])?];dispatch(c,Action::Walk(point),"native battlestaff pending Walk cancellation")?;ticks(c,l,6)?;assert_eq!(c.data.experience,xp);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),inv);println!("PG_CASE_PASS native battlestaff pending Walk cancellation");}
    let mut fine=130344311;let mut actions=0;
    for(index,(orb,out,award))in[(571,1395,5000),(575,1399,5625),(569,1393,6250),(573,1397,6875)].into_iter().enumerate(){for direction in 0..2{if crash && actions>0{continue;}ready(c,l)?;let staff_slot=4+index as i32*4+direction;let orb_slot=6+index as i32*4+direction;assert_eq!(c.inventories.item(93,staff_slot).id,1391);assert_eq!(c.inventories.item(93,orb_slot).id,orb);let before=c.outbound.len();if direction==0{use_slots(c,staff_slot,orb_slot)?;}else{use_slots(c,orb_slot,staff_slot)?;}let p=c.outbound.iter().skip(before).find(|p|p.opcode==27).ok_or("battlestaff held pair packet missing")?;let stale=(p.opcode,p.payload.clone());fine+=award;
        pump(c,l,30,|c,_|c.inventories.total(93,out)==direction+1 && c.data.experience[12]==fine/10)?;ticks(c,l,2)?;let inv=(0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>();let saved=c.data.experience;replay(c,l,&stale,"native stale selected battlestaff pair with duplicate ingredients retained")?;ticks(c,l,3)?;assert_eq!(c.data.experience,saved);assert_eq!((0..28).map(|s|{let i=c.inventories.item(93,s);(i.id,i.count)}).collect::<Vec<_>>(),inv);assert_eq!(c.inventories.item(93,0).id,1391);assert_eq!(c.inventories.item(93,1).id,1391);assert_eq!(c.inventories.item(93,22+index as i32).id,orb);assert_eq!(c.inventories.item(93,27).count,1000);for i in 0..25{if i!=12{assert_eq!(c.data.experience[i],xp[i]);}}assert!(!c.outbound.iter().skip(before).any(|p|p.opcode==115));actions+=1;println!("PG_BATTLESTAFF_COMMITTED index={index} direction={direction} staff_slot={staff_slot} orb_slot={orb_slot} output={out} fine_award={award} fine_total={fine}");
    }}
    if crash{println!("PG_CRASH_READY native battlestaff one unsaved selected product and5000fineXP; only the disposable owned game server is under test");let start=Instant::now();loop{match l.poll(c){Err(e)=>{println!("PG_EXPECTED_CRASH_DISCONNECT {e}");return Ok(());},Ok(())=>{if l.logged_out().is_some(){println!("PG_EXPECTED_CRASH_DISCONNECT native transport closed");return Ok(());}}}if start.elapsed()>Duration::from_secs(30){return Err("owned battlestaff recovery-test termination absent".into());}std::thread::sleep(Duration::from_millis(10));}}
    assert_eq!(actions,8);println!("PG_CASE_PASS native battlestaff eight manual actions both directions four profiles47500fineXP; stale refusal retained duplicates and no quantity/repeat");Ok(())
}
fn main()->R<()> {
    let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),7);
    let mut c=ClientState::load(DiskCache::open(&a[1],64<<20)?)?;let mut l=Connection::connect(&mut c,&a[2],&a[3],&a[4],&a[5])?;ready(&mut c,&mut l)?;
    if a[6]=="crab-carving"{crabs(&mut c,&mut l)?;l.session.close();return Ok(());}
    if a[6]=="crab-quantities"{crab_quantities(&mut c,&mut l)?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"crab-interrupt"|"crab-crash"){crab_partial(&mut c,&mut l,a[6]=="crab-crash")?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"battlestaves"|"battlestaff-crash"){battlestaves(&mut c,&mut l,a[6]=="battlestaff-crash")?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"sinew"|"sinew-crash"){sinew(&mut c,&mut l,a[6]=="sinew-crash")?;l.session.close();return Ok(());}
    if a[6]=="fallible-gems"{fallible_gems(&mut c,&mut l)?;l.session.close();return Ok(());}
    if a[6]=="fallible-gem-interrupt"{fallible_partial(&mut c,&mut l)?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"immediate-staff"|"immediate-limestone"|"immediate-certainty"|"immediate-crash"){immediate_tools(&mut c,&mut l,&a[6])?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"production-gems"|"production-shells"){production_profiles(&mut c,&mut l,a[6]=="production-shells")?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"production-gem-batch"|"production-shell-batch"){production_batch(&mut c,&mut l,a[6]=="production-shell-batch")?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"production-gem-x"|"production-gem-x-focus"){production_x_boundary(&mut c,&mut l,a[6]=="production-gem-x-focus")?;l.session.close();return Ok(());}
    if matches!(a[6].as_str(),"production-gem-interrupt"|"production-shell-interrupt"|"production-gem-crash"){production_partial(&mut c,&mut l,a[6]=="production-shell-interrupt",a[6]=="production-gem-crash")?;l.session.close();return Ok(());}
    match a[6].as_str(){"pouch"=>pouch(&mut c,&mut l)?,"glass-all"=>glass_all(&mut c,&mut l)?,"glass-interrupt"=>glass_interrupt(&mut c,&mut l)?,"glass-crash"=>glass_crash(&mut c,&mut l)?,"glass-remaining"=>glass_remaining(&mut c,&mut l)?,"glass-quantities"=>glass_quantities(&mut c,&mut l)?,"glass"=>glass(&mut c,&mut l)?,"shells"=>shells(&mut c,&mut l)?,"gem-effects"=>gem_effects(&mut c,&mut l)?,"gem-crash"=>gem_crash(&mut c,&mut l)?,"gems"=>gems(&mut c,&mut l)?,"prayer"=>prayer_burial(&mut c,&mut l)?,"chem-crash"=>chemistry_crash(&mut c,&mut l)?,"chem-interrupt"=>chemistry_interrupt(&mut c,&mut l)?,"food-negative"=>food_negative(&mut c,&mut l)?,"combat-interrupt"=>combat_interrupt(&mut c,&mut l)?,"chem-close"=>chemistry_close(&mut c,&mut l)?,"chem-negative"=>chemistry(&mut c,&mut l)?,"food"=>food(&mut c,&mut l)?,"verify"=>{ticks(&mut c,&mut l,6)?;println!("PG_CASE_PASS native relogin observation");},_=>return Err("unknown independent scenario".into())}
    l.session.close();Ok(())
}
