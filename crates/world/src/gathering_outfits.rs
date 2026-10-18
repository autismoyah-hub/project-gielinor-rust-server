//! Equipment XP descriptors; inventory transactions and rounding stay in gathering rewards.
use crate::inventory::{Catalog, Inventory};
#[derive(Clone, Debug)]
pub struct Piece {
    pub slot: usize,
    pub item_ids: Vec<i32>,
    pub basis_points: u16,
}
#[derive(Clone, Debug)]
pub struct Outfit {
    pub pieces: Vec<Piece>,
    pub full_set_basis_points: u16,
    pub members: bool,
}
impl Outfit {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.pieces.is_empty() || self.pieces.len() > 14 {
            return Err("Invalid gathering outfit size");
        }
        let mut slots = std::collections::BTreeSet::new();
        let mut items = std::collections::BTreeSet::new();
        let mut total = u32::from(self.full_set_basis_points);
        for piece in &self.pieces {
            if piece.slot >= 14
                || !slots.insert(piece.slot)
                || piece.item_ids.is_empty()
                || piece.basis_points == 0
                || piece.item_ids.iter().any(|&id| id < 0 || !items.insert(id))
            {
                return Err("Invalid gathering outfit piece");
            }
            total += u32::from(piece.basis_points);
        }
        if total > 10000 {
            return Err("Invalid gathering outfit bonus");
        }
        Ok(())
    }
    pub fn bonus_basis_points(
        &self,
        worn: &Inventory,
        catalog: &Catalog,
        member: bool,
    ) -> Result<u16, &'static str> {
        self.validate()?;
        if self.members && !member {
            return Ok(0);
        }
        let mut total = 0;
        let mut full = true;
        for piece in &self.pieces {
            let matches = worn.get(piece.slot).is_some_and(|item| {
                item.count == 1
                    && piece.item_ids.contains(&item.id)
                    && catalog.ordinary_nonstackable(item.id)
            });
            // Source matches worn type, so valid item-origin variables are retained.
            if matches {
                total += piece.basis_points;
            } else {
                full = false;
            }
        }
        if full {
            total += self.full_set_basis_points;
        }
        Ok(total)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Item, StackType, Template};
    #[test]
    fn prospector_outfit_components_mixed_variants_origin_vars_and_invalid_identity() {
        let mut cfg = Outfit {
            members: true,
            full_set_basis_points: 50,
            pieces: vec![
                Piece {
                    slot: 0,
                    item_ids: vec![12013, 25549],
                    basis_points: 40,
                },
                Piece {
                    slot: 4,
                    item_ids: vec![12014, 25551],
                    basis_points: 80,
                },
                Piece {
                    slot: 7,
                    item_ids: vec![12015, 25553],
                    basis_points: 60,
                },
                Piece {
                    slot: 10,
                    item_ids: vec![12016, 25555],
                    basis_points: 20,
                },
            ],
        };
        let mut worn = Inventory::new(14, StackType::Normal, false);
        let mut cat = Catalog::default();
        assert_eq!(cfg.bonus_basis_points(&worn, &cat, true).unwrap(), 0);
        for (slot, id, expected) in [
            (0, 12013, 40),
            (4, 25551, 120),
            (7, 12015, 180),
            (10, 25555, 250),
        ] {
            worn.restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 37,
                }),
            )
            .unwrap();
            assert_eq!(cfg.bonus_basis_points(&worn, &cat, true).unwrap(), expected);
        }
        assert_eq!(cfg.bonus_basis_points(&worn, &cat, false).unwrap(), 0);
        let before = worn.clone();
        cat.notes.insert(
            12013,
            Template {
                link: 0,
                template: 799,
            },
        );
        assert_eq!(cfg.bonus_basis_points(&worn, &cat, true).unwrap(), 160);
        assert_eq!(worn, before);
        cat.notes.clear();
        worn.restore_slot(4, None).unwrap();
        worn.restore_slot(
            1,
            Some(Item {
                id: 25551,
                count: 1,
                vars: 0,
            }),
        )
        .unwrap();
        assert_eq!(cfg.bonus_basis_points(&worn, &cat, true).unwrap(), 120);
        cfg.pieces.push(Piece {
            slot: 0,
            item_ids: vec![1],
            basis_points: 40,
        });
        assert!(cfg.validate().is_err());
    }
}
