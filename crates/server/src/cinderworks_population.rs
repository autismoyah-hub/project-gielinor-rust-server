//! Authored Floor One population adapted from audited f9180b23.
use gielinor_cache::{game::Definitions,symbols::Symbols};
use gielinor_world::CoordGrid;
pub fn resolve(
    origin: CoordGrid,
    spatial: &gielinor_world::cinderworks_floor::Spatial,
    definitions: &Definitions,
    symbols: &Symbols,
) -> Result<Vec<gielinor_world::npc::Npc>, String> {
    use gielinor_world::npc::Mode;
    let mut result = Vec::new();
    for (_x, _z, level, name, passive) in [
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
        const POS:[(i32,i32);20]=[(48,50),(57,48),(33,33),(36,32),(39,34),(39,37),(37,39),(33,36),(31,35),(36,36),(65,34),(72,34),(66,41),(71,42),(69,58),(70,60),(70,62),(69,65),(68,71),(81,56)];
        let (x,z)=spatial.supply_npc_position(result.len(),POS[result.len()]);
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
        let spatial=gielinor_world::cinderworks_floor::Spatial::parse("CINDERWORKS_SPATIAL 1\nN 2 31 72\n").unwrap();
        let actors = resolve(origin, &spatial, &definitions, &symbols).unwrap();
        assert_eq!(actors[2].spawn,origin.translate(31,72,0).unwrap());
        assert_eq!(actors[14].spawn,origin.translate(69,58,0).unwrap());
        assert_eq!(actors[18].spawn,origin.translate(68,71,0).unwrap());
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
        assert_eq!(actors[19].spawn, origin.translate(81, 56, 0).unwrap());
    }
}
