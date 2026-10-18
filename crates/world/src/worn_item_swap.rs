//! Staged worn-item mode swaps using the existing equipment and held exchange.
use crate::{
    equipment::{self, Event, WearCatalog},
    held_item_conversion,
    inventory::{Catalog, Inventory, StackType},
    stats::Stats,
};
#[derive(Clone, Copy, Debug)]
pub struct Rule {
    pub output: i32,
    pub from_slot: usize,
    pub to_slot: usize,
    pub button_operation: u8,
}
pub struct Plan {
    pub removed_inventory: Inventory,
    pub removed_equipment: Inventory,
    pub final_inventory: Inventory,
    pub final_equipment: Inventory,
    pub unequip_events: Vec<Event>,
    pub equip_events: Option<Vec<Event>>,
}
impl Rule {
    pub fn prepare(
        self,
        input: i32,
        inventory: &Inventory,
        equipment: &Inventory,
        catalog: &Catalog,
        wear: &WearCatalog,
        stats: &Stats,
        member: bool,
    ) -> Result<Plan, equipment::Error> {
        if self.from_slot == self.to_slot
            || self.from_slot >= 14
            || self.to_slot >= 14
            || inventory.stack != StackType::Normal
            || equipment.stack != StackType::Normal
            || inventory.placeholders
            || equipment.placeholders
            || equipment
                .get(self.from_slot)
                .is_none_or(|i| i.id != input || i.count != 1 || i.vars != 0)
        {
            return Err(equipment::Error::InvalidItem);
        }
        for (id, slot) in [(input, self.from_slot), (self.output, self.to_slot)] {
            if wear
                .types
                .get(&id)
                .is_none_or(|t| t.primary != Some(slot) || t.secondary.iter().any(Option::is_some))
            {
                return Err(equipment::Error::InvalidItem);
            }
        }
        let target_free = equipment.get(self.to_slot).is_none();
        let mut removed_inventory = inventory.clone();
        let mut removed_equipment = equipment.clone();
        let unequip_events = equipment::unequip(
            &mut removed_equipment,
            &mut removed_inventory,
            self.from_slot,
            catalog,
        )?;
        let added_slot = removed_inventory
            .slots()
            .iter()
            .enumerate()
            .find_map(|(slot, i)| {
                (inventory.get(slot).is_none()
                    && i.is_some_and(|i| i.id == input && i.count == 1 && i.vars == 0))
                .then_some(slot)
            })
            .ok_or(equipment::Error::InvalidItem)?;
        let mut final_inventory = held_item_conversion::Rule {
            output: self.output,
            members: false,
            button_operation: 4,
            message: None,
        }
        .prepare(input, added_slot, &removed_inventory, catalog, member)
        .map_err(|_| equipment::Error::InvalidItem)?;
        if final_inventory
            .get(added_slot)
            .is_none_or(|i| i.id != self.output || i.count != 1 || i.vars != 0)
        {
            return Err(equipment::Error::InvalidItem);
        }
        let mut final_equipment = removed_equipment.clone();
        // Wiki permits retaining the tool in F2P, but a new equip requires members.
        let equip_events = if target_free && member {
            Some(equipment::equip(
                &mut final_inventory,
                &mut final_equipment,
                added_slot,
                catalog,
                wear,
                stats,
            )?)
        } else {
            None
        };
        Ok(Plan {
            removed_inventory,
            removed_equipment,
            final_inventory,
            final_equipment,
            unequip_events,
            equip_events,
        })
    }
}
