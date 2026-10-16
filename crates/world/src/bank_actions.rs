//! Bank operations share the actor action boundary; persistent settings remain varbits.
use crate::{
    bank::{BankError, Banking, Tabs, Withdrawal},
    inventory::{Catalog, Error as InvError},
    simulation::PlayerData,
    variables::{self, Varbit, Variables, Varp},
};
use std::collections::HashSet;
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub base: Varp,
    pub bit: Varbit,
}
impl Binding {
    pub fn get(self, vars: &Variables) -> Result<i32, Error> {
        vars.bit(self.bit).map_err(Error::Variable)
    }
    pub fn set(self, vars: &mut Variables, value: i32) -> Result<Option<variables::Update>, Error> {
        vars.set_bit(self.base, self.bit, value, true)
            .map_err(Error::Variable)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{inventory::Item, stats::Stats, variables::Transmit, CoordGrid};
    pub(crate) fn config() -> Config {
        let binding = |id| Binding {
            base: Varp {
                id,
                permanent: true,
                transmit: Transmit::Always,
                bit_protect: false,
                client_code: -1,
            },
            bit: Varbit {
                base: id,
                lsb: 0,
                msb: 15,
            },
        };
        Config {
            sidebar: None,
            tabs: std::array::from_fn(|i| binding(i as u16)),
            selected: binding(10),
            capacity: binding(11),
            notes: binding(12),
            placeholders: binding(13),
            quantity: binding(14),
            last_quantity: binding(15),
            restricted: HashSet::new(),
        }
    }
    #[test]
    fn queued_counts_use_current_stack_and_tab_sizes_remain_in_variables() {
        let config = config();
        let mut data = PlayerData::new(1, "bank_actor".into(), CoordGrid::ZERO, Stats::default());
        config.capacity.set(&mut data.variables, 6).unwrap();
        config.selected.set(&mut data.variables, 1).unwrap();
        let mut items = Catalog::default();
        items.stackable.insert(995);
        data.inventory
            .restore_slot(
                5,
                Some(Item {
                    id: 995,
                    count: 100,
                    vars: 0,
                }),
            )
            .unwrap();
        let events = config
            .apply(
                &mut data,
                &items,
                Request::Button {
                    deposit: true,
                    slot: 5,
                    op: 4,
                },
            )
            .unwrap();
        assert_eq!(data.inventory.get(5).unwrap().count, 95);
        assert_eq!(data.bank.get(0).unwrap().count, 5);
        assert_eq!(config.tabs[0].get(&data.variables), Ok(1));
        assert!(matches!(
            events.first(),
            Some(Event::Variable(variables::Update { id: 0, value: 1 }))
        ));
        config
            .apply(
                &mut data,
                &items,
                Request::Button {
                    deposit: false,
                    slot: 0,
                    op: 8,
                },
            )
            .unwrap();
        assert_eq!(data.bank.get(0).unwrap().count, 1);
        assert_eq!(data.inventory.get(5).unwrap().count, 99);
        config
            .apply(
                &mut data,
                &items,
                Request::Button {
                    deposit: false,
                    slot: 0,
                    op: 8,
                },
            )
            .unwrap();
        assert_eq!(data.bank.get(0).unwrap().count, 1);
        config
            .apply(
                &mut data,
                &items,
                Request::Button {
                    deposit: false,
                    slot: 0,
                    op: 7,
                },
            )
            .unwrap();
        assert!(data.bank.get(0).is_none());
        assert_eq!(config.tabs[0].get(&data.variables), Ok(0));
        assert_eq!(config.selected.get(&data.variables), Ok(0));
    }
    #[test]
    fn transfers_refresh_sidebar_from_updated_inventory_without_duplicate_varp_writes() {
        use crate::bank_sidebar::{ItemOptions, Sidebar};
        let mut config = config();
        let mut outputs = [config.notes.base; 3];
        for (i, output) in outputs.iter_mut().enumerate() {
            output.id = 50 + i as u16;
        }
        config.sidebar = Some(Sidebar {
            outputs,
            items: [(
                1,
                ItemOptions {
                    special: None,
                    custom_text: false,
                    wear: true,
                    consumable: true,
                },
            )]
            .into_iter()
            .collect(),
        });
        let mut data = PlayerData::new(1, "bank_masks".into(), CoordGrid::ZERO, Stats::default());
        config.capacity.set(&mut data.variables, 6).unwrap();
        data.inventory
            .restore_slot(
                4,
                Some(Item {
                    id: 1,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let updates = |events: Vec<Event>| {
            events
                .into_iter()
                .filter_map(|event| match event {
                    Event::Variable(update) if update.id >= 50 => Some(update.value),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let events = config
            .apply(
                &mut data,
                &Catalog::default(),
                Request::Button {
                    deposit: true,
                    slot: 4,
                    op: 3,
                },
            )
            .unwrap();
        assert_eq!(updates(events), [0, 0, 0]);
        let events = config
            .apply(
                &mut data,
                &Catalog::default(),
                Request::Button {
                    deposit: false,
                    slot: 0,
                    op: 2,
                },
            )
            .unwrap();
        assert_eq!(updates(events), [0, 1, 0]);
        assert_eq!(data.inventory.get(0).unwrap().id, 1);
    }
    #[test]
    fn count_transfer_signed_boundaries_respect_binding_width_before_inventory_mutation() {
        // Fixture widths deliberately differ: source arithmetic alone does not
        // prove the prepared cache's remembered-count binding accepts MAX_VALUE.
        for width in [15, 30] {
            for input in [0i32, -5, i32::MIN, i32::MAX] {
                let mut config = config();
                config.last_quantity.bit.msb = width;
                let mut data =
                    PlayerData::new(1, "bank_bounds".into(), CoordGrid::ZERO, Stats::default());
                config.capacity.set(&mut data.variables, 6).unwrap();
                config.last_quantity.set(&mut data.variables, 3).unwrap();
                data.inventory
                    .restore_slot(
                        0,
                        Some(Item {
                            id: 995,
                            count: 20,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                let mut items = Catalog::default();
                items.stackable.insert(995);
                let result = config.resume_count(
                    &mut data,
                    &items,
                    CountPurpose::Transfer {
                        deposit: true,
                        slot: 0,
                    },
                    input,
                );
                let rejected = input == i32::MIN || (width == 15 && input == i32::MAX);
                let transferred = if rejected || input == 0 {
                    0
                } else if input == -5 {
                    5
                } else {
                    20
                };
                if rejected {
                    assert!(matches!(
                        result,
                        Err(Error::Variable(variables::Error::ValueOutOfRange))
                    ));
                    assert_eq!(config.last_quantity.get(&data.variables), Ok(3));
                } else {
                    assert!(result.is_ok());
                    assert_eq!(
                        config.last_quantity.get(&data.variables),
                        Ok(input.wrapping_abs())
                    );
                }
                assert_eq!(
                    data.inventory.get(0).map_or(0, |item| item.count),
                    20 - transferred
                );
                assert_eq!(data.bank.get(0).map_or(0, |item| item.count), transferred);
            }
        }
    }
    #[test]
    fn rejected_capacity_transfer_still_remembers_resumed_count() {
        let config = config();
        let mut data = PlayerData::new(1, "bank_full".into(), CoordGrid::ZERO, Stats::default());
        config.capacity.set(&mut data.variables, 1).unwrap();
        config.tabs[9].set(&mut data.variables, 1).unwrap();
        data.bank
            .restore_slot(
                0,
                Some(Item {
                    id: 1511,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        data.bank.clear_modified();
        let bank_before = data.bank.clone();
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 995,
                    count: 20,
                    vars: 0,
                }),
            )
            .unwrap();
        let mut items = Catalog::default();
        items.stackable.insert(995);
        let events = config
            .resume_count(
                &mut data,
                &items,
                CountPurpose::Transfer {
                    deposit: true,
                    slot: 0,
                },
                -5,
            )
            .unwrap();
        assert_eq!(config.last_quantity.get(&data.variables), Ok(5));
        assert_eq!(data.inventory.get(0).unwrap().count, 20);
        assert_eq!(data.bank, bank_before);
        assert!(events
            .iter()
            .any(|event| matches!(event, Event::Message(_))));
    }
    #[test]
    fn prepared_count_layout_preserves_neighbor_bit_and_persistent_signed_base() {
        // Prepared varbit3960 uses varp304 bits1..31 (metadata receipt
        // work/wiki-bank-bindings-001.json). This is a source-level replay,
        // not evidence that a live UI accepted these signed boundary inputs.
        for sibling in [0, 1] {
            let mut config = config();
            config.last_quantity.base.id = 304;
            config.last_quantity.bit = Varbit {
                base: 304,
                lsb: 1,
                msb: 31,
            };
            let mut data = PlayerData::new(
                1,
                "bank_packed_count".into(),
                CoordGrid::ZERO,
                Stats::default(),
            );
            data.variables
                .set(config.last_quantity.base, sibling, false);
            config.capacity.set(&mut data.variables, 6).unwrap();
            let purpose = CountPurpose::Transfer {
                deposit: true,
                slot: 0,
            };
            config
                .resume_count(&mut data, &Catalog::default(), purpose, i32::MAX)
                .unwrap();
            assert_eq!(config.last_quantity.get(&data.variables), Ok(i32::MAX));
            assert_eq!(data.variables.get(304), -2 + sibling);
            let saved = data
                .variables
                .persistent(&[(304, config.last_quantity.base)].into());
            let restored = Variables::restore(saved);
            assert_eq!(config.last_quantity.get(&restored), Ok(i32::MAX));
            assert_eq!(restored.get(304) & 1, sibling);
            assert!(matches!(
                config.resume_count(&mut data, &Catalog::default(), purpose, i32::MIN),
                Err(Error::Variable(variables::Error::ValueOutOfRange))
            ));
            assert_eq!(data.variables.get(304), -2 + sibling);
        }
    }
    #[test]
    fn setting_controls_and_quantity_prompt_update_authoritative_variables() {
        let config = config();
        let mut data =
            PlayerData::new(1, "bank_settings".into(), CoordGrid::ZERO, Stats::default());
        let items = Catalog::default();
        let request = |control| Request::Control {
            control,
            op: 1,
            slot: -1,
        };
        for _ in 0..2 {
            let events = config
                .apply(
                    &mut data,
                    &items,
                    request(Control::Toggle {
                        binding: config.notes,
                        refresh_side: true,
                    }),
                )
                .unwrap();
            assert!(matches!(events.last(), Some(Event::RefreshSideOperations)));
        }
        assert_eq!(config.notes.get(&data.variables), Ok(0));
        assert_eq!(
            config
                .apply(
                    &mut data,
                    &items,
                    Request::Control {
                        control: Control::QuantityX,
                        op: 2,
                        slot: -1
                    }
                )
                .unwrap(),
            vec![Event::QuantityInput(CountPurpose::QuantityMode)]
        );
        for (input, mode) in [(-5, 3), (0, 0), (1, 3)] {
            config
                .resume_count(&mut data, &items, CountPurpose::QuantityMode, input)
                .unwrap();
            assert_eq!(
                config.last_quantity.get(&data.variables),
                Ok(input.wrapping_abs())
            );
            assert_eq!(config.quantity.get(&data.variables), Ok(mode));
        }
        for (slot, mode) in [(1, 0), (3, 1), (5, 2)] {
            config
                .apply(
                    &mut data,
                    &items,
                    Request::Control {
                        control: Control::TabDisplay(config.selected),
                        op: 1,
                        slot,
                    },
                )
                .unwrap();
            assert_eq!(config.selected.get(&data.variables), Ok(mode));
        }
        assert_eq!(
            config.apply(
                &mut data,
                &items,
                Request::Control {
                    control: Control::TabDisplay(config.selected),
                    op: 1,
                    slot: 2
                }
            ),
            Err(Error::InvalidSetting)
        );
    }
    #[test]
    fn restricted_deposit_does_not_change_inventory_or_prompt_state() {
        let mut config = config();
        config.restricted.insert(7);
        let mut data = PlayerData::new(1, "bank_actor".into(), CoordGrid::ZERO, Stats::default());
        config.capacity.set(&mut data.variables, 6).unwrap();
        data.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 7,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        let before = data.inventory.clone();
        let events = config
            .apply(
                &mut data,
                &Catalog::default(),
                Request::Button {
                    deposit: true,
                    slot: 0,
                    op: 3,
                },
            )
            .unwrap();
        assert!(
            matches!(&events[..], [Event::Message(message)] if message.contains("magical force"))
        );
        assert_eq!(before, data.inventory);
        assert_eq!(
            config.quantity(&data.variables, true, 7, 1),
            Ok(Quantity::Prompt)
        );
        assert_eq!(
            config.quantity(&data.variables, false, 5, 1),
            Ok(Quantity::Count(1))
        );
    }
}
#[derive(Clone, Debug)]
pub struct Config {
    pub sidebar: Option<crate::bank_sidebar::Sidebar>,
    pub tabs: [Binding; 10],
    pub selected: Binding,
    pub capacity: Binding,
    pub notes: Binding,
    pub placeholders: Binding,
    pub quantity: Binding,
    pub last_quantity: Binding,
    pub restricted: HashSet<i32>,
}
#[derive(Clone, Copy, Debug)]
pub enum Control {
    DepositAll {
        worn: bool,
    },
    Tabs,
    Set {
        binding: Binding,
        value: i32,
    },
    Toggle {
        binding: Binding,
        refresh_side: bool,
    },
    TabDisplay(Binding),
    QuantityX,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountPurpose {
    Transfer { deposit: bool, slot: usize },
    QuantityMode,
}
/// Count resolution, not proof of an inventory transfer or original wire-dialog identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountOutcome {
    Stale,
    Delayed,
    Resolved,
    Error,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CountTrace {
    pub tick: i32,
    pub actor: crate::simulation::ActorId,
    pub token: u64,
    pub pending: Option<(u64, CountPurpose)>,
    pub input: i32,
    pub remembered_before: Option<i32>,
    pub remembered_after: Option<i32>,
    pub outcome: CountOutcome,
}
impl CountTrace {
    pub fn json(&self) -> String {
        let pending_token = self
            .pending
            .map(|p| p.0.to_string())
            .unwrap_or_else(|| "null".into());
        let (purpose, slot) = match self.pending.map(|p| p.1) {
            Some(CountPurpose::Transfer { deposit, slot }) => (
                if deposit { "deposit" } else { "withdraw" },
                slot.to_string(),
            ),
            Some(CountPurpose::QuantityMode) => ("quantity_mode", "null".into()),
            None => ("none", "null".into()),
        };
        let optional = |value: Option<i32>| {
            value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "null".into())
        };
        let outcome = match self.outcome {
            CountOutcome::Stale => "stale",
            CountOutcome::Delayed => "delayed",
            CountOutcome::Resolved => "resolved",
            CountOutcome::Error => "error",
        };
        format!(concat!("{{\"schema\":1,\"tick\":{},\"actor_slot\":{},\"actor_generation\":{},",
            "\"count_generation\":{},\"pending_count_generation\":{},\"purpose\":\"{}\",\"slot\":{},\"input\":{},",
            "\"remembered_before\":{},\"remembered_after\":{},\"outcome\":\"{}\",\"transfer_accepted\":null}}"),
            self.tick,self.actor.slot,self.actor.generation,self.token,pending_token,purpose,slot,self.input,
            optional(self.remembered_before),optional(self.remembered_after),outcome)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Request {
    Compress,
    DepositAll {
        worn: bool,
    },
    Tab {
        slot: i32,
        op: u8,
    },
    ReorderTabs {
        from: usize,
        into: usize,
    },
    DragIntoTab {
        from: usize,
        tab: usize,
        expected: i32,
    },
    DragMain {
        from: usize,
        into: usize,
        insert: Binding,
    },
    Control {
        control: Control,
        op: u8,
        slot: i32,
    },
    DragSide {
        from: usize,
        into: usize,
    },
    Button {
        deposit: bool,
        slot: usize,
        op: u8,
    },
    Deposit {
        slot: usize,
        count: i32,
    },
    Withdraw {
        slot: usize,
        count: i32,
        placeholder: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Sidebar(crate::bank_sidebar::Error),
    Variable(variables::Error),
    Bank(BankError),
    InvalidSetting,
}
impl From<BankError> for Error {
    fn from(e: BankError) -> Self {
        Self::Bank(e)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    WornDeposited { slot: usize, item: i32 },
    CloseSearch,
    Variable(variables::Update),
    Message(String),
    RefreshSideOperations,
    QuantityInput(CountPurpose),
    Examine { deposit: bool, slot: usize },
    ExtraOperation { slot: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quantity {
    Count(i32),
    Prompt,
    Examine,
    ExtraOperation,
}
impl Config {
    /// Kotlin countDialog uses Int.absoluteValue, including MIN_VALUE wrapping.
    pub fn resume_count(
        &self,
        data: &mut PlayerData,
        items: &Catalog,
        purpose: CountPurpose,
        input: i32,
    ) -> Result<Vec<Event>, Error> {
        let count = input.wrapping_abs();
        let mut events = Vec::new();
        if let Some(update) = self.last_quantity.set(&mut data.variables, count)? {
            events.push(Event::Variable(update));
        }
        events.extend(self.apply(
            data,
            items,
            match purpose {
                CountPurpose::QuantityMode => Request::Control {
                    control: Control::QuantityX,
                    op: 1,
                    slot: -1,
                },
                CountPurpose::Transfer {
                    deposit: true,
                    slot,
                } => Request::Deposit { slot, count },
                CountPurpose::Transfer {
                    deposit: false,
                    slot,
                } => Request::Withdraw {
                    slot,
                    count,
                    placeholder: false,
                },
            },
        )?);
        Ok(events)
    }
    pub fn load(&self, data: &PlayerData) -> Result<Banking, Error> {
        let mut sizes = [0; 10];
        for (i, binding) in self.tabs.iter().enumerate() {
            sizes[i] = usize::try_from(binding.get(&data.variables)?)
                .map_err(|_| Error::InvalidSetting)?;
        }
        let selected = self.selected.get(&data.variables)?;
        let selected = match selected {
            0 => 9,
            1..=9 => selected as usize - 1,
            _ => return Err(Error::InvalidSetting),
        };
        let banking = Banking {
            tabs: Tabs::restore(sizes, data.bank.slots().len())
                .map_err(|e| Error::Bank(e.into()))?,
            selected,
            capacity: usize::try_from(self.capacity.get(&data.variables)?)
                .map_err(|_| Error::InvalidSetting)?,
            withdraw_notes: self.notes.get(&data.variables)? != 0,
            always_placehold: self.placeholders.get(&data.variables)? != 0,
        };
        banking.validate(&data.bank)?;
        Ok(banking)
    }
    pub fn quantity(
        &self,
        vars: &Variables,
        deposit: bool,
        op: u8,
        stack: i32,
    ) -> Result<Quantity, Error> {
        let last = self.last_quantity.get(vars)?.max(1);
        let left = match self.quantity.get(vars)? {
            0 => 1,
            1 => 5,
            2 => 10,
            3 => last,
            4 => i32::MAX,
            _ => return Err(Error::InvalidSetting),
        };
        use Quantity::*;
        Ok(if deposit {
            match op {
                2 => Count(left),
                3 => Count(1),
                4 => Count(5),
                5 => Count(10),
                6 => Count(last),
                7 => Prompt,
                8 => Count(i32::MAX),
                9 => ExtraOperation,
                10 => Examine,
                _ => return Err(Error::InvalidSetting),
            }
        } else {
            match op {
                1 => Count(left),
                2 => Count(1),
                3 => Count(5),
                4 => Count(10),
                5 => Count(last),
                6 => Prompt,
                7 | 9 => Count(stack),
                8 => Count(stack.wrapping_sub(1)),
                10 => Examine,
                _ => return Err(Error::InvalidSetting),
            }
        })
    }
    pub fn apply(
        &self,
        data: &mut PlayerData,
        items: &Catalog,
        request: Request,
    ) -> Result<Vec<Event>, Error> {
        let events = self.apply_inner(data, items, request)?;
        let mut output = Vec::new();
        for event in events {
            if matches!(event, Event::RefreshSideOperations) {
                if let Some(sidebar) = &self.sidebar {
                    output.extend(
                        sidebar
                            .refresh(&data.inventory, &mut data.variables)
                            .map_err(Error::Sidebar)?
                            .into_iter()
                            .map(Event::Variable),
                    );
                }
            }
            output.push(event);
        }
        Ok(output)
    }
    fn apply_inner(
        &self,
        data: &mut PlayerData,
        items: &Catalog,
        request: Request,
    ) -> Result<Vec<Event>, Error> {
        if let Request::Control { control, op, slot } = request {
            let (binding, value, refresh) = match control {
                Control::DepositAll { worn } => {
                    return self.apply_inner(data, items, Request::DepositAll { worn })
                }
                Control::Tabs => return self.apply_inner(data, items, Request::Tab { slot, op }),
                Control::Set { binding, value } => (binding, value, false),
                Control::Toggle {
                    binding,
                    refresh_side,
                } => (
                    binding,
                    i32::from(binding.get(&data.variables)? == 0),
                    refresh_side,
                ),
                Control::TabDisplay(binding) => (
                    binding,
                    match slot {
                        1 => 0,
                        3 => 1,
                        5 => 2,
                        _ => return Err(Error::InvalidSetting),
                    },
                    false,
                ),
                Control::QuantityX if op == 2 => {
                    return Ok(vec![Event::QuantityInput(CountPurpose::QuantityMode)])
                }
                Control::QuantityX => (
                    self.quantity,
                    if self.last_quantity.get(&data.variables)? == 0 {
                        0
                    } else {
                        3
                    },
                    false,
                ),
            };
            let mut events = Vec::new();
            if let Some(update) = binding.set(&mut data.variables, value)? {
                events.push(Event::Variable(update));
            }
            if refresh {
                events.push(Event::RefreshSideOperations);
            }
            return Ok(events);
        }
        if let Request::DragSide { from, into } = request {
            // The script refreshes options even when the transaction fails.
            let _ = data.inventory.move_slot(items, from, into);
            return Ok(vec![Event::RefreshSideOperations]);
        }
        if let Request::Button { deposit, slot, op } = request {
            let inv = if deposit {
                &mut data.inventory
            } else {
                &mut data.bank
            };
            let Some(item) = inv.get(slot) else {
                if !inv.slots().is_empty() {
                    let first = inv.get(0);
                    let _ = inv.restore_slot(0, first);
                }
                return Ok(vec![]);
            };
            if !deposit
                && op != 10
                && items
                    .placeholders
                    .get(&item.id)
                    .is_some_and(|ty| ty.template != 0)
            {
                return self.apply_inner(
                    data,
                    items,
                    Request::Withdraw {
                        slot,
                        count: 0,
                        placeholder: false,
                    },
                );
            }
            return match self.quantity(&data.variables, deposit, op, item.count)? {
                Quantity::Count(count) => self.apply_inner(
                    data,
                    items,
                    if deposit {
                        Request::Deposit { slot, count }
                    } else {
                        Request::Withdraw {
                            slot,
                            count,
                            placeholder: op == 9,
                        }
                    },
                ),
                Quantity::Prompt => Ok(vec![Event::QuantityInput(CountPurpose::Transfer {
                    deposit,
                    slot,
                })]),
                Quantity::Examine => Ok(vec![Event::Examine { deposit, slot }]),
                Quantity::ExtraOperation => Ok(vec![Event::ExtraOperation { slot }]),
            };
        }
        let mut banking = self.load(data)?;
        let before = banking.clone();
        let mut events = vec![];
        match request {
            Request::Compress => banking.compact(&mut data.bank)?,
            Request::DepositAll { worn } => {
                let source = if worn {
                    &mut data.equipment
                } else {
                    &mut data.inventory
                };
                if source.slots().iter().all(Option::is_none) {
                    return Ok(vec![Event::Message("You have nothing to deposit.".into())]);
                }
                let before = source.clone();
                let result =
                    banking.deposit_all(source, &mut data.bank, items, &self.restricted)?;
                if result.capacity_error {
                    events.push(Event::Message("Your bank cannot hold your items.".into()));
                } else if result.restricted {
                    events.push(Event::Message(
                        if result.any_completed {
                            "Some of your items cannot be stored in the bank."
                        } else {
                            "Your items cannot be stored in the bank."
                        }
                        .into(),
                    ));
                }
                if result.refresh_side {
                    events.push(Event::RefreshSideOperations);
                }
                if worn {
                    for (slot, old) in before.slots().iter().enumerate() {
                        if let Some(item) = old.filter(|_| *old != source.get(slot)) {
                            events.push(Event::WornDeposited {
                                slot,
                                item: item.id,
                            });
                        }
                    }
                }
            }
            Request::Tab { slot, op } => {
                let tab = if (11..=19).contains(&slot) {
                    (slot - 11) as usize
                } else {
                    Tabs::MAIN
                };
                if tab != Tabs::MAIN && banking.tabs.sizes()[tab] == 0 {
                    return Ok(vec![Event::Message(
                        "To create a new tab, drag items from your bank onto this tab.".into(),
                    )]);
                }
                if tab != Tabs::MAIN {
                    events.push(Event::CloseSearch);
                }
                match op {
                    1 => {
                        if let Some(update) = self.selected.set(
                            &mut data.variables,
                            if tab == Tabs::MAIN { 0 } else { tab as i32 + 1 },
                        )? {
                            events.push(Event::Variable(update));
                        }
                        return Ok(events);
                    }
                    6 if tab != Tabs::MAIN => banking.collapse_tab(&mut data.bank, items, tab)?,
                    7 => {
                        if banking.release_tab_placeholders(&mut data.bank, items, tab)? == 0 {
                            events.push(Event::Message(
                                "You don't have any placeholders to release.".into(),
                            ));
                        }
                    }
                    _ if tab == Tabs::MAIN => (),
                    _ => return Err(Error::InvalidSetting),
                }
            }
            Request::ReorderTabs { from, into } => {
                if banking.reorder_tabs(&mut data.bank, from, into).is_err() {
                    let first = data.bank.get(0);
                    let _ = data.bank.restore_slot(0, first);
                }
            }
            Request::DragIntoTab {
                from,
                tab,
                expected,
            } => {
                let current = data.bank.get(from);
                if tab > 9 || current.is_none_or(|item| item.id != expected) {
                    let _ = data.bank.restore_slot(from, current);
                    return Ok(events);
                }
                let target =
                    data.bank.slots().len() + 18 + if tab == Tabs::MAIN { 0 } else { tab + 1 };
                if banking
                    .drag(&mut data.bank, items, from, target, false)
                    .is_err()
                {
                    let item = data.bank.get(from);
                    let _ = data.bank.restore_slot(from, item);
                }
            }
            Request::DragMain { from, into, insert } => {
                if banking
                    .drag(
                        &mut data.bank,
                        items,
                        from,
                        into,
                        insert.get(&data.variables)? != 0,
                    )
                    .is_err()
                {
                    let slot = if from < data.bank.slots().len() {
                        from
                    } else {
                        0
                    };
                    let _ = data.bank.restore_slot(slot, data.bank.get(slot));
                }
            }
            Request::Button { .. } | Request::Control { .. } | Request::DragSide { .. } => {
                unreachable!()
            }
            Request::Deposit { slot, count } => {
                let Some(item) = data.inventory.get(slot) else {
                    if !data.inventory.slots().is_empty() {
                        let first = data.inventory.get(0);
                        let _ = data.inventory.restore_slot(0, first);
                    }
                    return Ok(events);
                };
                let result = banking.deposit(
                    &mut data.inventory,
                    &mut data.bank,
                    items,
                    slot,
                    count,
                    self.restricted.contains(&item.id),
                );
                match result {
                    Err(BankError::RestrictedItem) => events.push(Event::Message(
                        "A magical force prevents you from banking this item!".into(),
                    )),
                    Err(BankError::Inventory(InvError::NotEnoughSpace)) => {
                        events.push(Event::Message(
                            if data.bank.slots().iter().flatten().count() >= banking.capacity {
                                "You don't have enough space in your bank account.".into()
                            } else {
                                "You already have a full stack of that item in the bank.".into()
                            },
                        ))
                    }
                    Ok(result) => {
                        if result.completed != result.requested {
                            events.push(Event::Message(
                                "You already have a full stack of that item in the bank.".into(),
                            ));
                        }
                        events.push(Event::RefreshSideOperations);
                    }
                    Err(BankError::Inventory(_)) => (),
                    Err(error) => return Err(error.into()),
                }
            }
            Request::Withdraw {
                slot,
                count,
                placeholder,
            } => {
                if data.bank.get(slot).is_none() {
                    if !data.bank.slots().is_empty() {
                        let first = data.bank.get(0);
                        let _ = data.bank.restore_slot(0, first);
                    }
                    return Ok(events);
                }
                match banking.withdraw(
                    &mut data.bank,
                    &mut data.inventory,
                    items,
                    slot,
                    count,
                    placeholder,
                ) {
                    Ok(Withdrawal::RemovedPlaceholder) => (),
                    Ok(Withdrawal::Moved {
                        result,
                        cannot_note,
                    }) => {
                        if cannot_note {
                            events.push(Event::Message(
                                "This item cannot be withdrawn as a note.".into(),
                            ));
                        }
                        if result.completed != result.requested {
                            events.push(Event::Message(
                                "You don't have enough inventory space to withdraw that many."
                                    .into(),
                            ));
                        }
                        events.push(Event::RefreshSideOperations);
                    }
                    Err(BankError::Inventory(InvError::NotEnoughSpace)) => events.push(
                        Event::Message("You don't have enough inventory space.".into()),
                    ),
                    Err(BankError::Inventory(_)) => (),
                    Err(error) => return Err(error.into()),
                }
            }
        }
        // Preserve tab compression that Kotlin performs even on a failed deposit.
        for i in 0..10 {
            if before.tabs.sizes()[i] != banking.tabs.sizes()[i] {
                if let Some(update) =
                    self.tabs[i].set(&mut data.variables, banking.tabs.sizes()[i] as i32)?
                {
                    events.push(Event::Variable(update));
                }
            }
        }
        if before.selected != banking.selected {
            let value = if banking.selected == 9 {
                0
            } else {
                banking.selected as i32 + 1
            };
            if let Some(update) = self.selected.set(&mut data.variables, value)? {
                events.push(Event::Variable(update));
            }
        }
        if matches!(request, Request::Deposit { .. }) {
            events.sort_by_key(|event| !matches!(event, Event::Variable(_)));
        }
        Ok(events)
    }
}
