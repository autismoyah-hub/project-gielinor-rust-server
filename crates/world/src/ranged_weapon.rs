//! Default PvN ranged weapon preparation, after cooldown and specialized handlers.
use crate::{
    inventory::Inventory,
    projectile,
    ranged_ammo::{self, Categories, Validation},
};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub category: i32,
    pub weapon: ranged_ammo::Weapon,
    pub charged: bool,
    pub thrown: bool,
    pub projectile: Option<i32>,
    pub travel: Option<u16>,
    pub launch: Option<u16>,
    pub animation: Option<u16>,
    pub sound: Option<u16>,
    pub recovery: Option<i32>,
    pub metallic: bool,
}
pub struct Catalog {
    pub items: BTreeMap<i32, Item>,
    pub styles: BTreeMap<i32, projectile::Style>,
    pub categories: Categories,
    pub javelin: i32,
    pub ballista_spot: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    MissingWeapon,
    MissingDefinition,
    MissingSelectedAmmo,
    Denied(&'static str),
}
#[derive(Clone, Copy, Debug)]
pub struct Prepared {
    pub ammo_id: i32,
    pub ammo_slot: usize,
    pub javelin: bool,
    pub style: projectile::Style,
    pub travel: u16,
    pub launch: u16,
    pub animation: u16,
    pub sound: Option<u16>,
    pub recovery: Option<i32>,
    pub metallic: bool,
}
impl Catalog {
    pub fn prepare(&self, worn: &Inventory) -> Result<Prepared, Error> {
        let weapon = worn.get(3).ok_or(Error::MissingWeapon)?;
        let weapon_type = self.items.get(&weapon.id).ok_or(Error::MissingDefinition)?;
        if weapon_type.charged {
            return Err(Error::Denied("The bow refuses to fire."));
        }
        let quiver = worn.get(13);
        let quiver_type = quiver
            .map(|item| self.items.get(&item.id).ok_or(Error::MissingDefinition))
            .transpose()?;
        let validation = self.categories.validate(
            weapon_type.weapon,
            quiver_type.map(|item| ranged_ammo::Ammo {
                category: item.category,
                level: item.weapon.level,
                bone: item.weapon.bone,
            }),
        );
        if validation != Validation::Valid {
            return Err(Error::Denied(
                validation.message(weapon_type.weapon.kind).unwrap(),
            ));
        }
        let (selected, selected_type, slot) = if weapon_type.thrown {
            (weapon, weapon_type, 3)
        } else {
            (
                quiver.ok_or(Error::MissingSelectedAmmo)?,
                quiver_type.ok_or(Error::MissingSelectedAmmo)?,
                13,
            )
        };
        let (projectile, travel) = weapon_type
            .projectile
            .zip(selected_type.travel)
            .ok_or(Error::Denied("You are unable to fire your ammunition."))?;
        let style = *self
            .styles
            .get(&projectile)
            .ok_or(Error::MissingDefinition)?;
        let animation = weapon_type
            .animation
            .ok_or(Error::Denied("The bow fails to fire."))?;
        let cape = worn.get(1).and_then(|item| self.items.get(&item.id));
        let body = worn.get(4).and_then(|item| self.items.get(&item.id));
        Ok(Prepared {
            ammo_id: selected.id,
            ammo_slot: slot,
            javelin: slot == 13 && selected_type.category == self.javelin,
            style,
            travel,
            launch: selected_type.launch.unwrap_or(u16::MAX),
            animation,
            sound: weapon_type.sound,
            recovery: cape.and_then(|item| item.recovery),
            metallic: body.is_some_and(|item| item.metallic),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ranged_ammo::Kind;
    fn fixture() -> (Catalog, Inventory) {
        let item = Item {
            category: 1,
            weapon: ranged_ammo::Weapon {
                kind: Kind::Bow,
                required: None,
                level: 1,
                bone: 0,
            },
            charged: false,
            thrown: false,
            projectile: Some(1),
            travel: Some(10),
            launch: None,
            animation: Some(20),
            sound: None,
            recovery: None,
            metallic: false,
        };
        let style = projectile::Style {
            start_height: 40,
            end_height: 30,
            delay: 20,
            angle: 15,
            length_adjustment: 0,
            progress: 11,
            step_multiplier: 5,
        };
        let mut worn = Inventory::new(14, crate::inventory::StackType::Normal, false);
        for (slot, id) in [(3, 100), (13, 101)] {
            worn.restore_slot(
                slot,
                Some(crate::inventory::Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        (
            Catalog {
                items: [(100, item), (101, item)].into(),
                styles: [(1, style)].into(),
                categories: Categories {
                    arrows: 1,
                    dragon_arrow: 2,
                    bolts: 3,
                    javelins: 4,
                },
                javelin: 4,
                ballista_spot: 30,
            },
            worn,
        )
    }
    #[test]
    fn ordered_validation_and_null_launch_preserve_reference_behavior() {
        let (mut catalog, mut worn) = fixture();
        let ready = catalog.prepare(&worn).unwrap();
        assert_eq!(
            (ready.ammo_id, ready.ammo_slot, ready.launch),
            (101, 13, 65535)
        );
        catalog.items.get_mut(&100).unwrap().animation = None;
        assert!(matches!(
            catalog.prepare(&worn),
            Err(Error::Denied("The bow fails to fire."))
        ));
        catalog.items.get_mut(&101).unwrap().travel = None;
        assert!(matches!(
            catalog.prepare(&worn),
            Err(Error::Denied("You are unable to fire your ammunition."))
        ));
        worn.restore_slot(13, None).unwrap();
        assert!(matches!(
            catalog.prepare(&worn),
            Err(Error::Denied("There is no ammo left in your quiver."))
        ));
        catalog.items.get_mut(&100).unwrap().charged = true;
        assert!(matches!(
            catalog.prepare(&worn),
            Err(Error::Denied("The bow refuses to fire."))
        ));
    }
    #[test]
    fn thrown_uses_weapon_graphics_and_unknown_other_without_quiver_is_an_error() {
        let (mut catalog, mut worn) = fixture();
        worn.restore_slot(13, None).unwrap();
        catalog.items.get_mut(&100).unwrap().weapon.kind = Kind::Other;
        assert!(matches!(
            catalog.prepare(&worn),
            Err(Error::MissingSelectedAmmo)
        ));
        catalog.items.get_mut(&100).unwrap().thrown = true;
        let ready = catalog.prepare(&worn).unwrap();
        assert_eq!((ready.ammo_id, ready.ammo_slot, ready.travel), (100, 3, 10));
        assert!(!ready.javelin);
    }
}
