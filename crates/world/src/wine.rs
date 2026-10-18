//! Plain wine mixing policy and atomic pack/bank fermentation over shared engines.
use crate::{
    bank::{Tabs, Withdrawal},
    bank_actions,
    inventory::{Catalog, Delete, Insert, Inventory, Item, StackType},
    npc::RandomSource,
    simulation::PlayerData,
    skills,
};
pub const TIMER: u16 = u16::MAX; // Private host timer key; never a target-cache identifier.
pub const INTERVAL: i32 = 20;
pub const LIMIT: i32 = 60_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub grapes: i32,
    pub water: i32,
    pub unfermented: i32,
    pub zamorak_unfermented: i32,
    pub wine: i32,
    pub bad: i32,
    pub placeholder: i32,
    pub animation: u16,
    pub spot: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Policy,
    Input,
    Limit,
    Bank,
    Space,
    Xp,
}
impl Error {
    pub fn message(self) -> &'static str {
        match self {
            Self::Limit => "You cannot make more than 60,000 unfermented wines at a time.",
            Self::Bank | Self::Space => {
                "Your wine needs suitable room in your bank before it can ferment."
            }
            _ => "Your wine cannot be processed with these items right now.",
        }
    }
}
pub struct Fermented {
    pub successes: i32,
    pub failures: i32,
    pub advancement: Option<skills::Advancement>,
    pub bank_events: Vec<bank_actions::Event>,
}
impl Config {
    pub fn recipe(self) -> crate::assembly::PreciseRecipe {
        crate::assembly::PreciseRecipe {
            recipe: crate::assembly::Recipe {
                first: self.grapes,
                second: self.water,
                output: self.unfermented,
                skill: 7,
                level: 35,
                base_xp: 0,
                members: false,
            },
            xp_tenths: 0,
            ticks: 3,
            animation: Some(self.animation),
            spot: Some(self.spot),
            required_tools: &[],
            tool_animation: None,
        }
    }
    pub fn matches(self, r: crate::assembly::PreciseRecipe) -> bool {
        let v = r.recipe;
        (v.first, v.second, v.output, v.skill, v.level, r.xp_tenths)
            == (self.grapes, self.water, self.unfermented, 7, 35, 0)
    }
    fn validate(self, items: &Catalog) -> Result<(), Error> {
        let ids = [
            self.grapes,
            self.water,
            self.unfermented,
            self.zamorak_unfermented,
            self.wine,
            self.bad,
        ];
        if ids
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 6
            || !ids.into_iter().all(|id| items.ordinary_nonstackable(id))
        {
            return Err(Error::Policy);
        }
        Ok(())
    }
    pub fn pending(self, data: &PlayerData) -> bool {
        data.inventory
            .slots()
            .iter()
            .chain(data.bank.slots())
            .flatten()
            .any(|i| i.id == self.unfermented && i.count > 0)
    }
    fn stock(self, data: &PlayerData) -> Result<(Vec<usize>, Option<(usize, i32)>, i32), Error> {
        if data.inventory.slots().len() != 28
            || data.inventory.stack != StackType::Normal
            || data.inventory.placeholders
            || data.bank.stack != StackType::Always
        {
            return Err(Error::Policy);
        }
        let mut total = 0i32;
        let mut slots = Vec::new();
        let mut bank_stock = None;
        let mut seen = std::collections::BTreeSet::new();
        for (s, i) in data
            .inventory
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(s, i)| i.map(|i| (s, i)))
        {
            if [self.unfermented, self.zamorak_unfermented].contains(&i.id) {
                if i.count != 1 || i.vars != 0 {
                    return Err(Error::Input);
                }
                total = total.checked_add(1).ok_or(Error::Limit)?;
                if i.id == self.unfermented {
                    slots.push(s);
                }
            }
        }
        for (s, i) in data
            .bank
            .slots()
            .iter()
            .enumerate()
            .filter_map(|(s, i)| i.map(|i| (s, i)))
        {
            if [
                self.unfermented,
                self.zamorak_unfermented,
                self.wine,
                self.bad,
            ]
            .contains(&i.id)
            {
                if i.count <= 0 || i.vars != 0 || !seen.insert(i.id) {
                    return Err(Error::Input);
                }
                if [self.unfermented, self.zamorak_unfermented].contains(&i.id) {
                    total = total.checked_add(i.count).ok_or(Error::Limit)?;
                }
                if i.id == self.unfermented {
                    bank_stock = Some((s, i.count));
                }
            }
        }
        Ok((slots, bank_stock, total))
    }
    pub fn mixing_allowed(
        self,
        data: &PlayerData,
        items: &Catalog,
        clock: i32,
    ) -> Result<(), Error> {
        self.validate(items)?;
        if clock.checked_add(INTERVAL).is_none() {
            return Err(Error::Policy);
        }
        if self.stock(data)?.2 >= LIMIT {
            return Err(Error::Limit);
        }
        Ok(())
    }
    fn pack_image(
        self,
        source: &Inventory,
        slots: &[usize],
        outcomes: &[bool],
        items: &Catalog,
    ) -> Result<Inventory, Error> {
        if slots.len() != outcomes.len() {
            return Err(Error::Policy);
        }
        let mut next = source.clone();
        for (&slot, &good) in slots.iter().zip(outcomes) {
            let mut debit = Delete::new(self.unfermented, 1);
            debit.strict_slot = Some(slot);
            if next
                .delete(items, debit)
                .map_err(|_| Error::Input)?
                .completed
                != 1
            {
                return Err(Error::Input);
            }
            let mut credit = Insert::new(if good { self.wine } else { self.bad }, 1);
            credit.strict_slot = Some(slot);
            if next
                .insert(items, credit)
                .map_err(|_| Error::Space)?
                .completed
                != 1
            {
                return Err(Error::Space);
            }
        }
        Ok(next)
    }
    fn bank_image(
        self,
        data: &PlayerData,
        config: Option<&bank_actions::Config>,
        stock: Option<(usize, i32)>,
        good: i32,
        bad: i32,
        items: &Catalog,
    ) -> Result<
        (
            Inventory,
            crate::variables::Variables,
            Vec<bank_actions::Event>,
        ),
        Error,
    > {
        let Some((slot, count)) = stock else {
            if good != 0 || bad != 0 {
                return Err(Error::Policy);
            }
            return Ok((data.bank.clone(), data.variables.clone(), Vec::new()));
        };
        if good < 0 || bad < 0 || good.checked_add(bad) != Some(count) {
            return Err(Error::Policy);
        }
        let config = config.ok_or(Error::Bank)?;
        if items
            .placeholders
            .get(&self.unfermented)
            .is_none_or(|p| p.template != 0 || p.link != self.placeholder)
            || items
                .placeholders
                .get(&self.placeholder)
                .is_none_or(|p| p.template == 0 || p.link != self.unfermented)
            || data
                .bank
                .slots()
                .iter()
                .flatten()
                .any(|i| i.id == self.placeholder)
        {
            return Err(Error::Policy);
        }
        let before = config.load(data).map_err(|_| Error::Bank)?;
        let mut banking = before.clone();
        let source_tab = banking.tabs.for_slot(slot).ok_or(Error::Bank)?;
        // One scratch slot represents the temporary source placeholder, never permanent capacity.
        // Conversion may retain existing over-capacity contents but cannot add another unique slot.
        let allowed = before.capacity.max(before.tabs.sizes().iter().sum());
        let mut bank = Inventory::new(
            data.bank.slots().len() + 1,
            data.bank.stack,
            data.bank.placeholders,
        );
        for (s, i) in data.bank.slots().iter().enumerate() {
            bank.restore_slot(s, *i).map_err(|_| Error::Bank)?;
        }
        banking.capacity = allowed + usize::from(!before.always_placehold);
        let mut temporary = Inventory::new(2, StackType::Always, false);
        banking.withdraw_notes = false;
        // Keep a temporary placeholder to anchor this tab until all products have been deposited.
        match banking
            .withdraw(&mut bank, &mut temporary, items, slot, count, true)
            .map_err(|_| Error::Bank)?
        {
            Withdrawal::Moved { result, .. } if result.completed == count => {}
            _ => return Err(Error::Bank),
        }
        if bank
            .get(slot)
            .is_none_or(|i| i.id != self.placeholder || i.count != 1 || i.vars != 0)
        {
            return Err(Error::Bank);
        }
        temporary = Inventory::new(2, StackType::Always, false);
        for (s, (id, n)) in [(self.wine, good), (self.bad, bad)].into_iter().enumerate() {
            if n > 0 {
                temporary
                    .restore_slot(
                        s,
                        Some(Item {
                            id,
                            count: n,
                            vars: 0,
                        }),
                    )
                    .map_err(|_| Error::Bank)?;
            }
        }
        banking.selected = source_tab;
        for (s, (id, n)) in [(self.wine, good), (self.bad, bad)].into_iter().enumerate() {
            if n > 0 {
                let result = banking
                    .deposit(
                        &mut temporary,
                        &mut bank,
                        items,
                        s,
                        n,
                        config.restricted.contains(&id),
                    )
                    .map_err(|_| Error::Bank)?;
                if result.completed != n {
                    return Err(Error::Space);
                }
            }
        }
        let mut selected = before.selected;
        if !before.always_placehold {
            let anchor = bank
                .slots()
                .iter()
                .position(|i| {
                    i.is_some_and(|i| i.id == self.placeholder && i.count == 1 && i.vars == 0)
                })
                .ok_or(Error::Bank)?;
            let removing_tab = source_tab != Tabs::MAIN && banking.tabs.sizes()[source_tab] == 1;
            if !matches!(
                banking
                    .withdraw(&mut bank, &mut temporary, items, anchor, 1, false)
                    .map_err(|_| Error::Bank)?,
                Withdrawal::RemovedPlaceholder
            ) {
                return Err(Error::Bank);
            }
            if removing_tab {
                if selected == source_tab {
                    selected = Tabs::MAIN;
                } else if selected > source_tab && selected < Tabs::MAIN {
                    selected -= 1;
                }
            }
        }
        banking.selected = selected;
        if banking.tabs.sizes().iter().sum::<usize>() > allowed
            || bank.get(data.bank.slots().len()).is_some()
        {
            return Err(Error::Space);
        }
        banking.capacity = before.capacity;
        let mut final_bank = data.bank.clone();
        for slot in 0..data.bank.slots().len() {
            if final_bank.get(slot) != bank.get(slot) {
                final_bank
                    .restore_slot(slot, bank.get(slot))
                    .map_err(|_| Error::Bank)?;
            }
        }
        bank = final_bank;
        banking.validate(&bank).map_err(|_| Error::Bank)?;
        let mut variables = data.variables.clone();
        let mut events = Vec::new();
        for i in 0..10 {
            if before.tabs.sizes()[i] != banking.tabs.sizes()[i] {
                if let Some(u) = config.tabs[i]
                    .set(&mut variables, banking.tabs.sizes()[i] as i32)
                    .map_err(|_| Error::Bank)?
                {
                    events.push(bank_actions::Event::Variable(u));
                }
            }
        }
        if before.selected != selected {
            if let Some(u) = config
                .selected
                .set(
                    &mut variables,
                    if selected == Tabs::MAIN {
                        0
                    } else {
                        selected as i32 + 1
                    },
                )
                .map_err(|_| Error::Bank)?
            {
                events.push(bank_actions::Event::Variable(u));
            }
        }
        Ok((bank, variables, events))
    }
    pub fn ferment(
        self,
        data: &mut PlayerData,
        items: &Catalog,
        bank_config: Option<&bank_actions::Config>,
        random: &mut impl RandomSource,
    ) -> Result<Option<Fermented>, Error> {
        self.validate(items)?;
        let (slots, stock, combined) = self.stock(data)?;
        if combined > LIMIT {
            return Err(Error::Limit);
        }
        let count = slots.len() as i32 + stock.map_or(0, |v| v.1);
        if count == 0 {
            return Ok(None);
        }
        let level = data.stats.current(7) as u8;
        let guaranteed = level >= 68;
        if data.xp_rate_hundreds <= 0 {
            return Err(Error::Xp);
        }
        let mut maximum = data.stats.clone();
        skills::advance(
            &mut maximum,
            7,
            200.0 * f64::from(count),
            f64::from(data.xp_rate_hundreds) / 100.0,
            99,
        )
        .map_err(|_| Error::Xp)?;
        // Prepare both pack credits and all bank capacity/count endpoints before the first draw.
        self.pack_image(&data.inventory, &slots, &vec![true; slots.len()], items)?;
        if !guaranteed {
            self.pack_image(&data.inventory, &slots, &vec![false; slots.len()], items)?;
        }
        let bank_count = stock.map_or(0, |v| v.1);
        let mut endpoints = vec![(bank_count, 0)];
        if !guaranteed {
            endpoints.push((0, bank_count));
            if bank_count >= 2 {
                endpoints.extend([(1, bank_count - 1), (bank_count - 1, 1)]);
            }
        }
        for (g, b) in endpoints {
            self.bank_image(data, bank_config, stock, g, b, items)?;
        }
        let chance = skills::success_rate(48, 352, i32::from(level), 99) * 256.0;
        let mut results = vec![false; slots.len()];
        let mut cursor = 0i32;
        let mut successes = 0i32;
        let batch = count / 15;
        let mut weights = Vec::new();
        if batch > 0 {
            weights.extend(std::iter::repeat_n(batch, 15));
        }
        weights.extend(std::iter::repeat_n(1, (count % 15) as usize));
        for weight in weights {
            let good = guaranteed || f64::from(random.below(256)) < chance;
            if good {
                successes += weight;
            }
            let end = (cursor + weight).min(slots.len() as i32);
            if cursor < end {
                for s in cursor..end {
                    results[s as usize] = good;
                }
            }
            cursor += weight;
        }
        let packed_good = results.iter().filter(|&&b| b).count() as i32;
        let bank_good = successes - packed_good;
        let bank_bad = bank_count - bank_good;
        let inventory = self.pack_image(&data.inventory, &slots, &results, items)?;
        let (bank, variables, bank_events) =
            self.bank_image(data, bank_config, stock, bank_good, bank_bad, items)?;
        let mut stats = data.stats.clone();
        let advancement = if successes == 0 {
            None
        } else {
            Some(
                skills::advance(
                    &mut stats,
                    7,
                    200.0 * f64::from(successes),
                    f64::from(data.xp_rate_hundreds) / 100.0,
                    99,
                )
                .map_err(|_| Error::Xp)?,
            )
        };
        data.inventory = inventory;
        data.bank = bank;
        data.variables = variables;
        data.stats = stats;
        Ok(Some(Fermented {
            successes,
            failures: count - successes,
            advancement,
            bank_events,
        }))
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{stats::Stats, CoordGrid};
    pub(crate) fn config() -> Config {
        Config {
            grapes: 1987,
            water: 1937,
            unfermented: 1995,
            zamorak_unfermented: 20752,
            wine: 1993,
            bad: 1991,
            placeholder: 19146,
            animation: 7529,
            spot: 47,
        }
    }
    pub(crate) fn catalog() -> Catalog {
        let mut c = Catalog::default();
        c.placeholders.insert(
            1995,
            crate::inventory::Template {
                link: 19146,
                template: 0,
            },
        );
        c.placeholders.insert(
            19146,
            crate::inventory::Template {
                link: 1995,
                template: 14401,
            },
        );
        c
    }
    pub(crate) fn player(pack: usize, bank: i32, level: i8) -> (PlayerData, bank_actions::Config) {
        let mut p = PlayerData::new(
            1,
            "Wine maker".into(),
            CoordGrid::new(3200, 3200, 0).unwrap(),
            Stats::default(),
        );
        p.stats.set_current(7, level);
        p.stats.set_base(7, level);
        p.stats
            .set_xp(7, crate::skills::xp_from_level(level as u8).unwrap())
            .unwrap();
        for s in 0..pack {
            p.inventory
                .restore_slot(
                    s,
                    Some(Item {
                        id: 1995,
                        count: 1,
                        vars: 0,
                    }),
                )
                .unwrap();
        }
        let c = bank_actions::tests::config();
        c.capacity.set(&mut p.variables, 100).unwrap();
        if bank > 0 {
            p.bank
                .restore_slot(
                    0,
                    Some(Item {
                        id: 1995,
                        count: bank,
                        vars: 0,
                    }),
                )
                .unwrap();
            c.tabs[9].set(&mut p.variables, 1).unwrap();
        }
        (p, c)
    }
    struct Fixed {
        draw: u32,
        calls: u32,
    }
    impl RandomSource for Fixed {
        fn below(&mut self, n: u32) -> u32 {
            assert_eq!(n, 256);
            self.calls += 1;
            self.draw
        }
    }
    fn count(inv: &Inventory, id: i32) -> i32 {
        inv.slots()
            .iter()
            .flatten()
            .filter(|i| i.id == id)
            .map(|i| i.count)
            .sum()
    }
    #[test]
    fn wine_guaranteed_pack_and_bank_ferment_together_without_draw_at_personal_rate() {
        let c = config();
        let items = catalog();
        let (mut p, b) = player(3, 100, 68);
        p.xp_rate_hundreds = 125;
        let before = p.clone();
        let mut rng = Fixed {
            draw: 255,
            calls: 0,
        };
        let out = c
            .ferment(&mut p, &items, Some(&b), &mut rng)
            .unwrap()
            .unwrap();
        assert_eq!((out.successes, out.failures, rng.calls), (103, 0, 0));
        assert_eq!(count(&p.inventory, 1993), 3);
        assert_eq!(count(&p.bank, 1993), 100);
        assert_eq!(count(&p.bank, 1995), 0);
        assert_eq!(p.stats.fine_xp(7) - before.stats.fine_xp(7), 257500);
        assert_eq!(p.equipment, before.equipment);
        assert_eq!(b.capacity.get(&p.variables).unwrap(), 100);
        assert_eq!(b.tabs[9].get(&p.variables).unwrap(), 1);
        assert_eq!(b.notes.get(&p.variables).unwrap(), 0);
    }
    #[test]
    fn wine_large_stack_uses_fifteen_batches_and_captures_level_before_any_xp() {
        struct Sequence {
            calls: u32,
        }
        impl RandomSource for Sequence {
            fn below(&mut self, n: u32) -> u32 {
                assert_eq!(n, 256);
                self.calls += 1;
                if self.calls == 1 {
                    0
                } else {
                    255
                }
            }
        }
        let (mut p, b) = player(0, 60000, 35);
        let baseline = p.stats.fine_xp(7);
        let mut rng = Sequence { calls: 0 };
        let out = config()
            .ferment(&mut p, &catalog(), Some(&b), &mut rng)
            .unwrap()
            .unwrap();
        assert_eq!((out.successes, out.failures, rng.calls), (4000, 56000, 15));
        assert_eq!(count(&p.bank, 1993), 4000);
        assert_eq!(count(&p.bank, 1991), 56000);
        assert_eq!(p.stats.fine_xp(7), baseline + 8_000_000);
        assert_eq!(b.tabs[9].get(&p.variables).unwrap(), 2);
        let (mut p, b) = player(3, 28, 35);
        let mut rng = Fixed { draw: 0, calls: 0 };
        let out = config()
            .ferment(&mut p, &catalog(), Some(&b), &mut rng)
            .unwrap()
            .unwrap();
        assert_eq!((out.successes, rng.calls), (31, 16));
    }
    #[test]
    fn wine_full_bank_conversion_preserves_tabs_selection_settings_and_placeholder_policy() {
        for mode in 0..4 {
            let (mut p, b) = player(0, 10, 68);
            b.capacity.set(&mut p.variables, 1).unwrap();
            b.tabs[9].set(&mut p.variables, 0).unwrap();
            b.tabs[2].set(&mut p.variables, 1).unwrap();
            b.selected.set(&mut p.variables, 3).unwrap();
            b.notes.set(&mut p.variables, 1).unwrap();
            if mode == 1 {
                b.capacity.set(&mut p.variables, 2).unwrap();
                b.placeholders.set(&mut p.variables, 1).unwrap();
            }
            if mode == 2 {
                p.bank
                    .restore_slot(
                        1,
                        Some(Item {
                            id: 1993,
                            count: 5,
                            vars: 0,
                        }),
                    )
                    .unwrap();
                b.tabs[9].set(&mut p.variables, 1).unwrap();
                b.capacity.set(&mut p.variables, 2).unwrap();
            }
            if mode == 3 {
                b.capacity.set(&mut p.variables, 0).unwrap();
            } // Existing over-capacity contents may transform without adding a slot.
            let before_capacity = b.capacity.get(&p.variables).unwrap();
            let before_len = p.bank.slots().len();
            let mut rng = Fixed {
                draw: 255,
                calls: 0,
            };
            let out = config()
                .ferment(&mut p, &catalog(), Some(&b), &mut rng)
                .unwrap()
                .unwrap();
            assert_eq!(out.successes, 10);
            assert_eq!(rng.calls, 0);
            assert_eq!(p.bank.slots().len(), before_len);
            assert_eq!(b.capacity.get(&p.variables).unwrap(), before_capacity);
            assert_eq!(b.notes.get(&p.variables).unwrap(), 1);
            assert_eq!(
                b.placeholders.get(&p.variables).unwrap(),
                i32::from(mode == 1)
            );
            assert_eq!(count(&p.bank, 1993), if mode == 2 { 15 } else { 10 });
            assert_eq!(
                b.selected.get(&p.variables).unwrap(),
                if mode == 2 { 0 } else { 3 }
            );
            assert_eq!(
                b.tabs[2].get(&p.variables).unwrap(),
                if mode == 1 {
                    2
                } else if mode == 2 {
                    0
                } else {
                    1
                }
            );
            assert_eq!(
                p.bank.slots().iter().flatten().any(|i| i.id == 19146),
                mode == 1
            );
        }
    }
    #[test]
    fn wine_bad_space_catalog_rate_and_stock_refuse_before_rng_or_any_publication() {
        for mode in 0..6 {
            let (mut p, b) = player(1, 10, 35);
            let mut items = catalog();
            match mode {
                0 => {
                    b.capacity.set(&mut p.variables, 1).unwrap();
                }
                1 => {
                    items.dummy.insert(1991);
                }
                2 => p.xp_rate_hundreds = 0,
                3 => {
                    p.inventory
                        .restore_slot(
                            0,
                            Some(Item {
                                id: 1995,
                                count: 1,
                                vars: 1,
                            }),
                        )
                        .unwrap();
                }
                4 => {
                    p.bank
                        .restore_slot(
                            1,
                            Some(Item {
                                id: 1995,
                                count: 1,
                                vars: 0,
                            }),
                        )
                        .unwrap();
                }
                _ => {
                    items.placeholders.remove(&19146);
                }
            }
            let before = p.clone();
            let mut rng = Fixed { draw: 0, calls: 0 };
            assert!(
                config()
                    .ferment(&mut p, &items, Some(&b), &mut rng)
                    .is_err(),
                "mode {mode}"
            );
            assert_eq!(rng.calls, 0);
            assert_eq!(p.inventory, before.inventory);
            assert_eq!(p.bank, before.bank);
            assert_eq!(p.stats, before.stats);
            assert_eq!(
                format!("{:?}", p.variables),
                format!("{:?}", before.variables)
            );
        }
    }
    #[test]
    fn wine_mix_cap_counts_both_kinds_in_pack_and_bank_without_mutation() {
        let (mut p, b) = player(0, 59999, 68);
        p.inventory
            .restore_slot(
                0,
                Some(Item {
                    id: 20752,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        assert_eq!(
            config().mixing_allowed(&p, &catalog(), 1),
            Err(Error::Limit)
        );
        p.inventory.restore_slot(0, None).unwrap();
        assert!(config().mixing_allowed(&p, &catalog(), 1).is_ok());
        assert_eq!(
            config().mixing_allowed(&p, &catalog(), i32::MAX - 10),
            Err(Error::Policy)
        );
        assert_eq!(b.tabs[9].get(&p.variables).unwrap(), 1);
    }
}
