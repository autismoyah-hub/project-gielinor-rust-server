//! Independent mutations of actual loaded target definitions and symbols.
use gielinor_cache::{game::{Definitions,GameCache},symbols};
use gielinor_server::content::gem_cutting;
use std::{collections::BTreeMap,path::Path};
fn copy(d:&Definitions)->Definitions { Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:d.items.clone(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:d.stats.clone()} }
#[test]
fn complete_actual_binding_and_159_independent_refusals(){
    let dir=std::env::var("PG_SYMBOLS").unwrap();
    let objects=symbols::load(&[Path::new(&dir)],"obj",None).unwrap();
    let seq=symbols::load(&[Path::new(&dir)],"seq",None).unwrap();
    let stat=symbols::load(&[Path::new(&dir)],"stat",None).unwrap();
    let cache=std::env::var("PG_GAME_CACHE").unwrap();
    let mut cache=GameCache::open(Path::new(&cache),0xff_ffff).unwrap();
    let items=cache.files(2,10,None).unwrap().unwrap();
    let stats=cache.files(2,200,None).unwrap().unwrap();
    let pairs=[("uncut_sapphire",1623,"sapphire",1607,20,500,false),("uncut_emerald",1621,"emerald",1605,27,675,false),("uncut_ruby",1619,"ruby",1603,34,850,false),("uncut_diamond",1617,"diamond",1601,43,1075,false),("uncut_dragonstone",1631,"dragonstone",1615,55,1375,true),("uncut_onyx",6571,"onyx",6573,67,1675,true),("uncut_zenyte",19496,"zenyte",19493,89,500,true)];
    let mut defs=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
    let mut ids=vec![("chisel",1755)];
    for (raw,id,cut,out,_,_,_) in pairs {ids.push((raw,id));ids.push((cut,out));}
    for (_,id) in &ids {defs.items.insert(*id as u32,gielinor_cache::definitions::ItemType::decode(&items[&(*id as u32)]).unwrap());}
    defs.stats.insert(12,gielinor_cache::definitions::StatType::decode(&stats[&12]).unwrap());
    let recipes=gem_cutting(&objects,&seq,&stat,&defs).unwrap();assert_eq!(recipes.len(),7);
    for (r,(_,id,_,out,level,xp,member)) in recipes.iter().zip(pairs) {
        assert_eq!((r.exchange.tool,r.exchange.input,r.exchange.output,r.level,r.xp_tenths,r.exchange.members,r.ticks,r.exchange.animation),(1755,id,out,level,xp,member,1,Some(8833)));
    }
    let mut refused=0;
    for (symbol,id) in ids {
        for case in 0..10 {
            let mut d=copy(&defs);let mut o=objects.clone();
            match case {
                0=>{d.items.remove(&(id as u32));},
                1=>d.items.get_mut(&(id as u32)).unwrap().name="wrong independent name".into(),
                2=>{let v=d.items.get_mut(&(id as u32)).unwrap();v.members=!v.members;},
                3=>d.items.get_mut(&(id as u32)).unwrap().stackable=true,
                4=>d.items.get_mut(&(id as u32)).unwrap().dummy=1,
                5=>d.items.get_mut(&(id as u32)).unwrap().cert.template=799,
                6=>d.items.get_mut(&(id as u32)).unwrap().placeholder.template=14401,
                7=>d.items.get_mut(&(id as u32)).unwrap().transform.template=1,
                8=>{o.insert(symbol.into(),id+100000);},
                9=>{o.remove(symbol);},_=>unreachable!()
            }
            assert!(gem_cutting(&o,&seq,&stat,&d).is_err(),"accepted {symbol} case{case}");refused+=1;
        }
    }
    for case in 0..9 {
        let mut d=copy(&defs);let mut s=stat.clone();let mut q=seq.clone();
        match case {
            0=>{s.remove("crafting");},1=>{s.insert("crafting".into(),13);},2=>{d.stats.remove(&12);},
            3=>d.stats.get_mut(&12).unwrap().name="Wrong skill".into(),4=>d.stats.get_mut(&12).unwrap().unreleased=true,
            5=>d.stats.get_mut(&12).unwrap().min_level=0,6=>d.stats.get_mut(&12).unwrap().max_level=98,
            7=>{q.remove("human_cutting");},8=>{q.insert("human_cutting".into(),8834);},_=>unreachable!()
        }
        assert!(gem_cutting(&objects,&q,&s,&d).is_err(),"accepted stat/effect case{case}");refused+=1;
    }
    assert_eq!(refused,159);assert_eq!(gem_cutting(&objects,&seq,&stat,&defs).unwrap().len(),7);
    println!("PG_GEM_BINDINGS actual_cache_items=15 profiles=7 independently_refused=159");
}
