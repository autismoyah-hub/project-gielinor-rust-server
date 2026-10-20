//! Private resource inputs, public read-only scene/collision diagnostic.
//! No account state, sockets, synthetic rewards or earned gameplay claims.
use gielinor_cache::game::GameCache;
use gielinor_server::{cinderworks_content::Content, startup::Names, tower_map};
use gielinor_world::{
    cinderworks_floor as floor,
    collision::{CollisionMap, Strategy},
    loc_reach::Target,
    reach::Rectangle,
    simulation::World,
};
use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    path::Path,
};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 {
        return Err("usage: cinderworks_floor_check GAME_CACHE SYMBOLS ART_TOWER".into());
    }
    let content = Content::load(Path::new(&args[3]))?;
    let names = Names::load(&[Path::new(&args[2])])?;
    let mut cache = GameCache::open(Path::new(&args[1]), 0xff_ffff)?;
    let mut definitions = cache.definitions_all()?;
    content
        .package()
        .install(&mut definitions, &cache.groups(7)?.into_iter().collect())?;
    let mut normal = CollisionMap::default();
    let mut locations = Vec::new();
    let maps = floor::template()
        .values()
        .map(|s| ((s.x / 8) << 8) | (s.z / 8))
        .collect::<BTreeSet<_>>();
    for id in maps {
        let map = cache.map_square(id)?.ok_or("Missing floor source map")?;
        map.terrain_collision(&mut normal);
        locations.extend(map.location_collision(&definitions, &mut normal)?);
    }
    let shell = content.prepare(&normal, &locations, &definitions, names.group("loc"))?;
    let published_locations = shell.map.locations.clone();
    let mut world = World::default();
    world.collision.set(3222, 3218, 0, 0x100);
    let (world, regions) =
        tower_map::publish_with_template(world, floor::origin(), shell, vec![], floor::template())?;
    assert_eq!(world.collision.get(3222, 3218, 0), 0x100);
    let spawn = content
        .spatial()
        .at(floor::origin(), 56, 56)
        .map_err(|e| format!("{e:?}"))?;
    assert_eq!((spawn.x(), spawn.z(), spawn.level()), (10420, 110, 0));
    assert_eq!(world.collision.get(spawn.x(), spawn.z(), 0) & 0x240100, 0);
    assert!(regions
        .rebuild(spawn)
        .map_err(|e| format!("{e:?}"))?
        .is_some());
    let mut seen = HashSet::from([spawn]);
    let mut queue = VecDeque::from([spawn]);
    while let Some(at) = queue.pop_front() {
        for dx in -1..=1 {
            for dz in -1..=1 {
                if world.collision.can_step(at, dx, dz, 1, 0, Strategy::Normal) {
                    let next = at.translate(dx, dz, 0).map_err(|e| format!("{e:?}"))?;
                    if seen.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
        }
    }
    // Exact gameplay anchors from the current floor handoff, not approximate
    // screen positions. Use production collision and production reach dispatch.
    for (x, z, id, op, shape) in [
        (10417, 114, 18491, 2, 10), // bank
        (10418, 105, 909, 1, 10),   // freestanding plaque
        (10419, 105, 57694, 1, 10), // reward chest (no claim exercised)
        (10424, 106, 409, 1, 10),   // altar
        (10428, 114, 57692, 1, 0),  // portal
        (10398, 104, 37947, 1, 10), // furnace
        (10401, 106, 2097, 1, 10),  // anvil
        (10432, 107, 9736, 1, 10),  // range
    ] {
        let loc = published_locations
            .iter()
            .find(|l| l.coords.x() == x && l.coords.z() == z && l.id == id)
            .ok_or("Missing exact functional anchor")?;
        assert_eq!(loc.shape, shape);
        let ty = &definitions.locations[&id];
        assert!(ty.ops[op - 1].is_some(), "missing operation {id}:{op}");
        let target = Target {
            rectangle: Rectangle {
                coords: loc.coords,
                width: ty.collision.width.into(),
                length: ty.collision.length.into(),
                angle: loc.angle,
                blocked: ty.force_approach,
                exclusive: false,
            },
            shape: loc.shape as i8,
        };
        assert!(
            seen.iter()
                .any(|at| target.reached(&world.collision, *at, 1)),
            "unreachable functional anchor {id} at{x},{z}"
        );
        println!("reachable anchor id={id} at={x},{z} op={op} shape={shape}");
    }
    assert_eq!(content.floor().cells.len(), 16384);
    assert_eq!(content.floor().objects.len(), 74);
    println!("authored floor prepared/published cells={} objects={} scene_locations={} reachable_tiles={} package=630bbb1bd738d4a105493f510b7273c51a6cfaca789bb7ae2d95c40d2bdb32d0",content.floor().cells.len(),content.floor().objects.len(),published_locations.len(),seen.len());
    Ok(())
}
fn main() {
    if let Err(error) = run() {
        eprintln!("Floor diagnostic: {error}");
        std::process::exit(1);
    }
}
