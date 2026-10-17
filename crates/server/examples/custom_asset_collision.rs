//! Audit the exact server-authored cache-backed shell and ordinary clipping.
use gielinor_cache::{game::GameCache,symbols};
use gielinor_world::{collision::{CollisionMap,Strategy},tower_layout,CoordGrid};
use std::{collections::BTreeSet,fs,path::PathBuf};
fn main()->Result<(),Box<dyn std::error::Error>>{
    let a:Vec<_>=std::env::args().collect(); let root=PathBuf::from(&a[1]);
    let mut cache=GameCache::open(&root.join("cache/game"),8<<20)?;
    let definitions=cache.definitions_all()?;
    let names=symbols::parse(&fs::read_to_string(root.join("symbols/loc.sym"))?,None).map_err(|e|format!("{e:?}"))?;
    let ids=tower_layout::template().values().map(|s|((s.x/8)<<8)|(s.z/8)).collect::<BTreeSet<_>>();
    let mut normal=CollisionMap::default();let mut locations=Vec::new();
    for id in ids {let square=cache.map_square(id)?.ok_or("map square")?;square.terrain_collision(&mut normal);locations.extend(square.location_collision(&definitions,&mut normal)?);}
    let origin=CoordGrid::new(10368,64,0).unwrap();
    let shell=gielinor_server::tower_map::prepare(origin,&normal,&locations,&definitions,&names)?;
    for (x,z,dx,dz) in [(55,40,0,1),(56,40,0,1),(60,39,1,0),(60,40,1,0)] {
        let before=origin.translate(x-dx,z-dz,0).unwrap();
        let at=origin.translate(x,z,0).unwrap();
        assert!(shell.map.collision.can_step(before,dx,dz,1,0,Strategy::Normal),"passage approach {x},{z}");
        assert!(shell.map.collision.can_step(at,dx,dz,1,0,Strategy::Normal),"passage exit {x},{z}");
    }
    for (x,z,dx,dz) in [(54,40,0,1),(57,40,0,1),(60,38,1,0),(60,41,1,0)] {
        let before=origin.translate(x-dx,z-dz,0).unwrap();
        assert!(!shell.map.collision.can_step(before,dx,dz,1,0,Strategy::Normal),"column must block {x},{z}");
    }
    assert_eq!(shell.map.locations.iter().filter(|l|l.id==65000).count(),2);
    assert_eq!(shell.map.locations.iter().filter(|l|l.id==65001).count(),4);
    println!("SERVER_COLLISION_PASS: both rotated arches published; four support tiles block normal steps; both two-tile passages permit entry and exit");
    Ok(())
}
