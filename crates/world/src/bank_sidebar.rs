use crate::{
    inventory::Inventory,
    variables::{self, Varbit, Variables, Varp},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub struct Special {
    pub bit: i32,
    pub condition: Varbit,
    pub flip: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct ItemOptions {
    pub special: Option<Special>,
    pub custom_text: bool,
    pub wear: bool,
    pub consumable: bool,
}
#[derive(Clone, Debug)]
pub struct Sidebar {
    pub outputs: [Varp; 3],
    pub items: BTreeMap<i32, ItemOptions>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownItem(i32),
    Variable(variables::Error),
}
impl Sidebar {
    pub fn masks(&self, inventory: &Inventory, vars: &Variables) -> Result<[i32; 3], Error> {
        let mut masks = [0; 3];
        for (slot, item) in inventory.slots().iter().enumerate() {
            let Some(item) = item else {
                continue;
            };
            let options = self
                .items
                .get(&item.id)
                .ok_or(Error::UnknownItem(item.id))?;
            if let Some(special) = options.special {
                let value = vars.bit(special.condition).map_err(Error::Variable)?;
                if special.flip && value == 0 || !special.flip && value > 0 {
                    masks[0] |= 1i32.wrapping_shl(special.bit as u32);
                    continue;
                }
            }
            if options.custom_text {
                continue;
            }
            if options.wear {
                masks[1] |= 1i32.wrapping_shl(slot as u32);
            } else if options.consumable {
                masks[2] |= 1i32.wrapping_shl(slot as u32);
            }
        }
        Ok(masks)
    }
    pub fn refresh(
        &self,
        inventory: &Inventory,
        vars: &mut Variables,
    ) -> Result<Vec<variables::Update>, Error> {
        let masks = self.masks(inventory, vars)?;
        Ok(self
            .outputs
            .iter()
            .zip(masks)
            .filter_map(|(&ty, value)| vars.set(ty, value, true))
            .collect())
    }
}
