use gielinor_cache::game::GameCache;
use std::{io, path::Path};
fn main() -> io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .expect("usage: inspect CACHE_PATH [MAP_SQUARE]");
    let mut cache = GameCache::open(Path::new(&path), 0xff_ffff)?;
    let defs = cache.definitions_all()?;
    let names = cache.names()?;
    let components = cache.components()?;
    println!(
        "components={} if3={} trailing={}",
        components.len(),
        components.values().filter(|component| component.v3).count(),
        components
            .values()
            .filter(|component| component.decoded_bytes != component.encoded.len())
            .count()
    );
    println!(
        "name groups={} human animations={:?}",
        names.groups.len(),
        names.human_animations().map_err(io::Error::other)?
    );
    println!(
        "items={} npcs={} locations={} inventories={} varps={} varbits={} stats={}",
        defs.items.len(),
        defs.npcs.len(),
        defs.locations.len(),
        defs.inventories.len(),
        defs.varps.len(),
        defs.varbits.len(),
        defs.stats.len()
    );
    let square: u16 = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "12850".into())
        .parse()
        .expect("map square integer");
    if let Some(map) = cache.map_square(square)? {
        let mut collision = gielinor_world::collision::CollisionMap::default();
        map.terrain_collision(&mut collision);
        let locations = map.location_collision(&defs, &mut collision)?;
        println!(
            "map={square} locations={} npcs={} objects={}",
            locations.len(),
            map.npcs.len(),
            map.objects.len()
        );
    }
    Ok(())
}
