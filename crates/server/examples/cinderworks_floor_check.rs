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
    if args.len() != 4 && args.len() != 5 {
        return Err("usage: cinderworks_floor_check GAME_CACHE SYMBOLS ART_TOWER [NEW_PRIVATE_REPORT_DIRECTORY]".into());
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
    // Exercise actual cache clipping with the authored population footprints,
    // not the obstacle-free arena unit fixture. Each warning permits four ticks;
    // require a legal safe position within at most three walking steps.
    let population=gielinor_server::cinderworks_population::resolve(floor::origin(),content.spatial(),&definitions,names.group("npc"))?;
    assert_eq!(population.len(),20);
    let mut occupied=HashSet::new();
    for npc in &population {
        for dx in 0..npc.movement.size {for dz in 0..npc.movement.size {
            occupied.insert(npc.spawn.translate(dx,dz,0).map_err(|e|format!("{e:?}"))?);
        }}
    }
    let mut escape_cases=0;
    for boss in [gielinor_world::cinderworks_encounter::Boss::Imp,gielinor_world::cinderworks_encounter::Boss::Warden] {
        let phases=if boss==gielinor_world::cinderworks_encounter::Boss::Imp {2}else{3};
        let aims=seen.iter().copied().filter(|at|boss.contains(floor::origin(),*at)&&!occupied.contains(at)).collect::<Vec<_>>();
        assert!(!aims.is_empty(),"No reachable authored guardian arena");
        for at in aims {for phase in 1..=phases {for cycle in 0..3 {
            let hot=boss.pattern(floor::origin(),phase,at,cycle);
            if !hot.contains(&at){continue;}
            let mut visited=HashSet::from([at]);let mut frontier=VecDeque::from([(at,0)]);let mut safe=false;
            while let Some((position,steps))=frontier.pop_front() {
                if !hot.contains(&position){safe=true;break;}
                if steps==3 {continue;}
                for dx in -1..=1 {for dz in -1..=1 {
                    if dx==0&&dz==0 {continue;}
                    if !world.collision.can_step(position,dx,dz,1,0,Strategy::Normal){continue;}
                    let next=position.translate(dx,dz,0).map_err(|e|format!("{e:?}"))?;
                    if !boss.contains(floor::origin(),next)||occupied.contains(&next){continue;}
                    if dx!=0&&dz!=0&&(occupied.contains(&position.translate(dx,0,0).unwrap())||occupied.contains(&position.translate(0,dz,0).unwrap())){continue;}
                    if visited.insert(next){frontier.push_back((next,steps+1));}
                }}
            }
            assert!(safe,"No authored escape: {boss:?} aim={at:?} phase={phase} cycle={cycle}");escape_cases+=1;
        }}}
    }
    println!("authored population={} collision-backed hazard escape cases={escape_cases}",population.len());
    // Optional local diagnostics adapted from the original broken_water_audit.
    // Existing output directories are refused. These derived cache reports are
    // private verification inputs, not distributable resource packages.
    if let Some(output)=args.get(4) {
        let output=Path::new(output);std::fs::create_dir(output)?;
        let origin=floor::origin();
        let mut scene=String::from("# Authoritative server template/locations; pinned floor.bwr is separately required.\n");
        for ((x,z,level),source) in floor::template(){scene.push_str(&format!("T {x} {z} {level} {} {} {} {}\n",source.x,source.z,source.level,source.rotation));}
        let mut objects=String::from("x\tz\tid\tshape\tangle\twidth\tlength\tblock_walk\tname\tops\n");
        for loc in &published_locations {
            let ty=&definitions.locations[&loc.id];let (x,z)=(loc.coords.x()-origin.x(),loc.coords.z()-origin.z());
            scene.push_str(&format!("L {x} {z} {} {} {} {}\n",loc.coords.level(),loc.id,loc.shape,loc.angle));
            objects.push_str(&format!("{x}\t{z}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:?}\n",loc.id,loc.shape,loc.angle,ty.collision.width,ty.collision.length,ty.collision.block_walk,ty.name.replace(['\t','\n','\r']," "),ty.ops));
        }
        let mut flags=String::from("x\tz\tflags\n");let mut edges=String::from("x\tz\tdx\tdz\n");
        for x in 0..128 {for z in 0..128 {
            let at=origin.translate(x,z,0).unwrap();flags.push_str(&format!("{x}\t{z}\t{}\n",world.collision.get(at.x(),at.z(),0)));
            if seen.contains(&at) {for dx in -1..=1 {for dz in -1..=1 {
                if (dx!=0||dz!=0)&&world.collision.can_step(at,dx,dz,1,0,Strategy::Normal){edges.push_str(&format!("{x}\t{z}\t{dx}\t{dz}\n"));}
            }}}
        }}
        let mut combat=String::from("boss\tx\tz\tdx\tdz\tcan_step\n");
        for boss in [gielinor_world::cinderworks_encounter::Boss::Imp,gielinor_world::cinderworks_encounter::Boss::Warden] {
            let (min,max)=boss.bounds();
            for x in min[0]..=max[0] {for z in min[1]..=max[1] {let at=origin.translate(x,z,0).unwrap();for dx in -1..=1 {for dz in -1..=1 {
                if dx!=0||dz!=0 {combat.push_str(&format!("{boss:?}\t{x}\t{z}\t{dx}\t{dz}\t{}\n",world.collision.can_step(at,dx,dz,1,0,Strategy::Normal)));}
            }}}}
        }
        let mut actors=String::from("index\tid\tx\tz\tsize\tblocked_static_cells\twaterborne\n");
        for (index,npc) in population.iter().enumerate(){
            let mut blocked=0;for dx in 0..npc.movement.size {for dz in 0..npc.movement.size {
                if world.collision.get(npc.spawn.x()+dx,npc.spawn.z()+dz,0)&0x200100!=0 {blocked+=1;}
            }}
            actors.push_str(&format!("{index}\t{}\t{}\t{}\t{}\t{blocked}\t{}\n",npc.type_id,npc.spawn.x()-origin.x(),npc.spawn.z()-origin.z(),npc.movement.size,(14..18).contains(&index)));
        }
        for (name,text) in [("prepared.scene",scene),("objects.tsv",objects),("collision-flags.tsv",flags),("walk-edges.tsv",edges),("combat-eight-direction-steps.tsv",combat),("npc-footprints.tsv",actors)] {std::fs::write(output.join(name),text)?;}
        std::fs::write(output.join("audit.json"),format!("{{\"scope\":\"Static authored preparation/collision; no earned gameplay\",\"origin\":[10368,64,0],\"objects\":{},\"reachable_tiles\":{},\"population\":{},\"hazard_escape_cases\":{escape_cases}}}\n",published_locations.len(),seen.len(),population.len()))?;
        println!("private authored reports written to {}",output.display());
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
