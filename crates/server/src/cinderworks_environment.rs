//! Opt-in original environment placement; existing gameplay anchors are immutable.
use gielinor_cache::{game::Definitions, tower_shell::Shell};
use gielinor_world::CoordGrid;
use std::{collections::BTreeSet, path::Path};
#[derive(Debug)]
struct Placement {
    id: u32,
    x: i32,
    z: i32,
    shape: u8,
    angle: u8,
    expected: Option<u32>,
    key: String,
}
fn parse(text: &str) -> Result<Vec<Placement>, String> {
    if text.len() > 131072 {
        return Err("Environment placement size".into());
    }
    let mut lines = text.lines();
    if lines.next() != Some("CINDERWORKS_ENVIRONMENT 1") {
        return Err("Environment placement version".into());
    }
    let mut rows = Vec::new();
    let mut keys = BTreeSet::new();
    let mut sites = BTreeSet::new();
    for line in lines {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() != 7 {
            return Err("Environment row fields".into());
        }
        let num = |i: usize| {
            f[i].parse::<i32>()
                .map_err(|_| "Environment integer".to_string())
        };
        let (id, x, z, shape, angle) = (num(0)?, num(1)?, num(2)?, num(3)?, num(4)?);
        let expected = if f[5] == "empty" {
            None
        } else {
            Some(num(5)? as u32)
        };
        if !(64700..=64725).contains(&id)
            || !(32..80).contains(&x)
            || !(32..80).contains(&z)
            || ![0, 2, 10, 22].contains(&shape)
            || !(0..4).contains(&angle)
        {
            return Err(format!("Environment identity/bounds: {line}"));
        }
        if !f[6].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || !keys.insert(f[6].to_string())
        {
            return Err("Environment duplicate/invalid key".into());
        }
        let layer = if shape < 4 {
            0
        } else if shape == 22 {
            3
        } else {
            2
        };
        if !sites.insert((x, z, layer)) {
            return Err("Environment duplicate layer site".into());
        }
        rows.push(Placement {
            id: id as u32,
            x,
            z,
            shape: shape as u8,
            angle: angle as u8,
            expected,
            key: f[6].into(),
        });
    }
    Ok(rows)
}
fn layer(s: u8) -> u8 {
    match s {
        0..=3 => 0,
        4..=8 => 1,
        9..=21 => 2,
        _ => 3,
    }
}
fn encounter(x: i32, z: i32) -> bool {
    (50..=62).contains(&x) && (67..=77).contains(&z)
        || (68..=75).contains(&x) && (68..=75).contains(&z)
}
pub fn apply(
    shell: &mut Shell,
    origin: CoordGrid,
    d: &Definitions,
    path: &Path,
) -> Result<(), String> {
    let rows = parse(&std::fs::read_to_string(path).map_err(|e| format!("Environment: {e}"))?)?;
    // Validate the complete transaction against the existing scene before stamping.
    for r in &rows {
        let p = origin
            .translate(r.x, r.z, 0)
            .map_err(|e| format!("{e:?}"))?;
        let old = shell
            .map
            .locations
            .iter()
            .find(|l| l.coords == p && layer(l.shape) == layer(r.shape));
        if old.map(|l| l.id) != r.expected {
            return Err(format!(
                "Environment {} would overwrite unexpected object {:?}",
                r.key,
                old.map(|l| l.id)
            ));
        }
        if let Some(id) = r.expected {
            if ![7016, 7017, 6143, 6145].contains(&id) {
                return Err(format!(
                    "Environment {} replaces protected content {id}",
                    r.key
                ));
            }
        }
        let t = d
            .locations
            .get(&r.id)
            .ok_or_else(|| format!("Environment definition absent {}", r.id))?;
        if t.ops.iter().any(Option::is_some) {
            return Err("Environment scenery cannot introduce gameplay operations".into());
        }
        let (mut w, mut h) = (t.collision.width as i32, t.collision.length as i32);
        if r.angle % 2 != 0 {
            std::mem::swap(&mut w, &mut h);
        }
        for x in r.x..r.x + w {
            for z in r.z..r.z + h {
                if x >= 80 || z >= 80 || encounter(x, z) {
                    return Err(format!(
                        "Environment {} intrudes into protected combat bounds or exterior",
                        r.key
                    ));
                }
            }
        }
    }
    for r in rows {
        shell.stamp_shaped(
            origin.translate(r.x, r.z, 0).unwrap(),
            r.id,
            r.shape,
            r.angle,
            d,
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_duplicates_and_unowned_namespaces() {
        assert!(parse(
            "CINDERWORKS_ENVIRONMENT 1\n64702 40 52 10 0 empty pump\n64703 40 52 10 0 empty arch\n"
        )
        .is_err());
        assert!(parse("CINDERWORKS_ENVIRONMENT 1\n2097 40 52 10 0 empty anvil\n").is_err());
    }
    #[test]
    fn exact_encounter_extents_are_reserved() {
        for x in 50..=62 {
            for z in 67..=77 {
                assert!(encounter(x, z));
            }
        }
        for x in 68..=75 {
            for z in 68..=75 {
                assert!(encounter(x, z));
            }
        }
        assert!(!encounter(56, 66));
        assert!(!encounter(72, 67));
    }
}
