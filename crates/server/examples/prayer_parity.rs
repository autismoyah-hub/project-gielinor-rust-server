//! Compare cache-resolved live prayer rules with actual compiled Kotlin output.
use gielinor_cache::game::GameCache;
use gielinor_world::{
    inventory::{Insert, Item},
    simulation::PlayerData,
    stats::Stats,
    CoordGrid,
};
use std::path::Path;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        return Err("expected GAME_CACHE SYMBOLS KOTLIN_FIXTURE".into());
    }
    let names = gielinor_server::startup::Names::load(&[Path::new(&args[2])])?;
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let defs = cache.definitions_all()?;
    let config = gielinor_server::prayer::resolve(&names, &defs)?;
    let mut cases = 0;
    for line in std::fs::read_to_string(&args[3])?.lines() {
        let f = line.split('|').collect::<Vec<_>>();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let mut p = PlayerData::new(
            1,
            "Prayer oracle".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        match f[0] {
            "C" => assert_eq!(
                config.requires_confirmation(CoordGrid::new(n(1), n(2), n(3)).unwrap()),
                f[4] == "true",
                "{line}"
            ),
            "R" => {
                let mut c = config.clone();
                c.bones.insert(n(1), 0.0);
                p.inventory
                    .insert(&Default::default(), Insert::new(n(1), 1))
                    .unwrap();
                if n(2) >= 0 {
                    p.equipment
                        .restore_slot(
                            2,
                            Some(Item {
                                id: n(2),
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                assert_eq!(c.prepare(&p, n(1)).unwrap().restore, n(3), "{line}");
            }
            "X" => {
                let mut c = config.clone();
                assert_eq!(
                    c.bones[&n(1)],
                    f[2].parse::<f64>().unwrap(),
                    "cached bone XP {line}"
                );
                p.xp_rate_hundreds = n(3);
                p.stats.set_base(5, n(4) as i8);
                p.stats.set_current(5, n(5) as i8);
                p.stats.set_fine_xp(5, n(6)).unwrap();
                if n(7) >= 0 {
                    p.equipment
                        .restore_slot(
                            2,
                            Some(Item {
                                id: n(7),
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                c.worn_xp_percent.insert(900, n(8));
                p.equipment
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 900,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                p.inventory
                    .insert(&Default::default(), Insert::new(n(1), 1))
                    .unwrap();
                let pending = c.prepare(&p, n(1)).unwrap();
                let result = c.finish(&mut p, &Default::default(), pending);
                assert_eq!(result.is_err(), f[12] == "true", "{line}");
                assert_eq!(
                    (
                        p.stats.current(5) as u8 as i32,
                        p.stats.base(5) as u8 as i32,
                        p.stats.fine_xp(5)
                    ),
                    (n(9), n(10), n(11)),
                    "{line}"
                );
            }
            _ => return Err("unknown prayer vector".into()),
        }
        cases += 1;
    }
    assert!(cases > 1300);
    println!(
        "Matched {cases} compiled Kotlin prayer cases; {} cache-resolved bones",
        config.bones.len()
    );
    Ok(())
}
