//! Resolve PrayerRepository definitions from the edited server cache.
use crate::startup::Names;
use gielinor_cache::{
    definitions::{ItemType, Param},
    enums::{EnumType, Value},
    game::Definitions,
    parameters::ParameterType,
};
use gielinor_world::{
    bank_actions::Binding,
    prayer_book::{self, Prayer, Regen},
    prayer_drain,
};
use std::{collections::BTreeMap, io};
pub struct Resolved {
    pub book: prayer_book::Config,
    pub drain: prayer_drain::Config,
    pub depletion_sound: u16,
}
pub fn resolve(
    names: &Names,
    defs: &Definitions,
    parameters: &BTreeMap<u32, ParameterType>,
    enums: &BTreeMap<u32, EnumType>,
) -> io::Result<Resolved> {
    let error = |message: String| io::Error::other(message);
    let binding = |id: u32| -> io::Result<Binding> {
        let bit = *defs
            .varbits
            .get(&id)
            .ok_or_else(|| error(format!("Missing prayer bit {id}")))?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| error(format!("Missing prayer base {}", bit.base)))?;
        Ok(Binding { base, bit })
    };
    let named = |name: &str| binding(names.id("varbit", name)?);
    let int = |item: &ItemType, name: &str| -> io::Result<Option<i32>> {
        let id = names.id("param", name)?;
        match item.params.get(&id) {
            Some(Param::Integer(n)) => Ok(Some(*n)),
            Some(_) => Err(error(format!("Noninteger prayer param {name}"))),
            None => Ok(parameters.get(&id).and_then(|p| p.default_int)),
        }
    };
    let required = |item: &ItemType, name: &str| -> io::Result<i32> {
        int(item, name)?.ok_or_else(|| error(format!("Missing prayer parameter {name}")))
    };
    let string = |item: &ItemType, name: &str| -> io::Result<Option<String>> {
        let id = names.id("param", name)?;
        match item.params.get(&id) {
            Some(Param::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(error(format!("Nonstring prayer param {name}"))),
            None => Ok(parameters.get(&id).and_then(|p| p.default_string.clone())),
        }
    };
    let table = |name: &str| -> io::Result<&EnumType> {
        enums
            .get(&names.id("enum", name)?)
            .ok_or_else(|| error(format!("Missing prayer enum {name}")))
    };
    let mut prayers = BTreeMap::new();
    let mut components = BTreeMap::new();
    let mut effects = Vec::new();
    let regen = [
        (
            names.id("varbit", "prayer_rapidrestore")?,
            Regen::RapidRestore,
        ),
        (names.id("varbit", "prayer_rapidheal")?, Regen::RapidHeal),
        (names.id("varbit", "prayer_preserve")?, Regen::Preserve),
    ];
    let objects = table("prayer_oc")?;
    let order =
        gielinor_cache::jvm_hash::order(&objects.insertion_order).map_err(io::Error::other)?;
    let mut prayer_order = Vec::new();
    for index in order {
        let value = &objects.values[&objects.insertion_order[index]];
        let Value::Int(item_id) = value else {
            return Err(error("Noninteger prayer object enum".into()));
        };
        if *item_id == -1 {
            continue;
        }
        let item = defs
            .items
            .get(&(*item_id as u32))
            .ok_or_else(|| error(format!("Missing prayer object {item_id}")))?;
        let id = u8::try_from(required(item, "prayer_id")?).map_err(io::Error::other)?;
        if id > 31 {
            return Err(error(format!("Prayer id exceeds bitset: {id}")));
        }
        let bit_id = required(item, "prayer_varbit")? as u32;
        let prayer = Prayer {
            id,
            name: string(item, "prayer_name")?
                .ok_or_else(|| error("Missing prayer name".into()))?,
            level: required(item, "prayer_levelreq")?,
            sound: u16::try_from(required(item, "prayer_sound")?).map_err(io::Error::other)?,
            enabled: binding(bit_id)?,
            overhead: int(item, "prayer_overhead")?
                .filter(|v| *v != -1)
                .map(u8::try_from)
                .transpose()
                .map_err(io::Error::other)?,
            unlock: int(item, "prayer_unlock_varbit")?
                .filter(|v| *v != -1)
                .map(|v| binding(v as u32))
                .transpose()?,
            unlock_state: required(item, "prayer_unlock_state")?,
            defence: int(item, "statreq1_level")?.filter(|v| *v != -1),
            locked_message: string(item, "prayer_locked_message")?,
            regen: regen
                .iter()
                .find(|(key, _)| *key == bit_id)
                .map(|(_, kind)| *kind),
        };
        let component = required(item, "prayer_component")? as u32;
        prayer_order.push(id);
        effects.push((id, required(item, "prayer_drain_effect")?));
        if prayers.insert(id, prayer).is_some() || components.insert(component, id).is_some() {
            return Err(error(format!(
                "Duplicate prayer id or component: {id}/{component}"
            )));
        }
    }
    if prayers.is_empty() {
        return Err(error("No prayer definitions".into()));
    }
    let mut collisions: BTreeMap<u8, Vec<u8>> = BTreeMap::new();
    for name in [
        "prayer_attack_collisions",
        "prayer_strength_collisions",
        "prayer_defence_collisions",
        "prayer_overhead_collisions",
    ] {
        let group = table(name)?;
        let ids = prayer_order
            .iter()
            .copied()
            .filter(|id| matches!(group.values.get(&(*id as i32)), Some(Value::Int(1))))
            .collect::<Vec<_>>();
        for id in &ids {
            let union = collisions.entry(*id).or_default();
            for other in &ids {
                if !union.contains(other) {
                    union.push(*other);
                }
            }
        }
    }
    let skill = u8::try_from(names.id("stat", "prayer")?).map_err(io::Error::other)?;
    let all_active = named("prayer_allactive")?;
    let counter = named("prayer_drain_counter")?;
    Ok(Resolved {
        book: prayer_book::Config {
            prayer_skill: skill,
            defence_skill: u8::try_from(names.id("stat", "defence")?).map_err(io::Error::other)?,
            all_active,
            quick_active: named("quickprayer_active")?,
            drain_counter: counter,
            disable_sound: names.short("synth", "prayer_disable")?,
            prayers,
            collisions,
            components,
        },
        drain: prayer_drain::Config {
            enabled: all_active,
            counter,
            resistance: named("prayer_drain_resistance")?,
            skill,
            effects,
        },
        depletion_sound: names.short("synth", "prayer_drain")?,
    })
}
