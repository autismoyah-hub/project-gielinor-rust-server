//! Ordinary static facility variants, independently pinned to target definitions.
//! Repair/build transforms and area access are separate from these registrations.
use super::{Definitions, Symbols};
use std::collections::BTreeMap;
pub struct FacilityLocations {
    pub spinning: BTreeMap<u32, u8>,
    pub weaving: BTreeMap<u32, u8>,
    pub shaping: BTreeMap<u32, u8>,
    pub firing: BTreeMap<u32, u8>,
}
// Group, exact target symbol/ID/name, footprint length, operation.
const VARIANTS: [(usize, &str, u32, &str, u8, u8); 15] = [
    (0, "viking_spinningwheel", 4309, "Spinning wheel", 1, 2),
    (
        0,
        "elf_village_spinning_wheel",
        8748,
        "Spinning wheel",
        1,
        2,
    ),
    (0, "contact_spinning_wheel", 20365, "Spinning wheel", 1, 2),
    (0, "iznot_spinning_wheel", 21304, "Spinning wheel", 1, 2),
    (0, "kr_spinningwheel", 25824, "Spinning wheel", 1, 2),
    (0, "spinningwheel_quetzacali", 55330, "Spinning wheel", 1, 2),
    (0, "spinningwheel_2", 56964, "Spinning wheel", 1, 2),
    (1, "regicide_loom", 787, "Loom", 1, 2),
    (2, "viking_potterywheel", 4310, "Potter's Wheel", 1, 1),
    (2, "contact_potterywheel", 20375, "Potter's wheel", 1, 1),
    (2, "darkm_poor_potterywheel", 39395, "Potter's Wheel", 1, 1),
    (2, "potterywheel_2", 50732, "Potter's Wheel", 1, 1),
    (3, "fai_barbarian_pottery_oven", 11601, "Pottery Oven", 2, 1),
    (3, "viking_potteryoven", 4308, "Pottery Oven", 2, 1),
    (3, "darkm_poor_pottery_oven", 39396, "Pottery Oven", 2, 1),
];
pub fn facility_locations(
    locations: &Symbols,
    defs: &Definitions,
) -> Result<FacilityLocations, String> {
    let mut groups: [BTreeMap<u32, u8>; 4] = Default::default();
    for (group, symbol, id, name, length, operation) in VARIANTS {
        if locations.get(symbol) != Some(&(id as i32)) {
            return Err(format!("Missing/aliased facility {symbol}"));
        }
        let loc = defs
            .locations
            .get(&id)
            .ok_or("Missing facility definition")?;
        let verb = match group {
            0 => "Spin",
            1 => "Weave",
            2 => "Use",
            _ => "Fire",
        };
        if loc.name != name
            || loc.ops[usize::from(operation) - 1].as_deref() != Some(verb)
            || loc.collision.width != 1
            || loc.collision.length != length
            || !loc.multi.is_empty()
            || loc.multi_default != -1
            || loc.multi_varbit != -1
            || loc.multi_varp != -1
        {
            return Err(format!("Invalid ordinary facility {id}"));
        }
        groups[group].insert(id, operation);
    }
    let [spinning, weaving, shaping, firing] = groups;
    Ok(FacilityLocations {
        spinning,
        weaving,
        shaping,
        firing,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{facility_crafting::Config, simulation::World};
    #[test]
    fn facility_locations_attest_fifteen_types_and_dispatch_only_their_real_operations() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut symbols = Symbols::new();
        for (group, symbol, id, name, length, operation) in VARIANTS {
            symbols.insert(symbol.into(), id as i32);
            let mut bytes = vec![2];
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&[15, length, 29 + operation]);
            bytes.extend_from_slice(match group {
                0 => b"Spin" as &[u8],
                1 => b"Weave",
                2 => b"Use",
                _ => b"Fire",
            });
            bytes.extend_from_slice(&[0, 0]);
            defs.locations.insert(
                id,
                gielinor_cache::locations::LocationType::decode(&bytes).unwrap(),
            );
        }
        let v = facility_locations(&symbols, &defs).unwrap();
        assert_eq!(
            (
                v.spinning.len(),
                v.weaving.len(),
                v.shaping.len(),
                v.firing.len()
            ),
            (7, 1, 4, 3)
        );
        let mut world = World::default();
        for locations in [v.spinning, v.weaving, v.shaping, v.firing] {
            let config = Config {
                locations,
                recipes: std::sync::Arc::new(vec![]),
                action: "test",
                empty_message: "test",
                animation: 894,
                location_animation: None,
            };
            for (&id, &operation) in &config.locations {
                world.facility_crafting.insert(id, config.clone());
                assert!(world.has_location_handler(id, operation));
                for wrong in [0, 3 - operation, 3, 4, 5] {
                    assert!(!world.has_location_handler(id, wrong));
                }
            }
        }
        for unrelated in [40733, 30934, 34959, 14889] {
            assert!(!world.has_location_handler(unrelated, 2));
        }
        symbols.insert("regicide_loom".into(), 8717);
        assert!(facility_locations(&symbols, &defs).is_err());
        symbols.insert("regicide_loom".into(), 787);
        defs.locations.get_mut(&4308).unwrap().collision.length = 1;
        assert!(facility_locations(&symbols, &defs).is_err());
        defs.locations.get_mut(&4308).unwrap().collision.length = 2;
        defs.locations.get_mut(&20375).unwrap().multi_varp = 1;
        assert!(facility_locations(&symbols, &defs).is_err());
        defs.locations.get_mut(&20375).unwrap().multi_varp = -1;
        defs.locations.get_mut(&787).unwrap().ops[1] = Some("Search".into());
        assert!(facility_locations(&symbols, &defs).is_err());
    }
}
