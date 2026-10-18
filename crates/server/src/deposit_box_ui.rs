//! Revision-bound deposit presentation; custom quantity prompts remain separate.
use gielinor_cache::symbols::Symbols;
use gielinor_world::ui::Component;
#[derive(Clone, Copy, Debug)]
pub struct Presentation {
    pub interface: u16,
    pub inventory: u16,
    pub disabled_inventory: u16,
    pub grid: Component,
    pub deposit_inventory: Component,
    pub deposit_worn: Component,
    pub animation: u16,
    pub presets: [(Component, i32); 4],
    pub quantity_x: Component,
}
impl Presentation {
    pub fn resolve(
        interfaces: &Symbols,
        components: &Symbols,
        sequences: &Symbols,
    ) -> Result<Self, String> {
        for (name, id) in [
            ("bank_depositbox", 192),
            ("inventory", 149),
            ("inventory_noops", 268),
        ] {
            if interfaces.get(name) != Some(&id) {
                return Err(format!("Invalid deposit interface {name}"));
            }
        }
        for (name, id) in [
            ("bank_depositbox:inventory", 12582935),
            ("bank_depositbox:deposit_inv", 12582941),
            ("bank_depositbox:deposit_worn", 12582942),
        ] {
            if components.get(name) != Some(&id) {
                return Err(format!("Invalid deposit component {name}"));
            }
        }
        if sequences.get("human_leverdown") != Some(&834) {
            return Err("Invalid deposit animation".into());
        }
        Ok(Self {
            interface: 192,
            inventory: 149,
            disabled_inventory: 268,
            grid: Component(12582935),
            deposit_inventory: Component(12582941),
            deposit_worn: Component(12582942),
            animation: 834,
            quantity_x: Component(12582949),
            presets: [
                (Component(12582946), 0),
                (Component(12582947), 1),
                (Component(12582948), 4),
                (Component(12582950), 2),
            ],
        })
    }
    pub fn worn_slot(self, component: Component) -> Option<usize> {
        if component.interface() != self.interface {
            return None;
        }
        component.child().checked_sub(9).and_then(|index| {
            [0, 1, 2, 3, 4, 5, 7, 9, 10, 12, 13]
                .get(index as usize)
                .copied()
        })
    }
    pub fn bulk(self, component: Component, op: u8) -> Option<bool> {
        if op != 1 {
            None
        } else if component == self.deposit_inventory {
            Some(false)
        } else if component == self.deposit_worn {
            Some(true)
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deposit_box_ui_exact_bulk_roles_never_map_grid_or_withdraw() {
        let i = [
            ("bank_depositbox".into(), 192),
            ("inventory".into(), 149),
            ("inventory_noops".into(), 268),
        ]
        .into();
        let c = [
            ("bank_depositbox:inventory".into(), 12582935),
            ("bank_depositbox:deposit_inv".into(), 12582941),
            ("bank_depositbox:deposit_worn".into(), 12582942),
        ]
        .into();
        let s = [("human_leverdown".into(), 834)].into();
        let p = Presentation::resolve(&i, &c, &s).unwrap();
        assert_eq!(p.bulk(p.deposit_inventory, 1), Some(false));
        assert_eq!(p.bulk(p.deposit_worn, 1), Some(true));
        assert_eq!(p.bulk(p.grid, 1), None);
        for (child, slot) in (9..=19).zip([0, 1, 2, 3, 4, 5, 7, 9, 10, 12, 13]) {
            assert_eq!(p.worn_slot(Component::new(192, child)), Some(slot));
        }
        for component in [
            Component::new(192, 8),
            Component::new(192, 20),
            Component::new(149, 9),
            p.grid,
        ] {
            assert_eq!(p.worn_slot(component), None);
        }

        assert_eq!(
            p.presets.map(|(c, m)| (c.child(), m)),
            [(34, 0), (35, 1), (36, 4), (38, 2)]
        );
        assert_eq!(p.bulk(p.deposit_inventory, 2), None);
    }
}
