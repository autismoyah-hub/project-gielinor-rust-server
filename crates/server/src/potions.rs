//! Resolve potion content against immutable item and stat definitions.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_world::potions::{self, Effect, Potion, ResolvedEffect};
use std::collections::BTreeMap;
pub fn resolve(
    objects: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<i32, Potion>, String> {
    let object = |name: &str| -> Result<i32, String> {
        let id = *objects
            .get(name)
            .ok_or_else(|| format!("Missing potion object {name}"))?;
        if !defs.items.contains_key(&(id as u32)) {
            return Err(format!("Missing potion type {id}"));
        }
        Ok(id)
    };
    let stat = |name: &str| -> Result<u8, String> {
        let id = *stats
            .get(name)
            .ok_or_else(|| format!("Missing potion stat {name}"))?;
        if !defs.stats.contains_key(&(id as u32)) {
            return Err(format!("Missing potion stat type {id}"));
        }
        u8::try_from(id).map_err(|_| "Potion stat out of range".into())
    };
    let vial = object(potions::EMPTY_VIAL)?;
    let mut result = BTreeMap::new();
    for dose in potions::catalog() {
        let id = object(dose.symbol)?;
        let effects = dose
            .effects
            .iter()
            .map(|e| {
                Ok(match *e {
                    Effect::Boost {
                        skill,
                        constant,
                        percent,
                    } => ResolvedEffect::Boost {
                        skill: stat(skill)?,
                        constant,
                        percent,
                    },
                    Effect::Recover {
                        skill,
                        constant,
                        percent,
                    } => ResolvedEffect::Recover {
                        skill: stat(skill)?,
                        constant,
                        percent,
                    },
                    Effect::Energy(percent) => ResolvedEffect::Energy(percent),
                })
            })
            .collect::<Result<_, String>>()?;
        let potion = Potion {
            id,
            next: object(dose.next)?,
            vial,
            sip: potions::sip(dose.name),
            remaining: potions::remaining(dose.left),
            effects,
        };
        if result.insert(id, potion).is_some() {
            return Err(format!("Duplicate potion ID {id}"));
        }
    }
    Ok(result)
}
