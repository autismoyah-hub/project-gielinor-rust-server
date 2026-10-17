// Private cache-backed design export; no running-world writes.
use gielinor_cache::{game::GameCache, symbols};
use gielinor_world::{collision::CollisionMap, tower_layout, CoordGrid};
use std::{collections::BTreeSet, fs, path::PathBuf};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "Usage: tower-scene-export WORKSPACE_ROOT OUTPUT_DIRECTORY"
    );
    let root = PathBuf::from(&args[1]).canonicalize().unwrap();
    let assets = root.join("resource-staging/server/work/integration/a-kotlin/.data");
    let mut c = GameCache::open(&assets.join("cache/game"), 8 << 20).unwrap();
    let d = c.definitions_all().unwrap();
    let symbols = symbols::parse(
        &fs::read_to_string(assets.join("symbols/loc.sym")).unwrap(),
        None,
    )
    .unwrap();
    let template = tower_layout::template();
    let ids = template
        .values()
        .map(|s| ((s.x / 8) << 8) | (s.z / 8))
        .collect::<BTreeSet<_>>();
    let mut normal = CollisionMap::default();
    let mut locs = vec![];
    for id in ids {
        let m = c.map_square(id).unwrap().unwrap();
        m.terrain_collision(&mut normal);
        locs.extend(m.location_collision(&d, &mut normal).unwrap());
    }
    let o = CoordGrid::new(10368, 64, 0).unwrap();
    let shell = gielinor_server::tower_map::prepare(o, &normal, &locs, &d, &symbols).unwrap();
    let mut out = String::from(
        "# OFFLINE authoritative prepared cold shell; no actors/UI/animation/zone lifecycle\n",
    );
    for (&(x, z, level), s) in &template {
        out += &format!(
            "T {x} {z} {level} {} {} {} {}\n",
            s.x, s.z, s.level, s.rotation
        );
    }
    let mut prepared_audit =
        String::from("local_x\tlocal_z\tid\tshape\tangle\tname\twidth\tlength\twalk\tops\n");
    for l in shell.map.locations {
        let (x, z) = (l.coords.x() - o.x(), l.coords.z() - o.z());
        if (48..=64).contains(&x) && (48..=64).contains(&z) && l.coords.level() == 0 {
            let ty = &d.locations[&l.id];
            prepared_audit += &format!(
                "{x}\t{z}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\n",
                l.id,
                l.shape,
                l.angle,
                ty.name,
                ty.collision.width,
                ty.collision.length,
                ty.collision.block_walk,
                ty.ops
            );
        }
        out += &format!(
            "L {} {} {} {} {} {}\n",
            l.coords.x() - o.x(),
            l.coords.z() - o.z(),
            l.coords.level(),
            l.id,
            l.shape,
            l.angle
        );
    }
    let dir = PathBuf::from(&args[2]);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("baseline.scene"), out).unwrap();
    fs::write(dir.join("prepared-hub-audit.tsv"), prepared_audit).unwrap();
    let mut audit = String::from("local_x\tlocal_z\tsymbol\tid\tname\twidth\tlength\twalk\tops\n");
    for row in gielinor_world::tower_furnishings::FURNISHINGS {
        if (48..=64).contains(&row.x) && (48..=64).contains(&row.z) {
            let id = symbols.get(row.symbol).unwrap();
            let t = &d.locations[&(*id as u32)];
            audit += &format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\n",
                row.x,
                row.z,
                row.symbol,
                id,
                t.name,
                t.collision.width,
                t.collision.length,
                t.collision.block_walk,
                t.ops
            );
        }
    }
    fs::write(dir.join("sanctuary-furnishings-audit.tsv"), audit).unwrap();
    println!("Exported exact template and prepared shell; sanctuary furnishings audited");
}
