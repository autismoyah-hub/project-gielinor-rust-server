//! Deposit-only adapters reuse the shared bank transfer implementation.
use crate::{
    bank_actions::{self, Config as BankConfig, Event},
    inventory::Catalog,
    simulation::PlayerData,
};
#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub animation: u16,
    pub quantities: Option<crate::deposit_box_items::Quantities>,
}
pub(crate) fn apply_in_main(
    bank: &BankConfig,
    data: &mut PlayerData,
    items: &Catalog,
    request: bank_actions::Request,
) -> Result<Vec<Event>, bank_actions::Error> {
    let previous = bank.selected.get(&data.variables)?;
    if !(0..=9).contains(&previous) {
        return Err(bank_actions::Error::InvalidSetting);
    }
    let mut staged = data.clone();
    let mut events = Vec::new();
    if previous != 0 {
        events.extend(
            bank.selected
                .set(&mut staged.variables, 0)?
                .map(Event::Variable),
        );
    }
    events.extend(bank.apply(&mut staged, items, request)?);
    if previous != 0 {
        events.extend(
            bank.selected
                .set(&mut staged.variables, previous)?
                .map(Event::Variable),
        );
    }
    *data = staged;
    Ok(events)
}
pub fn deposit_all(
    bank: &BankConfig,
    data: &mut PlayerData,
    items: &Catalog,
    worn: bool,
) -> Result<(Vec<Event>, bool), bank_actions::Error> {
    let before = if worn {
        data.equipment.free_space()
    } else {
        data.inventory.free_space()
    };
    let events = apply_in_main(
        bank,
        data,
        items,
        bank_actions::Request::DepositAll { worn },
    )?;
    let after = if worn {
        data.equipment.free_space()
    } else {
        data.inventory.free_space()
    };
    Ok((events, before != after))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{inventory::Item, stats::Stats, variables::Variables, CoordGrid};
    #[test]
    fn deposit_box_bulk_uses_main_tab_and_restores_selected_without_withdraw() {
        for worn in [false, true] {
            let bank = bank_actions::tests::config();
            let mut data = PlayerData::new(
                1,
                "Box".into(),
                CoordGrid::new(3200, 3200, 0).unwrap(),
                Stats::default(),
            );
            data.variables = Variables::restore([(0, 1), (10, 1), (11, 80)].into());
            data.bank
                .restore_slot(
                    0,
                    Some(Item {
                        id: 1,
                        count: 2,
                        vars: 0,
                    }),
                )
                .unwrap();
            let source = if worn {
                &mut data.equipment
            } else {
                &mut data.inventory
            };
            source
                .restore_slot(
                    if worn { 3 } else { 0 },
                    Some(Item {
                        id: 2,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
            let stats = data.stats.clone();
            let (events, changed) =
                deposit_all(&bank, &mut data, &Catalog::default(), worn).unwrap();
            assert!(changed);
            assert_eq!(bank.selected.get(&data.variables).unwrap(), 1);
            assert_eq!(bank.tabs[0].get(&data.variables).unwrap(), 1);
            assert!(data.bank.slots().iter().flatten().any(|i| i.id == 2));
            assert_eq!(data.stats, stats);
            let selection: Vec<_> = events
                .iter()
                .filter_map(|e| {
                    if let Event::Variable(v) = e {
                        (v.id == 10).then_some(v.value)
                    } else {
                        None
                    }
                })
                .collect();
            assert_eq!(selection, vec![0, 1]);
        }
    }
    #[test]
    fn deposit_box_bulk_invalid_setting_preserves_both_containers_and_flags() {
        let bank = bank_actions::tests::config();
        let mut data = PlayerData::new(
            1,
            "Box".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        data.variables = Variables::restore([(10, 99), (11, 80)].into());
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 2,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let before = data.clone();
        assert!(deposit_all(&bank, &mut data, &Catalog::default(), false).is_err());
        assert_eq!(data.inventory, before.inventory);
        assert_eq!(data.bank, before.bank);
        assert_eq!(data.variables.get(10), 99);
    }
}
