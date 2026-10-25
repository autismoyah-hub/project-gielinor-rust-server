//! Fixed weighted ordinary NPC loot. Content supplies an explicit guarantee
//! list, so ashes/no-remains profiles do not silently inherit generic bones.
//! Rows may delegate to a shared sub-table with its own weights, and a profile
//! may add tertiary rolls that are drawn independently of the main roll.
//! Ranged counts are drawn last, per product in batch order, as the kill hook
//! rolls each spawned item's count after the whole table.
use crate::{inventory::Catalog, npc::RandomSource, npc_death::DropPolicy, CoordGrid};
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Product {
    pub item: u16,
    pub count: i32,
}
impl Product {
    fn valid(self) -> bool {
        self.item != 0 && self.count > 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub weight: u32,
    pub product: Option<Product>,
}
/// Where the dying NPC is, for location-dependent rows (gem table talismans).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    Overworld,
    Underground,
}
impl Location {
    /// UndergroundDropChecks: a tile at z 6400 or above is underground.
    pub fn of(coords: CoordGrid) -> Self {
        if coords.z() >= 6400 {
            Self::Underground
        } else {
            Self::Overworld
        }
    }
}
/// Equipment is sampled from the eligible hero at the existing drop-roll
/// boundary. This is not a lethal-impact snapshot or a client variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub location: Location,
    pub wearing_wealth: bool,
}
pub const FINGER_SLOT: usize = 12;

