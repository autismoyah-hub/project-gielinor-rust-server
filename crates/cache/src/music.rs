//! MusicRepository's cache catalog. Track identity follows the reference's
//! Int2ObjectOpenHashMap values iterator, including custom area overrides.
use crate::{
    db::{Row, Table, Value},
    int_map_order,
    symbols::Symbols,
};
use gielinor_world::{
    music::{Catalog, Track},
    variables::Varp,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct Input<'a> {
    pub rows: &'a BTreeMap<u32, Row>,
    pub tables: &'a BTreeMap<u32, Table>,
    pub table_names: &'a Symbols,
    pub columns: &'a Symbols,
    pub area_ids: &'a BTreeSet<u16>,
    pub unlock_varps: &'a [Varp],
}
impl Input<'_> {
    fn values<'a>(&'a self, row: &'a Row, name: &str) -> Result<Option<&'a [Value]>, String> {
        let packed = *self
            .columns
            .get(name)
            .ok_or_else(|| format!("Missing music column {name}"))? as u32;
        if packed >> 16 != row.table as u32 {
            return Err(format!("Wrong music table for {name}"));
        }
        let column =
            u8::try_from(packed & 65535).map_err(|_| format!("Invalid music column {name}"))?;
        let table = self
            .tables
            .get(&(row.table as u32))
            .ok_or_else(|| format!("Missing music table {}", row.table))?;
        Ok(row.column(table, column))
    }
    fn ints(&self, row: &Row, name: &str) -> Result<Option<Vec<i32>>, String> {
        self.values(row, name)?
            .map(|values| {
                values
                    .iter()
                    .map(|v| match v {
                        Value::Int(n) => Ok(*n),
                        _ => Err(format!("Expected integer in {name}")),
                    })
                    .collect()
            })
            .transpose()
    }
    fn int(&self, row: &Row, name: &str) -> Result<Option<i32>, String> {
        // DbValueColumn invokes one codec read; remaining tuples are ignored.
        match self.values(row, name)? {
            None => Ok(None),
            Some([Value::Int(value), ..]) => Ok(Some(*value)),
            _ => Err(format!("Expected one value in {name}")),
        }
    }
    fn required_int(&self, row: &Row, name: &str) -> Result<i32, String> {
        self.int(row, name)?
            .ok_or_else(|| format!("Missing value in {name}"))
    }
    fn string(&self, row: &Row, name: &str) -> Result<String, String> {
        match self.values(row, name)? {
            Some([Value::String(value), ..]) => Ok(value.clone()),
            _ => Err(format!("Expected one string in {name}")),
        }
    }
    pub fn resolve(&self) -> Result<Catalog, String> {
        let table = |name: &str| {
            self.table_names
                .get(name)
                .copied()
                .ok_or_else(|| format!("Missing music table {name}"))
        };
        let music = table("music")?;
        let modern = table("music_modern")?;
        let classic = table("music_classic")?;
        let order = int_map_order::order(self.rows.keys().map(|id| *id as i32));
        let mut catalog = Catalog::default();
        let mut music_order = Vec::new();
        for row_id in &order {
            let row = &self.rows[&(*row_id as u32)];
            if row.table != music {
                continue;
            }
            let id =
                u16::try_from(catalog.tracks.len() + 1).map_err(|_| "Too many music tracks")?;
            let variable = self
                .ints(row, "music:variable")?
                .ok_or("Missing music variable")?;
            let [index, bit, ..] = variable.as_slice() else {
                return Err("Music variable requires two integers".into());
            };
            let track = Track {
                id,
                row: *row_id as u32,
                name: self.string(row, "music:displayname")?,
                unlock_hint: self.string(row, "music:unlockhint")?,
                duration: self.required_int(row, "music:duration")?,
                midi: self.required_int(row, "music:midi")?,
                unlock_varp: index
                    .checked_sub(1)
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| self.unlock_varps.get(n))
                    .copied(),
                unlock_bit: *bit,
                hidden: self.int(row, "music:hidden")?.unwrap_or(0) == 1,
                secondary: self.int(row, "music:secondary_track")?.map(|n| n as u32),
            };
            catalog.rows.insert(track.row, id);
            music_order.push(*row_id);
            catalog.tracks.insert(id, track);
            for (column, is_modern) in [("music:area", true), ("music:area_default", false)] {
                let Some(area) = self
                    .int(row, column)?
                    .filter(|n| *n > 0)
                    .and_then(|n| u16::try_from(n).ok())
                    .filter(|n| self.area_ids.contains(n))
                else {
                    continue;
                };
                if is_modern {
                    catalog.modern.entry(area).or_default().push(id);
                } else {
                    catalog.classic.insert(area, id);
                }
            }
        }
        for row_id in &order {
            let row = &self.rows[&(*row_id as u32)];
            if row.table != modern && row.table != classic {
                continue;
            }
            let prefix = if row.table == modern {
                "music_modern"
            } else {
                "music_classic"
            };
            let area = u16::try_from(self.required_int(row, &format!("{prefix}:area"))?)
                .map_err(|_| "Invalid custom music area")?;
            if !self.area_ids.contains(&area) {
                return Err(format!("Unknown custom music area {area}"));
            }
            let column = format!(
                "{prefix}:{}",
                if row.table == modern {
                    "tracks"
                } else {
                    "track"
                }
            );
            let mut rows = self
                .ints(row, &column)?
                .ok_or_else(|| format!("Missing {column}"))?;
            if row.table == classic {
                rows.truncate(1);
            }
            let tracks = rows
                .iter()
                .map(|id| {
                    catalog
                        .rows
                        .get(&(*id as u32))
                        .copied()
                        .ok_or_else(|| format!("Music row not found: {id}"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            if row.table == modern {
                catalog.modern.insert(area, tracks);
            } else {
                let [track, ..] = tracks.as_slice() else {
                    return Err("Classic music requires one track".into());
                };
                catalog.classic.insert(area, *track);
            }
        }
        catalog.all = int_map_order::copied_map_order(&music_order)
            .into_iter()
            .map(|row| catalog.rows[&(row as u32)])
            .collect();
        Ok(catalog)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_official_rows_custom_overrides_defaults_and_missing_references() {
        let table_names = Symbols::from([
            ("music".into(), 1),
            ("music_modern".into(), 2),
            ("music_classic".into(), 3),
        ]);
        let mut columns = Symbols::new();
        for (i, name) in [
            "displayname",
            "unlockhint",
            "duration",
            "midi",
            "variable",
            "hidden",
            "secondary_track",
            "area",
            "area_default",
        ]
        .iter()
        .enumerate()
        {
            columns.insert(format!("music:{name}"), (1 << 16) | i as i32);
        }
        for (table, name, track) in [(2, "music_modern", "tracks"), (3, "music_classic", "track")] {
            columns.insert(format!("{name}:area"), table << 16);
            columns.insert(format!("{name}:{track}"), (table << 16) | 1);
        }
        let int = |n| vec![Value::Int(n)];
        let table = Table {
            defaults: BTreeMap::from([
                (1, vec![Value::String("hint".into())]),
                (2, int(100)),
                (7, vec![Value::Int(9), Value::Int(10)]),
                (8, int(9)),
            ]),
            ..Default::default()
        };
        let tables = BTreeMap::from([(1, table), (2, Table::default()), (3, Table::default())]);
        let mut rows = BTreeMap::new();
        for id in [1, 12, 32] {
            rows.insert(
                id,
                Row {
                    table: 1,
                    data: BTreeMap::from([
                        (0, vec![Value::String(format!("track{id}"))]),
                        (3, int(id as i32)),
                        (4, vec![Value::Int(1), Value::Int(id as i32)]),
                    ]),
                    ..Default::default()
                },
            );
        }
        let area_ids = BTreeSet::from([9, 10]);
        let varps = [Varp {
            id: 20,
            permanent: true,
            transmit: gielinor_world::variables::Transmit::Always,
            bit_protect: false,
            client_code: -1,
        }];
        let resolve = |rows: &BTreeMap<u32, Row>| {
            Input {
                rows,
                tables: &tables,
                table_names: &table_names,
                columns: &columns,
                area_ids: &area_ids,
                unlock_varps: &varps,
            }
            .resolve()
        };
        let official = resolve(&rows).unwrap();
        let order = int_map_order::order(rows.keys().map(|n| *n as i32));
        assert_eq!(official.modern[&9], [1, 2, 3]);
        assert_eq!(official.classic[&9], 3);
        assert_eq!(official.tracks[&1].row, order[0] as u32);
        assert_eq!(official.tracks[&1].unlock_hint, "hint");
        assert_eq!(official.tracks[&1].unlock_varp.unwrap().id, varps[0].id);
        rows.insert(
            44,
            Row {
                table: 2,
                data: BTreeMap::from([
                    (0, int(9)),
                    (1, vec![Value::Int(32), Value::Int(1), Value::Int(32)]),
                ]),
                ..Default::default()
            },
        );
        rows.insert(
            45,
            Row {
                table: 3,
                data: BTreeMap::from([(0, int(9)), (1, int(12))]),
                ..Default::default()
            },
        );
        let custom = resolve(&rows).unwrap();
        assert_eq!(
            custom.modern[&9],
            [custom.rows[&32], custom.rows[&1], custom.rows[&32]]
        );
        assert_eq!(custom.classic[&9], custom.rows[&12]);
        rows.get_mut(&44).unwrap().data.insert(1, Vec::new());
        assert!(resolve(&rows).unwrap().modern[&9].is_empty());
        rows.get_mut(&44).unwrap().data.insert(1, int(999));
        assert!(resolve(&rows)
            .unwrap_err()
            .contains("Music row not found: 999"));
    }
}
