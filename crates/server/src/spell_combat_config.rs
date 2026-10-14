//! Cache-resolved metadata for PvN spell formula inputs.
use crate::startup::Names;
use gielinor_cache::{definitions::Param, game::Definitions, parameters::ParameterType};
use gielinor_world::{bank_actions::Binding, spell_combat};
use std::collections::BTreeMap;
pub fn resolve(
    names: &Names,
    parameters: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<spell_combat::Config, String> {
    let id = |group: &str, name: &str| {
        names
            .group(group)
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing spell_combat {group} {name}"))
    };
    let bit = |name: &str| -> Result<Binding, String> {
        let key = id("varbit", name)? as u32;
        let bit = *defs
            .varbits
            .get(&key)
            .ok_or_else(|| format!("Missing spell_combat varbit {name}"))?;
        Ok(Binding {
            bit,
            base: *defs
                .varps
                .get(&(bit.base as u32))
                .ok_or_else(|| format!("Missing spell_combat varbit base {name}"))?,
        })
    };
    let prayers = [
        "prayer_mysticwill",
        "prayer_mysticlore",
        "prayer_mysticmight",
        "prayer_augury",
        "prayer_mystic_vigour_unlocked",
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
                Some(_) => Err(format!(
                    "Invalid spell_combat NPC parameter {name} on {npc_id}"
                )),
                None => parameters
                    .get(&key)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing spell_combat NPC default {name}")),
            }
        };
        npcs.insert(
            u16::try_from(npc_id).map_err(|_| "Ranged NPC id out of range")?,
            spell_combat::NpcDefence {
                bonus: param("defence_magic")?,
                uses_defence: param("magic_defence_uses_defence_level")? == 1,
                weakness_percent: param("elemental_weakness_percent")?,
            },
        );
    }
    Ok(spell_combat::Config {
        equipment: crate::spell_equipment_config::resolve(
            names.group("obj"),
            names.group("param"),
            parameters,
            defs,
        )?,
        npcs,
        prayers,
        mark: bit("buff_mark_of_darkness_disabled")?,
        charge: bit("buff_charge_spell_disabled")?,
        stat: u8::try_from(id("stat", "magic")?).map_err(|_| "Ranged stat id out of range")?,
    })
}
