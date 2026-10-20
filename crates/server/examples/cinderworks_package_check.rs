//! Read-only package/cache admission check; no sockets, accounts or gameplay grants.
use gielinor_cache::{
    asset_package::{AssetPackage, Namespace},
    game::GameCache,
};
use gielinor_server::cinderworks_contract as contract;
use std::{collections::BTreeSet, path::Path};
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 {
        return Err("usage: cinderworks_package_check GAME_CACHE RUNTIME_PACKAGE".into());
    }
    let package = AssetPackage::load(
        Path::new(&args[2]),
        contract::RUNTIME_ID,
        Namespace {
            models: contract::MODELS,
            objects: contract::OBJECTS,
        },
    )?;
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let mut definitions = cache.definitions_all()?;
    let before = definitions.locations.len();
    let stock_models: BTreeSet<_> = cache.groups(7)?.into_iter().collect();
    package.install(&mut definitions, &stock_models)?;
    for id in contract::OBJECTS {
        assert_eq!(
            definitions.locations[id].encoded,
            package.object(*id).unwrap().encoded
        );
    }
    assert_eq!(
        definitions.locations.len(),
        before + contract::OBJECTS.len()
    );
    println!(
        "admitted content={} models={} objects={} stock_locations={} final_locations={}",
        package
            .identity()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        contract::MODELS.len(),
        contract::OBJECTS.len(),
        before,
        definitions.locations.len()
    );
    Ok(())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Package admission: {e}");
        std::process::exit(1);
    }
}
