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
    let mut shell = gielinor_cache::tower_shell::prepare(
        copied,
        origin,
        resolve("threshold_wall")?,
        definitions,
    )?;
    let mut put = |x, z, level, name| -> Result<(), String> {
        shell.stamp(
            origin
                .translate(x, z, level)
                .map_err(|e| format!("{e:?}"))?,
            resolve(name)?,
            definitions,
        )
    };
    // stampFloor through stampFloor2Stub, followed by the preferred stair
    // landing. ensureHubWalkable has opened that upper-hub tile already.
    for (x, z, level, name) in [
        (53, 60, 0, "aide_bankbooth"),
        (54, 51, 0, "plaque"),
        (55, 51, 0, "threshold_chest"),
        (53, 52, 0, "mind_altar"),
        (60, 52, 0, "altar"),
        (53, 58, 0, "threshold_vial_crate"),
        (60, 60, 0, "threshold_portal"),
        (53, 60, 1, "aide_bankbooth"),
        (56, 52, 1, "plaque"),
        (60, 60, 1, "threshold_portal"),
        (60, 52, 0, "altar"),
        (56, 76, 0, "threshold_stair"),
        (56, 60, 1, "threshold_stair"),
        (58, 43, 0, "crate_old"),
        (34, 70, 0, "tree"),
        (42, 70, 0, "tree"),
        (34, 76, 0, "tree"),
        (42, 76, 0, "tree"),
        (34, 54, 0, "tinrock1"),
        (34, 58, 0, "tinrock1"),
        (44, 54, 0, "copperrock1"),
        (44, 58, 0, "copperrock1"),
        (34, 50, 0, "newbiefurnace"),
        (37, 50, 0, "anvil"),
        (67, 53, 0, "newbierange"),
    ] {
        put(x, z, level, name)?;
    }
    for row in FURNISHINGS {
        put(row.x as i32, row.z as i32, 0, row.symbol)?;
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
        (52, 60, 1, "threshold_banker", true),
        (52, 60, 0, "threshold_banker", true),
        (61, 58, 0, "threshold_tanner", true),
        (50, 54, 0, "threshold_goat", false),
        (61, 54, 0, "threshold_goat", false),
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
        (51, 35, 0, "goblin", false),
        (60, 35, 0, "goblin", false),
        (51, 44, 0, "goblin", false),
        (60, 44, 0, "goblin", false),
        (74, 52, 0, "0_48_48_newbiefishing", true),
        (76, 54, 0, "0_48_48_newbiefishing", true),
        (74, 58, 0, "0_48_48_newbiefishing", true),
        (76, 56, 0, "0_48_48_newbiefishing", true),
        (72, 72, 0, "ash_imp", true),
        (56, 72, 0, "ash_warden", true),
    ] {
        let id = symbols
            .get(name)
            .and_then(|id| u16::try_from(*id).ok())
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
            npc.stats = [1, 1, 1, 24, 1, 80];
            npc.base_stats = npc.stats;
        } else if name == "ash_imp" {
            npc.stats[..4].copy_from_slice(&[1, 1, 1, 6]);
            npc.base_stats[..4].copy_from_slice(&[1, 1, 1, 6]);
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
            definitions.npcs.insert(index as u32, ty);
        }
        let origin = CoordGrid::new(10368, 64, 0).unwrap();
        let actors = npcs(origin, &definitions, &symbols).unwrap();
        assert_eq!(actors.len(), 27);
        assert!(actors.iter().all(|n| n.respawns));
        assert_eq!(actors[0].spawn.level(), 1);
        assert_eq!(actors[1].spawn.level(), 0);
        assert_eq!(actors[0].mode, gielinor_world::npc::Mode::None);
        assert_eq!(actors[5].mode, gielinor_world::npc::Mode::Wander);
        assert_eq!(actors[5].stats, [11, 12, 13, 14, 15, 16]);
        assert_eq!(actors[25].stats, [1, 1, 1, 6, 15, 16]);
        assert_eq!(actors[25].base_stats, actors[25].stats);
        assert_eq!(actors[26].stats, [1, 1, 1, 24, 1, 80]);
        assert_eq!(actors[26].base_stats, actors[26].stats);
        assert_eq!(actors[26].spawn, origin.translate(56, 72, 0).unwrap());
    }
}
