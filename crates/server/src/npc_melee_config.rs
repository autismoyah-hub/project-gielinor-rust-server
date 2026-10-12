use crate::startup::Names;
use gielinor_cache::{definitions::Param, game::Definitions, parameters::ParameterType};
use gielinor_world::{
    bank_actions::Binding,
    npc_melee::{Config, Definition},
};
use std::collections::{BTreeMap, BTreeSet};
pub fn resolve(
    names: &Names,
    defs: &Definitions,
    params: &BTreeMap<u32, ParameterType>,
) -> Result<Config, String> {
    let id = |group: &str, name: &str| names.id(group, name).map_err(|e| e.to_string());
    let bit = |name| -> Result<Binding, String> {
        let key = id("varbit", name)?;
        let bit = *defs
            .varbits
            .get(&key)
            .ok_or_else(|| format!("Missing NPC defence varbit {name}"))?;
        Ok(Binding {
            bit,
            base: *defs
                .varps
                .get(&(bit.base as u32))
                .ok_or("Missing NPC defence varp")?,
        })
    };
    let varp = |name| {
        defs.varps
            .get(&id("varp", name)?)
            .copied()
            .ok_or_else(|| format!("Missing NPC defence varp {name}"))
    };
    let object = |name: &str| id("obj", name).map(|v| v as i32);
    let prayers = [
        "prayer_piety",
        "prayer_rigour",
        "prayer_augury",
        "prayer_chivalry",
        "prayer_steelskin",
        "prayer_rockskin",
        "prayer_thickskin",
        "prayer_hawkeye",
        "prayer_deadeye_unlocked",
        "prayer_mysticmight",
        "prayer_mystic_vigour_unlocked",
    ]
    .map(bit)
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .try_into()
    .unwrap();
    let mut torag: [BTreeSet<i32>; 4] = Default::default();
    for (set, part) in torag.iter_mut().zip(["head", "body", "legs", "weapon"]) {
        for charge in [100, 75, 50, 25] {
            set.insert(object(&format!("barrows_torag_{part}_{charge}"))?);
        }
    }
    let param = |values: &BTreeMap<u32, Param>, name| -> Result<Option<i32>, String> {
        let key = id("param", name)?;
        match values.get(&key) {
            Some(Param::Integer(v)) => Ok(Some(*v)),
            None => Ok(params
                .get(&key)
                .ok_or_else(|| format!("Missing NPC melee param {name}"))?
                .default_int),
            _ => Err(format!("Invalid NPC melee param {name}")),
        }
    };
    let mut definitions = BTreeMap::new();
    let stab = id("category", "attacktype_stab")? as i32;
    let slash = id("category", "attacktype_slash")? as i32;
    for (&npc_id, npc) in &defs.npcs {
        let required = |name| {
            param(&npc.params, name)?.ok_or_else(|| format!("Missing NPC melee value {name}"))
        };
        let category = param(&npc.params, "npc_attack_type")?;
        let animation = required("attack_anim")?;
        if animation < 0 {
            continue;
        }
        definitions.insert(
            npc_id as u16,
            Definition {
                retreat_range: npc.max_range as i32,
                give_chase: npc.give_chase,
                attack_type: if category == Some(stab) {
                    1
                } else if category == Some(slash) {
                    2
                } else {
                    3
                },
                attack_bonus: required("attack_melee")?,
                strength_bonus: required("melee_strength")?,
                animation: animation as u16,
                sound: param(&npc.params, "attack_sound")?
                    .filter(|v| *v >= 0)
                    .map(|v| v as u16),
            },
        );
    }
    let bulwark = id("category", "dinhs_bulwark")? as i32;
    let mut item_defend = BTreeMap::new();
    for (&item_id, item) in &defs.items {
        if let Some(seq) = param(&item.params, "defend_anim")?.filter(|v| *v >= 0) {
            item_defend.insert(item_id as i32, seq as u16);
        }
    }
    let lit = id("hitmark", "regular_damage_lit")? as u16;
    let player_mark = gielinor_world::hit_display::Builder {
        target: lit,
        source: lit,
        public: Some(id("hitmark", "regular_damage_tint")? as u16),
        zero_lit: Some(id("hitmark", "zero_damage_lit")? as u16),
        zero_tint: Some(id("hitmark", "zero_damage_tint")? as u16),
        max_lit: Some(id("hitmark", "regular_damage_max")? as u16),
        target_threshold: i32::MAX,
        source_threshold: i32::MAX,
        delay: 0,
    };
    Ok(Config {
        player_mark,
        definitions,
        prayers,
        torag,
        damned: object("damned_amulet")?,
        elysian: object("elysian")?,
        justiciar: [
            object("justiciar_faceguard")?,
            object("justiciar_chestguard")?,
            object("justiciar_leg_guards")?,
        ],
        bulwarks: defs
            .items
            .iter()
            .filter(|(_, item)| item.category == bulwark)
            .map(|(&id, _)| id as i32)
            .collect(),
        passive_delay: varp("dinhs_passive_delay")?,
        defence_stat: id("stat", "defence")? as u8,
        health_stat: id("stat", "hitpoints")? as u8,
        unarmed_defend: id("seq", "human_unarmedblock")? as u16,
        item_defend,
    })
}
