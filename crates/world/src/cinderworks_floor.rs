//! Explicit authored-floor geometry and relocation. No process-global switches.
//! Adapted from audited f9180b23 broken_water_return.rs; original gameplay owners remain.
use crate::{
    tower_layout::{Source, Template},
    CoordError, CoordGrid,
};
use std::collections::BTreeMap;

/// The authored floor occupies one 128-square detached region at this origin.
pub const COOLANT_TILES: [(i32, i32); 4] = [(53, 71), (53, 74), (60, 71), (60, 74)];
pub fn origin() -> CoordGrid {
    CoordGrid::new(10368, 64, 0).unwrap()
}
pub fn template() -> Template {
    let mut zones = Template::new();
    for x in 3..12 {
        for z in 3..11 {
            zones.insert(
                (x, z, 0),
                Source {
                    x: 364,
                    z: 1245,
                    level: 0,
                    rotation: 0,
                },
            );
        }
    }
    zones
}
#[derive(Clone, Debug, Default)]
pub struct Spatial {
    landmarks: BTreeMap<(i32, i32), (i32, i32)>,
    supply_npcs: BTreeMap<usize, (i32, i32)>,
}
impl Spatial {
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 16384 || text.lines().next() != Some("CINDERWORKS_SPATIAL 1") {
            return Err("Facility spatial header/size".into());
        }
        let mut out = Self::default();
        for line in text
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let values = fields
                .get(1..)
                .ok_or("Missing spatial record")?
                .iter()
                .map(|v| v.parse::<i32>().map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            match fields[0] {
                "M" if values.len() == 4 => {
                    let key = (values[0], values[1]);
                    let destination = (values[2], values[3]);
                    if ![
                        (43, 50),
                        (72, 58),
                        (59, 49),
                        (55, 58),
                        (55, 78),
                        (71, 76),
                        (51, 78),
                        (60, 78),
                        (67, 53),
                    ]
                    .contains(&key)
                        || !(27..91).contains(&destination.0)
                        || !(29..83).contains(&destination.1)
                        || out.landmarks.insert(key, destination).is_some()
                    {
                        return Err("Facility landmark relocation is invalid/duplicate".into());
                    }
                }
                "N" if values.len() == 3 => {
                    // Only the unchanged eight cows and four chickens may move.
                    // Guardians, fish, aide and merchant identities stay pinned.
                    if !(2..14).contains(&values[0])
                        || !(27..91).contains(&values[1])
                        || !(29..83).contains(&values[2])
                        || out
                            .supply_npcs
                            .insert(values[0] as usize, (values[1], values[2]))
                            .is_some()
                    {
                        return Err("Facility supply spawn relocation is invalid/duplicate".into());
                    }
                }
                _ => return Err("Unknown facility spatial row".into()),
            }
        }
        Ok(out)
    }

    pub fn relocate(&self, x: i32, z: i32) -> (i32, i32) {
        if let Some(&position) = self.landmarks.get(&(x, z)) {
            return position;
        }
        match (x, z) {
            (56, 56) => (52, 46),
            (55, 56) => (51, 46),
            (56, 55) => (52, 45),
            (57, 56) => (53, 46),
            (56, 49) => (52, 39),
            (53, 60) => (49, 50),
            (54, 51) => (50, 41),
            (55, 51) => (51, 41),
            (60, 52) => (56, 42),
            (34, 50) => (30, 40),
            (37, 50) => (33, 42),
            (34, 54) => (31, 57),
            (34, 58) => (32, 62),
            (44, 54) => (38, 57),
            (44, 58) => (40, 61),
            (67, 53) => (61, 59),
            (69, 52) => (62, 60),
            (64, 60) => (60, 50),
            (72, 72) => (68, 71),
            (56, 72) => (81, 56),
            (53, 71) => (78, 55),
            (53, 74) => (78, 58),
            (60, 71) => (85, 55),
            (60, 74) => (85, 58),
            (34, 70) => (40, 71),
            (42, 70) => (48, 71),
            (34, 76) => (40, 77),
            (42, 76) => (48, 77),
            (43, 50) => (39, 40),
            (72, 58) => (68, 57),
            (59, 49) => (55, 39),
            (55, 58) => (51, 48),
            (55, 78) => (80, 62),
            (71, 76) => (67, 75),
            (51, 78) => (76, 62),
            (60, 78) => (85, 62),
            (67, 70) => (63, 69),
            (67, 74) => (63, 73),
            (76, 70) => (72, 69),
            (76, 74) => (72, 73),
            _ => (x, z),
        }
    }
    pub fn at(&self, origin: CoordGrid, x: i32, z: i32) -> Result<CoordGrid, CoordError> {
        let (x, z) = self.relocate(x, z);
        origin.translate(x, z, 0)
    }
    pub fn supply_npc_position(&self, index: usize, default: (i32, i32)) -> (i32, i32) {
        self.supply_npcs.get(&index).copied().unwrap_or(default)
    }
}
#[derive(Clone, Debug)]
pub struct Cell {
    pub x: i32,
    pub z: i32,
    pub height: i32,
    pub material: u16,
    pub walk: bool,
}
#[derive(Clone, Debug)]
pub struct Object {
    pub id: u32,
    pub x: i32,
    pub z: i32,
    pub shape: u8,
    pub rotation: u8,
}
pub struct Floor {
    pub cells: Vec<Cell>,
    pub objects: Vec<Object>,
}
pub fn parse(text: &str) -> Result<Floor, String> {
    if text.len() > 2_000_000 || text.lines().next() != Some("BROKEN_WATER_RETURN 1") {
        return Err("Invalid integration floor header/size".into());
    }
    let (mut cells, mut objects) = (Vec::new(), Vec::new());
    let mut seen = std::collections::BTreeSet::new();
    let mut sites = std::collections::BTreeSet::new();
    for l in text
        .lines()
        .skip(1)
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let f: Vec<_> = l.split_whitespace().collect();
        if f.len() != 6 {
            return Err(format!("Invalid floor row: {l}"));
        }
        let n = f[1..]
            .iter()
            .map(|s| s.parse::<i32>().map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        match f[0] {
            "V" => {
                if !(0..128).contains(&n[0])
                    || !(0..128).contains(&n[1])
                    || !(0..=1024).contains(&n[2])
                    // Zero means an authored mesh owns this non-walkable
                    // terrain surface (e.g. a vertical-sided drainage bed).
                    || (!(5802..=5808).contains(&n[3]) && !(n[3] == 0 && n[4] == 0))
                    || !(0..=1).contains(&n[4])
                    || !seen.insert((n[0], n[1]))
                {
                    return Err("Invalid/duplicate terrain cell".into());
                }
                cells.push(Cell {
                    x: n[0],
                    z: n[1],
                    height: n[2],
                    material: n[3] as u16,
                    walk: n[4] == 1,
                });
            }
            "O" => {
                // Reserved original interior-art namespace. Every referenced
                // definition is still required by detached publication.
                if !(64700..65300).contains(&n[0])
                    || !(24..96).contains(&n[1])
                    || !(24..88).contains(&n[2])
                    || ![10, 22].contains(&n[3])
                    || !(0..4).contains(&n[4])
                    || !sites.insert((n[1], n[2], n[3]))
                {
                    return Err("Invalid/duplicate original scenery placement".into());
                }
                objects.push(Object {
                    id: n[0] as u32,
                    x: n[1],
                    z: n[2],
                    shape: n[3] as u8,
                    rotation: n[4] as u8,
                });
            }
            _ => return Err("Unknown integration row".into()),
        }
    }
    if cells.len() != 128 * 128 {
        return Err("Floor requires every height and collision cell, including borders".into());
    }
    Ok(Floor { cells, objects })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_partial_and_out_of_namespace_floors() {
        assert!(parse("BROKEN_WATER_RETURN 1\nV 1 1 240 5802 1\n").is_err());
        assert!(parse("BROKEN_WATER_RETURN 1\nO 100 40 40 10 0\n").is_err());
    }
    #[test]
    fn interior_namespace_remains_bounded_and_requires_unique_valid_placements() {
        let mut floor = String::from("BROKEN_WATER_RETURN 1\n");
        for x in 0..128 {
            for z in 0..128 {
                floor.push_str(&format!("V {x} {z} 240 5802 1\n"));
            }
        }
        for id in [64700, 65000, 65100, 65299] {
            assert!(parse(&format!("{floor}O {id} 40 40 10 0\n")).is_ok());
        }
        for row in [
            "64699 40 40 10 0",
            "65300 40 40 10 0",
            "65100 23 40 10 0",
            "65100 40 40 9 0",
            "65100 40 40 10 4",
        ] {
            assert!(parse(&format!("{floor}O {row}\n")).is_err());
        }
        assert!(parse(&format!("{floor}O 65100 40 40 10 0\nO 65101 40 40 10 0\n")).is_err());
    }
    #[test]
    fn mesh_owned_surface_cannot_hide_walkable_ground() {
        let floor = |walk| {
            let mut text = String::from("BROKEN_WATER_RETURN 1\n");
            for x in 0..128 {
                for z in 0..128 {
                    text.push_str(&format!("V {x} {z} 240 0 {walk}\n"));
                }
            }
            text
        };
        assert!(parse(&floor(0)).is_ok());
        assert!(parse(&floor(1)).is_err());
    }
    #[test]
    fn facility_relocation_cannot_change_pinned_coolant_or_guardian_spawns() {
        assert!(Spatial::parse("CINDERWORKS_SPATIAL 1\nM 55 78 88 64\nN 2 31 72\n").is_ok());
        assert!(Spatial::parse("CINDERWORKS_SPATIAL 1\nM 53 71 88 64\n").is_err());
        assert!(Spatial::parse("CINDERWORKS_SPATIAL 1\nN 19 81 70\n").is_err());
        assert!(Spatial::parse("CINDERWORKS_SPATIAL 1\nN 2 31 72\nN 2 32 72\n").is_err());
    }

    #[test]
    fn explicit_spatial_instances_do_not_share_relocations() {
        let stock = Spatial::default();
        let authored = Spatial::parse("CINDERWORKS_SPATIAL 1\nM 67 53 64 43\nN 2 31 72\n").unwrap();
        assert_eq!(stock.relocate(67, 53), (61, 59));
        assert_eq!(authored.relocate(67, 53), (64, 43));
        assert_eq!(authored.supply_npc_position(2, (33, 33)), (31, 72));
        assert_eq!(stock.supply_npc_position(2, (33, 33)), (33, 33));
        assert_eq!(authored.relocate(56, 56), (52, 46));
        assert_eq!(authored.relocate(56, 72), (81, 56));
        assert_eq!(authored.relocate(53, 71), (78, 55));
        assert_eq!(template().len(), 72);
    }
}
