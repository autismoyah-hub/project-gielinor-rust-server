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
                if let Some(a)=c.interface_menu(point)?.into_iter().find(|e|match e.action{Action::Continue(id)|Action::Button{id,..}|Action::Interface{id,..}=>c.interfaces.runtime.get(&id).map_or(id,|r|r.logical_id)==logical,_=>false}).map(|e|e.action){
                    for (&id,w) in &c.interfaces.widgets {if id>>16==270 && !w.text.is_empty(){println!("PG_GLASS_WIDGET id={id} text={:?}",w.text);}}
                    println!("PG_GLASS_OPTION_READY index={index} logical={logical} tick={}",c.server_tick);return Ok(a);
                }
            }
        }
        if start.elapsed()>Duration::from_secs(30){return Err(format!("native glass choice{index} unavailable").into());}std::thread::sleep(Duration::from_millis(20));
    }
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
fn main()->R<()> {
    let a:Vec<_>=std::env::args().collect();assert_eq!(a.len(),7);
    let mut c=ClientState::load(DiskCache::open(&a[1],64<<20)?)?;let mut l=Connection::connect(&mut c,&a[2],&a[3],&a[4],&a[5])?;ready(&mut c,&mut l)?;
    match a[6].as_str(){"glass"=>glass(&mut c,&mut l)?,"shells"=>shells(&mut c,&mut l)?,"gem-effects"=>gem_effects(&mut c,&mut l)?,"gem-crash"=>gem_crash(&mut c,&mut l)?,"gems"=>gems(&mut c,&mut l)?,"prayer"=>prayer_burial(&mut c,&mut l)?,"chem-crash"=>chemistry_crash(&mut c,&mut l)?,"chem-interrupt"=>chemistry_interrupt(&mut c,&mut l)?,"food-negative"=>food_negative(&mut c,&mut l)?,"combat-interrupt"=>combat_interrupt(&mut c,&mut l)?,"chem-close"=>chemistry_close(&mut c,&mut l)?,"chem-negative"=>chemistry(&mut c,&mut l)?,"food"=>food(&mut c,&mut l)?,"verify"=>{ticks(&mut c,&mut l,6)?;println!("PG_CASE_PASS native relogin observation");},_=>return Err("unknown independent scenario".into())}
    l.session.close();Ok(())
}
