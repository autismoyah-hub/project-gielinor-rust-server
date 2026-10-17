//! Detached Floor 1 location preparation. Publication, NPCs and relocation are
//! deliberately separate so partial content cannot enter an accepting runtime.
use gielinor_cache::{
    game::{Definitions, StaticLocation},
    symbols::Symbols,
    tower_shell::Shell,
};
use gielinor_world::{
    collision::CollisionMap, tower_furnishings::FURNISHINGS, tower_layout, CoordGrid,
};

/// Consume an unpublished world so any validation failure discards the entire
/// attempted publication. NPC Create bindings must already be configured.
pub fn publish(
    mut world: gielinor_world::simulation::World,
    origin: CoordGrid,
    shell: Shell,
    population: Vec<gielinor_world::npc::Npc>,
) -> Result<
    (
        gielinor_world::simulation::World,
        crate::constructed::Regions,
    ),
    String,
> {
    use gielinor_world::locations::Location;
    let region = crate::constructed::Region::small(origin, tower_layout::template())?;
    if shell
        .base
        .iter()
        .chain(&shell.map.locations)
        .any(|loc| !region.contains(loc.coords))
        || population.iter().any(|npc| !region.contains(npc.spawn))
    {
        return Err("Tower publication contains an out-of-region entity".into());
    }
    world.locations.register_constructed(origin, 128)?;
    world
        .area_sources
        .register(origin, &tower_layout::template())?;
    world.collision.append_disjoint(shell.map.collision)?;
    // The client rebuilds the unmodified cache template. Load that baseline,
    // mask it, then restore the final script state. Exact retained locations
    // collapse back to map entries; replacements and removals remain deltas.
    // Collision is already prepared in script order: never apply it twice.
    for loc in shell.base {
        let loc = Location::new(loc.coords, loc.id, loc.shape, loc.angle)?;
        world.locations.load(loc);
        world.locations.delete(loc);
    }
    for loc in shell.map.locations {
        world
            .locations
            .add(Location::new(loc.coords, loc.id, loc.shape, loc.angle)?)?;
    }
    for npc in population {
        world
            .spawn_npc(npc)
            .map_err(|_| "Tower NPC capacity exceeded")?;
    }
    Ok((world, crate::constructed::Regions::new(vec![region])?))
}

