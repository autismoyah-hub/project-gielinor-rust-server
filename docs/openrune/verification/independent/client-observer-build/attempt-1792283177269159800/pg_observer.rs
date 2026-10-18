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
fn food(c:&mut ClientState,l:&mut Connection)->R<()> {
    let xp=c.data.experience;assert_eq!(c.data.base[3],10);assert_eq!(c.data.boosted[3],2);tab(c,"Inventory")?;ticks(c,l,2)?;
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
    match a[6].as_str(){"chem-negative"=>chemistry(&mut c,&mut l)?,"food"=>food(&mut c,&mut l)?,"verify"=>{ticks(&mut c,&mut l,6)?;println!("PG_CASE_PASS native relogin observation");},_=>return Err("unknown independent scenario".into())}
    l.session.close();Ok(())
}
