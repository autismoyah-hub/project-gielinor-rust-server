//! Read-only inspection of the exact rows/defaults feeding Kotlin's MusicRepository.
use gielinor_cache::{
    db::{Row, Value},
    game::GameCache,
    int_map_order, symbols,
};
use std::{io, path::Path};
fn main() -> io::Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        return Err(io::Error::other("usage: music_rows CACHE SYMBOLS"));
    }
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let rows = cache.db_rows()?;
    let tables = cache.db_tables()?;
    let names = symbols::load(&[Path::new(&args[2])], "dbtable", None)?;
    let columns = symbols::load(&[Path::new(&args[2])], "dbcol", Some(&names))?;
    let area_ids = cache.area_ids()?;
    let defs = cache.definitions_all()?;
    let varp_names = symbols::load(&[Path::new(&args[2])], "varp", None)?;
    let unlock_varps = (1..=25)
        .map(|i| {
            let name = format!("musicmulti_{i}");
            let id = *varp_names
                .get(&name)
                .ok_or_else(|| io::Error::other(format!("Missing {name}")))?;
            defs.varps
                .get(&(id as u32))
                .copied()
                .ok_or_else(|| io::Error::other(format!("Unknown {name}")))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let catalog = gielinor_cache::music::Input {
        rows: &rows,
        tables: &tables,
        table_names: &names,
        columns: &columns,
        area_ids: &area_ids,
        unlock_varps: &unlock_varps,
    }
    .resolve()
    .map_err(io::Error::other)?;
    eprintln!(
        "Resolved {} tracks, {} modern and {} classic areas",
        catalog.tracks.len(),
        catalog.modern.len(),
        catalog.classic.len()
    );
    let bit_names = symbols::load(&[Path::new(&args[2])], "varbit", None)?;
    for (&id, bit) in &defs.varbits {
        if bit.base == 375 || bit.base == 5481 {
            let names: Vec<_> = bit_names
                .iter()
                .filter(|(_, value)| **value as u32 == id)
                .map(|(name, _)| name.as_str())
                .collect();
            println!("Z|{id}|{}|{}|{}|{names:?}", bit.base, bit.lsb, bit.msb);
        }
    }
    let mut areas = gielinor_world::areas::Areas::default();
    for square in [12850, 12843] {
        if let Some(map) = cache.map_square(square)? {
            if let Some(index) = map.areas {
                index
                    .register(square, &mut areas)
                    .map_err(io::Error::other)?;
            }
        }
    }
    for (x, z) in [(3224, 3218), (3222, 3218), (3232, 2784)] {
        let point = gielinor_world::CoordGrid::new(x, z, 0).unwrap();
        println!("P|{x}|{z}|{:?}", areas.at(point));
        for area in areas.at(point) {
            if let Some(tracks) = catalog.modern.get(&area) {
                let mut vars = gielinor_world::variables::Variables::default();
                for id in tracks {
                    catalog.tracks[id]
                        .unlock(&mut vars, true, false, false)
                        .map_err(|_| io::Error::other("Cannot unlock area track"))?;
                }
                println!(
                    "U|{area}|{:?}",
                    unlock_varps
                        .iter()
                        .filter_map(|varp| vars
                            .contains(varp.id)
                            .then_some((varp.id, vars.get(varp.id))))
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    let get = |row: &Row, name: &str| -> io::Result<Option<Vec<Value>>> {
        let packed = *columns
            .get(name)
            .ok_or_else(|| io::Error::other(format!("Missing column {name}")))?
            as u32;
        if (packed >> 16) as i32 != row.table {
            return Err(io::Error::other("Wrong column table"));
        }
        let table = tables
            .get(&(row.table as u32))
            .ok_or_else(|| io::Error::other("Missing db table"))?;
        let column = u8::try_from(packed & 65535).map_err(io::Error::other)?;
        Ok(row.column(table, column).map(<[Value]>::to_vec))
    };
    let order = int_map_order::order(rows.keys().map(|id| *id as i32));
    let table_id = |name: &str| {
        names
            .get(name)
            .copied()
            .ok_or_else(|| io::Error::other(format!("Missing table {name}")))
    };
    let music = table_id("music")?;
    let mut track_id = 0;
    for id in &order {
        let row = &rows[&(*id as u32)];
        if row.table != music {
            continue;
        }
        track_id += 1;
        println!(
            "M|{id}|{track_id}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            get(row, "music:displayname")?,
            get(row, "music:variable")?,
            get(row, "music:midi")?,
            get(row, "music:duration")?,
            get(row, "music:area")?,
            get(row, "music:area_default")?
        );
    }
    for table_name in ["music_modern", "music_classic"] {
        let table = table_id(table_name)?;
        for id in &order {
            let row = &rows[&(*id as u32)];
            if row.table != table {
                continue;
            }
            println!(
                "A|{table_name}|{id}|{:?}|{:?}",
                get(row, &format!("{table_name}:area"))?,
                get(
                    row,
                    &format!(
                        "{table_name}:{}",
                        if table_name == "music_modern" {
                            "tracks"
                        } else {
                            "track"
                        }
                    )
                )?
            );
        }
    }
    // Kotlin groups all row IDs but resolves table metadata only for requested
    // tables. Unused rows can reference absent tables in the pinned cache.
    let unused_missing = rows
        .values()
        .filter(|row| !tables.contains_key(&(row.table as u32)))
        .count();
    eprintln!("Unused rows with absent tables: {unused_missing}");
    eprintln!(
        "Decoded {} rows, {} tables, {track_id} music tracks",
        rows.len(),
        tables.len()
    );
    Ok(())
}