pub fn prepare(
    origin: CoordGrid,
    normal: &CollisionMap,
    locations: &[StaticLocation],
    definitions: &Definitions,
    symbols: &Symbols,
) -> Result<Shell, String> {
    let resolve = |name: &str| {
        symbols
            .get(name)
            .and_then(|id| u32::try_from(*id).ok())
            .ok_or_else(|| format!("Missing tower loc symbol {name}"))
    };
    let copied = gielinor_cache::constructed::build_small(
        origin,
        &tower_layout::template(),
        normal,
        locations,
        definitions,
    )?;
    let palette = gielinor_world::tower_scenery::MASONRY;
    for id in palette {
        let ty = definitions
            .locations
            .get(&id)
            .ok_or("Missing fortress masonry")?;
        if ty.ops.iter().any(Option::is_some) {
            return Err(format!("Interactive fortress masonry {id}"));
        }
    }
    let mut shell =
        gielinor_cache::tower_shell::prepare_palette(copied, origin, palette, definitions)?;
    // The revision 239.1 JS5 map contains five conveyor-frame decorations
    // which the paired server game-map intentionally omits. They still arrive
    // in the native constructed scene. Register their exact cache baseline so
    // normal zone deletes remove them, including on relog and region rebuild.
    // Do not change the cache or suppress models in the client renderer.
    for (&(x, z, level), source) in &tower_layout::template() {
        if (source.x, source.z, source.level, source.rotation) != (242, 620, 0, 0) {
            continue;
        }
        for (dx, id) in [(2, 9109), (3, 9110), (4, 9111), (5, 9112), (6, 9113)] {
            if !definitions.locations.contains_key(&id) {
                return Err(format!("Missing client-only conveyor definition {id}"));
            }
            let loc = StaticLocation {
                id,
                coords: origin
                    .translate(x as i32 * 8 + dx, z as i32 * 8 + 7, level as i32)
                    .map_err(|e| format!("{e:?}"))?,
                shape: 10,
                angle: 1,
            };
            shell.base.push(loc.clone());
            shell.removed.push(loc);
        }
    }
    let mut put = |x, z, level, name| -> Result<(), String> {
        shell.stamp(
            origin
                .translate(x, z, level)
                .map_err(|e| format!("{e:?}"))?,
            resolve(name)?,
            definitions,
        )
    };
    // Authored working props only: no prototype staff crate, unused altar,
    // vial crate, goats, goblin training room or unreachable upstairs scene.
    for (x, z, level, name) in [
        (53, 60, 0, "aide_bankbooth"),
        (55, 51, 0, "threshold_chest"),
        (60, 52, 0, "altar"),
        (34, 70, 0, "tree"),
        (42, 70, 0, "tree"),
        (34, 76, 0, "tree"),
        (42, 76, 0, "tree"),
        (34, 54, 0, "tinrock1"),
        (34, 58, 0, "tinrock1"),
        (44, 54, 0, "copperrock1"),
        (44, 58, 0, "copperrock1"),
        (34, 50, 0, "tut2_furnace"),
        (37, 50, 0, "anvil"),
        (67, 53, 0, "newbierange"),
    ] {
        put(x, z, level, name)?;
    }
    for (x, z) in gielinor_world::tower_cinderworks::COOLANT_TILES {
        put(x, z, 0, "mistmyst_barrel_water")?;
    }
    for row in FURNISHINGS {
        put(row.x as i32, row.z as i32, 0, row.symbol)?;
    }
    // Exact real cache pairs share clipping and expose no gameplay actions.
    // Their per-restorer state is derived in tower_scene after ordinary zones.
    for row in gielinor_world::tower_scenery::LANDMARKS {
        let cold = definitions
            .locations
            .get(&row.dormant)
            .ok_or("Missing dormant landmark")?;
        let hot = definitions
            .locations
            .get(&row.restored)
            .ok_or("Missing restored landmark")?;
        let clip = |t: &gielinor_cache::locations::LocationType| {
            let c = t.collision;
            (
                c.width,
                c.length,
                c.block_walk,
                c.block_range,
                c.break_route_finding,
            )
        };
        if clip(cold) != clip(hot) || cold.ops.iter().chain(&hot.ops).any(Option::is_some) {
            return Err(format!(
                "Unsafe restoration pair {} / {}",
                row.dormant, row.restored
            ));
        }
        if row.angle > 3 {
            return Err("Invalid landmark rotation".into());
        }
        let (width, length) = if row.angle & 1 == 0 {
            (cold.collision.width, cold.collision.length)
        } else {
            (cold.collision.length, cold.collision.width)
        };
        if row.x + width as i32 > 80 || row.z + length as i32 > 80 {
            return Err("Landmark extends beyond authored floor".into());
        }
        shell.stamp_shaped(
            origin.translate(row.x, row.z, 0).unwrap(),
            row.dormant,
            10,
            row.angle,
            definitions,
        )?;
    }
    for f in gielinor_world::tower_scenery::FIXTURES {
        let t = definitions
            .locations
            .get(&f.id)
            .ok_or("Missing fortress fixture")?;
        if t.ops.iter().any(Option::is_some) {
            return Err(format!("Unimplemented fixture operation {}", f.id));
        }
        shell.stamp_shaped(
            origin.translate(f.x, f.z, 0).unwrap(),
            f.id,
            f.shape,
            f.angle,
            definitions,
        )?;
    }
    let (x, z) = gielinor_world::tower_scenery::TRAINING_COOLANT;
    shell.stamp(
        origin.translate(x, z, 0).unwrap(),
        resolve("mistmyst_barrel_water")?,
        definitions,
    )?;
    shell.stamp_shaped(
        origin.translate(54, 51, 0).unwrap(),
        resolve("plaque")?,
        4,
        1,
        definitions,
    )?;
    shell.stamp_shaped(
        origin.translate(64, 60, 0).unwrap(),
        resolve("threshold_portal")?,
        0,
        0,
        definitions,
    )?;
    if std::env::var_os("TOWER_ASSET_PACKAGE").is_some() {
        // Experimental compound collision: open visual object plus ordinary
        // single-tile ground-location clipping underneath each stone column.
        for (x,z,angle) in [(54,40,0),(60,38,1)] {
            shell.stamp_shaped(origin.translate(x,z,0).unwrap(),65000,10,angle,definitions)?;
            for (dx,dz) in if angle==0 { [(0,0),(3,0)] } else { [(0,0),(0,3)] } {
                shell.stamp_shaped(origin.translate(x+dx,z+dz,0).unwrap(),65001,22,0,definitions)?;
            }
        }
        shell.stamp_shaped(origin.translate(52,40,0).unwrap(),7016,10,0,definitions)?;
    }
    Ok(shell)
}

