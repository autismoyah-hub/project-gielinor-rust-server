//! Parameters used by the generic ranged branch of the original PvNCombat.
use crate::startup::Names;
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, projectiles::ProjectileType,
};
use gielinor_world::{
    ranged_ammo::{Categories, Kind, Weapon},
    ranged_weapon::{Catalog, Item},
};
use std::collections::BTreeMap;
pub fn resolve(
    names: &Names,
    parameters: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
    projectiles: &BTreeMap<u32, ProjectileType>,
) -> Result<Catalog, String> {
    let id = |group: &str, name: &str| {
        names
            .group(group)
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing ranged weapon {group} {name}"))
    };
    let bow = id("category", "bow")?;
    let crossbow = id("category", "crossbow")?;
    let ballista = id("category", "ballista")?;
    let charged = id("category", "chargebow")?;
    let thrown = id("category", "throwing_weapon")?;
    let categories = Categories {
        arrows: id("category", "arrows")?,
        dragon_arrow: id("category", "dragon_arrow")?,
        bolts: id("category", "crossbow_bolt")?,
        javelins: id("category", "javelin")?,
    };
    let mut items = BTreeMap::new();
    for (&item_id, item) in &defs.items {
        let param = |name: &str| -> Result<Option<i32>, String> {
            let key = id("param", name)? as u32;
            match item.params.get(&key) {
                Some(Param::Integer(v)) => Ok(Some(*v)),
                Some(_) => Err(format!("Invalid ranged weapon {item_id} parameter {name}")),
                None => Ok(parameters
                    .get(&key)
                    .ok_or_else(|| format!("Missing ranged parameter definition {name}"))?
                    .default_int),
            }
        };
        let required =
            |name: &str| param(name)?.ok_or_else(|| format!("Missing ranged default {name}"));
        let typed =
            |name: &str| -> Result<Option<i32>, String> { Ok(param(name)?.filter(|v| *v != -1)) };
        let effect = |name: &str| -> Result<Option<u16>, String> {
            typed(name)?
                .map(|v| {
                    u16::try_from(v).map_err(|_| format!("Ranged effect {name} out of range: {v}"))
                })
                .transpose()
        };
        items.insert(
            item_id as i32,
            Item {
                category: item.category,
                weapon: Weapon {
                    kind: if item.category == crossbow {
                        Kind::Crossbow
                    } else if item.category == bow {
                        Kind::Bow
                    } else if item.category == ballista {
                        Kind::Ballista
                    } else {
                        Kind::Other
                    },
                    required: typed("required_ammo")?,
                    level: required("levelrequire")?,
                    bone: required("bone_weapon")?,
                },
                charged: item.category == charged,
                thrown: item.category == thrown,
                projectile: typed("proj_type")?,
                travel: effect("proj_travel")?,
                launch: effect("proj_launch")?,
                animation: effect("attack_anim_stance1")?,
                sound: effect("attack_sound_stance1")?,
                recovery: param("ammo_recovery_rate")?,
                metallic: required("metallic_interference")? == 1,
            },
        );
    }
    Ok(Catalog {
        items,
        categories,
        javelin: categories.javelins,
        ballista_spot: u16::try_from(id("spotanim", "ballista_special")?)
            .map_err(|_| "Ballista spot animation out of range")?,
        styles: projectiles
            .iter()
            .map(|(&id, value)| (id as i32, value.style))
            .collect(),
    })
}
