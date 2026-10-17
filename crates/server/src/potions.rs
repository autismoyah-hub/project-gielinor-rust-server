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
        if matches!(dose.name,"ancient brew"|"forgotten brew") {
            validate_brew_type(&dose,&defs.items[&(id as u32)])?;
        }

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
                    Effect::OverRestore{skill,constant,percent,cap_percent} => ResolvedEffect::OverRestore{skill:stat(skill)?,constant,percent,cap_percent},
                    Effect::DrainCurrent{skill,constant,percent,minimum} => ResolvedEffect::DrainCurrent{skill:stat(skill)?,constant,percent,minimum},
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

fn validate_brew_type(dose:&potions::Dose,item:&gielinor_cache::definitions::ItemType)->Result<(),String>{
    let name=match dose.name {"ancient brew"=>"Ancient brew","forgotten brew"=>"Forgotten brew",_=>return Err("Unexpected brew family".into())};
    if dose.left>3 || item.name!=format!("{name}({})",dose.left+1) || !item.members || item.stackable || item.cert.template!=0 || item.placeholder.template!=0 || item.transform.template!=0 || item.dummy==1 || item.inventory_ops[0].as_deref()!=Some("Drink") {return Err(format!("Invalid ordinary brew dose {}",dose.symbol));}
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brew_potion_bindings_require_exact_ordinary_member_drink_doses() {
        for dose in potions::catalog().into_iter().filter(|d|matches!(d.name,"ancient brew"|"forgotten brew")) {
            let mut item=gielinor_cache::definitions::ItemType::decode(&[16,0]).unwrap();let name=if dose.name=="ancient brew"{"Ancient brew"}else{"Forgotten brew"};item.name=format!("{name}({})",dose.left+1);item.inventory_ops[0]=Some("Drink".into());assert!(validate_brew_type(&dose,&item).is_ok());
            for mode in 0..7 {let mut bad=item.clone();match mode {0=>bad.stackable=true,1=>bad.members=false,2=>bad.cert.template=799,3=>bad.placeholder.template=14401,4=>bad.transform.template=1,5=>bad.name="Wrong brew(4)".into(),_=>bad.inventory_ops[0]=Some("Eat".into())};assert!(validate_brew_type(&dose,&bad).is_err());}
        }
    }
}
