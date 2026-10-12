use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::{combat_xp, melee_modifiers::Npc, npc_combat::Definition};
use std::collections::BTreeMap;
pub fn resolve(
    params: &Symbols,
    categories: &Symbols,
    npcs: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<BTreeMap<u16, Definition>, String> {
    let symbol = |map: &Symbols, name: &str| {
        map.get(name)
            .copied()
            .ok_or_else(|| format!("Missing NPC combat symbol {name}"))
    };
    let vampire = symbol(categories, "vampyres")?;
    let corp = symbol(npcs, "corp_beast")?;
    let mut result = BTreeMap::new();
    for (&id, npc) in &defs.npcs {
        let int = |name: &str| -> Result<Option<i32>, String> {
            let key = symbol(params, name)? as u32;
            match npc.params.get(&key) {
                Some(Param::Integer(v)) => Ok(Some(*v)),
                Some(_) => Err(format!("Invalid NPC combat param {name} on {id}")),
                None => Ok(parameter_types
                    .get(&key)
                    .ok_or_else(|| format!("Missing NPC combat default {name}"))?
                    .default_int),
            }
        };
        let required =
            |name: &str| int(name)?.ok_or_else(|| format!("Null NPC combat default {name}"));
        let defence = [
            required("defence_stab")?,
            required("defence_slash")?,
            required("defence_crush")?,
        ];
        let xp_multiplier = combat_xp::npc_multiplier(
            int("npc_com_xp_multiplier")?,
            [npc.stats[0], npc.stats[2], npc.stats[1], npc.stats[3]],
            defence,
            required("attack_melee")?,
            required("melee_strength")?,
        );
        let mut attributes = 0;
        let mut add = |a: Npc| {
            attributes |= 1u64 << a as u8;
        };
        for (name, attr) in [
            ("revenant", Npc::Revenant),
            ("undead", Npc::Undead),
            ("demon", Npc::Demon),
            ("demonbane_resistant", Npc::DemonbaneResistance),
            ("draconic", Npc::Draconic),
            ("kalphite", Npc::Kalphite),
            ("golem", Npc::Golem),
            ("leafy", Npc::Leafy),
            ("rat", Npc::Rat),
            ("shade", Npc::Shade),
            ("xerician", Npc::Xerician),
            ("amascutnpc", Npc::Amascut),
        ] {
            if required(name)? != 0 {
                add(attr);
            }
        }
        if npc.category == vampire {
            add(Npc::Vampyre);
        }
        if id as i32 == corp {
            add(Npc::CorporealBeast);
        }
        if required("tormented_demon")? != 0 && required("td_shield_active")? != 1 {
            add(Npc::TormentedDemonUnshielded);
        }
        match npc.size {
            2 => add(Npc::Size2),
            3 => add(Npc::Size3),
            4 => add(Npc::Size4),
            5.. => add(Npc::Size5OrMore),
            _ => {}
        }
        match int("elemental_weakness_type")? {
            Some(0) => add(Npc::WindWeakness),
            Some(1) => add(Npc::WaterWeakness),
            Some(2) => add(Npc::EarthWeakness),
            Some(3) => add(Npc::FireWeakness),
            _ => {}
        }
        result.insert(
            u16::try_from(id).map_err(|_| format!("NPC id {id} exceeds protocol range"))?,
            Definition {
                xp_multiplier,
                defence,
                attributes,
                attack_rate: required("attackrate")?,
                retreat: required("retreat")?,
                aggression_range: npc.max_range as i32 + npc.attack_range as i32,
            },
        );
    }
    Ok(result)
}
