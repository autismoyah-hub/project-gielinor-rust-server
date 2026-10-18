//! Captured material plans reuse ordinary retained-tool production and XP.
use crate::{
    assembly,
    inventory::{Catalog, Delete, Insert, Inventory, StackType},
    stats::Stats,
    tool_crafting::{Error, Job, Prepared},
};
pub fn prepare(
    job: Job,
    input_count: u16,
    inv: &Inventory,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<Prepared, Error> {
    prepare_with_slots(
        job,
        input_count,
        &[job.input_slot],
        inv,
        items,
        stats,
        rate,
        member,
    )
}
pub fn prepare_with_slots(
    job: Job,
    input_count: u16,
    slots: &[usize],
    inv: &Inventory,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<Prepared, Error> {
    if input_count == 1 {
        return job.prepare(inv, items, stats, rate, member);
    }
    let stack = items.stackable.contains(&job.recipe.exchange.input);
    if input_count == 0 || slots.len() != if stack { 1 } else { usize::from(input_count) } {
        return Err(Error::Policy);
    }
    let parts = slots
        .iter()
        .map(|&slot| {
            (
                job.recipe.exchange.input,
                slot,
                if stack { i32::from(input_count) } else { 1 },
            )
        })
        .collect::<Vec<_>>();
    prepare_plan(job, &parts, inv, items, stats, rate, member, None)
}
pub fn prepare_materials(
    job: Job,
    materials: &[(i32, usize)],
    inv: &Inventory,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<Prepared, Error> {
    if materials.len() < 2
        || materials
            .iter()
            .any(|&(id, _)| !items.ordinary_nonstackable(id))
    {
        return Err(Error::Policy);
    }
    let parts = materials
        .iter()
        .map(|&(id, slot)| (id, slot, 1))
        .collect::<Vec<_>>();
    prepare_plan(job, &parts, inv, items, stats, rate, member, None)
}
/// Stacked material counts and ordinary singles share the same staged transaction.
pub fn prepare_material_counts(
    job: Job,
    materials: &[(i32, usize, u16)],
    inv: &Inventory,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<Prepared, Error> {
    prepare_material_counts_inner(job, materials, inv, items, stats, rate, member, None)
}
pub fn prepare_material_counts_worn(
    job: Job,
    materials: &[(i32, usize, u16)],
    inv: &Inventory,
    worn: &Inventory,
    slot: usize,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
) -> Result<Prepared, Error> {
    prepare_material_counts_inner(
        job,
        materials,
        inv,
        items,
        stats,
        rate,
        member,
        Some((worn, slot)),
    )
}
fn prepare_material_counts_inner(
    job: Job,
    materials: &[(i32, usize, u16)],
    inv: &Inventory,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
    worn: Option<(&Inventory, usize)>,
) -> Result<Prepared, Error> {
    if materials.len() < 2
        || materials.iter().any(|&(id, _, n)| {
            n == 0
                || id < 0
                || items.dummy.contains(&id)
                || items.notes.get(&id).is_some_and(|t| t.template != 0)
                || items.placeholders.get(&id).is_some_and(|t| t.template != 0)
                || items.transforms.get(&id).is_some_and(|t| t.template != 0)
                || (!items.stackable.contains(&id) && (!items.ordinary_nonstackable(id) || n != 1))
        })
    {
        return Err(Error::Policy);
    }
    let parts = materials
        .iter()
        .map(|&(id, slot, n)| (id, slot, i32::from(n)))
        .collect::<Vec<_>>();
    prepare_plan(job, &parts, inv, items, stats, rate, member, worn)
}
fn prepare_plan(
    job: Job,
    parts: &[(i32, usize, i32)],
    inv: &Inventory,
    items: &Catalog,
    stats: &Stats,
    rate: i32,
    member: bool,
    worn: Option<(&Inventory, usize)>,
) -> Result<Prepared, Error> {
    let r = job.recipe;
    let e = r.exchange;
    if (r.output_count > 1 && !items.stackable.contains(&e.output))
        || parts.is_empty()
        || r.failure.is_some()
        || r.skill >= 23
        || r.level == 0
        || r.level > 99
        || r.output_count == 0
        || inv.stack != StackType::Normal
        || inv.placeholders
        || e.tool == e.input
        || e.output == e.input
        || e.output == e.tool
        || parts.iter().any(|&(id, _, count)| {
            count <= 0
                || id == e.tool
                || id == e.output
                || (!items.stackable.contains(&id)
                    && (!items.ordinary_nonstackable(id) || count != 1))
        })
    {
        return Err(Error::Policy);
    }
    if e.members && !member {
        return Err(Error::Members);
    }
    if (stats.current(r.skill) as u8) < r.level {
        return Err(Error::Level);
    }
    let retained_tool = match worn {
        Some((gear, slot)) => {
            if gear.stack != StackType::Normal || gear.placeholders {
                return Err(Error::Policy);
            }
            gear.get(slot)
        }
        None => inv.get(job.tool_slot),
    };
    if !retained_tool.is_some_and(|i| i.id == e.tool && i.count == 1 && i.vars == 0)
        || parts.first().map(|&(id, slot, _)| (id, slot)) != Some((e.input, job.input_slot))
        || parts
            .iter()
            .map(|v| v.1)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != parts.len()
        || parts.iter().any(|&(id, slot, count)| {
            (worn.is_none() && slot == job.tool_slot)
                || !inv.get(slot).is_some_and(|i| {
                    i.id == id
                        && i.vars == 0
                        && if items.stackable.contains(&id) {
                            i.count >= count
                        } else {
                            i.count == 1
                        }
                })
        })
    {
        return Err(Error::Exchange(crate::grinding::Outcome::Stale));
    }
    let mut inventory = inv.clone();
    for &(id, slot, count) in parts {
        let mut debit = Delete::new(id, count);
        debit.strict_slot = Some(slot);
        inventory
            .delete(items, debit)
            .map_err(|_| Error::Exchange(crate::grinding::Outcome::Capacity))?;
    }
    inventory
        .insert(items, Insert::new(e.output, i32::from(r.output_count)))
        .map_err(|_| Error::Exchange(crate::grinding::Outcome::Capacity))?;
    let (stats, advancement) =
        assembly::prepare_reward_value(stats, r.skill, r.xp_tenths as f64 / 10.0, rate)
            .map_err(|_| Error::Policy)?;
    Ok(Prepared {
        inventory,
        stats,
        advancement,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{grinding, inventory::Item, tool_crafting::Recipe};
    #[test]
    fn counted_tool_material_transaction_capacity_last_stack_and_live_refusals() {
        let recipe = Recipe {
            skill: 12,
            exchange: grinding::Recipe {
                tool: 29920,
                input: 28624,
                output: 28618,
                members: true,
                animation: None,
            },
            level: 2,
            xp_tenths: 150,
            output_count: 1,
            ticks: 2,
            name: "Pheasant boots",
            action: "make",
            failure: None,
        };
        for case in 0..8 {
            let mut inv = Inventory::new(3, StackType::Normal, false);
            let mut items = Catalog::default();
            items.stackable.insert(28624);
            let mut stats = Stats::default();
            stats.set_current(12, 2);
            let mut job = Job {
                recipe,
                tool_slot: 0,
                input_slot: 1,
            };
            inv.restore_slot(
                0,
                Some(Item {
                    id: 29920,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            inv.restore_slot(
                1,
                Some(Item {
                    id: 28624,
                    count: if case == 1 { 15 } else { 30 },
                    vars: 0,
                }),
            )
            .unwrap();
            inv.restore_slot(
                2,
                Some(Item {
                    id: 99,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            match case {
                2 => {
                    inv.restore_slot(2, None).unwrap();
                }
                3 => {
                    inv.restore_slot(2, None).unwrap();
                    job.tool_slot = 2;
                }
                4 => {
                    inv.restore_slot(2, None).unwrap();
                    stats.set_current(12, 1);
                }
                5 => {
                    inv.restore_slot(2, None).unwrap();
                    items.dummy.insert(28618);
                }
                6 | 7 => {
                    inv.restore_slot(2, None).unwrap();
                }
                _ => {}
            }
            let old = (inv.clone(), stats.clone());
            let result = prepare(
                job,
                15,
                &inv,
                &items,
                &stats,
                if case == 7 { 0 } else { 100 },
                case != 6,
            );
            if case == 1 || case == 2 {
                let p = result.unwrap();
                assert_eq!(p.stats.fine_xp(12), 150);
                assert_eq!(p.inventory.get(0), inv.get(0));
                assert_eq!(
                    p.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 28618)
                        .count(),
                    1
                );
                assert_eq!(
                    p.inventory
                        .get(1)
                        .map_or(0, |i| if i.id == 28624 { i.count } else { 0 }),
                    if case == 1 { 0 } else { 15 }
                );
            } else {
                assert!(result.is_err());
            }
            assert_eq!((inv, stats), old);
        }
    }
}

#[cfg(test)]
mod discrete_tests {
    use super::*;
    use crate::{grinding, inventory::Item, tool_crafting::Recipe};
    #[test]
    fn discrete_materials_stage_all_slots_and_reject_duplicates_stale_credit_and_reward() {
        let recipe = Recipe {
            skill: 12,
            exchange: grinding::Recipe {
                tool: 29920,
                input: 1745,
                output: 1135,
                members: true,
                animation: None,
            },
            level: 63,
            xp_tenths: 1860,
            output_count: 1,
            ticks: 2,
            name: "Green d'hide body",
            action: "make",
            failure: None,
        };
        let job = Job {
            recipe,
            tool_slot: 0,
            input_slot: 1,
        };
        for case in 0..7 {
            let mut inventory = Inventory::new(5, StackType::Normal, false);
            let mut items = Catalog::default();
            let mut stats = Stats::default();
            stats.set_current(12, 63);
            inventory
                .restore_slot(
                    0,
                    Some(Item {
                        id: 29920,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            for slot in 1..5 {
                inventory
                    .restore_slot(
                        slot,
                        Some(Item {
                            id: 1745,
                            count: 1,
                            vars: 0,
                        }),
                    )
                    .unwrap();
            }
            let mut slots = vec![1, 2, 3];
            match case {
                1 => slots[1] = 1,
                2 => {
                    inventory
                        .restore_slot(
                            2,
                            Some(Item {
                                id: 1745,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap();
                }
                3 => {
                    items.dummy.insert(1135);
                }
                4 => slots[1] = 0,
                _ => {}
            }
            let before = (inventory.clone(), stats.clone());
            let result = prepare_with_slots(
                job,
                3,
                &slots,
                &inventory,
                &items,
                &stats,
                if case == 5 { 0 } else { 100 },
                case != 6,
            );
            if case == 0 {
                let p = result.unwrap();
                assert_eq!(p.stats.fine_xp(12), 1860);
                assert_eq!(p.inventory.get(0), inventory.get(0));
                assert_eq!(p.inventory.get(4), inventory.get(4));
                assert_eq!(
                    p.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 1135)
                        .count(),
                    1
                );
                assert_eq!(
                    p.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 1745)
                        .count(),
                    1
                );
            } else {
                assert!(result.is_err());
            }
            assert_eq!((inventory, stats), before);
        }
    }
}

#[cfg(test)]
mod mixed_tests {
    use super::*;
    use crate::{grinding, inventory::Item, tool_crafting::Recipe};
    #[test]
    fn mixed_parts_stage_together_and_reject_secondary_duplicate_credit_rate_member() {
        let job = Job {
            recipe: Recipe {
                skill: 12,
                exchange: grinding::Recipe {
                    tool: 29920,
                    input: 3239,
                    output: 3385,
                    members: true,
                    animation: None,
                },
                level: 61,
                xp_tenths: 1240,
                output_count: 1,
                ticks: 2,
                name: "Splitbark helm",
                action: "make",
                failure: None,
            },
            tool_slot: 0,
            input_slot: 1,
        };
        for case in 0..8 {
            let mut inv = Inventory::new(6, StackType::Normal, false);
            let mut items = Catalog::default();
            let mut stats = Stats::default();
            stats.set_current(12, 61);
            for (slot, id) in [
                (0, 29920),
                (1, 3239),
                (2, 3239),
                (3, 3470),
                (4, 3470),
                (5, 3470),
            ] {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count: 1,
                        vars: if case == 2 && slot == 4 { 1 } else { 0 },
                    }),
                )
                .unwrap();
            }
            let mut parts = vec![(3239, 1), (3239, 2), (3470, 3), (3470, 4)];
            match case {
                1 => parts[3].1 = 3,
                3 => {
                    items.dummy.insert(3385);
                }
                4 => parts[3] = (29920, 0),
                7 => parts[3] = (3239, 4),
                _ => {}
            }
            let before = (inv.clone(), stats.clone());
            let result = prepare_materials(
                job,
                &parts,
                &inv,
                &items,
                &stats,
                if case == 5 { 0 } else { 125 },
                case != 6,
            );
            if case == 0 {
                let p = result.unwrap();
                assert_eq!(p.stats.fine_xp(12), 1550);
                assert_eq!(p.inventory.get(0), inv.get(0));
                assert_eq!(p.inventory.get(5), inv.get(5));
                assert_eq!(
                    p.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 3385)
                        .count(),
                    1
                );
                assert_eq!(p.inventory.free_space(), 3);
            } else {
                assert!(result.is_err(), "case{case}");
            }
            assert_eq!((inv, stats), before);
        }
    }
}

#[cfg(test)]
mod stack_mixed_tests {
    use super::*;
    use crate::{
        grinding,
        inventory::{Item, Template},
        tool_crafting::Recipe,
    };
    #[test]
    fn stack_mixed_nails_and_all_singles_are_staged_before_credit_or_reward() {
        let mut job = Job {
            recipe: Recipe {
                skill: 12,
                exchange: grinding::Recipe {
                    tool: 2347,
                    input: 1743,
                    output: 22269,
                    members: true,
                    animation: None,
                },
                level: 41,
                xp_tenths: 700,
                output_count: 1,
                ticks: 5,
                name: "Hard leather shield",
                action: "make",
                failure: None,
            },
            tool_slot: 0,
            input_slot: 1,
        };
        for case in 0..10 {
            let mut inv = Inventory::new(6, StackType::Normal, false);
            let mut items = Catalog::default();
            items.stackable.insert(4819);
            let mut stats = Stats::default();
            stats.set_current(12, 41);
            for (slot, id, count) in [
                (0, 2347, 1),
                (1, 1743, 1),
                (2, 1743, 1),
                (3, 22251, 1),
                (4, 4819, if case == 3 { 14 } else { 30 }),
                (5, 1511, 1),
            ] {
                inv.restore_slot(
                    slot,
                    Some(Item {
                        id,
                        count,
                        vars: if case == 2 && slot == 4 { 9 } else { 0 },
                    }),
                )
                .unwrap();
            }
            let mut parts = vec![(1743, 1, 1), (1743, 2, 1), (22251, 3, 1), (4819, 4, 15)];
            match case {
                1 => parts[2].1 = 2,
                4 => {
                    items.dummy.insert(22269);
                }
                7 => parts[3].2 = 0,
                8 => {
                    items.notes.insert(
                        4819,
                        Template {
                            link: 1,
                            template: 1,
                        },
                    );
                }
                9 => job.recipe.output_count = 2,
                _ => {}
            }
            let before = (inv.clone(), stats.clone());
            let result = prepare_material_counts(
                job,
                &parts,
                &inv,
                &items,
                &stats,
                if case == 5 { 0 } else { 125 },
                case != 6,
            );
            if case == 0 {
                let p = result.unwrap();
                assert_eq!(p.stats.fine_xp(12), 875);
                assert_eq!(p.inventory.get(4).unwrap().count, 15);
                assert_eq!(p.inventory.free_space(), 2);
                assert_eq!(p.inventory.get(0), inv.get(0));
                assert_eq!(p.inventory.get(5), inv.get(5));
                assert_eq!(
                    p.inventory
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == 22269)
                        .count(),
                    1
                );
            } else {
                assert!(result.is_err(), "case{case}");
            }
            assert_eq!((inv, stats), before);
        }
    }
}
