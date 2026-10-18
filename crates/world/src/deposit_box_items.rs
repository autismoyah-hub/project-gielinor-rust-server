//! Quantity and selected-item adapter; bank transfer semantics remain shared.
use crate::{
    bank_actions::{self, Binding, Event},
    deposit_box::apply_in_main,
    inventory::Catalog,
    simulation::PlayerData,
    variables::Varp,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountPurpose {
    Mode,
    Item { slot: usize, expected: i32 },
}
#[derive(Clone, Copy, Debug)]
pub struct Quantities {
    pub mode: Binding,
    pub requested: Varp,
}
impl Quantities {
    pub fn amount(self, data: &PlayerData, operation: u8) -> Result<i32, bank_actions::Error> {
        Ok(match operation {
            1 => match self.mode.get(&data.variables)? {
                1 => 5,
                2 => i32::MAX,
                3 => data.variables.get(self.requested.id).max(1),
                4 => 10,
                _ => 1,
            },
            2 => 1,
            3 => 5,
            4 => 10,
            6 => i32::MAX,
            _ => return Err(bank_actions::Error::InvalidSetting),
        })
    }
    /// Source presets use 2=All and 4=Ten, unlike the full-bank mode.
    pub fn select(
        self,
        data: &mut PlayerData,
        value: i32,
    ) -> Result<Vec<Event>, bank_actions::Error> {
        if ![0, 1, 2, 4].contains(&value) {
            return Err(bank_actions::Error::InvalidSetting);
        }
        Ok(self
            .mode
            .set(&mut data.variables, value)?
            .map(Event::Variable)
            .into_iter()
            .collect())
    }
    pub fn input(
        self,
        data: &mut PlayerData,
        amount: i32,
    ) -> Result<Vec<Event>, bank_actions::Error> {
        if amount <= 0 {
            return Ok(vec![]);
        }
        let mut staged = data.clone();
        let mut events = Vec::new();
        let mode = match amount {
            1 => 0,
            5 => 1,
            10 => 4,
            _ => {
                events.extend(
                    staged
                        .variables
                        .set(self.requested, amount, true)
                        .map(Event::Variable),
                );
                3
            }
        };
        events.extend(
            self.mode
                .set(&mut staged.variables, mode)?
                .map(Event::Variable),
        );
        *data = staged;
        Ok(events)
    }
    pub fn deposit(
        self,
        bank: &bank_actions::Config,
        data: &mut PlayerData,
        items: &Catalog,
        clicked: usize,
        expected: i32,
        operation: u8,
    ) -> Result<(Vec<Event>, bool), bank_actions::Error> {
        if !data
            .inventory
            .get(clicked)
            .is_some_and(|i| i.id == expected && i.count > 0)
        {
            return Ok((vec![], false));
        }
        let count = self.amount(data, operation)?;
        self.deposit_amount(bank, data, items, clicked, expected, count)
    }
    pub fn deposit_amount(
        self,
        bank: &bank_actions::Config,
        data: &mut PlayerData,
        items: &Catalog,
        clicked: usize,
        expected: i32,
        count: i32,
    ) -> Result<(Vec<Event>, bool), bank_actions::Error> {
        if count <= 0
            || !data
                .inventory
                .get(clicked)
                .is_some_and(|i| i.id == expected && i.count > 0)
        {
            return Ok((vec![], false));
        }
        // Source DEPOSIT_TOP_TO_BOTTOM=true selects first matching id, not the clicked duplicate.
        let slot = data
            .inventory
            .slots()
            .iter()
            .position(|i| i.is_some_and(|i| i.id == expected))
            .unwrap();
        let before = data.inventory.clone();
        let events = apply_in_main(
            bank,
            data,
            items,
            bank_actions::Request::Deposit { slot, count },
        )?;
        Ok((events, before.slots() != data.inventory.slots()))
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        inventory::Item,
        stats::Stats,
        variables::{Transmit, Varbit},
        CoordGrid,
    };
    pub(crate) fn quantities() -> Quantities {
        let base = Varp {
            id: 867,
            permanent: true,
            transmit: Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        Quantities {
            mode: Binding {
                base,
                bit: Varbit {
                    base: 867,
                    lsb: 28,
                    msb: 31,
                },
            },
            requested: Varp { id: 1794, ..base },
        }
    }
    fn data() -> PlayerData {
        let mut d = PlayerData::new(1, "Item deposit".into(), CoordGrid::ZERO, Stats::default());
        bank_actions::tests::config()
            .capacity
            .set(&mut d.variables, 80)
            .unwrap();
        d
    }
    #[test]
    fn deposit_box_selected_modes_presets_and_fixed_amounts_preserve_main_and_other_bits() {
        let q = quantities();
        let bank = bank_actions::tests::config();
        let mut items = Catalog::default();
        items.stackable.insert(995);
        for (mode, op, want) in [
            (0, 1, 1),
            (1, 1, 5),
            (2, 1, 100),
            (4, 1, 10),
            (3, 1, 7),
            (15, 1, 1),
            (0, 2, 1),
            (0, 3, 5),
            (0, 4, 10),
            (0, 6, 100),
        ] {
            let mut d = data();
            d.variables.set(q.mode.base, 123, true);
            q.mode.set(&mut d.variables, mode).unwrap();
            d.variables.set(q.requested, 7, true);
            bank.selected.set(&mut d.variables, 0).unwrap();
            d.inventory
                .restore_slot(
                    5,
                    Some(Item {
                        id: 995,
                        count: 100,
                        vars: 0,
                    }),
                )
                .unwrap();
            let (_, changed) = q.deposit(&bank, &mut d, &items, 5, 995, op).unwrap();
            assert!(changed);
            assert_eq!(d.bank.get(0).unwrap().count, want);
            assert_eq!(d.inventory.get(5).map_or(0, |i| i.count), 100 - want);
            assert_eq!(d.variables.get(867) as u32 & 0x0fffffff, 123);
            assert_eq!(q.mode.get(&d.variables).unwrap(), mode);
        }
        for value in [0, 1, 4, 2] {
            let mut d = data();
            assert!(q.select(&mut d, value).is_ok());
            assert_eq!(q.mode.get(&d.variables).unwrap(), value);
        }
        for value in [-1, 3, 5, 15] {
            let mut d = data();
            let before = d.variables.get(867);
            assert!(q.select(&mut d, value).is_err());
            assert_eq!(d.variables.get(867), before);
        }
    }
    #[test]
    fn deposit_box_top_to_bottom_stale_click_restriction_and_failed_setting() {
        let q = quantities();
        let mut bank = bank_actions::tests::config();
        let mut d = data();
        let items = Catalog::default();
        for slot in [0, 5] {
            d.inventory
                .restore_slot(
                    slot,
                    Some(Item {
                        id: 2,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let before = d.inventory.clone();
        assert!(!q.deposit(&bank, &mut d, &items, 5, 99, 2).unwrap().1);
        assert_eq!(d.inventory, before);
        bank.restricted.insert(2);
        assert!(!q.deposit(&bank, &mut d, &items, 5, 2, 2).unwrap().1);
        assert_eq!(d.inventory, before);
        bank.restricted.clear();
        bank.selected.set(&mut d.variables, 10).unwrap();
        assert!(q.deposit(&bank, &mut d, &items, 5, 2, 2).is_err());
        assert_eq!(d.inventory, before);
        bank.selected.set(&mut d.variables, 0).unwrap();
        assert!(q.deposit(&bank, &mut d, &items, 5, 2, 2).unwrap().1);
        assert!(d.inventory.get(0).is_none());
        assert!(d.inventory.get(5).is_some());
        assert_eq!(d.bank.get(0).unwrap().id, 2);
    }
}
