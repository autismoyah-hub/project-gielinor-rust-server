//! Cache-resolved metadata for ordinary PvN ranged formula inputs.
use crate::startup::Names;
use gielinor_cache::{definitions::Param, game::Definitions, parameters::ParameterType};
use gielinor_world::{bank_actions::Binding, ranged};
use std::collections::BTreeMap;
pub fn resolve(
    names: &Names,
    parameters: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<ranged::Config, String> {
    let id = |group: &str, name: &str| {
        names
            .group(group)
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing ranged {group} {name}"))
    };
    let bit = |name: &str| -> Result<Binding, String> {
        let key = id("varbit", name)? as u32;
        let bit = *defs
            .varbits
            .get(&key)
            .ok_or_else(|| format!("Missing ranged varbit {name}"))?;
        Ok(Binding {
            bit,
            base: *defs
                .varps
                .get(&(bit.base as u32))
                .ok_or_else(|| format!("Missing ranged varbit base {name}"))?,
        })
    };
    let prayers = [
        "prayer_sharpeye",
        "prayer_hawkeye",
        "prayer_eagleeye",
        "prayer_rigour",
        "prayer_deadeye_unlocked",
    ]
    .map(bit)
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .try_into()
    .unwrap();
    let mut npcs = BTreeMap::new();
    for (&npc_id, npc) in &defs.npcs {
        let param = |name: &str| -> Result<i32, String> {
            let key = id("param", name)? as u32;
            match npc.params.get(&key) {
                Some(Param::Integer(value)) => Ok(*value),
                Some(_) => Err(format!("Invalid ranged NPC parameter {name} on {npc_id}")),
                None => parameters
                    .get(&key)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing ranged NPC default {name}")),
            }
        };
        npcs.insert(
            u16::try_from(npc_id).map_err(|_| "Ranged NPC id out of range")?,
            ranged::NpcDefence {
                bonuses: [
                    param("defence_light")?,
                    param("defence_standard")?,
                    param("defence_heavy")?,
                ],
                magic_attack: param("attack_magic")?,
            },
        );
    }
    Ok(ranged::Config {
        equipment: crate::ranged_equipment_config::resolve(
            names.group("obj"),
            names.group("param"),
            names.group("category"),
            parameters,
            defs,
        )?,
        npcs,
        prayers,
        stat: u8::try_from(id("stat", "ranged")?).map_err(|_| "Ranged stat id out of range")?,
    })
}