/// Permanent startup population in Kotlin's spawn order. Trainer NPCs retain
/// their cache default mode; passive actors and the two scripted bosses use None.
pub fn npcs(
    origin: CoordGrid,
    definitions: &Definitions,
    symbols: &Symbols,
) -> Result<Vec<gielinor_world::npc::Npc>, String> {
    use gielinor_world::npc::Mode;
    let mut result = Vec::new();
    for (x, z, level, name, passive) in [
        (52, 60, 0, "threshold_banker", true),
        (61, 58, 0, "threshold_tanner", true),
        (34, 34, 0, "cow", false),
        (42, 34, 0, "cow", false),
        (34, 42, 0, "cow", false),
        (42, 42, 0, "cow", false),
        (38, 34, 0, "cow", false),
        (34, 38, 0, "cow", false),
        (42, 38, 0, "cow", false),
        (38, 42, 0, "cow", false),
        (66, 34, 0, "chicken", false),
        (76, 34, 0, "chicken", false),
        (66, 44, 0, "chicken", false),
        (76, 44, 0, "chicken", false),
        (72, 50, 0, "0_48_48_newbiefishing", true),
        (72, 52, 0, "0_48_48_newbiefishing", true),
        (72, 54, 0, "0_48_48_newbiefishing", true),
        (72, 56, 0, "0_48_48_newbiefishing", true),
        (72, 72, 0, "ash_imp", true),
        (56, 72, 0, "ash_warden", true),
    ] {
        let id = if name == "ash_warden" {
            Some(gielinor_world::tower_cinderworks::WARDEN)
        } else {
            symbols.get(name).and_then(|id| u16::try_from(*id).ok())
        }
        .ok_or_else(|| format!("Missing tower npc symbol {name}"))?;
        let ty = definitions
            .npcs
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing tower npc definition {id}"))?;
        let coords = origin
            .translate(x, z, level)
            .map_err(|e| format!("{e:?}"))?;
        let mut npc = ty.spawn(id, coords)?;
        if passive {
            npc.mode = Mode::None;
        }
        // NpcRepository.add(Int.MAX_VALUE) owns this flag, not Npc's constructor.
        npc.respawns = true;
        if name == "ash_warden" {
            npc.regen_rate = 0;
            npc.respawn_rate = 8;
            npc.stats = [1, 1, 1, 42, 1, 80];
            npc.base_stats = npc.stats;
        } else if name == "ash_imp" {
            npc.regen_rate = 0;
            npc.respawn_rate = 8;
            npc.stats[..4].copy_from_slice(&[1, 1, 1, 18]);
            npc.base_stats[..4].copy_from_slice(&[1, 1, 1, 18]);
        }
        result.push(npc);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_preserves_cache_deltas_and_prepared_collision() {
        use gielinor_world::{locations::DELETED_ID, simulation::World};
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let loc = |x, id| StaticLocation {
            coords: origin.translate(x, 56, 0).unwrap(),
            id,
            shape: 10,
            angle: 0,
        };
        let retained = loc(58, 100);
        let removed = loc(56, 101);
        let replaced = loc(57, 102);
        let replacement = loc(57, 103);
        let mut collision = CollisionMap::default();
        collision.set(origin.x() + 56, origin.z() + 56, 0, 0x200000);
        let shell = Shell {
            map: gielinor_cache::constructed::Built {
                collision,
                locations: vec![retained.clone(), replacement.clone()],
                remapped: vec![],
            },
            base: vec![retained.clone(), removed.clone(), replaced],
            removed: vec![removed.clone()],
        };
        let mut normal = World::default();
        normal.collision.set(3222, 3218, 0, 0x100);
        let (world, regions) = publish(normal, origin, shell, vec![]).unwrap();
        assert_eq!(world.collision.get(3222, 3218, 0), 0x100);
        assert_eq!(
            world
                .collision
                .get(removed.coords.x(), removed.coords.z(), 0),
            0x200000
        );
        assert!(world
            .locations
            .find_type(retained.coords, retained.id)
            .is_some());
        assert!(world
            .locations
            .find_type(removed.coords, removed.id)
            .is_none());
        assert!(world
            .locations
            .find_type(replacement.coords, replacement.id)
            .is_some());
        let deltas = world.locations.spawned(retained.coords).collect::<Vec<_>>();
        assert_eq!(deltas.len(), 2);
        assert!(deltas
            .iter()
            .any(|l| l.coords == removed.coords && l.id == DELETED_ID));
        assert!(deltas
            .iter()
            .any(|l| l.coords == replacement.coords && l.id == replacement.id));
        assert_eq!(
            regions.rebuild(retained.coords).unwrap().unwrap().opcode,
            125
        );
    }
    #[test]
    fn floor_population_preserves_modes_respawns_and_partial_boss_overrides() {
        let mut definitions = Definitions {
            inventories: Default::default(),
            items: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            varps: Default::default(),
            varbits: Default::default(),
            stats: Default::default(),
        };
        let mut symbols = Symbols::new();
        for (index, name) in [
            "threshold_banker",
            "threshold_tanner",
            "threshold_goat",
            "cow",
            "chicken",
            "goblin",
            "0_48_48_newbiefishing",
            "ash_imp",
            "ash_warden",
        ]
        .iter()
        .enumerate()
        {
            symbols.insert((*name).into(), index as i32);
            let mut ty = gielinor_cache::npcs::NpcType::decode(&[0]).unwrap();
            ty.stats = [11, 12, 13, 14, 15, 16];
            ty.default_mode = 1;
            definitions.npcs.insert(index as u32, ty.clone());
            if *name == "ash_warden" {
                definitions
                    .npcs
                    .insert(gielinor_world::tower_cinderworks::WARDEN as u32, ty);
            }
        }
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let actors = npcs(origin, &definitions, &symbols).unwrap();
        assert_eq!(actors.len(), 20);
        assert!(actors.iter().all(|n| n.spawn.level() == 0));
        assert!(actors.iter().all(|n| n.respawns));
        assert_eq!(actors[0].spawn.level(), 0);
        assert_eq!(actors[1].spawn.level(), 0);
        assert_eq!(actors[0].mode, gielinor_world::npc::Mode::None);
        assert_eq!(actors[2].mode, gielinor_world::npc::Mode::Wander);
        assert_eq!(actors[2].stats, [11, 12, 13, 14, 15, 16]);
        assert_eq!(actors[18].stats, [1, 1, 1, 18, 15, 16]);
        assert_eq!(actors[18].base_stats, actors[18].stats);
        assert_eq!(actors[19].stats, [1, 1, 1, 42, 1, 80]);
        assert_eq!(actors[19].base_stats, actors[19].stats);
        assert_eq!(actors[19].spawn, origin.translate(56, 72, 0).unwrap());
    }
}