/// Only attested worn rings qualify; carrying the same item has no effect.
pub fn wearing_wealth(worn: &crate::inventory::Inventory, rings: &std::collections::BTreeSet<i32>) -> bool {
    worn.get(FINGER_SLOT).is_some_and(|item| rings.contains(&item.id))
}
#[derive(Clone, Debug)]
pub enum Outcome {
    Nothing,
    Product(Product),
    /// A noted product (rare drop table silver ore); the catalog must know it as a note.
    Noted(Product),
    /// One product chosen by the dying NPC's location.
    ByLocation {
        overworld: Product,
        underground: Product,
    },
    /// Roll a shared sub-table with its own weights.
    Table(Weighted),
    /// An item whose count is drawn inclusively in `min..=max` after every table roll.
    Range { item: u16, min: i32, max: i32 },
}
#[derive(Clone, Debug)]
pub struct Row {
    pub weight: u32,
    pub outcome: Outcome,
}
/// An independent `chance`-in-`out_of` roll after the main table.
#[derive(Clone, Debug)]
pub struct Tertiary {
    pub chance: u32,
    pub out_of: u32,
    pub outcome: Outcome,
}
#[derive(Clone, Debug)]
pub struct Weighted {
    total: u32,
    entries: Arc<[Row]>,
    wealth: Option<(u32, Arc<[Row]>)>,
}
/// The products of one kill and the main-table draw (for the drop trace).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rolled {
    pub draw: Option<u32>,
    pub products: Vec<(u16, i32)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidTotal,
    InvalidWeight,
    InvalidProduct,
    InvalidChance,
}
impl Outcome {
    fn valid(&self) -> bool {
        match self {
            Self::Nothing | Self::Table(_) => true,
            Self::Product(p) | Self::Noted(p) => p.valid(),
            Self::ByLocation {
                overworld,
                underground,
            } => overworld.valid() && underground.valid(),
            Self::Range { item, min, max } => *item != 0 && 0 < *min && min <= max,
        }
    }
    fn resolve(&self, random: &mut impl RandomSource, context: Context, out: &mut Batch) {
        match self {
            Self::Nothing => {}
            Self::Product(p) | Self::Noted(p) => out.products.push((p.item, p.count)),
            Self::Range { item, min, max } => {
                out.ranges.push((out.products.len(), *min, *max));
                out.products.push((*item, *min));
            }
            Self::ByLocation {
                overworld,
                underground,
            } => {
                let p = if context.location == Location::Underground {
                    underground
                } else {
                    overworld
                };
                out.products.push((p.item, p.count));
            }
            Self::Table(table) => table.roll_nested(random, context, out),
        }
    }
    fn validate_catalog(&self, c: &Catalog, ordinary: &impl Fn(&Product) -> bool) -> bool {
        match self {
            Self::Nothing => true,
            Self::Product(p) => ordinary(p),
            Self::Noted(p) => c
                .notes
                .get(&i32::from(p.item))
                .is_some_and(|v| v.template != 0),
            Self::ByLocation {
                overworld,
                underground,
            } => ordinary(overworld) && ordinary(underground),
            Self::Range { item, min, .. } => ordinary(&Product { item: *item, count: *min }),
            Self::Table(table) => table
                .entries
                .iter()
                .all(|row| row.outcome.validate_catalog(c, ordinary)),
        }
    }
}
impl Weighted {
    pub fn new(total: u32, entries: Vec<Entry>) -> Result<Self, Error> {
        let rows = entries
            .into_iter()
            .map(|e| Row {
                weight: e.weight,
                outcome: e.product.map_or(Outcome::Nothing, Outcome::Product),
            })
            .collect();
        Self::with_rows(total, rows)
    }
    pub fn with_rows(total: u32, rows: Vec<Row>) -> Result<Self, Error> {
        if total == 0 || rows.is_empty() {
            return Err(Error::InvalidTotal);
        }
        if rows.iter().any(|r| r.weight == 0) {
            return Err(Error::InvalidWeight);
        }
        if rows.iter().any(|r| !r.outcome.valid()) {
            return Err(Error::InvalidProduct);
        }
        if rows
            .iter()
            .try_fold(0u32, |sum, r| sum.checked_add(r.weight))
            != Some(total)
        {
            return Err(Error::InvalidTotal);
        }
        Ok(Self {
            total,
            entries: rows.into(),
            wealth: None,
        })
    }
    /// Explicitly mark a shared gem/mega-rare table. Ordinary Nothing rows in
    /// unmarked monster tables remain eligible. Build both variants once.
    pub fn with_wealth_without_nothing(mut self) -> Result<Self, Error> {
        let rows: Vec<_> = self.entries.iter()
            .filter(|r| !matches!(r.outcome, Outcome::Nothing)).cloned().collect();
        let total = rows.iter().try_fold(0u32, |sum, r| sum.checked_add(r.weight))
            .ok_or(Error::InvalidTotal)?;
        if total == 0 || rows.len() == self.entries.len() || self.wealth.is_some() {
            return Err(Error::InvalidTotal);
        }
        self.wealth = Some((total, rows.into()));
        Ok(self)
    }
    fn variant(&self, context: Context) -> (u32, &[Row]) {
        if context.wearing_wealth {
            if let Some((total, rows)) = &self.wealth { return (*total, rows); }
        }
        (self.total, &self.entries)
    }
    pub fn total_with_context(&self, context: Context) -> u32 {
        self.variant(context).0
    }
    pub fn total(&self) -> u32 {
        self.total
    }
    fn selected(&self, draw: u32) -> Option<&Outcome> {
        self.selected_with_context(draw, Context { location: Location::Overworld, wearing_wealth: false })
    }
    fn selected_with_context(&self, draw: u32, context: Context) -> Option<&Outcome> {
        let (total, entries) = self.variant(context);
        if draw >= total {
            return None;
        }
        let mut at = draw;
        for row in entries {
            if at < row.weight {
                return Some(&row.outcome);
            }
            at -= row.weight;
        }
        unreachable!("validated weighted total")
    }
    /// RSWeightedTable: one draw below the table's own total; a single-row
    /// table yields its row without drawing.
    fn roll_nested(&self, random: &mut impl RandomSource, context: Context, out: &mut Batch) {
        let (total, entries) = self.variant(context);
        let outcome = if entries.len() == 1 {
            &entries[0].outcome
        } else {
            self.selected_with_context(random.below(total), context)
                .expect("RandomSource must return a value below its exclusive bound")
        };
        outcome.resolve(random, context, out);
    }
}
/// Products in batch order, with the ranged counts still to draw.
#[derive(Default)]
struct Batch {
    products: Vec<(u16, i32)>,
    ranges: Vec<(usize, i32, i32)>,
}
#[derive(Clone, Debug)]
pub struct Profile {
    guaranteed: Arc<[Product]>,
    main: Option<Weighted>,
    tertiary: Arc<[Tertiary]>,
    policy: DropPolicy,
}
impl Profile {
    pub fn new(
        policy: DropPolicy,
        guaranteed: Vec<Product>,
        main: Option<Weighted>,
    ) -> Result<Self, Error> {
        if guaranteed.iter().any(|p| !p.valid()) {
            return Err(Error::InvalidProduct);
        }
        Ok(Self {
            guaranteed: guaranteed.into(),
            main,
            tertiary: Arc::new([]),
            policy,
        })
    }
    pub fn with_tertiary(mut self, tertiary: Vec<Tertiary>) -> Result<Self, Error> {
        if tertiary
            .iter()
            .any(|t| t.chance == 0 || t.out_of == 0 || t.chance > t.out_of)
        {
            return Err(Error::InvalidChance);
        }
        if tertiary.iter().any(|t| !t.outcome.valid()) {
            return Err(Error::InvalidProduct);
        }
        self.tertiary = tertiary.into();
        Ok(self)
    }
    /// Guarantees, then one main-table draw (sub-tables draw again with their own
    /// totals), then each tertiary independently: a draw below `out_of`, a hit when
    /// below `chance`. A certain tertiary does not draw. Ranged counts draw last, in
    /// product order; a range of one value does not draw.
    pub fn roll(&self, random: &mut impl RandomSource, at: Location) -> Rolled {
        self.roll_with_context(random, Context { location: at, wearing_wealth: false })
    }
    pub fn roll_with_context(&self, random: &mut impl RandomSource, context: Context) -> Rolled {
        let mut products = Batch {
            products: self.guaranteed.iter().map(|p| (p.item, p.count)).collect(),
            ranges: vec![],
        };
        let draw = self.main.as_ref().map(|t| random.below(t.total_with_context(context)));
        if let (Some(table), Some(draw)) = (&self.main, draw) {
            table
                .selected_with_context(draw, context)
                .expect("RandomSource must return a value below its exclusive bound")
                .resolve(random, context, &mut products);
        }
        for t in self.tertiary.iter() {
            if t.chance >= t.out_of || random.below(t.out_of) < t.chance {
                t.outcome.resolve(random, context, &mut products);
            }
        }
        for &(index, min, max) in &products.ranges {
            if max > min {
                products.products[index].1 = min + random.below((max - min + 1) as u32) as i32;
            }
        }
        Rolled { draw, products: products.products }
    }
    pub fn policy(&self) -> DropPolicy {
        self.policy
    }
    pub fn bound(&self) -> Option<u32> {
        self.main.as_ref().map(|t| t.total)
    }
    /// None is a missing/out-of-range draw, distinct from an explicit Nothing row.
    /// Flat tables only: a sub-table or location row needs [`Profile::roll`].
    pub fn products(&self, draw: Option<u32>) -> Option<Vec<(u16, i32)>> {
        let selected = match (&self.main, draw) {
            (Some(t), Some(d)) => match t.selected(d)? {
                Outcome::Nothing => None,
                Outcome::Product(p) | Outcome::Noted(p) => Some(*p),
                Outcome::ByLocation { .. } | Outcome::Table(_) | Outcome::Range { .. } => return None,
            },
            (None, None) => None,
            _ => return None,
        };
        let mut out = self
            .guaranteed
            .iter()
            .map(|p| (p.item, p.count))
            .collect::<Vec<_>>();
        if let Some(p) = selected {
            out.push((p.item, p.count));
        }
        Some(out)
    }
    pub fn validate_catalog(&self, c: &Catalog) -> bool {
        let ordinary = |p: &Product| {
            let id = i32::from(p.item);
            !c.dummy.contains(&id)
                && !c.notes.get(&id).is_some_and(|v| v.template != 0)
                && !c.placeholders.get(&id).is_some_and(|v| v.template != 0)
                && !c.transforms.get(&id).is_some_and(|v| v.template != 0)
        };
        self.guaranteed.iter().all(ordinary)
            && self
                .main
                .as_ref()
                .is_none_or(|t| Outcome::Table(t.clone()).validate_catalog(c, &ordinary))
            && self
                .tertiary
                .iter()
                .all(|t| t.outcome.validate_catalog(c, &ordinary))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weighted_boundaries_guarantees_and_nothing_are_distinct() {
        let table = Weighted::new(
            128,
            vec![
                Entry {
                    weight: 64,
                    product: Some(Product {
                        item: 995,
                        count: 5,
                    }),
                },
                Entry {
                    weight: 32,
                    product: Some(Product {
                        item: 995,
                        count: 15,
                    }),
                },
                Entry {
                    weight: 32,
                    product: None,
                },
            ],
        )
        .unwrap();
        let p = Profile::new(
            DropPolicy::Imp,
            vec![Product {
                item: 25766,
                count: 1,
            }],
            Some(table),
        )
        .unwrap();
        let mut distribution = std::collections::BTreeMap::new();
        for draw in 0..128 {
            let v = p.products(Some(draw)).unwrap();
            assert_eq!(v[0], (25766, 1));
            *distribution.entry(v.get(1).map_or(0, |v| v.1)).or_insert(0) += 1;
        }
        assert_eq!(distribution, [(0, 32), (5, 64), (15, 32)].into());
        assert_eq!(p.products(None), None);
        assert_eq!(p.products(Some(128)), None);
        let no_main = Profile::new(DropPolicy::Imp, vec![], None).unwrap();
        assert_eq!(no_main.bound(), None);
        assert_eq!(no_main.products(None), Some(vec![]));
        assert_eq!(no_main.products(Some(0)), None);
    }
    #[test]
    fn malformed_totals_weights_products_and_late_catalog_templates_fail_closed() {
        for (total, entries) in [
            (0, vec![]),
            (1, vec![]),
            (
                1,
                vec![Entry {
                    weight: 0,
                    product: None,
                }],
            ),
            (
                2,
                vec![Entry {
                    weight: 1,
                    product: None,
                }],
            ),
            (
                u32::MAX,
                vec![
                    Entry {
                        weight: u32::MAX,
                        product: None,
                    },
                    Entry {
                        weight: 1,
                        product: None,
                    },
                ],
            ),
        ] {
            assert!(Weighted::new(total, entries).is_err());
        }
        for product in [
            Product { item: 0, count: 1 },
            Product {
                item: 995,
                count: 0,
            },
            Product {
                item: 995,
                count: -1,
            },
        ] {
            assert!(Weighted::new(
                1,
                vec![Entry {
                    weight: 1,
                    product: Some(product)
                }]
            )
            .is_err());
            assert!(Profile::new(DropPolicy::Imp, vec![product], None).is_err());
        }
        let p = Profile::new(
            DropPolicy::Imp,
            vec![Product {
                item: 25766,
                count: 1,
            }],
            Some(
                Weighted::new(
                    1,
                    vec![Entry {
                        weight: 1,
                        product: Some(Product {
                            item: 995,
                            count: i32::MAX,
                        }),
                    }],
                )
                .unwrap(),
            ),
        )
        .unwrap();
        for id in [995, 25766] {
            for case in 0..4 {
                let mut c = Catalog::default();
                assert!(p.validate_catalog(&c));
                match case {
                    0 => {
                        c.dummy.insert(id);
                    }
                    1 => {
                        c.notes.insert(
                            id,
                            crate::inventory::Template {
                                link: 995,
                                template: 799,
                            },
                        );
                    }
                    2 => {
                        c.placeholders.insert(
                            id,
                            crate::inventory::Template {
                                link: 995,
                                template: 14401,
                            },
                        );
                    }
                    _ => {
                        c.transforms.insert(
                            id,
                            crate::inventory::Template {
                                link: 995,
                                template: 1,
                            },
                        );
                    }
                }
                assert!(!p.validate_catalog(&c));
            }
        }
    }
    /// Replays scripted draws and records each exclusive bound requested.
    struct Script {
        draws: std::collections::VecDeque<u32>,
        bounds: Vec<u32>,
    }
    impl Script {
        fn new(draws: &[u32]) -> Self {
            Self {
                draws: draws.iter().copied().collect(),
                bounds: vec![],
            }
        }
    }
    impl RandomSource for Script {
        fn below(&mut self, bound: u32) -> u32 {
            self.bounds.push(bound);
            let draw = self.draws.pop_front().expect("unscripted draw");
            assert!(draw < bound, "scripted draw {draw} not below {bound}");
            draw
        }
    }
    fn product(item: u16, count: i32) -> Outcome {
        Outcome::Product(Product { item, count })
    }
    fn row(weight: u32, outcome: Outcome) -> Row {
        Row { weight, outcome }
    }
    fn sub_table() -> Weighted {
        Weighted::with_rows(4, vec![row(3, product(1623, 1)), row(1, product(1617, 1))]).unwrap()
    }
    fn main_with_sub_table() -> Profile {
        let main = Weighted::with_rows(
            10,
            vec![
                row(6, Outcome::Nothing),
                row(2, product(995, 7)),
                row(2, Outcome::Table(sub_table())),
            ],
        )
        .unwrap();
        Profile::new(DropPolicy::Imp, vec![Product { item: 526, count: 1 }], Some(main)).unwrap()
    }
    #[test]
    fn flat_profiles_roll_exactly_as_before() {
        let entries = (0..4)
            .map(|i| Entry {
                weight: 32,
                product: (i > 0).then_some(Product { item: 995, count: i }),
            })
            .collect();
        let p = Profile::new(
            DropPolicy::Imp,
            vec![Product { item: 25766, count: 1 }],
            Some(Weighted::new(128, entries).unwrap()),
        )
        .unwrap();
        for draw in 0..128 {
            let mut random = Script::new(&[draw]);
            let rolled = p.roll(&mut random, Location::Overworld);
            assert_eq!(random.bounds, [128], "one main draw, nothing else");
            assert_eq!(rolled.draw, Some(draw));
            assert_eq!(Some(rolled.products), p.products(Some(draw)));
        }
        let guaranteed = Profile::new(DropPolicy::Imp, vec![Product { item: 526, count: 1 }], None).unwrap();
        let mut random = Script::new(&[]);
        assert_eq!(guaranteed.roll(&mut random, Location::Overworld).products, [(526, 1)]);
        assert!(random.bounds.is_empty());
    }
    #[test]
    fn a_sub_table_row_rolls_the_sub_table_with_its_own_weights() {
        let p = main_with_sub_table();
        let mut counts = std::collections::BTreeMap::new();
        for main in 0..10 {
            for sub in 0..4 {
                let mut random = Script::new(&[main, sub]);
                let rolled = p.roll(&mut random, Location::Overworld);
                let delegated = main >= 8;
                assert_eq!(random.bounds, if delegated { vec![10, 4] } else { vec![10] });
                if !delegated && sub > 0 {
                    continue;
                }
                assert_eq!(rolled.products[0], (526, 1));
                *counts.entry(rolled.products.get(1).copied()).or_insert(0) += 1;
            }
        }
        // 6 nothing, 2 coins, and the sub-table's 2 split 3:1 over its 4 draws.
        assert_eq!(
            counts,
            [(None, 6), (Some((995, 7)), 2), (Some((1617, 1)), 2), (Some((1623, 1)), 6)].into()
        );
        assert_eq!(p.products(Some(8)), None, "flat lookup refuses a sub-table row");
    }
    #[test]
    fn nested_single_row_tables_and_locations_need_no_extra_draw() {
        let single = Weighted::with_rows(1, vec![row(1, product(1249, 1))]).unwrap();
        let talisman = Outcome::ByLocation {
            overworld: Product { item: 1462, count: 1 },
            underground: Product { item: 1452, count: 1 },
        };
        let main = Weighted::with_rows(2, vec![row(1, Outcome::Table(single)), row(1, talisman)]).unwrap();
        let p = Profile::new(DropPolicy::Imp, vec![], Some(main)).unwrap();
        let mut random = Script::new(&[0]);
        assert_eq!(p.roll(&mut random, Location::Overworld).products, [(1249, 1)]);
        assert_eq!(random.bounds, [2]);
        for (at, item) in [(Location::Overworld, 1462), (Location::Underground, 1452)] {
            let mut random = Script::new(&[1]);
            assert_eq!(p.roll(&mut random, at).products, [(item, 1)]);
        }
        assert_eq!(Location::of(CoordGrid::new(3200, 6399, 0).unwrap()), Location::Overworld);
        assert_eq!(Location::of(CoordGrid::new(3200, 6400, 0).unwrap()), Location::Underground);
    }
    #[test]
    fn tertiary_rolls_are_independent_of_the_main_roll() {
        let p = main_with_sub_table()
            .with_tertiary(vec![
                Tertiary { chance: 1, out_of: 128, outcome: product(2677, 1) },
                Tertiary { chance: 1, out_of: 1, outcome: product(1947, 2) },
                Tertiary { chance: 3, out_of: 50, outcome: Outcome::Table(sub_table()) },
            ])
            .unwrap();
        // Main "nothing" (draw 0) still rolls every tertiary: 1/128 hit, certain, 3/50 hit.
        let mut random = Script::new(&[0, 0, 2, 3]);
        let rolled = p.roll(&mut random, Location::Overworld);
        assert_eq!(random.bounds, [10, 128, 50, 4], "the certain tertiary draws nothing");
        assert_eq!(rolled.draw, Some(0));
        assert_eq!(rolled.products, [(526, 1), (2677, 1), (1947, 2), (1617, 1)]);
        // Main coins (draw 6) with tertiary misses at their boundaries (1 of 128, 3 of 50).
        let mut random = Script::new(&[6, 1, 3]);
        let rolled = p.roll(&mut random, Location::Overworld);
        assert_eq!(random.bounds, [10, 128, 50]);
        assert_eq!(rolled.products, [(526, 1), (995, 7), (1947, 2)]);
        // A main sub-table roll consumes its own draw before the tertiaries.
        let mut random = Script::new(&[9, 0, 127, 49]);
        let rolled = p.roll(&mut random, Location::Overworld);
        assert_eq!(random.bounds, [10, 4, 128, 50]);
        assert_eq!(rolled.products, [(526, 1), (1623, 1), (1947, 2)]);
    }
    #[test]
    fn malformed_rows_and_tertiaries_and_note_rows_fail_closed() {
        let bad = Product { item: 995, count: 0 };
        assert!(Weighted::with_rows(1, vec![row(1, Outcome::Noted(bad))]).is_err());
        assert!(Weighted::with_rows(
            1,
            vec![row(1, Outcome::ByLocation { overworld: bad, underground: Product { item: 1, count: 1 } })]
        )
        .is_err());
        assert!(Weighted::with_rows(3, vec![row(1, Outcome::Nothing), row(1, Outcome::Nothing)]).is_err());
        let p = Profile::new(DropPolicy::Imp, vec![], None).unwrap();
        for (chance, out_of) in [(0, 1), (1, 0), (2, 1)] {
            assert_eq!(
                p.clone()
                    .with_tertiary(vec![Tertiary { chance, out_of, outcome: Outcome::Nothing }])
                    .err(),
                Some(Error::InvalidChance)
            );
        }
        let noted = Profile::new(
            DropPolicy::Imp,
            vec![],
            Some(Weighted::with_rows(1, vec![row(1, Outcome::Noted(Product { item: 443, count: 100 }))]).unwrap()),
        )
        .unwrap();
        let mut c = Catalog::default();
        assert!(!noted.validate_catalog(&c), "a noted row must be a known note");
        c.notes.insert(443, crate::inventory::Template { link: 442, template: 799 });
        assert!(noted.validate_catalog(&c));
        let nested = main_with_sub_table()
            .with_tertiary(vec![Tertiary { chance: 1, out_of: 2, outcome: product(2677, 1) }])
            .unwrap();
        for id in [1617, 2677] {
            let mut c = Catalog::default();
            assert!(nested.validate_catalog(&c));
            c.dummy.insert(id);
            assert!(!nested.validate_catalog(&c), "sub-table and tertiary products are checked");
        }
    }
    #[test]
    fn ranged_counts_draw_inclusively_after_every_table_roll() {
        let bolts = Outcome::Range { item: 877, min: 2, max: 12 };
        let main = Weighted::with_rows(2, vec![row(1, bolts), row(1, Outcome::Table(sub_table()))]).unwrap();
        let p = Profile::new(DropPolicy::Imp, vec![Product { item: 526, count: 1 }], Some(main))
            .unwrap()
            .with_tertiary(vec![Tertiary { chance: 1, out_of: 4, outcome: product(2677, 1) }])
            .unwrap();
        // Main bolts, tertiary hit, then the count: 11 inclusive values 2..=12.
        for (count_draw, count) in [(0, 2), (5, 7), (10, 12)] {
            let mut random = Script::new(&[0, 0, count_draw]);
            let rolled = p.roll(&mut random, Location::Overworld);
            assert_eq!(random.bounds, [2, 4, 11], "count drawn after the tertiary");
            assert_eq!(rolled.products, [(526, 1), (877, count), (2677, 1)]);
        }
        // A sub-table row has no range, so nothing extra is drawn.
        let mut random = Script::new(&[1, 3, 3]);
        assert_eq!(p.roll(&mut random, Location::Overworld).products, [(526, 1), (1617, 1)]);
        assert_eq!(random.bounds, [2, 4, 4]);
        assert_eq!(p.products(Some(0)), None, "flat lookup refuses a ranged row");
        // A one-value range does not draw.
        let single = Weighted::with_rows(1, vec![row(1, Outcome::Range { item: 877, min: 3, max: 3 })]).unwrap();
        let p = Profile::new(DropPolicy::Imp, vec![], Some(single)).unwrap();
        let mut random = Script::new(&[0]);
        assert_eq!(p.roll(&mut random, Location::Overworld).products, [(877, 3)]);
        assert_eq!(random.bounds, [1]);
        for (item, min, max) in [(0, 2, 12), (877, 0, 12), (877, 5, 4), (877, -1, 3)] {
            assert!(Weighted::with_rows(1, vec![row(1, Outcome::Range { item, min, max })]).is_err());
        }
        let mut c = Catalog::default();
        assert!(p.validate_catalog(&c));
        c.dummy.insert(877);
        assert!(!p.validate_catalog(&c), "ranged products are catalog-checked");
    }
}
