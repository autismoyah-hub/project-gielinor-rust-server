//! Standard finite GP-shop core. No named shop or UI is registered implicitly.
//! Pricing/restock boundaries follow the original PG standard profile; imported
//! Kronos pricing and infinite-stock shops require separate explicit policies.
use crate::inventory::{Catalog, Delete, Insert, Inventory, Item, StackType};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug)]
pub enum Quantity {
    One,
    Five,
    Ten,
    Fifty,
}
#[derive(Clone, Copy, Debug)]
pub enum Request {
    Value {
        slot: usize,
        item: i32,
        player_buys: bool,
    },
    Buy {
        slot: usize,
        item: i32,
        quantity: Quantity,
    },
    Sell {
        slot: usize,
        item: i32,
        quantity: Quantity,
    },
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Session {
    pub shop: u32,
    pub generation: u64,
    pub buttons: Option<ButtonBinding>,
}
/// Components supplied by attested content after the host's UI permission check.
/// No revision-specific interface IDs or named-shop policy are inferred here.
#[derive(Clone, Copy, Debug)]
pub struct ButtonBinding {
    stock: crate::ui::Component,
    carried: crate::ui::Component,
}
impl ButtonBinding {
    pub(crate) fn includes_interface(self, interface: u16) -> bool {
        self.stock.interface() == interface || self.carried.interface() == interface
    }
    pub fn new(stock: crate::ui::Component, carried: crate::ui::Component) -> Result<Self, Error> {
        if stock == carried {
            return Err(Error::InvalidProfile);
        }
        Ok(Self { stock, carried })
    }
    pub fn request(
        self,
        component: crate::ui::Component,
        sub: i32,
        operation: u8,
        item: i32,
    ) -> Option<Request> {
        if item < 0 {
            return None;
        }
        let player_buys = component == self.stock;
        if operation == 1 {
            let slot = if player_buys {
                usize::try_from(sub.checked_sub(1)?).ok()?
            } else if component == self.carried {
                usize::try_from(sub).ok()?
            } else {
                return None;
            };
            return Some(Request::Value {
                slot,
                item,
                player_buys,
            });
        }
        let quantity = Quantity::from_operation(operation)?;
        if component == self.stock {
            // Standard PG ShopScript.shopInvButton: sub zero is not stock.
            let slot = usize::try_from(sub.checked_sub(1)?).ok()?;
            Some(Request::Buy {
                slot,
                item,
                quantity,
            })
        } else if component == self.carried {
            let slot = usize::try_from(sub).ok()?;
            Some(Request::Sell {
                slot,
                item,
                quantity,
            })
        } else {
            None
        }
    }
}
impl Quantity {
    /// Original standard shop button mapping; value/examine and unused options
    /// must not accidentally become a purchase or sale.
    pub fn from_operation(operation: u8) -> Option<Self> {
        match operation {
            2 => Some(Self::One),
            3 => Some(Self::Five),
            4 => Some(Self::Ten),
            5 => Some(Self::Fifty),
            _ => None,
        }
    }
    fn count(self) -> i32 {
        match self {
            Self::One => 1,
            Self::Five => 5,
            Self::Ten => 10,
            Self::Fifty => 50,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Stock {
    pub item: i32,
    pub count: i32,
    pub interval: i32,
}
#[derive(Clone, Debug)]
pub struct Profile {
    pub currency: i32,
    pub capacity: usize,
    pub stock: Vec<Stock>,
    /// Explicitly accepted unnoted item values; absent items cannot be sold.
    pub values: BTreeMap<i32, i32>,
    pub player_pays_percent: f64,
    pub shop_pays_percent: f64,
    pub change_percent: f64,
    pub nonstock_interval: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidProfile,
    StaleItem,
    NotTraded,
    OutOfStock,
    Currency,
    Capacity,
    Overflow,
    Inventory(crate::inventory::Error),
}
impl From<crate::inventory::Error> for Error {
    fn from(e: crate::inventory::Error) -> Self {
        Self::Inventory(e)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exchange {
    pub count: i32,
    pub currency: i32,
}
pub struct Shop {
    profile: Profile,
    inventory: Inventory,
    modified: bool,
}
impl Shop {
    pub fn new(profile: Profile) -> Result<Self, Error> {
        let mut seen = std::collections::BTreeSet::new();
        if profile.currency < 0
            || profile.capacity == 0
            || profile.stock.len() > profile.capacity
            || profile.nonstock_interval <= 0
            || [
                profile.player_pays_percent,
                profile.shop_pays_percent,
                profile.change_percent,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
            || profile
                .values
                .iter()
                .any(|(&id, &cost)| id < 0 || id == profile.currency || cost < 0)
            || profile.stock.iter().any(|s| {
                s.count < 0
                    || s.interval <= 0
                    || !profile.values.contains_key(&s.item)
                    || !seen.insert(s.item)
            })
        {
            return Err(Error::InvalidProfile);
        }
        let mut inventory = Inventory::new(profile.capacity, StackType::Always, false);
        for (slot, stock) in profile.stock.iter().enumerate() {
            inventory.restore_slot(
                slot,
                Some(Item {
                    id: stock.item,
                    count: stock.count,
                    vars: 0,
                }),
            )?;
        }
        Ok(Self {
            profile,
            inventory,
            modified: false,
        })
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    /// Host calls only after every current observer has encoded this cycle.
    pub fn clear_transmitted(&mut self) {
        self.inventory.clear_modified();
    }
    pub fn needs_restock(&self) -> bool {
        self.modified
    }
    fn initial(&self, item: i32) -> i32 {
        self.profile
            .stock
            .iter()
            .find(|s| s.item == item)
            .map_or(0, |s| s.count)
    }
    fn price(&self, item: i32, current: i32, player_buys: bool) -> Result<i32, Error> {
        let base = *self.profile.values.get(&item).ok_or(Error::NotTraded)? as f64;
        let percent = if player_buys {
            self.profile.player_pays_percent
        } else {
            self.profile.shop_pays_percent
        };
        let difference = self.initial(item) as i64 - current as i64;
        let value = (base * (percent / 100.0)
            + difference as f64 * base * (self.profile.change_percent / 100.0))
            .floor()
            .max(base * if player_buys { 0.3 } else { 0.1 })
            .max(if player_buys { 1.0 } else { 0.0 });
        if !value.is_finite() || value > i32::MAX as f64 {
            return Err(Error::Overflow);
        }
        Ok(value as i32)
    }
    pub fn quote(&self, slot: usize, expected: i32, player_buys: bool) -> Result<i32, Error> {
        let item = self
            .inventory
            .get(slot)
            .filter(|i| i.id == expected)
            .ok_or(Error::StaleItem)?;
        self.price(item.id, item.count, player_buys)
    }
    /// A carried-item quote uses the accepted unnoted item and current shared
    /// stock, including zero stock for an accepted item not yet in the shop.
    /// It does not reserve stock or mutate either inventory.
    pub fn quote_sale(
        &self,
        held: &Inventory,
        catalog: &Catalog,
        slot: usize,
        expected: i32,
    ) -> Result<i32, Error> {
        let source = held
            .get(slot)
            .filter(|i| i.id == expected && i.vars == 0)
            .ok_or(Error::StaleItem)?;
        let item = catalog
            .notes
            .get(&source.id)
            .filter(|n| n.template != 0)
            .map_or(source.id, |n| n.link);
        let count = self
            .inventory
            .slots()
            .iter()
            .flatten()
            .find(|i| i.id == item)
            .map_or(0, |i| i.count);
        self.price(item, count, false)
    }
    /// One world-thread transaction against current stock, never a client quote.
    pub fn buy(
        &mut self,
        held: &mut Inventory,
        catalog: &Catalog,
        slot: usize,
        expected: i32,
        quantity: Quantity,
    ) -> Result<Exchange, Error> {
        if !catalog.stackable.contains(&self.profile.currency) {
            return Err(Error::InvalidProfile);
        }
        let item = self
            .inventory
            .get(slot)
            .filter(|i| i.id == expected)
            .ok_or(Error::StaleItem)?;
        if item.count <= 0 {
            return Err(Error::OutOfStock);
        }
        let total = |id| {
            held.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == id)
                .map(|i| i.count as i64)
                .sum::<i64>()
        };
        let available = total(self.profile.currency);
        let capacity = if catalog.stackable.contains(&item.id) {
            (i32::MAX as i64 - total(item.id)).max(0)
        } else {
            held.free_space() as i64
        };
        // Original standard path allows a one-unit transaction to free its coin slot.
        let request = quantity
            .count()
            .min(item.count)
            .min(capacity.min(i32::MAX as i64) as i32)
            .max(1);
        let mut result = Exchange {
            count: 0,
            currency: 0,
        };
        for unit in 0..request {
            let next =
                self.price(item.id, item.count - unit, true)? as i64 + result.currency as i64;
            if next > available || next > i32::MAX as i64 {
                break;
            }
            result.count += 1;
            result.currency = next as i32;
        }
        if result.count == 0 {
            return Err(Error::Currency);
        }
        let mut next = held.clone();
        next.delete(catalog, Delete::new(self.profile.currency, result.currency))?;
        next.insert(catalog, Insert::new(item.id, result.count))?;
        let mut stock = self.inventory.clone();
        stock.restore_slot(
            slot,
            if item.count == result.count && slot >= self.profile.stock.len() {
                None
            } else {
                Some(Item {
                    count: item.count - result.count,
                    ..item
                })
            },
        )?;
        *held = next;
        self.inventory = stock;
        self.modified = true;
        Ok(result)
    }
    /// Explicit accepted item values determine sellability. Currency and unknown
    /// items are rejected before inventory mutation; notes use the existing catalog.
    pub fn sell(
        &mut self,
        held: &mut Inventory,
        catalog: &Catalog,
        slot: usize,
        expected: i32,
        quantity: Quantity,
    ) -> Result<Exchange, Error> {
        if !catalog.stackable.contains(&self.profile.currency) {
            return Err(Error::InvalidProfile);
        }
        let source = held
            .get(slot)
            .filter(|i| i.id == expected && i.vars == 0)
            .ok_or(Error::StaleItem)?;
        let item = catalog
            .notes
            .get(&source.id)
            .filter(|n| n.template != 0)
            .map_or(source.id, |n| n.link);
        if !self.profile.values.contains_key(&item) {
            return Err(Error::NotTraded);
        }
        let existing = self
            .inventory
            .slots()
            .iter()
            .position(|v| v.is_some_and(|i| i.id == item));
        let target = existing
            .or_else(|| self.inventory.slots().iter().position(Option::is_none))
            .ok_or(Error::Capacity)?;
        let count = existing.map_or(0, |s| self.inventory.get(s).unwrap().count);
        let held_count = held
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == source.id && i.vars == 0)
            .map(|i| i.count as i64)
            .sum::<i64>();
        let coins = held
            .slots()
            .iter()
            .flatten()
            .filter(|i| i.id == self.profile.currency)
            .map(|i| i.count as i64)
            .sum::<i64>();
        let request = quantity
            .count()
            .min(held_count.min(i32::MAX as i64) as i32)
            .min(i32::MAX - count);
        let mut result = Exchange {
            count: 0,
            currency: 0,
        };
        for unit in 0..request {
            let next = result.currency as i64 + self.price(item, count + unit, false)? as i64;
            if next > i32::MAX as i64 - coins {
                break;
            }
            result.count += 1;
            result.currency = next as i32;
        }
        if result.count == 0 {
            return Err(Error::Capacity);
        }
        let mut next = held.clone();
        // Select only ordinary items. A same-type variable item elsewhere must
        // not be consumed by a generic multi-slot deletion.
        for _ in 0..result.count {
            let eligible = next
                .slots()
                .iter()
                .position(|v| v.is_some_and(|i| i.id == source.id && i.vars == 0))
                .ok_or(Error::StaleItem)?;
            let mut deletion = Delete::new(source.id, 1);
            deletion.strict_slot = Some(eligible);
            next.delete(catalog, deletion)?;
        }
        if result.currency > 0 {
            next.insert(catalog, Insert::new(self.profile.currency, result.currency))?;
        }
        let mut stock = self.inventory.clone();
        stock.restore_slot(
            target,
            Some(Item {
                id: item,
                count: count + result.count,
                vars: 0,
            }),
        )?;
        *held = next;
        self.inventory = stock;
        self.modified = true;
        Ok(result)
    }
    /// Standard PG LateCycle global phase: one step on a divisible clock, not
    /// a fresh per-sale countdown. Unknown/infinite intervals cannot construct this profile.
    pub fn restock(&mut self, clock: i32) {
        if !self.modified {
            return;
        }
        let mut pending = false;
        for slot in 0..self.inventory.slots().len() {
            let Some(mut item) = self.inventory.get(slot) else {
                continue;
            };
            let initial = self.profile.stock.get(slot);
            let target = initial.map_or(0, |s| s.count);
            if item.count == target {
                continue;
            }
            if clock.rem_euclid(initial.map_or(self.profile.nonstock_interval, |s| s.interval)) == 0
            {
                item.count += if item.count < target { 1 } else { -1 };
                self.inventory
                    .restore_slot(
                        slot,
                        if initial.is_none() && item.count == 0 {
                            None
                        } else {
                            Some(item)
                        },
                    )
                    .expect("existing stock slot");
            }
            pending |= item.count != target;
        }
        self.modified = pending;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires an immutable original Kotlin fixture from scripts/shop-price-parity.ps1"]
    fn original_standard_price_oracle_matches_selected_finite_vectors() {
        let path = std::env::var("GIELINOR_SHOP_PRICES").expect("explicit shop oracle fixture");
        let fixture = std::fs::read_to_string(path).unwrap();
        assert_eq!(fixture.lines().count(), 1296);
        for (case, line) in fixture.lines().enumerate() {
            let columns = line.split('|').collect::<Vec<_>>();
            assert_eq!(columns.len(), 3);
            let args = columns[0].split(',').collect::<Vec<_>>();
            assert_eq!(args.len(), 6);
            let initial = args[0].parse().unwrap();
            let current = args[1].parse().unwrap();
            let profile = Profile {
                currency: 995,
                capacity: 1,
                stock: vec![Stock {
                    item: 100,
                    count: initial,
                    interval: 100,
                }],
                values: [(100, args[2].parse().unwrap())].into(),
                player_pays_percent: args[3].parse().unwrap(),
                shop_pays_percent: args[4].parse().unwrap(),
                change_percent: args[5].parse().unwrap(),
                nonstock_interval: 100,
            };
            let shop = Shop::new(profile).unwrap();
            assert_eq!(
                shop.price(100, current, true).unwrap(),
                columns[1].parse::<i32>().unwrap(),
                "buy case {case}"
            );
            assert_eq!(
                shop.price(100, current, false).unwrap(),
                columns[2].parse::<i32>().unwrap(),
                "sell case {case}"
            );
        }
    }
    fn fixture() -> (Shop, Catalog, Inventory) {
        let profile = Profile {
            currency: 995,
            capacity: 3,
            stock: vec![Stock {
                item: 100,
                count: 2,
                interval: 100,
            }],
            values: [(100, 10), (101, 10)].into(),
            player_pays_percent: 100.0,
            shop_pays_percent: 40.0,
            change_percent: 10.0,
            nonstock_interval: 100,
        };
        let mut catalog = Catalog::default();
        catalog.stackable.insert(995);
        let mut held = Inventory::new(3, StackType::Normal, false);
        held.restore_slot(
            0,
            Some(Item {
                id: 995,
                count: 100,
                vars: 0,
            }),
        )
        .unwrap();
        (Shop::new(profile).unwrap(), catalog, held)
    }
    #[test]
    fn bulk_buy_reprices_each_unit_and_restock_uses_global_boundaries() {
        let (mut shop, catalog, mut held) = fixture();
        assert_eq!(
            shop.buy(&mut held, &catalog, 0, 100, Quantity::Five)
                .unwrap(),
            Exchange {
                count: 2,
                currency: 21
            }
        );
        assert_eq!(held.get(0).unwrap().count, 79);
        assert_eq!(
            held.slots()
                .iter()
                .flatten()
                .filter(|i| i.id == 100)
                .count(),
            2
        );
        assert_eq!(shop.inventory().get(0).unwrap().count, 0);
        shop.restock(99);
        assert_eq!(shop.inventory().get(0).unwrap().count, 0);
        shop.restock(100);
        assert_eq!(shop.inventory().get(0).unwrap().count, 1);
        shop.restock(101);
        assert_eq!(shop.inventory().get(0).unwrap().count, 1);
        shop.restock(200);
        assert_eq!(shop.inventory().get(0).unwrap().count, 2);
        assert!(!shop.needs_restock());
        shop.buy(&mut held, &catalog, 0, 100, Quantity::One)
            .unwrap_err(); // Full backpack, coins remain.
        assert!(!shop.needs_restock());
    }
    #[test]
    fn rejected_purchase_preserves_both_containers_and_does_not_enroll_restock() {
        let (mut shop, catalog, mut held) = fixture();
        held.restore_slot(
            0,
            Some(Item {
                id: 995,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let before = (held.clone(), shop.inventory().clone());
        assert_eq!(
            shop.buy(&mut held, &catalog, 0, 100, Quantity::One),
            Err(Error::Currency)
        );
        assert_eq!((&held, shop.inventory()), (&before.0, &before.1));
        assert_eq!(
            shop.buy(&mut held, &catalog, 0, 101, Quantity::One),
            Err(Error::StaleItem)
        );
        assert!(!shop.needs_restock());
    }
    #[test]
    fn buyers_cannot_both_take_the_last_unit_and_coin_slot_can_be_reused() {
        let (mut shop, catalog, mut first) = fixture();
        shop.buy(&mut first, &catalog, 0, 100, Quantity::One)
            .unwrap();
        let mut second = Inventory::new(1, StackType::Normal, false);
        second
            .restore_slot(
                0,
                Some(Item {
                    id: 995,
                    count: 11,
                    vars: 0,
                }),
            )
            .unwrap();
        assert_eq!(
            shop.buy(&mut second, &catalog, 0, 100, Quantity::One)
                .unwrap()
                .currency,
            11
        );
        assert_eq!(second.get(0).unwrap().id, 100);
        assert_eq!(
            shop.buy(&mut first, &catalog, 0, 100, Quantity::One),
            Err(Error::OutOfStock)
        );
        assert_eq!(shop.inventory().get(0).unwrap().count, 0);
    }
    #[test]
    fn sold_nonstock_disappears_and_overstock_steps_toward_default() {
        let (mut shop, catalog, mut held) = fixture();
        held.restore_slot(
            1,
            Some(Item {
                id: 101,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            shop.sell(&mut held, &catalog, 1, 101, Quantity::One)
                .unwrap(),
            Exchange {
                count: 1,
                currency: 4
            }
        );
        assert_eq!(held.get(0).unwrap().count, 104);
        assert_eq!(shop.inventory().get(1).unwrap().count, 1);
        shop.restock(99);
        assert!(shop.inventory().get(1).is_some());
        shop.restock(100);
        assert!(shop.inventory().get(1).is_none());
        assert!(!shop.needs_restock());
        held.restore_slot(
            1,
            Some(Item {
                id: 100,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        shop.sell(&mut held, &catalog, 1, 100, Quantity::One)
            .unwrap();
        assert_eq!(shop.inventory().get(0).unwrap().count, 3);
        shop.restock(200);
        assert_eq!(shop.inventory().get(0).unwrap().count, 2);
    }
    #[test]
    fn price_floors_and_invalid_intervals_are_explicit() {
        let (shop, _, _) = fixture();
        assert_eq!(shop.price(100, 1000, true).unwrap(), 3);
        assert_eq!(shop.price(100, 1000, false).unwrap(), 1);
        for interval in [-1, 0] {
            let mut profile = shop.profile.clone();
            profile.stock[0].interval = interval;
            assert!(matches!(Shop::new(profile), Err(Error::InvalidProfile)));
        }
        let mut profile = shop.profile.clone();
        profile.change_percent = f64::NAN;
        assert!(matches!(Shop::new(profile), Err(Error::InvalidProfile)));
    }

    #[test]
    fn noted_sale_preserves_variable_items_and_overflow_rejects_atomically() {
        let (mut shop, mut catalog, mut held) = fixture();
        catalog.notes.insert(
            102,
            crate::inventory::Template {
                link: 100,
                template: 799,
            },
        );
        held.restore_slot(
            1,
            Some(Item {
                id: 102,
                count: 1,
                vars: 7,
            }),
        )
        .unwrap();
        held.restore_slot(
            2,
            Some(Item {
                id: 102,
                count: 5,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            shop.sell(&mut held, &catalog, 2, 102, Quantity::Five)
                .unwrap(),
            Exchange {
                count: 5,
                currency: 11
            }
        );
        assert_eq!(held.get(1).unwrap().vars, 7);
        assert_eq!(held.get(1).unwrap().count, 1);
        assert!(held.get(2).is_none());
        assert_eq!(shop.inventory().get(0).unwrap().count, 7);
        held.restore_slot(
            0,
            Some(Item {
                id: 995,
                count: i32::MAX,
                vars: 0,
            }),
        )
        .unwrap();
        held.restore_slot(
            2,
            Some(Item {
                id: 100,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        let before = (held.clone(), shop.inventory().clone());
        assert_eq!(
            shop.sell(&mut held, &catalog, 2, 100, Quantity::One),
            Err(Error::Capacity)
        );
        assert_eq!((&held, shop.inventory()), (&before.0, &before.1));
    }

    #[test]
    fn world_late_cycle_drives_registered_stock_partition() {
        let (mut shop, catalog, mut held) = fixture();
        shop.buy(&mut held, &catalog, 0, 100, Quantity::One)
            .unwrap();
        let mut world = crate::simulation::World::default();
        world.shops.insert(7, shop);
        while world.clock.0 < 99 {
            world.tick().unwrap();
            world.drain_events();
        }
        assert_eq!(world.shops[&7].inventory().get(0).unwrap().count, 1);
        world.tick().unwrap();
        assert_eq!(world.clock.0, 100);
        assert_eq!(world.shops[&7].inventory().get(0).unwrap().count, 2);
        assert!(!world.shops[&7].needs_restock());
    }

    #[test]
    fn aggregate_purchase_cost_does_not_wrap_and_unused_buttons_do_not_trade() {
        let (shop, mut catalog, mut held) = fixture();
        let mut profile = shop.profile;
        profile.values.insert(100, i32::MAX);
        profile.change_percent = 0.0;
        let mut shop = Shop::new(profile).unwrap();
        catalog.stackable.insert(100);
        held.restore_slot(
            0,
            Some(Item {
                id: 995,
                count: i32::MAX,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(
            shop.buy(&mut held, &catalog, 0, 100, Quantity::Fifty)
                .unwrap(),
            Exchange {
                count: 1,
                currency: i32::MAX
            }
        );
        assert_eq!(shop.inventory().get(0).unwrap().count, 1);
        assert!(!held.slots().iter().flatten().any(|i| i.id == 995));
        assert_eq!(
            held.slots()
                .iter()
                .flatten()
                .find(|i| i.id == 100)
                .unwrap()
                .count,
            1
        );
        assert_eq!(
            (0..=10)
                .filter_map(|op| Quantity::from_operation(op).map(|q| (op, q.count())))
                .collect::<Vec<_>>(),
            vec![(2, 1), (3, 5), (4, 10), (5, 50)]
        );
    }
}
