//! Detached design feasibility only: no server, account, save or runtime writes.
use gielinor_cache::game::GameCache;
use gielinor_world::{
    collision::{CollisionMap, Strategy},
    loc_collision,
    tower_layout::{Source, Template},
    CoordGrid,
};
use std::{
    collections::{BTreeSet, VecDeque},
    env, fs,
    path::PathBuf,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        return Err("Usage: design_audit WORKSPACE_ROOT CANDIDATE.scene".into());
    }
    let root = PathBuf::from(&args[1]);
    let text = fs::read_to_string(&args[2])?;
    let mut cache = GameCache::open(
        &root.join("resource-staging/server/work/integration/a-kotlin/.data/cache/game"),
        8 << 20,
    )?;
    let defs = cache.definitions_all()?;
    let mut template = Template::new();
    for l in text.lines().filter(|l| l.starts_with("T ")) {
        let v = l
            .split_whitespace()
            .skip(1)
            .map(str::parse::<u16>)
            .collect::<Result<Vec<_>, _>>()?;
        template.insert(
            (v[0] as u8, v[1] as u8, v[2] as u8),
            Source {
                x: v[3],
                z: v[4],
                level: v[5] as u8,
                rotation: v[6] as u8,
            },
        );
    }
    let mut normal = CollisionMap::default();
    for id in template
        .values()
        .map(|s| ((s.x / 8) << 8) | (s.z / 8))
        .collect::<BTreeSet<_>>()
    {
        cache
            .map_square(id)?
            .ok_or("Missing map")?
            .terrain_collision(&mut normal);
    }
    let o = CoordGrid::new(10368, 64, 0).map_err(|e| format!("{e:?}"))?;
    let at = |x: i32, z: i32| o.translate(x, z, 0).unwrap();
    let mut map =
        gielinor_cache::constructed::build_small(o, &template, &normal, &[], &defs)?.collision;
    let mut loaded = BTreeSet::new();
    let mut wet = BTreeSet::new();
    for (&(zx, zz, _), s) in &template {
        for dx in 0..8 {
            for dz in 0..8 {
                let (x, z) = (zx as i32 * 8 + dx, zz as i32 * 8 + dz);
                loaded.insert((x, z));
                if (s.x, s.z) == (187, 566) {
                    assert_ne!(map.get(o.x() + x, o.z() + z, 0) & 0x200000, 0);
                    wet.insert((x, z));
                } else {
                    map.set(o.x() + x, o.z() + z, 0, 0);
                }
            }
        }
    }
    let mut objects = vec![];
    let mut ids = BTreeSet::new();
    for l in text.lines().filter(|l| l.starts_with("L ")) {
        let v = l
            .split_whitespace()
            .skip(1)
            .map(str::parse::<u32>)
            .collect::<Result<Vec<_>, _>>()?;
        let d = defs.locations.get(&v[3]).ok_or("Unknown location")?;
        loc_collision::toggle(
            &mut map,
            at(v[0] as i32, v[1] as i32),
            v[4] as u8,
            v[5] as u8,
            d.collision,
            true,
        )?;
        ids.insert(v[3]);
        objects.push(v);
    }
    let points = text
        .lines()
        .filter(|l| l.starts_with("P "))
        .map(|l| {
            let v: Vec<_> = l.split_whitespace().collect();
            (
                v[1].to_string(),
                v[2].parse::<i32>().unwrap(),
                v[3].parse::<i32>().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let start = points.iter().find(|p| p.0 == "entry").ok_or("No entry")?;
    let mut seen = BTreeSet::from([(start.1, start.2)]);
    let mut queue = VecDeque::from([(start.1, start.2)]);
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
    while let Some((x, z)) = queue.pop_front() {
        for &(dx, dz) in &steps {
            let p = (x + dx, z + dz);
            if loaded.contains(&p)
                && map.can_step(at(x, z), dx, dz, 1, 0, Strategy::Normal)
                && seen.insert(p)
            {
                queue.push_back(p);
            }
        }
    }
    let routes = points
        .iter()
        .filter(|p| p.0.starts_with("route-"))
        .collect::<Vec<_>>();
    for (index, pair) in routes.windows(2).enumerate() {
        let from = (pair[0].1, pair[0].2);
        let target = (pair[1].1, pair[1].2);
        let mut q = VecDeque::from([from]);
        let mut parents = std::collections::BTreeMap::from([(from, from)]);
        while let Some(p) = q.pop_front() {
            if p == target {
                break;
            }
            for &(dx, dz) in &steps {
                let n = (p.0 + dx, p.1 + dz);
                if loaded.contains(&n)
                    && !parents.contains_key(&n)
                    && map.can_step(at(p.0, p.1), dx, dz, 1, 0, Strategy::Normal)
                {
                    parents.insert(n, p);
                    q.push_back(n);
                }
            }
        }
        if parents.contains_key(&target) {
            let mut path = vec![target];
            let mut p = target;
            while p != from {
                p = parents[&p];
                path.push(p);
            }
            path.reverse();
            println!("SEGMENT {index} steps={}", path.len() - 1);
            for (x, z) in path {
                println!("PATH {index} {x} {z}");
            }
        }
    }
    let mut issues = vec![];
    for (name, x, z) in &points {
        if !seen.contains(&(*x, *z)) || map.get(o.x() + x, o.z() + z, 0) & 0x240100 != 0 {
            issues.push(format!("unreachable/blocked waypoint {name} {x},{z}"));
        }
    }
    for v in &objects {
        let d = &defs.locations[&v[3]];
        if !d.ops.iter().any(Option::is_some) {
            continue;
        }
        let (w, h) = if v[5] & 1 == 0 {
            (d.collision.width, d.collision.length)
        } else {
            (d.collision.length, d.collision.width)
        };
        let (x, z) = (v[0] as i32, v[1] as i32);
        let adjacent = (x - 1..=x + w as i32)
            .flat_map(|xx| (z - 1..=z + h as i32).map(move |zz| (xx, zz)))
            .any(|p| {
                seen.contains(&p)
                    && ((p.0 == x - 1 || p.0 == x + w as i32) && (z..z + h as i32).contains(&p.1)
                        || (p.1 == z - 1 || p.1 == z + h as i32)
                            && (x..x + w as i32).contains(&p.0))
            });
        if !adjacent {
            issues.push(format!(
                "no cardinal interaction approach for {} at{x},{z}",
                v[3]
            ));
        }
    }
    let mut escapes = 0;
    for l in text.lines().filter(|l| l.starts_with("A ")) {
        let q: Vec<_> = l.split_whitespace().collect();
        let v = q[2..]
            .iter()
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>();
        let (minx, minz, maxx, maxz, sx, sz, size) = (v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
        let mut occupied = BTreeSet::new();
        for x in sx..sx + size {
            for z in sz..sz + size {
                occupied.insert((x, z));
                assert_eq!(
                    map.get(o.x() + x, o.z() + z, 0) & 0x240100,
                    0,
                    "Boss footprint obstructed"
                );
            }
        }
        for x in minx..=maxx {
            for z in minz..=maxz {
                if occupied.contains(&(x, z)) || map.get(o.x() + x, o.z() + z, 0) & 0x240100 != 0 {
                    continue;
                }
                if !seen.contains(&(x, z)) {
                    issues.push(format!("unreachable {} court {x},{z}", q[1]));
                    continue;
                }
                for phase in 1..=3 {
                    for cycle in 0..3 {
                        let hot = |xx: i32, zz: i32| {
                            if q[1] == "imp" {
                                if phase == 1 {
                                    xx == x && zz == z
                                } else {
                                    zz == z && (xx - x).abs() <= 1
                                }
                            } else {
                                match phase {
                                    1 => xx == x,
                                    2 => xx == x || zz == z,
                                    _ => (xx + zz + cycle) % 3 == 0,
                                }
                            }
                        };
                        if !hot(x, z) {
                            continue;
                        }
                        let escape = steps.iter().any(|&(dx, dz)| {
                            let p = (x + dx, z + dz);
                            (minx..=maxx).contains(&p.0)
                                && (minz..=maxz).contains(&p.1)
                                && !occupied.contains(&p)
                                && !(dx != 0
                                    && dz != 0
                                    && (occupied.contains(&(x + dx, z))
                                        || occupied.contains(&(x, z + dz))))
                                && map.can_step(at(x, z), dx, dz, 1, 0, Strategy::Normal)
                                && !hot(p.0, p.1)
                        });
                        if !escape {
                            issues.push(format!("no {} phase{phase} escape {x},{z}", q[1]));
                        }
                        escapes += 1;
                    }
                }
            }
        }
        println!(
            "ARENA {} {}x{} spawn_footprint={}x{}",
            q[1],
            maxx - minx + 1,
            maxz - minz + 1,
            size,
            size
        );
    }
    for l in text.lines().filter(|l| l.starts_with("F ")) {
        let q: Vec<_> = l.split_whitespace().collect();
        let (x, z) = (q[2].parse::<i32>()?, q[3].parse::<i32>()?);
        if !wet.contains(&(x, z))
            || !seen
                .iter()
                .any(|p| (p.0 - x).abs().max((p.1 - z).abs()) <= 3)
        {
            issues.push(format!("fishing reach failure {}", q[1]));
        }
    }
    println!(
        "SUMMARY loaded={} reached={} water={} hazard_escapes={} issues={}",
        loaded.len(),
        seen.len(),
        wet.len(),
        escapes,
        issues.len()
    );
    for id in ids {
        let d = &defs.locations[&id];
        println!(
            "ASSET {id} {:?} {}x{} walk={} ops={:?}",
            d.name, d.collision.width, d.collision.length, d.collision.block_walk, d.ops
        );
    }
    for id in [16300, 16295, 2075, 3317, 1173, 1017] {
        if let Some(n) = defs.npcs.get(&id) {
            println!("NPC {id} size={} block={:?}", n.size, n.block_walk);
        }
    }
    for issue in &issues {
        println!("ISSUE {issue}");
    }
    if !issues.is_empty() {
        return Err("Candidate static feasibility issues; not accepted".into());
    }
    Ok(())
}
