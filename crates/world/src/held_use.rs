//! HeldUInteractions verification and directional handler precedence. A default
//! in the selected direction wins over an exact pair in the reverse direction.
use crate::inventory::Inventory;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Type {
    pub id: i32,
    pub content: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub ty: Type,
    pub slot: usize,
}
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub selected_slot: i32,
    pub selected_id: i32,
    pub target_slot: i32,
    pub target_id: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preparation {
    pub abort_dialog: bool,
}
#[derive(Clone, Debug)]
pub enum Handler {
    CountedAssembly(crate::counted_assembly::Recipe),
    AssemblyBatch(std::sync::Arc<crate::assembly::BatchRecipe>),
    Assembly(crate::assembly::Recipe),
    AssemblyImmediate(crate::assembly::Recipe),
    AssemblyPrecise(crate::assembly::PreciseRecipe),
    AssemblyProduction { recipe:crate::assembly::PreciseRecipe, cycle_ticks:u8 },
    AssemblyTimed {
        recipe: crate::assembly::Recipe,
        ticks: u8,
    },
    Grinding(crate::grinding::Recipe),
    ToolCrafting(crate::tool_crafting::Recipe),
    ToolCraftingImmediate(crate::tool_crafting::Recipe),
    ToolCraftingChoices{recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8},
    ToolCraftingCountedChoices{recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,input_count:u16},
    ToolCraftingMaterialChoices{recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,input_counts:std::sync::Arc<Vec<u16>>},
    ToolCraftingEquivalentMixedChoices{recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,materials:crate::tool_crafting_materials::Specifications,tools:std::sync::Arc<Vec<i32>>,worn_tools:std::sync::Arc<Vec<(i32,usize)>>},
    ToolCraftingMixedChoices{recipes:std::sync::Arc<Vec<crate::tool_crafting::Recipe>>,cycle_ticks:u8,materials:crate::tool_crafting_materials::Specifications},
    TutorialDough(crate::tutorial_dough::Config),
    TutorialFire(crate::tutorial_fire::Config),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Scope {
    Types,
    ContentType,
    Contents,
    DefaultType,
    DefaultContent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Duplicate,
    SameSlot,
    InvalidSlot,
    StaleItem,
    UnknownType,
}
pub struct Selected<'a, T> {
    pub handler: &'a T,
    pub scope: Scope,
    pub first: Item,
    pub second: Item,
}
pub struct Registry<T> {
    bindings: [BTreeMap<i64, T>; 5],
}
impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            bindings: std::array::from_fn(|_| BTreeMap::new()),
        }
    }
}
fn pair(first: i32, second: i32) -> i64 {
    ((first as i64) << 32) | second as i64
}
impl<T> Registry<T> {
    pub fn register(
        &mut self,
        scope: Scope,
        first: i32,
        second: i32,
        handler: T,
    ) -> Result<(), Error> {
        let key = match scope {
            Scope::DefaultType | Scope::DefaultContent => first as i64,
            _ => pair(first, second),
        };
        let entries = &mut self.bindings[scope as usize];
        if entries.contains_key(&key)
            || (matches!(scope, Scope::Types | Scope::Contents)
                && entries.contains_key(&pair(second, first)))
        {
            return Err(Error::Duplicate);
        }
        entries.insert(key, handler);
        Ok(())
    }
    fn direction(&self, first: Item, second: Item) -> Option<Selected<'_, T>> {
        for (scope, key) in [
            (Scope::Types, pair(first.ty.id, second.ty.id)),
            (Scope::ContentType, pair(first.ty.content, second.ty.id)),
            (Scope::Contents, pair(first.ty.content, second.ty.content)),
            (Scope::DefaultType, first.ty.id as i64),
            (Scope::DefaultContent, first.ty.content as i64),
        ] {
            if let Some(handler) = self.bindings[scope as usize].get(&key) {
                return Some(Selected {
                    handler,
                    scope,
                    first,
                    second,
                });
            }
        }
        None
    }
    pub fn resolve(
        &self,
        inventory: &Inventory,
        types: &BTreeMap<i32, Type>,
        selected_slot: i32,
        selected_id: i32,
        target_slot: i32,
        target_id: i32,
    ) -> Result<Option<Selected<'_, T>>, Error> {
        if selected_slot == target_slot {
            return Err(Error::SameSlot);
        }
        let verify = |slot: i32, id: i32| -> Result<Item, Error> {
            let slot = usize::try_from(slot).map_err(|_| Error::InvalidSlot)?;
            let current = inventory.get(slot).ok_or(Error::StaleItem)?;
            if current.id != id {
                return Err(Error::StaleItem);
            }
            let ty = *types.get(&id).ok_or(Error::UnknownType)?;
            Ok(Item { ty, slot })
        };
        let first = verify(selected_slot, selected_id)?;
        let second = verify(target_slot, target_id)?;
        Ok(self
            .direction(first, second)
            .or_else(|| self.direction(second, first)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Inventory, BTreeMap<i32, Type>) {
        let mut inv = Inventory::new(4, crate::inventory::StackType::Normal, false);
        for (slot, id) in [(0, 10), (1, 20)] {
            inv.restore_slot(
                slot,
                Some(crate::inventory::Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        (
            inv,
            [
                (
                    10,
                    Type {
                        id: 10,
                        content: 100,
                    },
                ),
                (
                    20,
                    Type {
                        id: 20,
                        content: 200,
                    },
                ),
            ]
            .into(),
        )
    }
    #[test]
    fn reverse_pair_preserves_registered_item_and_slot_order_but_forward_default_wins() {
        let (inv, types) = fixture();
        let mut registry = Registry::default();
        registry.register(Scope::Types, 20, 10, "reverse").unwrap();
        let found = registry
            .resolve(&inv, &types, 0, 10, 1, 20)
            .unwrap()
            .unwrap();
        assert_eq!(
            (*found.handler, found.first.slot, found.second.slot),
            ("reverse", 1, 0)
        );
        registry
            .register(Scope::DefaultContent, 100, 0, "forward default")
            .unwrap();
        let found = registry
            .resolve(&inv, &types, 0, 10, 1, 20)
            .unwrap()
            .unwrap();
        assert_eq!((*found.handler, found.first.slot), ("forward default", 0));
        assert_eq!(
            registry.register(Scope::Types, 10, 20, "duplicate"),
            Err(Error::Duplicate)
        );
    }
    #[test]
    fn same_direction_precedence_and_stale_slots() {
        let (mut inv, types) = fixture();
        let mut registry = Registry::default();
        registry.register(Scope::DefaultType, 10, 0, 4).unwrap();
        registry.register(Scope::Contents, 100, 200, 3).unwrap();
        registry.register(Scope::ContentType, 100, 20, 2).unwrap();
        assert_eq!(
            *registry
                .resolve(&inv, &types, 0, 10, 1, 20)
                .unwrap()
                .unwrap()
                .handler,
            2
        );
        registry.register(Scope::Types, 10, 20, 1).unwrap();
        assert_eq!(
            *registry
                .resolve(&inv, &types, 0, 10, 1, 20)
                .unwrap()
                .unwrap()
                .handler,
            1
        );
        assert!(matches!(
            registry.resolve(&inv, &types, 0, 10, 0, 10),
            Err(Error::SameSlot)
        ));
        assert!(matches!(
            registry.resolve(&inv, &types, -1, 10, 1, 20),
            Err(Error::InvalidSlot)
        ));
        inv.restore_slot(1, None).unwrap();
        assert!(matches!(
            registry.resolve(&inv, &types, 0, 10, 1, 20),
            Err(Error::StaleItem)
        ));
    }
    #[test]
    fn signed_low_content_key_matches_event_bus_aliasing() {
        let (inv, mut types) = fixture();
        types.get_mut(&20).unwrap().content = -1;
        let mut registry = Registry::default();
        registry.register(Scope::Contents, 999, -1, 1).unwrap();
        assert_eq!(
            *registry
                .resolve(&inv, &types, 0, 10, 1, 20)
                .unwrap()
                .unwrap()
                .handler,
            1
        );
    }
}
