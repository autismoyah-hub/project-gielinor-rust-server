//! Independent native controls and packet observation. No local gameplay mutation.
use gielinor_client::{client_state::ClientState,disk_cache::DiskCache,live_runtime::Connection,context_menu::Action};
use std::time::{Duration,Instant};
type R<T> = Result<T,Box<dyn std::error::Error>>;
fn pump(c:&mut ClientState,l:&mut Connection,seconds:u64,ready:impl Fn(&ClientState,&Connection)->bool)->R<()> {
    let start=Instant::now();let mut cycle=Instant::now();let mut progress=Instant::now();
    loop {
        l.poll(c)?;let seq=sequence(c);if seq==830{println!("A5_SEQUENCE_OBSERVATION id=830 tick={}",c.server_tick);}if l.logged_out().is_some(){return Err("unexpected native disconnect".into());}
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
fn native_cape_frame(c:&mut ClientState,l:&mut Connection,name:&str)->R<()>{
    use std::io::Write;use sha2::Digest;
    let size=c.canvas;let mut frame=l.paint(c,size[0]as usize,size[1]as usize)?;
    let layout=c.interfaces.layout(c.canvas)?;let mut renderer=gielinor_client::interface_render::Renderer::load(&mut c.scripts.source.cache,c.preferences.device.brightness)?;
    renderer.minimap=l.minimap_view(c);renderer.world_drawn=l.world.is_some();renderer.player=c.player_state.as_ref().and_then(|state|state.players[state.local as usize].as_ref()).and_then(|player|player.appearance.as_ref()).map(|a|a.appearance.clone());
    let report=renderer.draw_with_variables(&mut c.scripts.source.cache,&c.items,&c.interfaces,&layout,&mut frame,Some(&c.scripts.variables))?;
    assert!(report.drawn>0,"native interface overlay drew no widgets");println!("PG_CAPE_UI_RENDER name={name} widgets={} report={report:?};actual existing viewport interface renderer",layout.widgets.len());let bytes=frame.bmp()?;
    let path=format!("{name}.bmp");let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(&path)?;file.write_all(&bytes)?;
    println!("PG_CAPE_NATIVE_FRAME name={name} width={} height={} sha256={:x} actual_current={} tick={};private native rendering,not published assets",frame.width(),frame.height(),sha2::Sha256::digest(&bytes),c.data.boosted[12],c.server_tick);Ok(())
}

fn sequence(c:&ClientState)->i32{c.player_state.as_ref().and_then(|s|s.players[s.local as usize].as_ref()).map(|p|p.sequence.id).unwrap_or(-1)}
fn main()->R<()>{
 let a=std::env::args().collect::<Vec<_>>();assert_eq!(a.len(),7);
 let mut c=ClientState::load(DiskCache::open(&a[1],64<<20)?)?;let mut l=Connection::connect(&mut c,&a[2],&a[3],&a[4],&a[5])?;
 ready(&mut c,&mut l)?;tab(&mut c,"Inventory")?;ticks(&mut c,&mut l,2)?;
 let image=|c:&ClientState,inv,size|(0..size).map(|s|{let i=c.inventories.item(inv,s);(i.id,i.count)}).collect::<Vec<_>>();let bag=image(&c,93,28);let gear=image(&c,94,14);let xp=c.data.experience;let stats=c.data.boosted;let pos=c.data.position;let text="You dig a hole in the ground... but find nothing.";
 assert!(bag.iter().all(|(_,n)|*n>0));for s in [0,7,27]{assert_eq!(bag[s],(952,1));}
 assert_eq!(c.game_messages.iter().filter(|m|m.text==text).count(),0);ticks(&mut c,&mut l,6)?;assert_eq!(c.game_messages.iter().filter(|m|m.text==text).count(),0);println!("A5_NATIVE_QUIET mode={} position={pos:?} full_inventory=true",a[6]);
 let slots:&[i32]=if a[6]=="verify"{&[7]}else{&[0,7,27]};
 for (repeat,&slot) in slots.iter().enumerate(){
  let action=item_action(&mut c,slot,false,"Dig")?;println!("A5_NATIVE_MENU slot={slot} action={action:?}");dispatch(&mut c,action,"actual native Dig")?;
  pump(&mut c,&mut l,30,|c,_|sequence(c)==830||c.game_messages.iter().filter(|m|m.text==text).count()>repeat)?;
  if sequence(&c)!=830{return Err("INCONCLUSIVE_NATIVE_SEQUENCE: message arrived before sequence830 was observed".into());}
  println!("A5_NATIVE_SEQUENCE slot={slot} id=830 tick={}",c.server_tick);
  if repeat==0{native_cape_frame(&mut c,&mut l,if a[6]=="verify"{"spade-reconnect-active"}else{"spade-active"})?;}
  pump(&mut c,&mut l,30,|c,_|c.game_messages.iter().filter(|m|m.text==text).count()>repeat)?;ticks(&mut c,&mut l,6)?;
  assert_eq!(c.game_messages.iter().filter(|m|m.text==text).count(),repeat+1);assert_eq!(image(&c,93,28),bag);assert_eq!(image(&c,94,14),gear);assert_eq!(c.data.experience,xp);assert_eq!(c.data.boosted,stats);assert_eq!(c.data.position,pos);assert_eq!(sequence(&c),-1);
  println!("A5_NATIVE_COMMIT slot={slot} messages={} tick={} sequence_reset=true unchanged_bag_gear_XP_stats_position=true",repeat+1,c.server_tick);
 }
 native_cape_frame(&mut c,&mut l,if a[6]=="verify"{"spade-reconnect-final"}else{"spade-final"})?;
 println!("A5_NATIVE_PASS spade mode={} genuine_native_Dig no_auto_repeat_reward=true",a[6]);Ok(())
}
