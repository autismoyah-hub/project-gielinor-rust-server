//! Independent Man/Woman pickpocket registration; base policy only.
use super::{Definitions, Symbols};
use gielinor_world::thieving::{Config, Target};
pub fn citizen_pickpockets(
    npcs: &Symbols,
    sequences: &Symbols,
    spots: &Symbols,
    sounds: &Symbols,
    stats: &Symbols,
    defs: &Definitions,
) -> Result<Config, String> {
    let short = |symbols: &Symbols, name: &str, id: i32| -> Result<u16, String> {
        if symbols.get(name) != Some(&id) {
            return Err(format!("Missing/aliased pickpocket binding {name}"));
        }
        u16::try_from(id).map_err(|_| "Invalid pickpocket binding".into())
    };
    let skill = short(stats, "thieving", 17)? as u8;
    let stat = defs
        .stats
        .get(&(skill as u32))
        .ok_or("Missing Thieving stat")?;
    if stat.name != "Thieving" || stat.unreleased || stat.min_level != 1 || stat.max_level != 99 {
        return Err("Invalid Thieving stat".into());
    }
    let mut targets = std::collections::BTreeMap::new();
    for (name, id, owner) in [
        ("man", 3106, "man"),
        ("man2", 3107, "man"),
        ("man3", 3108, "man"),
        ("woman", 3111, "woman"),
        ("woman2", 3112, "woman"),
        ("woman3", 3113, "woman"),
    ] {
        let id = short(npcs, name, id)?;
        let d = defs
            .npcs
            .get(&(id as u32))
            .ok_or("Missing pickpocket NPC")?;
        if d.name != if owner == "man" { "Man" } else { "Woman" }
            || d.size != 1
            || !d.active
            || !d.multi.is_empty()
            || d.multi_default.is_some()
            || d.ops[2].as_deref() != Some("Pickpocket")
        {
            return Err(format!("Invalid pickpocket NPC {id}"));
        }
        targets.insert(id, Target { owner });
    }
    Ok(Config {
        targets,
        skill,
        pouch: 22521,
        animation: short(sequences, "human_pickpocket", 881)?,
        block: short(sequences, "human_unarmedblock", 424)?,
        spot: short(spots, "stunned_thieving", 245)?,
        sound: short(sounds, "pick", 2581)?,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn six_exact_third_operations_and_stat_are_required() {
        let mut defs = Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::new(),
            stats: BTreeMap::new(),
        };
        defs.stats.insert(
            17,
            gielinor_cache::definitions::StatType::decode(b"\x01Thieving\0\0").unwrap(),
        );
        let mut names = Symbols::new();
        for (name, id, label) in [
            ("man", 3106, "Man"),
            ("man2", 3107, "Man"),
            ("man3", 3108, "Man"),
            ("woman", 3111, "Woman"),
            ("woman2", 3112, "Woman"),
            ("woman3", 3113, "Woman"),
        ] {
            names.insert(name.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(label.as_bytes());
            raw.extend_from_slice(b"\0\x20Pickpocket\0\0");
            defs.npcs.insert(
                id as u32,
                gielinor_cache::npcs::NpcType::decode(&raw).unwrap(),
            );
        }
        let seq = [
            ("human_pickpocket".into(), 881),
            ("human_unarmedblock".into(), 424),
        ]
        .into();
        let spot = [("stunned_thieving".into(), 245)].into();
        let sound = [("pick".into(), 2581)].into();
        let stat = [("thieving".into(), 17)].into();
        let c = citizen_pickpockets(&names, &seq, &spot, &sound, &stat, &defs).unwrap();
        assert_eq!(c.targets.len(), 6);
        assert_eq!(c.targets[&3113].owner, "woman");
        defs.npcs.get_mut(&3113).unwrap().ops[2] = Some("Talk-to".into());
        assert!(citizen_pickpockets(&names, &seq, &spot, &sound, &stat, &defs).is_err());
    }
}
