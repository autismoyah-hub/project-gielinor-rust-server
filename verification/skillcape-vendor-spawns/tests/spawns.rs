//! Native scenario preparation from actual target map spawns and collision.
use gielinor_cache::game::GameCache;
use gielinor_world::collision::{CollisionMap, Strategy};
use std::path::Path;
#[test]
fn actual_vendor_spawns_and_passable_native_admission_tiles() {
    let path = std::env::var("PG_GAME_CACHE").unwrap();
    let mut c = GameCache::open(Path::new(&path), 0xff_ffff).unwrap();
    let defs = c.definitions_all().unwrap();
    for (npc, mx, mz) in [(5810u16, 45u16, 51u16), (2658, 49, 53)] {
        let mut maps = Vec::new();
        for x in mx - 1..=mx + 1 {
            for z in mz - 1..=mz + 1 {
                if let Some(m) = c.map_square((x << 8) | z).unwrap() {
                    maps.push(m);
                }
            }
        }
        let mut collision = CollisionMap::default();
        for m in &maps {
            m.terrain_collision(&mut collision);
        }
        for m in &maps {
            m.location_collision(&defs, &mut collision).unwrap();
        }
        let mut found = 0;
        for m in &maps {
            for s in &m.npcs {
                if s.id != npc {
                    continue;
                }
                let p = m.coords(s.x, s.z, s.level).unwrap();
                let mut adjacent = Vec::new();
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if collision.can_step(p, dx, dz, 1, 0, Strategy::Normal) {
                        adjacent.push((p.x() + dx, p.z() + dz, p.level()));
                    }
                }
                assert!(!adjacent.is_empty());
                println!("PG_VENDOR_ACTUAL_SPAWN npc={npc} position={},{},{} passable_admission={adjacent:?}",p.x(),p.z(),p.level());
                found += 1;
            }
        }
        assert!(
            found > 0,
            "actual target vendor{npc} absent around its guild"
        );
    }
    println!("PG_VENDOR_SPAWNS twoactualNPCs withtargetterrain/locationcollision passableadmissiontiles;preparation only PASS");
}
