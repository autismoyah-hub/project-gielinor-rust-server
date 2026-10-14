//! FloorOneScript.takeStaff inventory/persistence decisions, after Tower bounds.
use crate::{
    inventory::{Catalog, Insert},
    simulation::PlayerData,
};
use std::collections::BTreeSet;
#[derive(Clone, Debug)]
pub struct Config {
    pub staff: i32,
    pub talisman: i32,
    /// All types whose names contain "staff" with Kotlin ignoreCase semantics.
    pub staff_names: BTreeSet<i32>,
    /// Canonical air_staff or names equal to its packed name / "Staff of air".
    /// ownsType accepts certificate aliases; do not apply ownsNamed's exclusion.
    pub staff_owned: BTreeSet<i32>,
    pub talisman_owned: BTreeSet<i32>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct UnknownBonesOffered;
fn owns(data: &PlayerData, ids: &BTreeSet<i32>) -> bool {
    [&data.inventory, &data.equipment, &data.bank]
        .into_iter()
        .any(|inv| {
            inv.slots()
                .iter()
                .flatten()
                .any(|item| ids.contains(&item.id))
        })
}
impl Config {
    pub fn owns_staff(&self, data: &PlayerData) -> bool {
        data.equipment
            .get(3)
            .is_some_and(|i| self.staff_names.contains(&i.id))
            || data
                .inventory
                .slots()
                .iter()
                .flatten()
                .any(|i| self.staff_names.contains(&i.id))
            || owns(data, &self.staff_owned)
    }
    pub fn owns_talisman(&self, data: &PlayerData) -> bool {
        owns(data, &self.talisman_owned)
    }
    /// Synchronous inventory portion of restoreStaffInPack. The host must run
    /// its normal invEquip handler for the returned slot, then emit the wield
    /// message even if that handler rejects the item, as the source does.
    pub fn prepare_restore(&self, data: &mut PlayerData, catalog: &Catalog) -> Restore {
        let mut result = Restore::default();
        if !data.tower_arrival.staff_taken {
            return result;
        }
        if !self.owns_staff(data) {
            if data.inventory.free_space() == 0 {
                result
                    .messages
                    .push("Your staff of air is waiting. Drop something, then search the crate.");
                return result;
            }
            let _ = data.inventory.insert(catalog, Insert::new(self.staff, 1));
            result.messages.push("You still have the staff of air.");
        }
        if let Some(message) = self.restore_talisman(data, catalog) {
            result.messages.push(message);
        }
        if !data
            .equipment
            .get(3)
            .is_some_and(|i| self.staff_names.contains(&i.id))
        {
            result.equip_slot = data
                .inventory
                .slots()
                .iter()
                .position(|i| i.is_some_and(|i| self.staff_names.contains(&i.id)));
        }
        result
    }
    pub fn restore_talisman(
        &self,
        data: &mut PlayerData,
        catalog: &Catalog,
    ) -> Option<&'static str> {
        if !data.tower_arrival.staff_taken || self.owns_talisman(data) {
            return None;
        }
        if data.inventory.free_space() == 0 {
            return Some("Your mind talisman is waiting. Drop something, then search the crate.");
        }
        let _ = data
            .inventory
            .insert(catalog, Insert::new(self.talisman, 1));
        Some("You still have the mind talisman.")
    }
    pub fn needs_search(&self, data: &PlayerData) -> bool {
        !data.tower_arrival.staff_taken && !(self.owns_staff(data) && self.owns_talisman(data))
    }
    pub fn search(
        &self,
        data: &mut PlayerData,
        catalog: &Catalog,
    ) -> Result<&'static str, UnknownBonesOffered> {
        let staff = self.owns_staff(data);
        let talisman = self.owns_talisman(data);
        if staff && talisman {
            data.tower_arrival.staff_taken = true;
            return Ok("The crate is empty.");
        }
        if !data
            .tower_arrival
            .bones_offered
            .ok_or(UnknownBonesOffered)?
        {
            return Ok("Offer bones at the altar first. The crate is sealed.");
        }
        let slots = usize::from(!staff) + usize::from(!talisman);
        if data.inventory.free_space() < slots {
            return Ok("You don't have enough inventory space.");
        }
        // Original flag is persisted before separately committed invAdd calls.
        // The handler does not branch on either transaction result.
        data.tower_arrival.staff_taken = true;
        if !staff {
            let _ = data.inventory.insert(catalog, Insert::new(self.staff, 1));
        }
        if !talisman {
            let _ = data
                .inventory
                .insert(catalog, Insert::new(self.talisman, 1));
        }
        Ok("You take a staff of air and a mind talisman. Craft mind runes at the hub altar.")
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        inventory::{Inventory, Item, StackType},
        stats::Stats,
        CoordGrid,
    };
    fn setup() -> (Config, PlayerData, Catalog) {
        (
            Config {
                staff: 1381,
                talisman: 1448,
                staff_names: [1381, 1391, 900].into(),
                staff_owned: [1381, 1382].into(),
                talisman_owned: [1448, 1449].into(),
            },
            PlayerData::new(
                1,
                "Staff crate".into(),
                CoordGrid::new(10425, 107, 0).unwrap(),
                Stats::default(),
            ),
            Catalog::default(),
        )
    }
    fn put(inv: &mut Inventory, slot: usize, id: i32) {
        inv.restore_slot(
            slot,
            Some(Item {
                id,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
    }
    #[test]
    fn restore_requires_saved_flag_and_never_grants_mind_runes() {
        let (c, mut p, cat) = setup();
        assert_eq!(c.prepare_restore(&mut p, &cat), Restore::default());
        p.tower_arrival.staff_taken = true;
        let result = c.prepare_restore(&mut p, &cat);
        assert_eq!(
            result.messages,
            [
                "You still have the staff of air.",
                "You still have the mind talisman."
            ]
        );
        assert_eq!(result.equip_slot, Some(0));
        assert_eq!(
            p.inventory
                .slots()
                .iter()
                .flatten()
                .map(|i| i.id)
                .collect::<Vec<_>>(),
            [1381, 1448]
        );
        assert!(c.prepare_restore(&mut p, &cat).messages.is_empty());
    }
    #[test]
    fn restore_one_free_slot_inserts_staff_then_requests_equip_without_talisman() {
        let (c, mut p, cat) = setup();
        p.tower_arrival.staff_taken = true;
        p.inventory = Inventory::new(1, StackType::Normal, false);
        let result = c.prepare_restore(&mut p, &cat);
        assert_eq!(
            result.messages,
            [
                "You still have the staff of air.",
                "Your mind talisman is waiting. Drop something, then search the crate."
            ]
        );
        assert_eq!(result.equip_slot, Some(0));
        assert_eq!(p.inventory.get(0).unwrap().id, 1381);
    }
    #[test]
    fn banked_staff_is_not_withdrawn_and_full_pack_returns_before_talisman() {
        let (c, mut p, cat) = setup();
        p.tower_arrival.staff_taken = true;
        put(&mut p.bank, 0, 1382);
        let result = c.prepare_restore(&mut p, &cat);
        assert_eq!(result.messages, ["You still have the mind talisman."]);
        assert_eq!(result.equip_slot, None);
        p.bank.restore_slot(0, None).unwrap();
        p.inventory = Inventory::new(0, StackType::Normal, false);
        let result = c.prepare_restore(&mut p, &cat);
        assert_eq!(
            result.messages,
            ["Your staff of air is waiting. Drop something, then search the crate."]
        );
        assert_eq!(result.equip_slot, None);
    }
    #[test]
    fn unknown_or_unearned_offering_cannot_mint_a_kit() {
        let (c, mut p, cat) = setup();
        assert_eq!(c.search(&mut p, &cat), Err(UnknownBonesOffered));
        assert_eq!(p.inventory.free_space(), 28);
        p.tower_arrival.bones_offered = Some(false);
        assert_eq!(
            c.search(&mut p, &cat).unwrap(),
            "Offer bones at the altar first. The crate is sealed."
        );
        assert!(!p.tower_arrival.staff_taken);
    }
    #[test]
    fn preflight_needs_only_missing_items_and_owned_kit_wins_before_gate() {
        let (c, mut p, cat) = setup();
        put(&mut p.bank, 0, 1382); // Certificate name alias accepted by ownsType.
        put(&mut p.equipment, 0, 1449);
        assert_eq!(c.search(&mut p, &cat).unwrap(), "The crate is empty.");
        assert!(p.tower_arrival.staff_taken);
        assert!(!c.needs_search(&p));
        assert_eq!(p.tower_arrival.bones_offered, None);
        p.equipment.restore_slot(0, None).unwrap();
        p.tower_arrival.bones_offered = Some(true);
        p.inventory = Inventory::new(1, StackType::Normal, false);
        c.search(&mut p, &cat).unwrap();
        assert_eq!(p.inventory.get(0).unwrap().id, 1448);
    }
    #[test]
    fn arbitrary_staff_in_bank_does_not_count_but_inventory_or_weapon_does() {
        let (c, mut p, _) = setup();
        put(&mut p.bank, 0, 1391);
        assert!(!c.owns_staff(&p));
        put(&mut p.equipment, 0, 1391);
        assert!(!c.owns_staff(&p));
        put(&mut p.equipment, 3, 1391);
        assert!(c.owns_staff(&p));
        p.equipment.restore_slot(3, None).unwrap();
        put(&mut p.inventory, 0, 900);
        assert!(c.owns_staff(&p));
    }
    #[test]
    fn full_inventory_preserves_flag_and_successful_repeat_does_not_duplicate() {
        let (c, mut p, cat) = setup();
        p.tower_arrival.bones_offered = Some(true);
        p.inventory = Inventory::new(1, StackType::Normal, false);
        assert_eq!(
            c.search(&mut p, &cat).unwrap(),
            "You don't have enough inventory space."
        );
        assert!(!p.tower_arrival.staff_taken);
        p.inventory = Inventory::new(2, StackType::Normal, false);
        c.search(&mut p, &cat).unwrap();
        assert_eq!(p.inventory.get(0).unwrap().id, 1381);
        assert_eq!(p.inventory.get(1).unwrap().id, 1448);
        assert!(p.tower_arrival.staff_taken);
        assert_eq!(c.search(&mut p, &cat).unwrap(), "The crate is empty.");
    }
}

#[derive(Clone, Debug)]
pub struct HandlerConfig {
    pub location: u32,
    pub origin: crate::CoordGrid,
    pub inventory: Config,
}
/// ASCII substring target with Kotlin's simple case comparison, including long S.
pub fn staff_name(name: &str) -> bool {
    let chars = name.chars().collect::<Vec<_>>();
    chars
        .windows(5)
        .any(|part| crate::tower_chest::name_matches(&part.iter().collect::<String>(), "staff"))
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Restore {
    pub messages: Vec<&'static str>,
    pub equip_slot: Option<usize>,
}
