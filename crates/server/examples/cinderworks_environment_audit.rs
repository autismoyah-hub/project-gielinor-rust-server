//! Read-only exact prepared scene and ordinary server collision observations.
use gielinor_cache::{game::GameCache, symbols};
use gielinor_world::{
    collision::{CollisionMap, Strategy},
    tower_layout, CoordGrid,
};
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    path::PathBuf,
    time::Instant,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().collect();
    if a.len() != 3 {
        return Err("DATA_ROOT OUTPUT".into());
    }
    let started = Instant::now();
    let root = PathBuf::from(&a[1]);
    let out = PathBuf::from(&a[2]);
    fs::create_dir_all(&out)?;
    let mut cache = GameCache::open(&root.join("cache/game"), 8 << 20)?;
    let d = cache.definitions_all()?;
    let names = symbols::parse(&fs::read_to_string(root.join("symbols/loc.sym"))?, None)
        .map_err(|e| format!("{e:?}"))?;
    let template = tower_layout::template();
    let ids = template
        .values()
        .map(|s| ((s.x / 8) << 8) | (s.z / 8))
        .collect::<BTreeSet<_>>();
    let mut normal = CollisionMap::default();
    let mut locations = Vec::new();
    for id in ids {
        let m = cache.map_square(id)?.ok_or("source map missing")?;
        m.terrain_collision(&mut normal);
        locations.extend(m.location_collision(&d, &mut normal)?);
    }
    let o = CoordGrid::new(10368, 64, 0).unwrap();
    let shell = gielinor_server::tower_map::prepare(o, &normal, &locations, &d, &names)?;
    let mut scene = String::from("# Authoritative prepared Floor One; private cache identifiers\n");
    for (&(x, z, l), s) in &template {
        scene += &format!("T {x} {z} {l} {} {} {} {}\n", s.x, s.z, s.level, s.rotation);
    }
    let mut inventory =
        String::from("x\tz\tid\tshape\tangle\twidth\tlength\tblock_walk\tname\tops\n");
    for l in &shell.map.locations {
        let t = &d.locations[&l.id];
        let (x, z) = (l.coords.x() - o.x(), l.coords.z() - o.z());
        scene += &format!(
            "L {x} {z} {} {} {} {}\n",
            l.coords.level(),
            l.id,
            l.shape,
            l.angle
        );
        inventory += &format!(
            "{x}\t{z}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\n",
            l.id,
            l.shape,
            l.angle,
            t.collision.width,
            t.collision.length,
            t.collision.block_walk,
            t.name.replace(['\t', '\n'], " "),
            t.ops
        );
    }
    let mut edges = String::from("x\tz\tdx\tdz\n");
    let mut flags = String::from("x\tz\tflags\n");
    let mut reached = BTreeSet::new();
    let mut q = VecDeque::new();
    reached.insert((56, 56));
    q.push_back((56, 56));
    while let Some((x, z)) = q.pop_front() {
        let p = o.translate(x, z, 0).unwrap();
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let n = (x + dx, z + dz);
            if (32..80).contains(&n.0)
                && (32..80).contains(&n.1)
                && shell
                    .map
                    .collision
                    .can_step(p, dx, dz, 1, 0, Strategy::Normal)
                && reached.insert(n)
            {
                q.push_back(n);
            }
        }
    }
    for x in 32..80 {
        for z in 32..80 {
            let p = o.translate(x, z, 0).unwrap();
            flags += &format!("{x}\t{z}\t{}\n", shell.map.collision.get(p.x(), p.z(), 0));
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                if (32..80).contains(&(x + dx))
                    && (32..80).contains(&(z + dz))
                    && shell
                        .map
                        .collision
                        .can_step(p, dx, dz, 1, 0, Strategy::Normal)
                {
                    edges += &format!("{x}\t{z}\t{dx}\t{dz}\n");
                }
            }
        }
    }
    fs::write(out.join("prepared.scene"), scene)?;
    fs::write(out.join("objects.tsv"), inventory)?;
    fs::write(out.join("walk-edges.tsv"), edges)?;
    fs::write(out.join("collision-flags.tsv"), flags)?;
    let mut combat = String::from("x\tz\tdx\tdz\tcan_step\n");
    for (min, max) in [([50, 67], [62, 77]), ([68, 68], [75, 75])] {
        for x in min[0]..=max[0] {
            for z in min[1]..=max[1] {
                let p = o.translate(x, z, 0).unwrap();
                for (dx, dz) in [
                    (1, 0),
                    (-1, 0),
                    (0, 1),
                    (0, -1),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ] {
                    combat += &format!(
                        "{x}\t{z}\t{dx}\t{dz}\t{}\n",
                        shell
                            .map
                            .collision
                            .can_step(p, dx, dz, 1, 0, Strategy::Normal)
                    );
                }
            }
        }
    }
    fs::write(out.join("combat-eight-direction-steps.tsv"), combat)?;
    fs::write(out.join("audit.json"),format!("{{\"origin\":[10368,64,0],\"reachable_tiles\":{},\"objects\":{},\"elapsed_ms\":{:.3},\"authoritative_prepared_collision\":true}}\n",reached.len(),shell.map.locations.len(),started.elapsed().as_secs_f64()*1000.))?;
    println!(
        "PREPARED_SCENE {} objects; {} reachable tiles; {:.1}ms",
        shell.map.locations.len(),
        reached.len(),
        started.elapsed().as_secs_f64() * 1000.
    );
    Ok(())
}
