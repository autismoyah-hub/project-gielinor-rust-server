//! Actual loaded pouch and bank restriction; native banking remains separate.
use gielinor_cache::{game::GameCache,symbols};
use gielinor_world::{inventory::Item,simulation::PlayerData,stats::Stats,bank_actions::{Request,Event},CoordGrid};
use std::path::Path;
#[test]
fn actual_pouch_flags_bank_resolver_restricted_deposit_and_readonly_nearby_bankers(){
    let directory=std::env::var("PG_SYMBOLS").unwrap();let path=std::env::var("PG_GAME_CACHE").unwrap();let directory=Path::new(&directory);
    let objects=symbols::load(&[directory],"obj",None).unwrap();let params=symbols::load(&[directory],"param",None).unwrap();let bits=symbols::load(&[directory],"varbit",None).unwrap();
    let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let mut defs=cache.definitions_all().unwrap();
    let policy=gielinor_server::content::coin_pouches(&objects,&params,&defs).unwrap();assert_eq!(policy.len(),1);let spec=policy[&22521];assert_eq!((spec.rule.output,spec.rule.count,spec.rule.noted,spec.members),(995,3,false,true));
    let bank=gielinor_server::content::banking(&bits,&params,&defs).unwrap();assert!(bank.restricted.contains(&22521));assert!(!bank.restricted.contains(&995));
    let mut data=PlayerData::new(1,"Independent restricted deposit".into(),CoordGrid::new(3200,3200,0).unwrap(),Stats::default());bank.capacity.set(&mut data.variables,400).unwrap();
    data.inventory.restore_slot(5,Some(Item{id:22521,count:28,vars:0})).unwrap();data.inventory.restore_slot(6,Some(Item{id:22521,count:2,vars:0})).unwrap();data.inventory.restore_slot(7,Some(Item{id:995,count:10,vars:0})).unwrap();
    let catalog=defs.inventory_catalog();let before=(data.inventory.clone(),data.bank.clone(),data.stats.clone());
    let events=bank.apply(&mut data,&catalog,Request::Deposit{slot:5,count:28}).unwrap();assert!(events.iter().any(|e|matches!(e,Event::Message(s) if s.contains("prevents you from banking"))));assert_eq!((data.inventory.clone(),data.bank.clone(),data.stats.clone()),before);
    bank.apply(&mut data,&catalog,Request::Deposit{slot:7,count:10}).unwrap();assert!(data.inventory.get(7).is_none());assert_eq!(data.bank.slots().iter().flatten().filter(|i|i.id==995).map(|i|i.count).sum::<i32>(),10);assert_eq!(data.inventory.get(5).unwrap().count,28);assert_eq!(data.inventory.get(6).unwrap().count,2);assert_eq!(data.stats,before.2);
    let saved=defs.items[&22521].clone();let mut refused=0;
    for mode in 0..11{
        defs.items.insert(22521,saved.clone());let p=defs.items.get_mut(&22521).unwrap();
        match mode{0=>p.name="Wrong pouch".into(),1=>p.members=false,2=>p.stackable=false,3=>p.dummy=1,4=>p.cert.template=799,5=>p.placeholder.template=14401,6=>p.transform.template=1,7=>p.tradeable=true,8=>p.stockmarket=true,9=>p.inventory_ops[0]=Some("Open".into()),_=>{p.params.remove(&(params["no_bank"] as u32));}}
        assert!(gielinor_server::content::coin_pouches(&objects,&params,&defs).is_err(),"mode{mode}");refused+=1;
    }
    defs.items.insert(22521,saved);assert_eq!(gielinor_server::content::coin_pouches(&objects,&params,&defs).unwrap().len(),1);assert_eq!(refused,11);
    // These are existing decoded cache actors; no NPC or world state is created.
    let map=cache.map_square((50<<8)|50).unwrap().unwrap();let mut bankers=0;
    for spawn in &map.npcs{let npc=&defs.npcs[&(spawn.id as u32)];if npc.name.to_ascii_lowercase().contains("banker"){let coords=map.coords(spawn.x,spawn.z,spawn.level).unwrap();println!("PG_CACHE_BANKER id={} name={:?} coords={coords:?}",spawn.id,npc.name);bankers+=1;}}
    println!("PG_POUCH_BINDINGS selected_pouch=22521 output=995 coins=3 metadata_refusals=11 no_bank_resolver=true restricted_deposit_refused=true coin_control_deposited=true nearby_cache_bankers={bankers}");
}
