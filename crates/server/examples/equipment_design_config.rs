//! Validate new runtime catalogs against the owned canonical cache, offline.
use gielinor_cache::game::GameCache;
use gielinor_server::{
    magic_config::Inputs,
    player_design::{Bindings, Kits},
    startup::Names,
};
use std::{fs::OpenOptions, io::Write, path::Path};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 && args.len() != 5 {
        return Err("expected CACHE SYMBOLS [RAW_OUTPUT CATALOG_OUTPUT]".into());
    }
    let names = Names::load(&[Path::new(&args[2])])?;
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let mut output: Box<dyn Write> = Box::new(std::io::sink());
    if args.len() == 5 {
        let mut raw = std::io::BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[3])?,
        );
        for group in [38, 39] {
            for (id, bytes) in cache.files(2, group, None)?.ok_or("Missing DB group")? {
                writeln!(
                    raw,
                    "{group}|{id}|{}",
                    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>()
                )?;
            }
        }
        raw.flush()?;
        output = Box::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[4])?,
        );
    }
    let defs = cache.definitions_all()?;
    let parameters = cache.parameters()?;
    let enums = cache.enums()?;
    let input = Inputs {
        objs: names.group("obj"),
        params: names.group("param"),
        bits: names.group("varbit"),
        enums: names.group("enum"),
        categories: names.group("category"),
        parameter_types: &parameters,
        tables: &enums,
        defs: &defs,
    };
    let wear = input.wearpos(
        names.group("varp"),
        names.group("synth"),
        names.combat_stats()?,
    )?;
    assert_eq!(wear.items[&1277].category, 17);
    assert_eq!(wear.special.id, 301);
    assert_eq!(wear.category.bit.base, 843);
    let bulwark = input.bulwark(names.group("varp"), names.group("spotanim"))?;
    assert_eq!(bulwark.glow, 1336);
    assert_eq!(bulwark.passive_delay.id, 65517);
    assert!(!bulwark.items.is_empty());
    let rows = cache.db_rows()?;
    let tables = cache.db_tables()?;
    let kits = Kits::resolve(&rows, &tables, names.group("dbtable"), names.group("dbcol"))?;
    for (body, slots) in kits.options.iter().enumerate() {
        for (slot, options) in slots.iter().enumerate() {
            println!("Creator body={body} slot={slot} ordered kits={options:?}");
            writeln!(
                output,
                "Creator body={body} slot={slot} ordered kits={options:?}"
            )?;
        }
    }
    let controls = Bindings::resolve(kits, names.group("component"), names.group("varbit"), &defs)?;
    let tutorial = gielinor_server::tutorial_ui::Config::resolve(&names)?;
    assert_eq!(tutorial.overlay, 614);
    assert_eq!(tutorial.panels.len(), 14);
    let character = gielinor_server::tutorial_character::Config::resolve(&names, &defs)?;
    assert_eq!(character.creator, 679);
    assert_eq!(character.experience, 929);
    // Exercise every step's resolved symbols and packet arguments offline.
    for step in gielinor_world::tutorial_steps::Step::ALL {
        tutorial.apply(step, None)?;
    }
    assert_eq!(controls.buttons.len(), 26);
    assert_eq!(controls.pronoun_component >> 16, 679);
    println!(
        "Resolved {} equipment types, {} bulwarks, and {} creator controls",
        wear.items.len(),
        bulwark.items.len(),
        controls.buttons.len() + 1
    );
    Ok(())
}
