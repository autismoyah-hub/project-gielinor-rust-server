//! Fixed weighted ordinary NPC loot. Content supplies an explicit guarantee
//! list, so ashes/no-remains profiles do not silently inherit generic bones.
use crate::{inventory::Catalog, npc_death::DropPolicy};
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
#[derive(Clone, Debug)]
pub struct Weighted {
    total: u32,
    entries: Arc<[Entry]>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidTotal,
    InvalidWeight,
    InvalidProduct,
}
impl Weighted {
    pub fn new(total: u32, entries: Vec<Entry>) -> Result<Self, Error> {
        if total == 0 || entries.is_empty() {
            return Err(Error::InvalidTotal);
        }
        if entries.iter().any(|e| e.weight == 0) {
            return Err(Error::InvalidWeight);
        }
        if entries
            .iter()
            .any(|e| e.product.is_some_and(|p| !p.valid()))
        {
            return Err(Error::InvalidProduct);
        }
        if entries
            .iter()
            .try_fold(0u32, |sum, e| sum.checked_add(e.weight))
            != Some(total)
        {
            return Err(Error::InvalidTotal);
        }
        Ok(Self {
            total,
            entries: entries.into(),
        })
    }
    fn selected(&self, draw: u32) -> Option<Option<Product>> {
        if draw >= self.total {
            return None;
        }
        let mut at = draw;
        for e in self.entries.iter() {
            if at < e.weight {
                return Some(e.product);
            }
            at -= e.weight;
        }
        unreachable!("validated weighted total")
    }
}
#[derive(Clone, Debug)]
pub struct Profile {
    guaranteed: Arc<[Product]>,
    main: Option<Weighted>,
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
            policy,
        })
    }
    pub fn policy(&self) -> DropPolicy {
        self.policy
    }
    pub fn bound(&self) -> Option<u32> {
        self.main.as_ref().map(|t| t.total)
    }
    /// None is a missing/out-of-range draw, distinct from an explicit Nothing row.
    pub fn products(&self, draw: Option<u32>) -> Option<Vec<(u16, i32)>> {
        let selected = match (&self.main, draw) {
            (Some(t), Some(d)) => t.selected(d)?,
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
            && self.main.as_ref().is_none_or(|t| {
                t.entries
                    .iter()
                    .all(|e| e.product.as_ref().is_none_or(ordinary))
            })
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
}
