//! Bounded read-only inventory of actual NPC spawn files; no synthetic actors.
use gielinor_cache::{game::GameCache, maps};
use std::path::Path;
#[test]
fn inventory_every_actual_target_map_npc_file_for_both_vendor_ids() {
    let path = std::env::var("PG_GAME_CACHE").unwrap();
    let mut c = GameCache::open(Path::new(&path), 0xff_ffff).unwrap();
    let groups = c.groups(5).unwrap();
    let mut files = 0;
    let mut total = 0;
    let mut found = [0, 0];
    for g in groups.iter().copied().filter(|g| *g <= u16::MAX as u32) {
        let f = c.files(5, g, None).unwrap().unwrap();
        let Some(raw) = f.get(&100) else {
            continue;
        };
        files += 1;
        for s in maps::npcs(raw).unwrap() {
            total += 1;
            if let Some(i) = [5810, 2658].iter().position(|&id| id == s.id) {
                found[i] += 1;
                let x = ((g >> 8) as i32) * 64 + i32::from(s.x);
                let z = ((g & 255) as i32) * 64 + i32::from(s.z);
                println!(
                    "PG_VENDOR_GLOBAL_SPAWN npc={} map={g} position={x},{z},{}",
                    s.id, s.level
                );
            }
        }
    }
    println!("PG_VENDOR_GLOBAL_SPAWN_INVENTORY map_groups={} npc_files={files} npc_spawns={total} vendor5810={} vendor2658={};diagnostic inventory only,no native/runtime reachability claim",groups.len(),found[0],found[1]);
    assert!(
        files > 0 && total > 0,
        "No map NPC files were available, so absence cannot establish a vendor-specific dependency"
    );
}
