//! Immutable mixed-material plans use the existing retained-tool production.
use crate::{
    held_use,
    inventory::{Catalog, Inventory},
    tool_crafting::{Recipe, Request},
    CoordGrid,
};
use std::{collections::BTreeSet, sync::Arc};
pub type Specifications = Arc<Vec<Vec<(i32, u16)>>>;
pub struct Capture {
    pub slots: Vec<usize>,
    pub ids: Arc<Vec<i32>>,
    pub counts: Arc<Vec<u16>>,
    pub stride: usize,
}
pub fn bind(
    recipes: Arc<Vec<Recipe>>,
    cycle: u8,
    held: held_use::Request,
    inv: &Inventory,
    items: &Catalog,
    origin: CoordGrid,
    movement: i32,
    specs: Specifications,
) -> Option<Request> {
    bind_inner(
        recipes, cycle, held, inv, items, origin, movement, specs, None,
    )
}
pub(crate) fn bind_worn(
    recipes: Arc<Vec<Recipe>>,
    cycle: u8,
    held: held_use::Request,
    inv: &Inventory,
    items: &Catalog,
    worn: &Inventory,
    worn_slot: usize,
    origin: CoordGrid,
    movement: i32,
    specs: Specifications,
) -> Option<Request> {
    bind_inner(
        recipes,
        cycle,
        held,
        inv,
        items,
        origin,
        movement,
        specs,
        Some((worn, worn_slot)),
    )
}
fn bind_inner(
    recipes: Arc<Vec<Recipe>>,
    cycle: u8,
    held: held_use::Request,
    inv: &Inventory,
    items: &Catalog,
    origin: CoordGrid,
    movement: i32,
    specs: Specifications,
    worn: Option<(&Inventory, usize)>,
) -> Option<Request> {
    let first = *recipes.first()?;
    let stacks = specs
        .iter()
        .flatten()
        .filter_map(|&(id, _)| items.stackable.contains(&id).then_some(id))
        .collect::<BTreeSet<_>>();
    if specs.len() != recipes.len()
        || recipes.iter().any(|r| r.failure.is_some())
        || specs.iter().any(|s| {
            s.len() < 2
                || s.first().map(|v| v.0) != Some(first.exchange.input)
                || s.iter().any(|&(id, n)| {
                    id < 0
                        || n == 0
                        || id == first.exchange.tool
                        || (!stacks.contains(&id) && !items.ordinary_nonstackable(id))
                        || items.dummy.contains(&id)
                        || items.notes.get(&id).is_some_and(|t| t.template != 0)
                        || items.placeholders.get(&id).is_some_and(|t| t.template != 0)
                        || items.transforms.get(&id).is_some_and(|t| t.template != 0)
                })
                || s.iter().map(|v| usize::from(v.1)).sum::<usize>() > 27
                || s.iter().map(|v| v.0).collect::<BTreeSet<_>>().len() != s.len()
        })
    {
        return None;
    }
    let mut selected = Vec::new();
    let mut chosen_tool = None;
    for (slot, id) in [
        (held.selected_slot, held.selected_id),
        (held.target_slot, held.target_id),
    ] {
        let slot = usize::try_from(slot).ok()?;
        let i = inv.get(slot)?;
        if i.id != id
            || (if stacks.contains(&id) {
                i.count <= 0
            } else {
                i.count != 1
            })
            || i.vars != 0
        {
            return None;
        }
        if id == first.exchange.tool {
            if worn.is_some() {
                return None;
            }
            if chosen_tool.replace(slot).is_some() {
                return None;
            }
        } else {
            if !specs.iter().all(|s| s.iter().any(|v| v.0 == id)) {
                return None;
            }
            selected.push((id, slot));
        }
    }
    if held.selected_slot == held.target_slot
        || held.selected_id == held.target_id
        || selected.is_empty()
    {
        return None;
    }
    let primary = selected
        .iter()
        .find(|v| v.0 == first.exchange.input)
        .map(|v| v.1)
        .or_else(|| {
            inv.slots().iter().position(|i| {
                i.is_some_and(|i| i.id == first.exchange.input && i.count == 1 && i.vars == 0)
            })
        })?;
    let mut request = if let Some((gear, slot)) = worn {
        if !items.ordinary_nonstackable(first.exchange.tool) {
            return None;
        }
        Request::bind_worn(recipes, cycle, inv, gear, slot, primary, origin, movement)?
    } else {
        let tool = chosen_tool.or_else(|| {
            inv.slots().iter().position(|i| {
                i.is_some_and(|i| i.id == first.exchange.tool && i.count == 1 && i.vars == 0)
            })
        })?;
        let canonical = held_use::Request {
            selected_slot: i32::try_from(tool).ok()?,
            selected_id: first.exchange.tool,
            target_slot: i32::try_from(primary).ok()?,
            target_id: first.exchange.input,
        };
        Request::bind(recipes, cycle, canonical, inv, origin, movement)?
    };
    request.material_specs = Some(specs);
    request.material_stacks = Arc::new(stacks);
    request.selected_materials = selected;
    Some(request)
}
pub fn available(request: &Request, index: usize, inv: &Inventory) -> usize {
    let Some(spec) = request.material_specs.as_ref().and_then(|s| s.get(index)) else {
        return 0;
    };
    if spec.is_empty()
        || request.selected_materials.iter().any(|&(id, slot)| {
            !inv.get(slot).is_some_and(|i| {
                i.id == id
                    && (if request.material_stacks.contains(&id) {
                        i.count > 0
                    } else {
                        i.count == 1
                    })
                    && i.vars == 0
            })
        })
    {
        return 0;
    }
    spec.iter()
        .map(|&(id, n)| {
            if n == 0 {
                0
            } else {
                let quantity = if request.material_stacks.contains(&id) {
                    stack_slot(request, id, inv)
                        .and_then(|slot| inv.get(slot))
                        .map_or(0, |i| i.count as usize)
                } else {
                    inv.slots()
                        .iter()
                        .flatten()
                        .filter(|i| i.id == id && i.count == 1 && i.vars == 0)
                        .count()
                };
                quantity / usize::from(n)
            }
        })
        .min()
        .unwrap_or(0)
        .min(28)
}
fn stack_slot(request: &Request, id: i32, inv: &Inventory) -> Option<usize> {
    request
        .selected_materials
        .iter()
        .find(|v| v.0 == id)
        .map(|v| v.1)
        .or_else(|| {
            inv.slots()
                .iter()
                .position(|i| i.is_some_and(|i| i.id == id && i.count > 0 && i.vars == 0))
        })
}
pub fn capture(request: &Request, index: usize, inv: &Inventory, count: usize) -> Option<Capture> {
    let spec = request.material_specs.as_ref()?.get(index)?;
    let jobs = available(request, index, inv).min(count);
    if jobs == 0 {
        return None;
    }
    let stride = spec
        .iter()
        .map(|&(id, n)| {
            if request.material_stacks.contains(&id) {
                1
            } else {
                usize::from(n)
            }
        })
        .sum();
    let mut used = if request.worn_tool_slot.is_some() {
        BTreeSet::new()
    } else {
        BTreeSet::from([request.tool_slot])
    };
    let mut slots = Vec::new();
    let mut ids = Vec::new();
    let mut counts = Vec::new();
    for _ in 0..jobs {
        for &(id, n) in spec {
            if request.material_stacks.contains(&id) {
                let slot = stack_slot(request, id, inv)?;
                if slot == request.tool_slot {
                    return None;
                }
                slots.push(slot);
                ids.push(id);
                counts.push(n);
            } else {
                for _ in 0..n {
                    let slot = request
                        .selected_materials
                        .iter()
                        .find(|&&(wanted, slot)| wanted == id && !used.contains(&slot))
                        .map(|v| v.1)
                        .or_else(|| {
                            inv.slots().iter().enumerate().find_map(|(slot, i)| {
                                (!used.contains(&slot)
                                    && i.is_some_and(|i| i.id == id && i.count == 1 && i.vars == 0))
                                .then_some(slot)
                            })
                        })?;
                    used.insert(slot);
                    slots.push(slot);
                    ids.push(id);
                    counts.push(1);
                }
            }
        }
    }
    Some(Capture {
        slots,
        ids: Arc::new(ids),
        counts: Arc::new(counts),
        stride,
    })
}
