//! Read-only, bounded asset metadata search for private environmental studies.
use gielinor_cache::game::GameCache;
use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        return Err("Usage: environment_assets WORKSPACE_ROOT NAME_SUBSTRING...".into());
    }
    let root = PathBuf::from(&args[1]);
    let mut cache = GameCache::open(
        &root.join("resource-staging/server/work/integration/a-kotlin/.data/cache/game"),
        8 << 20,
    )?;
    let defs = cache.definitions_all()?;
    for term in &args[2..] {
        println!("QUERY {term}");
        let mut rows: Vec<_> = defs.locations.iter().filter(|(_, d)| {
            d.name.to_lowercase().contains(&term.to_lowercase())
        }).collect();
        rows.sort_by_key(|(id, _)| **id);
        for (id, d) in rows.into_iter().take(20) {
            println!("{id}\t{}\t{}x{}\twalk={}\tops={:?}", d.name,
                d.collision.width, d.collision.length, d.collision.block_walk, d.ops);
        }
    }
    Ok(())
}
