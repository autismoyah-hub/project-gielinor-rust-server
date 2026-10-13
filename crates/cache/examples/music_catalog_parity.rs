//! Export raw, pinned cache inputs and canonical Rust music catalog for a
//! separate original-Kotlin MusicRepository comparison. Refuses overwrites.
use gielinor_cache::{game::GameCache, music::Input, symbols};
use std::{
    fs::OpenOptions,
    io::{self, Write},
    path::Path,
};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn main() -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err(io::Error::other(
            "usage: music_catalog_parity CACHE SYMBOLS RAW_OUTPUT RUST_OUTPUT",
        ));
    }
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let mut raw = io::BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[3])?,
    );
    for group in [18, 38, 39] {
        for (id, bytes) in cache
            .files(2, group, None)?
            .ok_or_else(|| io::Error::other("Missing config group"))?
        {
            writeln!(raw, "{group}|{id}|{}", hex(&bytes))?;
        }
    }
    raw.flush()?;
    let directories = [Path::new(&args[2])];
    let table_names = symbols::load(&directories, "dbtable", None)?;
    let columns = symbols::load(&directories, "dbcol", Some(&table_names))?;
    let varps = symbols::load(&directories, "varp", None)?;
    let defs = cache.definitions_all()?;
    let unlock_varps = (1..=25)
        .map(|i| {
            let name = format!("musicmulti_{i}");
            let id = *varps
                .get(&name)
                .ok_or_else(|| io::Error::other(format!("Missing {name}")))?;
            defs.varps
                .get(&(id as u32))
                .copied()
                .ok_or_else(|| io::Error::other(format!("Missing metadata {name}")))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let rows = cache.db_rows()?;
    let tables = cache.db_tables()?;
    let area_ids = cache.area_ids()?;
    let catalog = Input {
        rows: &rows,
        tables: &tables,
        table_names: &table_names,
        columns: &columns,
        area_ids: &area_ids,
        unlock_varps: &unlock_varps,
    }
    .resolve()
    .map_err(io::Error::other)?;
    let mut out = io::BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[4])?,
    );
    for (row, id) in &catalog.rows {
        let track = &catalog.tracks[id];
        writeln!(
            out,
            "T|{row}|{id}|{}|{}|{}|{}|{}|{}|{}|{}",
            hex(track.name.as_bytes()),
            hex(track.unlock_hint.as_bytes()),
            track.duration,
            track.midi,
            track.unlock_varp.map(|v| v.id as i32).unwrap_or(-1),
            track.unlock_bit,
            track.hidden,
            track.secondary.map(|n| n as i32).unwrap_or(-1)
        )?;
    }
    writeln!(
        out,
        "O|{}",
        catalog
            .all
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    )?;
    for (area, tracks) in &catalog.modern {
        writeln!(
            out,
            "M|{area}|{}",
            tracks
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        )?;
    }
    for (area, track) in &catalog.classic {
        writeln!(out, "C|{area}|{track}")?;
    }
    out.flush()?;
    eprintln!(
        "Catalog comparison input exported; {} tracks",
        catalog.tracks.len()
    );
    Ok(())
}
