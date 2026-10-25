//! Revision-239 registration for OpenRune Cleave. Wiki Dragon longsword
//! rev15183370 supports cost/max hit/Slash defence; the additional accuracy
//! multiplier is explicitly OpenRune-backed, not independently Wiki-attested.
use crate::startup::Names;
use gielinor_cache::{components::ComponentType, enums::{EnumType, Value}, game::Definitions};
use gielinor_world::{special_attacks::{self, Melee}, variables::Transmit};
use std::collections::BTreeMap;

pub struct Config {
    pub world: special_attacks::Config,
    pub buttons: Vec<u32>,
}

pub fn resolve(names: &Names, defs: &Definitions, tables: &BTreeMap<u32, EnumType>, components: &BTreeMap<u32, ComponentType>) -> Result<Config, String> {
    let id = |group: &str, name: &str, expected: u32| -> Result<u32, String> {
        let found = names.id(group, name).map_err(|e| e.to_string())?;
        if found != expected { return Err(format!("special attacks: {group}.{name}={found}, expected {expected}")); }
        Ok(found)
    };
    let weapon = id("obj", "dragon_longsword", 1305)? as i32;
    if defs.items.get(&(weapon as u32)).is_none_or(|d| d.name != "Dragon longsword" || d.cert.template != 0) {
        return Err("special attacks: unattested Dragon longsword definition".into());
    }
    let enum_id = id("enum", "sa_energy_requirements", 906)?;
    let cost = match tables.get(&enum_id).and_then(|e| e.values.get(&weapon)) {
        Some(Value::Int(250)) => 250,
        other => return Err(format!("special attacks: DLS cache cost {other:?}, expected Wiki25%=250")),
    };
    let varp = |name: &str, expected: u32| {
        let key = id("varp", name, expected)?;
        defs.varps.get(&key).copied().ok_or_else(|| format!("special attacks: missing {name}"))
    };
    let energy = varp("sa_energy", 300)?;
    let armed = varp("sa_attack", 301)?;
    if !energy.permanent || energy.transmit == Transmit::Never || armed.transmit == Transmit::Never {
        return Err("special attacks: energy must persist and both varps must transmit".into());
    }
    let world = special_attacks::Config {
        energy, armed,
        melee: [(weapon, Melee {
            cost, accuracy: 1.25, max_hit: 1.25, block_type: 2,
            animation: id("seq", "cleave", 1058)? as u16,
            spot: id("spotanim", "sp_attack_cleave_spotanim", 248)? as u16,
            spot_slot: 1, spot_height: 96,
        })].into(),
    };
    world.validate().map_err(str::to_string)?;
    // The rev239 symbol names the bar container, not its operation-bearing
    // child. Attest the child's parent, cached Op1 and exact menu operation;
    // never translate arbitrary descendants or assume symbol + 1 is a button.
    let bar = id("component", "combat_interface:special_attack", (593 << 16) | 38)?;
    if components.get(&bar).is_none_or(|c| !c.v3 || c.kind != 0) {
        return Err("special attacks: missing combat bar container".into());
    }
    let controls: Vec<_> = components.iter().filter_map(|(&key, c)| {
        (c.v3 && c.layer == Some(bar) && c.events & 2 != 0
            && c.operations.first().is_some_and(|op| op == "Use <col=00ff00>Special Attack</col>"))
            .then_some(key)
    }).collect();
    if controls != [(593 << 16) | 39] {
        return Err(format!("special attacks: unattested operative combat bar child {controls:?}"));
    }
    // This symbol names the orb sprite. Its parent's sibling control receives
    // the Use/Special Attack action from cache scripts2069 ->2792 (arg5).
    let orb_sprite = id("component", "orbs:specbutton", (160 << 16) | 35)?;
    let orb_parent = (160 << 16) | 34;
    if components.get(&orb_sprite).is_none_or(|c| !c.v3 || c.kind != 5 || c.layer != Some(orb_parent))
        || components.get(&orb_parent).is_none_or(|c| !c.v3 || c.kind != 0)
    {
        return Err("special attacks: unattested orb sprite/container".into());
    }
    let orb_controls: Vec<_> = components.iter().filter_map(|(&key, c)| {
        (c.v3 && c.kind == 0 && c.layer == Some(orb_parent) && c.events & 2 != 0
            && c.operations.first().is_some_and(|op| op == "*"))
            .then_some(key)
    }).collect();
    if orb_controls != [(160 << 16) | 36] {
        return Err(format!("special attacks: unattested operative orb child {orb_controls:?}"));
    }
    let buttons = vec![controls[0], orb_controls[0]];
    Ok(Config { world, buttons })
}
