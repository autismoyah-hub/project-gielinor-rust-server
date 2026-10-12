use crate::startup::Names;
use gielinor_cache::{definitions::Param, game::Definitions, parameters::ParameterType};
use gielinor_world::{bank_actions::Binding, bonuses::ItemBonuses, melee, variables::Varp};
use std::collections::BTreeMap;
pub fn resolve(
    names: &Names,
    parameters: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
    bonuses: BTreeMap<i32, ItemBonuses>,
) -> Result<melee::Config, String> {
    let id = |group: &str, name: &str| {
        names
            .group(group)
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing melee {group} {name}"))
    };
    let varp = |name: &str| -> Result<Varp, String> {
        let key = id("varp", name)? as u32;
        defs.varps
            .get(&key)
            .copied()
            .ok_or_else(|| format!("Missing melee varp {name}"))
    };
    let bit = |name: &str| -> Result<Binding, String> {
        let key = id("varbit", name)? as u32;
        let bit = *defs
            .varbits
            .get(&key)
            .ok_or_else(|| format!("Missing melee varbit {name}"))?;
        Ok(Binding {
            bit,
            base: *defs
                .varps
                .get(&(bit.base as u32))
                .ok_or_else(|| format!("Missing melee varbit base {name}"))?,
        })
    };
    let attack_prayers = [
        "prayer_clarityofthought",
        "prayer_improvedreflexes",
        "prayer_incrediblereflexes",
        "prayer_chivalry",
        "prayer_piety",
    ]
    .map(bit)
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .try_into()
    .unwrap();
    let strength_prayers = [
        "prayer_burstofstrength",
        "prayer_superhumanstrength",
        "prayer_ultimatestrength",
        "prayer_chivalry",
        "prayer_piety",
    ]
    .map(bit)
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .try_into()
    .unwrap();
    let seq = |name: &str| {
        u16::try_from(id("seq", name)?).map_err(|_| format!("Invalid melee sequence {name}"))
    };
    let sound = |name: &str| {
        u16::try_from(id("synth", name)?).map_err(|_| format!("Invalid melee sound {name}"))
    };
    let punch = (seq("human_unarmedpunch")?, sound("human_unarmedpunch")?);
    let kick = (seq("human_unarmedkick")?, sound("human_unarmedkick")?);
    let unarmed_fx = [punch, kick, punch, punch];
    let mut weapon_fx = BTreeMap::new();
    for (&item_id, item) in &defs.items {
        let mut fx = unarmed_fx;
        for (index, entry) in fx.iter_mut().enumerate() {
            let get = |name: &str| -> Result<Option<u16>, String> {
                let key = id("param", name)? as u32;
                let value = match item.params.get(&key) {
                    Some(Param::Integer(v)) => Some(*v),
                    Some(_) => return Err(format!("Invalid melee effect {name}")),
                    None => {
                        parameters
                            .get(&key)
                            .ok_or_else(|| format!("Missing melee effect default {name}"))?
                            .default_int
                    }
                };
                value
                    .filter(|v| *v != -1)
                    .map(|v| u16::try_from(v).map_err(|_| format!("Melee effect out of range {v}")))
                    .transpose()
            };
            if let Some(v) = get(&format!("attack_anim_stance{}", index + 1))? {
                entry.0 = v;
            }
            if let Some(v) = get(&format!("attack_sound_stance{}", index + 1))? {
                entry.1 = v;
            }
        }
        weapon_fx.insert(item_id as i32, fx);
    }
    let mut npc_marks = BTreeMap::new();
    let mut npc_defend = BTreeMap::new();
    let mut npc_defend_sounds = BTreeMap::new();
    let regular = [
        id("hitmark", "regular_damage_lit")?,
        id("hitmark", "regular_damage_tint")?,
        id("hitmark", "regular_damage_max")?,
    ];
    for (&npc_id, npc) in &defs.npcs {
        let key = id("param", "defend_sound")? as u32;
        let sound = match npc.params.get(&key) {
            Some(Param::Integer(v)) => Some(*v),
            None => parameters.get(&key).and_then(|p| p.default_int),
            _ => return Err("Invalid NPC defend sound".into()),
        };
        if let Some(sound) = sound.filter(|v| *v >= 0) {
            npc_defend_sounds.insert(
                npc_id as u16,
                u16::try_from(sound).map_err(|_| "Invalid NPC defend sound")?,
            );
        }
        let key = id("param", "defend_anim")? as u32;
        let sequence = match npc.params.get(&key) {
            Some(Param::Integer(v)) => Some(*v),
            None => parameters.get(&key).and_then(|p| p.default_int),
            _ => return Err("Invalid NPC defend animation".into()),
        };
        if let Some(sequence) = sequence.filter(|v| *v >= 0) {
            npc_defend.insert(
                npc_id as u16,
                u16::try_from(sequence).map_err(|_| "Invalid NPC defend sequence")?,
            );
        }
        let mut group = [0i32; 3];
        for (value, name) in group
            .iter_mut()
            .zip(["hitmark_lit", "hitmark_tint", "hitmark_max"])
        {
            let key = id("param", name)? as u32;
            *value = match npc.params.get(&key) {
                Some(Param::Integer(v)) => *v,
                None => parameters
                    .get(&key)
                    .and_then(|p| p.default_int)
                    .ok_or_else(|| format!("Missing NPC hitmark default {name}"))?,
                _ => return Err(format!("Invalid NPC hitmark {name}")),
            };
        }
        let is_regular = group == regular;
        let lit = u16::try_from(group[0]).map_err(|_| "Invalid NPC hitmark lit")?;
        npc_marks.insert(
            npc_id as u16,
            gielinor_world::hit_display::Builder {
                target: lit,
                source: lit,
                public: Some(if group[1] < 0 { lit } else { group[1] as u16 }),
                zero_lit: if is_regular {
                    Some(id("hitmark", "zero_damage_lit")? as u16)
                } else {
                    None
                },
                zero_tint: if is_regular {
                    Some(id("hitmark", "zero_damage_tint")? as u16)
                } else {
                    None
                },
                max_lit: (group[2] >= 0).then_some(group[2] as u16),
                target_threshold: i32::MAX,
                source_threshold: i32::MAX,
                delay: 0,
            },
        );
    }
    Ok(melee::Config {
        npc_marks,
        npc_defend,
        npc_defend_sounds,
        hit_threshold: bit("settings_hitsplat_threshold")?,
        equipment: crate::melee_equipment_config::resolve(
            names.group("obj"),
            names.group("param"),
            names.group("category"),
            parameters,
            defs,
        )?,
        bonuses,
        stance: varp("com_mode")?,
        attack_prayers,
        strength_prayers,
        souls: varp("soulreaper_stacks")?,
        invocation: bit("toa_client_raid_level")?,
        max_hit: varp("com_maxhit")?,
        stats: [
            id("stat", "attack")? as u8,
            id("stat", "strength")? as u8,
            id("stat", "hitpoints")? as u8,
        ],
        unarmed_fx,
        weapon_fx,
    })
}
