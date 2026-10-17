//! Compare real cache clipping within unchanged encounter bounds.
//! This checks static movement only; live combat and hazard tests remain required.
use gielinor_cache::game::{Definitions, GameCache};
use gielinor_world::{
    cinderworks_encounter::Boss,
    collision::{CollisionMap, Strategy},
    loc_collision, CoordGrid,
};
use std::{env, error::Error, fs, path::PathBuf};

fn collision(text: &str, definitions: &Definitions) -> Result<CollisionMap, Box<dyn Error>> {
    let mut map = CollisionMap::default();
    // TowerFloor clears copied terrain throughout both combat courts before
    // stamping its authored shell. Water lies outside the compared bounds.
    for x in 32..80 {
        for z in 32..80 {
            map.allocate(10368 + x, 64 + z, 0);
            map.set(10368 + x, 64 + z, 0, 0);
        }
    }
    for line in text.lines().filter(|line| line.starts_with("L ")) {
        let values = line
            .split_whitespace()
            .skip(1)
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        if values.len() != 6 {
            return Err("Invalid location record".into());
        }
        let location = definitions
            .locations
            .get(&values[3])
            .ok_or("Unknown cache location")?;
        let at = CoordGrid::new(
            10368 + values[0] as i32,
            64 + values[1] as i32,
            values[2] as i32,
        )
        .map_err(|error| format!("{error:?}"))?;
        loc_collision::toggle(
            &mut map,
            at,
            values[4] as u8,
            values[5] as u8,
            location.collision,
            true,
        )?;
    }
    Ok(map)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        return Err("Usage: arena_routes WORKSPACE_ROOT BEFORE.scene AFTER.scene".into());
    }
    let workspace = PathBuf::from(&args[1]).canonicalize()?;
    let mut cache = GameCache::open(
        &workspace.join("resource-staging/server/work/integration/a-kotlin/.data/cache/game"),
        8 << 20,
    )?;
    let definitions = cache.definitions_all()?;
    let before = collision(&fs::read_to_string(&args[2])?, &definitions)?;
    let after = collision(&fs::read_to_string(&args[3])?, &definitions)?;
    let steps = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];
    for boss in [Boss::Imp, Boss::Warden] {
        let (min, max) = boss.bounds();
        let mut checked = 0;
        for x in min[0]..=max[0] {
            for z in min[1]..=max[1] {
                let at =
                    CoordGrid::new(10368 + x, 64 + z, 0).map_err(|error| format!("{error:?}"))?;
                assert_eq!(
                    before.get(at.x(), at.z(), 0) & 0x240100,
                    after.get(at.x(), at.z(), 0) & 0x240100,
                    "Standing tile changed for {boss:?} at{x},{z}"
                );
                for (dx, dz) in steps {
                    if !(min[0]..=max[0]).contains(&(x + dx))
                        || !(min[1]..=max[1]).contains(&(z + dz))
                    {
                        continue;
                    }
                    assert_eq!(
                        before.can_step(at, dx, dz, 1, 0, Strategy::Normal),
                        after.can_step(at, dx, dz, 1, 0, Strategy::Normal),
                        "Combat lane changed for {boss:?} at{x},{z} toward{dx},{dz}"
                    );
                    checked += 1;
                }
            }
        }
        println!(
            "{boss:?}: standing tiles and {checked} directed cache-clipped movements unchanged"
        );
    }
    Ok(())
}
