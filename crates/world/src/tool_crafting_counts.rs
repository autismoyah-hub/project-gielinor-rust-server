//! Counted stack materials reuse ordinary tool choices, weak scheduling and XP.
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
    let r = job.recipe;
    let e = r.exchange;
    let stack = items.stackable.contains(&e.input);
    if (r.output_count > 1 && !items.stackable.contains(&e.output))
        || input_count == 0
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
        || (!stack && !items.ordinary_nonstackable(e.input))
    {
        return Err(Error::Policy);
    }
    if e.members && !member {
        return Err(Error::Members);
    }
    if (stats.current(r.skill) as u8) < r.level {
        return Err(Error::Level);
    }
    if !inv
        .get(job.tool_slot)
        .is_some_and(|i| i.id == e.tool && i.count == 1 && i.vars == 0)
        || slots.first() != Some(&job.input_slot)
        || slots.len() != if stack { 1 } else { usize::from(input_count) }
        || slots
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != slots.len()
        || slots.iter().any(|&slot| {
            slot == job.tool_slot
                || !inv.get(slot).is_some_and(|i| {
                    i.id == e.input
                        && i.vars == 0
                        && if stack {
                            i.count >= i32::from(input_count)
                        } else {
                            i.count == 1
                        }
                })
        })
    {
        return Err(Error::Exchange(crate::grinding::Outcome::Stale));
    }
    let mut inventory = inv.clone();
    for &slot in slots {
        let mut debit = Delete::new(e.input, if stack { i32::from(input_count) } else { 1 });
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
