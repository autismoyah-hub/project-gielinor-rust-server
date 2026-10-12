//! PlayerCommons attack selection/range and WeaponSpeeds, before damage modifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    Melee {
        kind: Option<u8>,
        style: Option<u8>,
        stance: u8,
    },
    Ranged {
        kind: u8,
        style: Option<u8>,
    },
    Staff {
        style: Option<u8>,
    },
    Spell {
        defensive: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidStance,
    WrongStyle,
    MissingWeapon,
    InvalidRange,
    MissingDefinition,
}
#[derive(Clone, Copy, Debug)]
pub struct Weapon {
    pub category: u8,
    pub range: i32,
    pub speed: i32,
}
#[derive(Default)]
pub struct Catalog {
    pub weapons: std::collections::BTreeMap<i32, Weapon>,
    pub types: std::collections::BTreeMap<i32, i32>,
    pub styles: std::collections::BTreeMap<i32, i32>,
}
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub kind: Option<u8>,
    pub style: Option<u8>,
    pub range: i32,
    pub speed: i32,
}
impl Catalog {
    pub fn selection(
        &self,
        weapon: Option<i32>,
        stance: i32,
    ) -> Result<(Option<u8>, Option<u8>), Error> {
        let weapon = weapon
            .map(|id| self.weapons.get(&id).ok_or(Error::MissingDefinition))
            .transpose()?;
        let category = crate::combat_stance::category(weapon.map(|w| w.category));
        Ok((
            attack_type(self.types.get(&category).copied().unwrap_or(0), stance)?,
            crate::bonuses::attack_style(self.styles.get(&category).copied().unwrap_or(0), stance)
                .map_err(|_| Error::InvalidStance)?,
        ))
    }
    pub fn profile(
        &self,
        weapon: Option<i32>,
        stance: i32,
        autocast: i32,
    ) -> Result<Profile, Error> {
        let (kind, style) = self.selection(weapon, stance)?;
        let weapon = weapon
            .map(|id| self.weapons.get(&id).ok_or(Error::MissingDefinition))
            .transpose()?;
        Ok(Profile {
            kind,
            style,
            range: range(weapon.map(|w| w.range), style, autocast)?,
            speed: speed(weapon.map(|w| w.speed), style),
        })
    }
}
pub fn attack_type(packed: i32, stance: i32) -> Result<Option<u8>, Error> {
    if !(0..=3).contains(&stance) {
        return Err(Error::InvalidStance);
    }
    let kind = ((packed >> (stance * 3)) & 7) as u8;
    Ok((1..=7).contains(&kind).then_some(kind))
}
pub fn resolve(
    weapon: bool,
    kind: Option<u8>,
    style: Option<u8>,
    spell: bool,
    defensive: bool,
    stance: i32,
) -> Result<Attack, Error> {
    if spell {
        return Ok(Attack::Spell { defensive });
    }
    match kind {
        Some(4) => {
            if style.is_some_and(|s| !(8..=9).contains(&s)) {
                return Err(Error::WrongStyle);
            }
            if !weapon {
                return Err(Error::MissingWeapon);
            }
            Ok(Attack::Staff { style })
        }
        Some(kind @ 5..=7) => {
            if style.is_some_and(|s| !(5..=7).contains(&s)) {
                return Err(Error::WrongStyle);
            }
            if !weapon {
                return Err(Error::MissingWeapon);
            }
            Ok(Attack::Ranged { kind, style })
        }
        _ => Ok(Attack::Melee {
            kind: kind.filter(|k| (1..=3).contains(k)),
            style: style.filter(|s| (1..=4).contains(s)),
            stance: if (0..=3).contains(&stance) {
                stance as u8
            } else {
                0
            },
        }),
    }
}
pub fn range(weapon_range: Option<i32>, style: Option<u8>, autocast_id: i32) -> Result<i32, Error> {
    if autocast_id > 0 {
        return Ok(10);
    }
    let value = weapon_range
        .map(|r| r.wrapping_add(if style == Some(7) { 2 } else { 0 }))
        .unwrap_or(1);
    if value <= 0 {
        return Err(Error::InvalidRange);
    }
    Ok(value.min(10))
}
pub fn speed(weapon_speed: Option<i32>, style: Option<u8>) -> i32 {
    let base = weapon_speed.unwrap_or(4);
    if style == Some(6) {
        base.wrapping_sub(1).max(1)
    } else {
        base
    }
}
