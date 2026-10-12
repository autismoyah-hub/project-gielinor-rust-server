//! Offline, read-only cache preparation with no listener or account database.
use gielinor_cache::{game::GameCache, symbols};
use gielinor_world::{collision::CollisionMap, CoordGrid};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        return Err("expected GAME_CACHE SYMBOL_DIRECTORY NEW_OUTPUT_DIRECTORY".into());
    }
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let definitions = cache.definitions_all()?;
    let locs = symbols::load(&[Path::new(&args[2])], "loc", None)?;
    let npc_names = symbols::load(&[Path::new(&args[2])], "npc", None)?;
    let mut ids = std::collections::BTreeSet::new();
    for source in gielinor_world::tower_layout::template().values() {
        ids.insert(((source.x / 8) << 8) | (source.z / 8));
    }
    let mut maps = Vec::new();
    for id in ids {
        maps.push(
            cache
                .map_square(id)?
                .ok_or_else(|| format!("Missing tower source map {id}"))?,
        );
    }
    let mut normal = CollisionMap::default();
    for map in &maps {
        map.terrain_collision(&mut normal);
    }
    let mut original = Vec::new();
    for map in &maps {
        original.extend(map.location_collision(&definitions, &mut normal)?);
    }
    let origin = CoordGrid::new(10368, 64, 0).map_err(|e| format!("{e:?}"))?;
    let mut shell =
        gielinor_server::tower_map::prepare(origin, &normal, &original, &definitions, &locs)?;
    let population = gielinor_server::tower_map::npcs(origin, &definitions, &npc_names)?;
    // Match the accepting runtime's Npc Create hook instead of comparing raw
    // cache stats with Kotlin's post-Kronos population.
    let catalog = gielinor_server::kronos_combat::Catalog::load(Path::new(
        gielinor_server::kronos_combat::DEFAULT_DIRECTORY,
    ));
    if !catalog.failures.is_empty() {
        return Err(format!("NPC runtime bindings failed: {:?}", catalog.failures).into());
    }
    let mut world = gielinor_world::simulation::World::default();
    world.collision = shell.map.collision;
    world.configure_npc_bindings(catalog.bindings(&cache.sequences()?));
    let mut npcs = Vec::new();
    for npc in population {
        let id = world
            .spawn_npc(npc)
            .map_err(|_| "Tower NPC capacity exceeded")?;
        npcs.push(world.npc(id).unwrap().clone());
    }
    shell.map.collision = world.collision;
    let output = Path::new(&args[3]);
    fs::create_dir(output)?;
    let mut loc_file = BufWriter::new(File::create(output.join("floor-locations.tsv"))?);
    shell
        .map
        .locations
        .sort_by_key(|l| (l.coords.level(), l.coords.x(), l.coords.z(), l.shape));
    for l in &shell.map.locations {
        if l.coords.level() <= 1 {
            writeln!(
                loc_file,
                "{}\t{}\t{}\t{}\t{}\t{}",
                l.coords.x() - origin.x(),
                l.coords.z() - origin.z(),
                l.coords.level(),
                l.id,
                l.shape,
                l.angle
            )?;
        }
    }
    loc_file.flush()?;
    let mut collision = BufWriter::new(File::create(output.join("floor-collision.tsv"))?);
    for level in 0..=1 {
        for x in 0..128 {
            for z in 0..128 {
                writeln!(
                    collision,
                    "{x}\t{z}\t{level}\t{}",
                    shell
                        .map
                        .collision
                        .get(origin.x() + x, origin.z() + z, level) as i32
                )?;
            }
        }
    }
    collision.flush()?;
    let mut npc_file = BufWriter::new(File::create(output.join("floor-npcs.tsv"))?);
    for n in &npcs {
        writeln!(
            npc_file,
            "{}\t{}\t{}\t{}\t{:?}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            n.type_id,
            n.spawn.x() - origin.x(),
            n.spawn.z() - origin.z(),
            n.spawn.level(),
            n.mode,
            n.stats[0],
            n.stats[1],
            n.stats[2],
            n.stats[3],
            n.stats[4],
            n.stats[5],
            n.respawns
        )?;
    }
    npc_file.flush()?;
    fs::write(output.join("snapshot.txt"),"offline_startup_snapshot=true\norigin=10368,64,0\ncollision_includes_npc_occupancy=true\nnot_a_live_gameplay_test=true\n")?;
    println!(
        "Prepared {} locations, {} deleted cache locations and {} NPCs",
        shell.map.locations.len(),
        shell.removed.len(),
        npcs.len()
    );
    Ok(())
}
