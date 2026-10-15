use crate::startup::Names;
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, sequences::SequenceType,
};
use gielinor_world::npc_death::{Chicken, Config, Definition};
use std::collections::BTreeMap;
pub fn resolve(
    names: &Names,
    defs: &Definitions,
    params: &BTreeMap<u32, ParameterType>,
    sequences: &BTreeMap<u32, SequenceType>,
) -> Result<Config, String> {
    let id = |group: &str, name: &str| names.id(group, name).map_err(|e| e.to_string());
    let animation_key = id("param", "death_anim")?;
    let sound_key = id("param", "death_sound")?;
    let value = |values: &BTreeMap<u32, Param>, key: u32| -> Result<Option<i32>, String> {
        match values.get(&key) {
            Some(Param::Integer(v)) => Ok(Some(*v)),
            None => Ok(params
                .get(&key)
                .ok_or("Missing death parameter definition")?
                .default_int),
            _ => Err("Invalid death parameter value".into()),
        }
    };
    let mut definitions = BTreeMap::new();
    for (&npc_id, npc) in &defs.npcs {
        let animation = value(&npc.params, animation_key)?.ok_or("Missing death animation")?;
        if animation < 0 {
            continue;
        }
        let ticks = sequences
            .get(&(animation as u32))
            .map_or(0, SequenceType::tick_duration);
        let sound = value(&npc.params, sound_key)?
            .filter(|id| *id >= 0)
            .map(u16::try_from)
            .transpose()
            .map_err(|_| "Invalid death sound")?;
        definitions.insert(
            u16::try_from(npc_id).map_err(|_| "Invalid NPC id")?,
            Definition {
                animation: u16::try_from(animation).map_err(|_| "Invalid death animation")?,
                animation_ticks: ticks,
                sound,
            },
        );
    }
    let bones = u16::try_from(id("obj", "bones")?).map_err(|_| "Invalid bones id")?;
    let raw = u16::try_from(id("obj", "raw_chicken")?).map_err(|_| "Invalid raw chicken id")?;
    let feather = u16::try_from(id("obj", "feather")?).map_err(|_| "Invalid feather id")?;
    if bones == raw || bones == feather || raw == feather {
        return Err("Chicken drop products must be distinct".into());
    }
    for (item, stackable) in [(bones, false), (raw, false), (feather, true)] {
        let definition = defs
            .items
            .get(&(item as u32))
            .ok_or("Missing chicken drop definition")?;
        if definition.cert.template != 0
            || definition.placeholder.template != 0
            || definition.dummy == 1
            || definition.stackable != stackable
        {
            return Err("Invalid chicken drop product definition".into());
        }
    }
    let mut chickens = BTreeMap::new();
    // Only the two variants assessed for GR-12 revision239.1; do not import
    // OpenRune241's broader tutorial/farm/miscellaneous variant registration.
    for symbol in ["chicken", "chicken_brown"] {
        let type_id = u16::try_from(id("npc", symbol)?).map_err(|_| "Invalid chicken id")?;
        let npc = defs
            .npcs
            .get(&(type_id as u32))
            .ok_or("Missing chicken definition")?;
        if npc.stats[3] != 3
            || npc.ops[1].as_deref() != Some("Attack")
            || !definitions
                .get(&type_id)
                .is_some_and(|death| death.animation_ticks > 0)
        {
            return Err("Unattested chicken combat/death definition".into());
        }
        chickens.insert(type_id, Chicken { raw, feather });
    }
    Ok(Config {
        chickens,
        definitions,
        bones,
    })
}
