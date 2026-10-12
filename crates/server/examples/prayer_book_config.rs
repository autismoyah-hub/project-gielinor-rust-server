//! Offline validation of the same cache-resolved definitions used by the runtime.
use gielinor_cache::game::GameCache;
use gielinor_world::{stats::Stats, variables::Variables};
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err("expected CACHE SYMBOLS".into());
    }
    let names = gielinor_server::startup::Names::load(&[Path::new(&args[2])])?;
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let defs = cache.definitions_all()?;
    let parameters = cache.parameters()?;
    let enums = cache.enums()?;
    let config = gielinor_server::prayer_book::resolve(&names, &defs, &parameters, &enums)?;
    assert_eq!(config.book.prayers.len(), 29);
    assert_eq!(config.book.components.len(), 29);
    let mut stats = Stats::default();
    stats.set_base(5, 99);
    stats.set_current(5, 99);
    stats.set_base(1, 99);
    for (&id, prayer) in &config.book.prayers {
        let mut vars = Variables::default();
        if let Some(binding) = prayer.unlock {
            vars.set_bit(binding.base, binding.bit, prayer.unlock_state, false)
                .unwrap();
        }
        config
            .book
            .toggle(id, &mut vars, &stats, None, true)
            .unwrap();
        assert_eq!(
            vars.bit(config.book.all_active.bit).unwrap(),
            1i32.wrapping_shl(id as u32),
            "{}",
            prayer.name
        );
        config.drain.update_resistance(&mut vars, 0, true).unwrap();
        config
            .drain
            .tick(&mut vars, &mut stats.clone(), true)
            .unwrap();
        config
            .book
            .toggle(id, &mut vars, &stats, prayer.overhead, true)
            .unwrap();
        assert_eq!(vars.bit(config.book.all_active.bit).unwrap(), 0);
        assert_eq!(vars.bit(config.book.drain_counter.bit).unwrap(), 0);
    }
    println!(
        "Resolved and exercised {} packed prayer definitions",
        config.book.prayers.len()
    );
    Ok(())
}
