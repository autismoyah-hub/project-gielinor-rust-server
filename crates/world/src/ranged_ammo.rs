//! RangedAmmunition validation and ordered conservation/drop decisions.
use crate::npc::RandomSource;
#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub slot: usize,
    pub id: i32,
    pub recovery_rate: Option<i32>,
    pub metallic_body: bool,
    pub drop_coord: crate::CoordGrid,
    pub drop_delay: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bow,
    Crossbow,
    Ballista,
    Other,
}
#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    pub kind: Kind,
    pub required: Option<i32>,
    pub level: i32,
    pub bone: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Ammo {
    pub category: i32,
    pub level: i32,
    pub bone: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Categories {
    pub arrows: i32,
    pub dragon_arrow: i32,
    pub bolts: i32,
    pub javelins: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Validation {
    Valid,
    Missing,
    Incorrect,
    BoneWeaponIncorrect,
    ExpectedBoneWeapon,
    LevelTooHigh,
}
impl Categories {
    pub fn validate(self, weapon: Weapon, ammo: Option<Ammo>) -> Validation {
        if weapon.kind == Kind::Other {
            return Validation::Valid;
        }
        let Some(ammo) = ammo else {
            return Validation::Missing;
        };
        let required = weapon.required.unwrap_or(match weapon.kind {
            Kind::Bow => self.arrows,
            Kind::Crossbow => self.bolts,
            Kind::Ballista => self.javelins,
            Kind::Other => unreachable!(),
        });
        let alternative = weapon.kind == Kind::Bow
            && required == self.arrows
            && ammo.category == self.dragon_arrow;
        if ammo.category != required && !alternative {
            return if weapon.kind == Kind::Crossbow && weapon.bone != 0 {
                Validation::BoneWeaponIncorrect
            } else {
                Validation::Incorrect
            };
        }
        if weapon.kind == Kind::Crossbow && ammo.bone != 0 && weapon.bone == 0 {
            return Validation::ExpectedBoneWeapon;
        }
        if weapon.kind != Kind::Ballista && ammo.level > weapon.level {
            return Validation::LevelTooHigh;
        }
        Validation::Valid
    }
}
impl Validation {
    pub fn message(self, kind: Kind) -> Option<&'static str> {
        match (self, kind) {
            (Self::Valid, _) => None,
            (Self::Missing, Kind::Ballista) => Some("There are no javelins in your quiver."),
            (Self::Missing, _) => Some("There is no ammo left in your quiver."),
            (Self::BoneWeaponIncorrect, _) => {
                Some("You can't use that ammo with your bone crossbow.")
            }
            (Self::ExpectedBoneWeapon, _) => {
                Some("Bone bolts are only usable with a bone crossbow.")
            }
            (Self::LevelTooHigh, Kind::Crossbow) => {
                Some("Your crossbow isn't powerful enough for those bolts.")
            }
            (Self::LevelTooHigh, _) => Some("Your bow isn't powerful enough for those arrows."),
            (_, Kind::Crossbow) => Some("You can't use that ammo with your crossbow."),
            (_, Kind::Ballista) => Some("You can't use that ammo with your ballista."),
            _ => Some("You can't use that ammo with your bow."),
        }
    }
}
/// No recovery draw when the cape has no rate or metal armour interferes.
pub fn conserve(rate: Option<i32>, metallic_body: bool, random: &mut impl RandomSource) -> bool {
    match rate {
        Some(rate) if !metallic_body => rate > random.below(100) as i32,
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Detraction {
    pub depleted: bool,
    pub rebuild_appearance: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConsumptionError {
    WrongWornType,
    NotEnough,
    InvalidDropChance,
}
/// RangedAmmunition.detractAmmo keeps item variables when reducing the stack.
pub fn detract(
    worn: &mut crate::inventory::Inventory,
    slot: usize,
    expected: i32,
    count: i32,
) -> Result<Detraction, ConsumptionError> {
    let item = worn
        .get(slot)
        .filter(|item| item.id == expected)
        .ok_or(ConsumptionError::WrongWornType)?;
    if item.count < count {
        return Err(ConsumptionError::NotEnough);
    }
    let remaining = item.count.wrapping_sub(count);
    worn.restore_slot(
        slot,
        (remaining != 0).then_some(crate::inventory::Item {
            count: remaining,
            ..item
        }),
    )
    .expect("Existing worn slot");
    Ok(Detraction {
        depleted: remaining == 0,
        rebuild_appearance: remaining == 0 && slot == 3,
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Consumption {
    /// None means the conservation draw succeeded; no worn validation occurred.
    pub detracted: Option<Detraction>,
    /// Collision is checked at launch, after the drop draw, as in the reference.
    pub drop: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Use {
    pub slot: usize,
    pub id: i32,
    pub count: i32,
    pub recovery_rate: Option<i32>,
    pub metallic_body: bool,
    pub drop_chance: i32,
    pub drop_blocked: bool,
}
pub fn consume(
    worn: &mut crate::inventory::Inventory,
    request: Use,
    random: &mut impl RandomSource,
) -> Result<Consumption, ConsumptionError> {
    if conserve(request.recovery_rate, request.metallic_body, random) {
        return Ok(Consumption {
            detracted: None,
            drop: false,
        });
    }
    let detracted = detract(worn, request.slot, request.id, request.count)?;
    // The original RNG rejects this only after inventory mutation/unequip.
    if request.drop_chance <= 0 {
        return Err(ConsumptionError::InvalidDropChance);
    }
    let drop = random.below(request.drop_chance as u32) == 0;
    Ok(Consumption {
        detracted: Some(detracted),
        drop: drop && !request.drop_blocked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ammo_mutation_and_drop_draws_preserve_launch_order_and_last_slot_behavior() {
        use crate::inventory::{Inventory, Item, StackType};
        struct Replay(std::collections::VecDeque<(u32, u32)>);
        impl RandomSource for Replay {
            fn below(&mut self, bound: u32) -> u32 {
                let (b, v) = self.0.pop_front().unwrap();
                assert_eq!(b, bound);
                v
            }
        }
        for slot in [3, 13] {
            for blocked in [false, true] {
                let mut worn = Inventory::new(14, StackType::Normal, false);
                worn.restore_slot(
                    slot,
                    Some(Item {
                        id: 100,
                        count: 2,
                        vars: 77,
                    }),
                )
                .unwrap();
                let request = Use {
                    slot,
                    id: 100,
                    count: 1,
                    recovery_rate: Some(80),
                    metallic_body: false,
                    drop_chance: 5,
                    drop_blocked: blocked,
                };
                let mut random = Replay([(100, 80), (5, 0), (100, 79), (100, 99), (5, 4)].into());
                let first = consume(&mut worn, request, &mut random).unwrap();
                assert_eq!(
                    first,
                    Consumption {
                        detracted: Some(Detraction {
                            depleted: false,
                            rebuild_appearance: false
                        }),
                        drop: !blocked
                    }
                );
                assert_eq!(
                    worn.get(slot),
                    Some(Item {
                        id: 100,
                        count: 1,
                        vars: 77
                    })
                );
                let saved = consume(&mut worn, request, &mut random).unwrap();
                assert_eq!(saved.detracted, None);
                assert_eq!(worn.get(slot).unwrap().count, 1);
                let last = consume(&mut worn, request, &mut random).unwrap();
                assert_eq!(
                    last.detracted,
                    Some(Detraction {
                        depleted: true,
                        rebuild_appearance: slot == 3
                    })
                );
                assert!(!last.drop);
                assert!(worn.get(slot).is_none());
                assert!(random.0.is_empty());
            }
        }
    }
    #[test]
    fn conserved_ammo_skips_type_check_and_failed_drop_range_keeps_detraction() {
        use crate::inventory::{Inventory, Item, StackType};
        let mut worn = Inventory::new(14, StackType::Normal, false);
        let mut rng = Draw { value: 0, calls: 0 };
        let request = Use {
            slot: 13,
            id: 100,
            count: 1,
            recovery_rate: Some(100),
            metallic_body: false,
            drop_chance: 0,
            drop_blocked: false,
        };
        assert_eq!(
            consume(&mut worn, request, &mut rng).unwrap().detracted,
            None
        );
        worn.restore_slot(
            13,
            Some(Item {
                id: 100,
                count: 2,
                vars: 55,
            }),
        )
        .unwrap();
        assert_eq!(
            consume(
                &mut worn,
                Use {
                    recovery_rate: None,
                    ..request
                },
                &mut rng
            ),
            Err(ConsumptionError::InvalidDropChance)
        );
        assert_eq!(
            worn.get(13),
            Some(Item {
                id: 100,
                count: 1,
                vars: 55
            })
        );
        assert_eq!(rng.calls, 1);
    }
    struct Draw {
        value: u32,
        calls: u32,
    }
    impl RandomSource for Draw {
        fn below(&mut self, bound: u32) -> u32 {
            assert_eq!(bound, 100);
            self.calls += 1;
            self.value
        }
    }
    #[test]
    fn conservation_preserves_random_draw_schedule_even_for_zero_or_negative_rates() {
        for rate in [
            None,
            Some(-1),
            Some(0),
            Some(1),
            Some(80),
            Some(100),
            Some(101),
        ] {
            for metallic in [false, true] {
                for value in [0, 1, 79, 80, 99] {
                    let mut random = Draw { value, calls: 0 };
                    let actual = conserve(rate, metallic, &mut random);
                    assert_eq!(actual, !metallic && rate.is_some_and(|r| r > value as i32));
                    assert_eq!(random.calls, u32::from(rate.is_some() && !metallic));
                }
            }
        }
    }
    #[test]
    fn crossbow_error_precedence_and_javelin_level_bypass_follow_source() {
        let categories = Categories {
            arrows: 10,
            dragon_arrow: 11,
            bolts: 12,
            javelins: 13,
        };
        let bone = Weapon {
            kind: Kind::Crossbow,
            required: None,
            level: 1,
            bone: 1,
        };
        assert_eq!(
            categories.validate(
                bone,
                Some(Ammo {
                    category: 10,
                    level: 99,
                    bone: 1
                })
            ),
            Validation::BoneWeaponIncorrect
        );
        let ordinary = Weapon { bone: 0, ..bone };
        assert_eq!(
            categories.validate(
                ordinary,
                Some(Ammo {
                    category: 12,
                    level: 99,
                    bone: 1
                })
            ),
            Validation::ExpectedBoneWeapon
        );
        let ballista = Weapon {
            kind: Kind::Ballista,
            ..ordinary
        };
        assert_eq!(
            categories.validate(
                ballista,
                Some(Ammo {
                    category: 13,
                    level: 99,
                    bone: 1
                })
            ),
            Validation::Valid
        );
        assert_eq!(
            categories.validate(
                Weapon {
                    kind: Kind::Other,
                    ..ordinary
                },
                None
            ),
            Validation::Valid
        );
        assert_eq!(
            Validation::Missing.message(Kind::Ballista),
            Some("There are no javelins in your quiver.")
        );
    }
}
