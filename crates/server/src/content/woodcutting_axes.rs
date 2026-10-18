//! Seven ordinary metal axe bindings; special axes and new tree species are separate.
use super::{Definitions, Symbols};
use gielinor_world::woodcutting::Axe;
pub struct WoodcuttingTool {
    pub id: i32,
    pub axe: Axe,
    pub regular_rates: (i32, i32),
    pub oak_rates: (i32, i32),
}
const ROWS: [(&str, i32, u16, i32, (i32, i32), (i32, i32)); 7] = [
    ("bronze", 1351, 879, 1, (64, 200), (32, 100)),
    ("iron", 1349, 877, 1, (96, 300), (48, 150)),
    ("steel", 1353, 875, 6, (128, 400), (64, 200)),
    ("black", 1361, 873, 11, (144, 450), (72, 225)),
    ("mithril", 1355, 871, 21, (160, 500), (80, 250)),
    ("adamant", 1357, 869, 31, (192, 600), (96, 300)),
    ("rune", 1359, 867, 41, (224, 700), (112, 350)),
];
pub fn woodcutting_axes(
    objects: &Symbols,
    sequences: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Vec<WoodcuttingTool>, String> {
    if stats.get("woodcutting") != Some(&8) {
        return Err("Missing/aliased Woodcutting stat".into());
    }
    let stat = defs.stats.get(&8).ok_or("Missing Woodcutting stat")?;
    if stat.name != "Woodcutting" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99
    {
        return Err("Invalid Woodcutting stat".into());
    }
    let mut result = Vec::new();
    for (metal, id, animation, level, regular_rates, oak_rates) in ROWS {
        let symbol = format!("{metal}_axe");
        let name = format!("{}{} axe", metal[..1].to_uppercase(), &metal[1..]);
        if objects.get(&symbol) != Some(&id)
            || sequences.get(&format!("human_woodcutting_{metal}_axe"))
                != Some(&i32::from(animation))
        {
            return Err(format!("Missing/aliased woodcutting tool {metal}"));
        }
        let d = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing woodcutting axe")?;
        if d.name != name
            || d.members
            || d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err(format!("Invalid ordinary woodcutting axe {id}"));
        }
        result.push(WoodcuttingTool {
            id,
            axe: Axe { members: false, level, animation },
            regular_rates,
            oak_rates,
        });
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn woodcutting_axes_require_exact_ordinary_tools_and_animations() {
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        defs.stats.insert(
            8,
            gielinor_cache::definitions::StatType::decode(b"\x01Woodcutting\0\0").unwrap(),
        );
        let mut obj = Symbols::new();
        let mut seq = Symbols::new();
        for (metal, id, animation, _, _, _) in ROWS {
            obj.insert(format!("{metal}_axe"), id);
            seq.insert(
                format!("human_woodcutting_{metal}_axe"),
                i32::from(animation),
            );
            let name = format!("{}{} axe", metal[..1].to_uppercase(), &metal[1..]);
            let mut b = vec![2];
            b.extend_from_slice(name.as_bytes());
            b.extend_from_slice(&[0, 0]);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&b).unwrap(),
            );
        }
        let names = [("woodcutting".into(), 8)].into();
        let rows = woodcutting_axes(&obj, &seq, &names, &defs).unwrap();
        assert_eq!(rows.len(), 7);
        assert_eq!(
            (rows[1].axe.level, rows[1].regular_rates, rows[1].oak_rates),
            (1, (96, 300), (48, 150))
        );
        defs.items.get_mut(&1359).unwrap().placeholder.template = 799;
        assert!(woodcutting_axes(&obj, &seq, &names, &defs).is_err());
    }
}

#[cfg(test)]
mod dispatch_tests {
    use gielinor_world::{
        inventory::Item,
        loc_collision::LocCollision,
        locations::Location,
        simulation::{Action, PlayerData, World, WorldEvent},
        stats::Stats,
        woodcutting::{Axe, Config, Tree},
        CoordGrid,
    };
    struct Success;
    impl gielinor_world::npc::RandomSource for Success {
        fn below(&mut self, _: u32) -> u32 {
            0
        }
    }
    #[test]
    fn woodcutting_axes_actual_reached_oak_selects_iron_over_bronze_without_attack_gate() {
        let mut w = World::default();
        for x in (3192..3224).step_by(8) {
            for z in (3192..3224).step_by(8) {
                w.collision.allocate(x, z, 0);
            }
        }
        let coords = CoordGrid::new(3201, 3200, 0).unwrap();
        let loc = Location::new(coords, 1281, 10, 0).unwrap();
        let ty = LocCollision {
            width: 2,
            length: 2,
            block_walk: 2,
            block_range: true,
            break_route_finding: false,
        };
        w.location_types.insert(1281, ty);
        w.locations.load(loc);
        gielinor_world::loc_collision::toggle(&mut w.collision, coords, 10, 0, ty, true).unwrap();
        w.woodcutting = Some(Config {
            skill: 8,
            axes: [
                (
                    1351,
                    Axe {
                        members: false,
                        level: 1,
                        animation: 879,
                    },
                ),
                (
                    1349,
                    Axe {
                        members: false,
                        level: 1,
                        animation: 877,
                    },
                ),
                (
                    1359,
                    Axe {
                        members: false,
                        level: 41,
                        animation: 867,
                    },
                ),
            ]
            .into(),
            trees: [(
                1281,
                Tree {
                    members: false,
                    level: 15,
                    product: 1521,
                    product_name: "Oak logs".into(),
                    xp: 37.5,
                    stump: 1356,
                    deplete_chance: 0,
                    chopping_ticks: Some(45),
                    respawn: [14, 14],
                    rates: [(1351, (32, 100)), (1349, (48, 150)), (1359, (112, 350))].into(),
                },
            )]
            .into(),
            full_sound: 0,
            fall_sound: 0,
        });
        let mut stats = Stats::default();
        stats.set_base(8, 15);
        stats.set_current(8, 15);
        stats.set_current(0, 0);
        let mut data = PlayerData::new(
            1,
            "Axe chooser".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            stats,
        );
        data.xp_rate_hundreds = 125;
        for (slot, id) in [(0, 1351), (1, 1349), (2, 1359)] {
            data.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        data.equipment
            .restore_slot(
                3,
                Some(Item {
                    id: 1351,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        w.queue_login(1, data).unwrap();
        w.tick().unwrap();
        let a = w.actors().next().unwrap();
        w.drain_events();
        w.submit(
            a,
            Action::OperateLocation {
                coords,
                id: 1281,
                operation: 1,
                control: false,
            },
        )
        .unwrap();
        let mut success = false;
        let mut iron_animation = false;
        for _ in 0..15 {
            w.tick_with_random(&mut Success).unwrap();
            for e in w.drain_events() {
                match e {
                    WorldEvent::Woodcutting { trace, .. } => {
                        assert_eq!(trace.tool, Some(1349));
                        if trace.result == "success" {
                            success = true;
                            assert_eq!(trace.fine_xp_delta, 468);
                        }
                    }
                    WorldEvent::PlayerCombatAnimation { sequence: 877, .. } => {
                        iron_animation = true
                    }
                    _ => {}
                }
            }
            if success {
                break;
            }
        }
        assert!(success && iron_animation);
        let d = w.data(a).unwrap();
        assert_eq!(
            d.inventory
                .slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 1521)
                .count(),
            1
        );
        for (slot, id) in [(0, 1351), (1, 1349), (2, 1359)] {
            assert_eq!(d.inventory.get(slot).unwrap().id, id);
        }
        assert_eq!(d.equipment.get(3).unwrap().id, 1351);
    }
}
