//! Independent target-cache glass variant/stat/sequence refusal matrix.
use gielinor_cache::{game::{Definitions,GameCache},symbols,definitions::{ItemType,StatType}};
use gielinor_server::content::glassblowing;
use std::{collections::BTreeMap,path::Path};
fn copy(d:&Definitions)->Definitions {Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:d.items.clone(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:d.stats.clone()}}
#[test]
fn actual_eight_glass_outputs_member_recipe_distinction_and_109_refusals(){
    let dir=std::env::var("PG_SYMBOLS").unwrap();let obj=symbols::load(&[Path::new(&dir)],"obj",None).unwrap();let seq=symbols::load(&[Path::new(&dir)],"seq",None).unwrap();let stat=symbols::load(&[Path::new(&dir)],"stat",None).unwrap();
    let path=std::env::var("PG_GAME_CACHE").unwrap();let mut cache=GameCache::open(Path::new(&path),0xff_ffff).unwrap();let items=cache.files(2,10,None).unwrap().unwrap();let stats=cache.files(2,200,None).unwrap().unwrap();
    let ids=[("glassblowingpipe",1785),("molten_glass",1775),("beer_glass",1919),("candle_lantern_empty",4527),("oil_lamp_empty",4525),("vial_empty",229),("fishbowl_empty",6667),("stafforb",567),("bullseye_lantern_lens",4542),("dorgesh_lightbulb_nofilament",10980)];
    let mut d=Definitions{inventories:BTreeMap::new(),locations:BTreeMap::new(),npcs:BTreeMap::new(),items:BTreeMap::new(),varbits:BTreeMap::new(),varps:BTreeMap::new(),stats:BTreeMap::new()};
    for (_,id) in ids{d.items.insert(id as u32,ItemType::decode(&items[&(id as u32)]).unwrap());}d.stats.insert(12,StatType::decode(&stats[&12]).unwrap());
    let recipes=glassblowing(&obj,&seq,&stat,&d).unwrap();assert_eq!(recipes.len(),8);
    for (r,(output,level,xp)) in recipes.iter().zip([(1919,1,175),(4527,4,190),(4525,12,250),(229,33,350),(6667,42,425),(567,46,525),(4542,49,550),(10980,87,700)]) {
        assert_eq!((r.exchange.tool,r.exchange.input,r.exchange.output,r.level,r.xp_tenths,r.ticks,r.exchange.animation,r.exchange.members,r.action),(1785,1775,output,level,xp,2,Some(884),true,"blow"));
    }
    assert!(!d.items[&1919].members && !d.items[&229].members);assert_eq!(d.items[&6667].name,"Empty fishbowl");
    let mut refused=0;
    for (symbol,id) in ids{for mode in 0..10{
        let mut defs=copy(&d);let mut objects=obj.clone();
        match mode{
            0=>{defs.items.remove(&(id as u32));},1=>defs.items.get_mut(&(id as u32)).unwrap().name="Wrong variant".into(),
            2=>{let item=defs.items.get_mut(&(id as u32)).unwrap();item.members=!item.members;},
            3=>defs.items.get_mut(&(id as u32)).unwrap().stackable=true,4=>defs.items.get_mut(&(id as u32)).unwrap().dummy=1,
            5=>defs.items.get_mut(&(id as u32)).unwrap().cert.template=799,6=>defs.items.get_mut(&(id as u32)).unwrap().placeholder.template=14401,7=>defs.items.get_mut(&(id as u32)).unwrap().transform.template=1,
            8=>{objects.insert(symbol.into(),id+1);},_=>{objects.remove(symbol);}
        }
        assert!(glassblowing(&objects,&seq,&stat,&defs).is_err(),"id{id} mode{mode}");refused+=1;
    }}
    for mode in 0..9{
        let mut defs=copy(&d);let mut stats=stat.clone();let mut sequences=seq.clone();
        match mode{0=>{stats.remove("crafting");},1=>{stats.insert("crafting".into(),13);},2=>{defs.stats.remove(&12);},3=>defs.stats.get_mut(&12).unwrap().name="Wrong skill".into(),4=>defs.stats.get_mut(&12).unwrap().unreleased=true,5=>defs.stats.get_mut(&12).unwrap().min_level=0,6=>defs.stats.get_mut(&12).unwrap().max_level=98,7=>{sequences.remove("human_glassblowing");},_=>{sequences.insert("human_glassblowing".into(),885);}}
        assert!(glassblowing(&obj,&sequences,&stats,&defs).is_err());refused+=1;
    }
    assert_eq!(refused,109);assert_eq!(glassblowing(&obj,&seq,&stat,&d).unwrap().len(),8);
    println!("PG_GLASS_BINDINGS actual_cache_items=10 profiles=8 independent_refusals=109");
}
