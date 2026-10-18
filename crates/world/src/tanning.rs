//! Ordinary paid tanning. Tower session conversion remains a separate policy.
use crate::inventory::{Catalog, Delete, Inventory, StackType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    pub input: i32,
    pub output: i32,
    pub cost: i32,
    pub members: bool,
}
#[derive(Clone, Debug)]
pub struct Request {
    pub(crate) target: crate::simulation::NpcId,
    pub(crate) npc_type: u16,
    pub(crate) origin: crate::CoordGrid,
    pub(crate) last_movement: i32,
    pub(crate) choices: Vec<(Recipe, Vec<usize>)>,
}
impl Request {
    pub fn recipes(&self) -> impl Iterator<Item = Recipe> + '_ {
        self.choices.iter().map(|(recipe, _)| *recipe)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Invalid,
    Members,
    Hides,
    Coins,
    Credit,
}
fn ordinary_coin(catalog: &Catalog) -> bool {
    catalog.stackable.contains(&995)
        && !catalog.dummy.contains(&995)
        && !catalog.notes.get(&995).is_some_and(|t| t.template != 0)
        && !catalog
            .placeholders
            .get(&995)
            .is_some_and(|t| t.template != 0)
        && !catalog
            .transforms
            .get(&995)
            .is_some_and(|t| t.template != 0)
}
pub(crate) fn hide_slots(inv: &Inventory, id: i32) -> Vec<usize> {
    inv.slots()
        .iter()
        .enumerate()
        .filter_map(|(slot, item)| {
            item.filter(|i| i.id == id && i.count == 1 && i.vars == 0)
                .map(|_| slot)
        })
        .collect()
}
/// Stage the fee and every hide/output together. Strict selected slots and full
/// output credit are required; neither failed credit nor malformed items cost GP.
pub(crate) fn exchange(
    inv: &mut Inventory,
    catalog: &Catalog,
    recipe: Recipe,
    slots: &[usize],
    requested: i32,
    member: bool,
) -> Result<i32, Error> {
    if requested <= 0
        || recipe.cost < 0
        || recipe.input == recipe.output
        || !matches!(inv.stack, StackType::Normal)
        || inv.placeholders
        || !catalog.ordinary_nonstackable(recipe.input)
        || !catalog.ordinary_nonstackable(recipe.output)
        || !ordinary_coin(catalog)
    {
        return Err(Error::Invalid);
    }
    if recipe.members && !member {
        return Err(Error::Members);
    }
    // Capture one ordinary currency stack. Multiple or variable stacks indicate
    // malformed inventory and must not be silently merged or stripped of vars.
    let coins: Vec<_> = inv
        .slots()
        .iter()
        .enumerate()
        .filter_map(|(s, i)| i.filter(|i| i.id == 995).map(|i| (s, i)))
        .collect();
    if coins.len() > 1
        || coins
            .first()
            .is_some_and(|(_, i)| i.vars != 0 || i.count <= 0)
    {
        return Err(Error::Invalid);
    }
    let held = i32::try_from(slots.len()).map_err(|_| Error::Invalid)?;
    if held == 0 {
        return Err(Error::Hides);
    }
    let affordable = if recipe.cost == 0 {
        i32::MAX
    } else {
        coins.first().map_or(0, |(_, i)| i.count) / recipe.cost
    };
    let count = requested.min(held).min(affordable);
    if count <= 0 {
        return Err(Error::Coins);
    }
    let cost = recipe.cost.checked_mul(count).ok_or(Error::Invalid)?;
    let mut next = inv.clone();
    if cost > 0 {
        let mut debit = Delete::new(995, cost);
        debit.strict_slot = Some(coins[0].0);
        next.delete(catalog, debit).map_err(|_| Error::Coins)?;
    }
    let inputs: Vec<_> = slots
        .iter()
        .take(count as usize)
        .map(|&s| (recipe.input, s))
        .collect();
    let outputs = vec![recipe.output; count as usize];
    next = crate::assembly::exchange_slots(&next, catalog, &inputs, &outputs, true)
        .map_err(|_| Error::Credit)?;
    *inv = next;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Item;
    fn fixture() -> (Inventory, Catalog, Recipe) {
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for s in 0..3 {
            inv.restore_slot(
                s,
                Some(Item {
                    id: 1739,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        inv.restore_slot(
            4,
            Some(Item {
                id: 995,
                count: 8,
                vars: 0,
            }),
        )
        .unwrap();
        let mut cat = Catalog::default();
        cat.stackable.insert(995);
        (
            inv,
            cat,
            Recipe {
                input: 1739,
                output: 1743,
                cost: 3,
                members: false,
            },
        )
    }
    #[test]
    fn tanning_quantity_is_bounded_by_fee_and_selected_hides_without_xp() {
        let (mut inv, cat, r) = fixture();
        assert_eq!(
            exchange(&mut inv, &cat, r, &[0, 1, 2], i32::MAX, false),
            Ok(2)
        );
        assert_eq!(inv.get(4).unwrap().count, 2);
        assert_eq!(hide_slots(&inv, 1739), vec![2]);
        assert_eq!(
            inv.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 1743)
                .count(),
            2
        );
    }
    #[test]
    fn tanning_stale_credit_currency_and_membership_fail_atomically() {
        for case in 0..8 {
            let (mut inv, mut cat, mut r) = fixture();
            let mut slots = vec![0, 1, 2];
            match case {
                0 => {
                    cat.dummy.insert(1743);
                }
                1 => {
                    inv.restore_slot(
                        0,
                        Some(Item {
                            id: 1739,
                            count: 1,
                            vars: 1,
                        }),
                    )
                    .unwrap();
                }
                2 => slots[1] = 0,
                3 => {
                    r.members = true;
                }
                4 => {
                    inv.restore_slot(
                        4,
                        Some(Item {
                            id: 995,
                            count: 8,
                            vars: 1,
                        }),
                    )
                    .unwrap();
                }
                5 => {
                    inv.restore_slot(4, None).unwrap();
                }
                6 => {
                    cat.stackable.insert(1743);
                }
                _ => r.cost = -1,
            }
            let before = inv.clone();
            assert!(
                exchange(&mut inv, &cat, r, &slots, 3, false).is_err(),
                "case {case}"
            );
            assert_eq!(inv, before, "case {case}");
        }
    }
}
