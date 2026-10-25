//! NPC combat stat bindings from the rev-239 server cache. OpenRune's `Npc` takes its six
//! levels from `type.attack..magic` (cache opcodes 74-79) and respawns after
//! `type.respawnRate` (packed server opcode204, default100). The producing pack's
//! historical provenance is not established. Ordinary respawn profiles may
//! override only the effective rate; the optional Kronos override remains last.
//! A Kronos-format directory named by `OSRS_NPC_COMBAT_DIRECTORY` may override them.
//!
//! Kronos-only fields with no cache/OpenRune source keep explicit defaults:
//! - `spawn_animation`: [`SPAWN_ANIMATION`] (none), as Kronos' own `-1` default.
//! - `combat_xp_modifier`: not bound ([`XP_MULTIPLIER`]); the multiplier configured from
//!   the OpenRune `npc_com_xp_multiplier` param in `npc_combat_config` applies.
use crate::kronos_combat;
use gielinor_cache::{npcs::NpcType, sequences::SequenceType};
use gielinor_world::npc_bind::Binding;
use std::{collections::BTreeMap, path::PathBuf};

pub const OVERRIDE_VARIABLE: &str = "OSRS_NPC_COMBAT_DIRECTORY";
pub const SPAWN_ANIMATION: Option<u16> = None;
pub const XP_MULTIPLIER: Option<i32> = None;

pub fn from_cache(npcs: &BTreeMap<u32, NpcType>) -> BTreeMap<u16, Binding> {
    npcs.iter()
        .filter_map(|(&id, ty)| {
            Some((
                u16::try_from(id).ok()?,
                Binding {
                    stats: ty.stats,
                    xp_multiplier: XP_MULTIPLIER,
                    spawn_animation: SPAWN_ANIMATION,
                    respawn_ticks: ty.respawn_rate as i32,
                },
            ))
        })
        .collect()
}

#[derive(Debug, PartialEq)]
pub enum Override {
    None,
    Missing(PathBuf),
    Loaded {
        directory: PathBuf,
        ids: usize,
        failures: Vec<String>,
    },
}

pub struct Loaded {
    pub bindings: BTreeMap<u16, Binding>,
    pub cache_ids: usize,
    pub ordinary_respawn_ids: usize,
    pub ordinary_hitpoints_ids: usize,
    pub source: Override,
}

/// Cache bindings, then the optional override directory on top.
pub fn load(
    npcs: &BTreeMap<u32, NpcType>,
    sequences: &BTreeMap<u32, SequenceType>,
    override_directory: Option<PathBuf>,
) -> Loaded {
    load_bindings(from_cache(npcs), sequences, override_directory, 0, 0)
}

/// Raw cache -> respawn-only rows -> explicit optional whole-binding overrides.
pub fn load_with_respawns(
    npcs: &BTreeMap<u32, NpcType>,
    sequences: &BTreeMap<u32, SequenceType>,
    respawns: &BTreeMap<u16, i32>,
    override_directory: Option<PathBuf>,
) -> Result<Loaded, String> {
    load_with_ordinary(npcs, sequences, respawns, &BTreeMap::new(), override_directory)
}

/// Raw cache -> respawn-only profiles -> HP-only corrections -> Kronos last.
pub fn load_with_ordinary(
    npcs: &BTreeMap<u32, NpcType>,
    sequences: &BTreeMap<u32, SequenceType>,
    respawns: &BTreeMap<u16, i32>,
    hitpoints: &BTreeMap<u16, u16>,
    override_directory: Option<PathBuf>,
) -> Result<Loaded, String> {
    let mut bindings = from_cache(npcs);
    for (&id, &ticks) in respawns {
        if ticks <= 0 { return Err(format!("ordinary_respawn: invalid rate for {id}")); }
        let binding = bindings.get_mut(&id)
            .ok_or_else(|| format!("ordinary_respawn: missing cache binding {id}"))?;
        binding.respawn_ticks = ticks;
    }
    for (&id, &hp) in hitpoints {
        if hp == 0 { return Err(format!("ordinary_hitpoints: invalid HP for {id}")); }
        let binding = bindings.get_mut(&id)
            .ok_or_else(|| format!("ordinary_hitpoints: missing cache binding {id}"))?;
        binding.stats[3] = i32::from(hp);
    }
    Ok(load_bindings(bindings, sequences, override_directory, respawns.len(), hitpoints.len()))
}

fn load_bindings(
    mut bindings: BTreeMap<u16, Binding>,
    sequences: &BTreeMap<u32, SequenceType>,
    override_directory: Option<PathBuf>,
    ordinary_respawn_ids: usize,
    ordinary_hitpoints_ids: usize,
) -> Loaded {
    let cache_ids = bindings.len();
    let source = match override_directory {
        None => Override::None,
        Some(directory) if !directory.is_dir() => Override::Missing(directory),
        Some(directory) => {
            let catalog = kronos_combat::Catalog::load(&directory);
            let overrides = catalog.bindings(sequences);
            let ids = overrides.len();
            bindings.extend(overrides);
            Override::Loaded {
                directory,
                ids,
                failures: catalog.failures,
            }
        }
    };
    Loaded {
        bindings,
        cache_ids,
        ordinary_respawn_ids,
        ordinary_hitpoints_ids,
        source,
    }
}

/// Startup's source: the cache plus `OSRS_NPC_COMBAT_DIRECTORY` when set.
pub fn load_from_environment(
    npcs: &BTreeMap<u32, NpcType>,
    sequences: &BTreeMap<u32, SequenceType>,
) -> Loaded {
    load(
        npcs,
        sequences,
        std::env::var_os(OVERRIDE_VARIABLE).map(PathBuf::from),
    )
}

impl Loaded {
    /// The single startup log line.
    pub fn summary(&self) -> String {
        let mut line = format!(
            "NPC combat data: {} ids from cache (stats opcodes 74-79, respawn opcode 204)",
            self.cache_ids
        );
        if self.ordinary_respawn_ids != 0 {
            line.push_str(&format!("; {} ordinary respawn-only profiles before explicit overrides", self.ordinary_respawn_ids));
        }
        if self.ordinary_hitpoints_ids != 0 {
            line.push_str(&format!("; {} ordinary HP-only corrections before explicit overrides", self.ordinary_hitpoints_ids));
        }
        match &self.source {
            Override::None => {}
            Override::Missing(directory) => line.push_str(&format!(
                "; {OVERRIDE_VARIABLE}={} does not exist, override ignored",
                directory.display()
            )),
            Override::Loaded {
                directory,
                ids,
                failures,
            } => {
                line.push_str(&format!(
                    "; {ids} ids overridden from {OVERRIDE_VARIABLE}={}",
                    directory.display()
                ));
                if let Some(first) = failures.first() {
                    line.push_str(&format!(
                        " ({} file failures, first: {first})",
                        failures.len()
                    ));
                }
            }
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn npc(stats: [u16; 6], respawn: Option<u16>) -> NpcType {
        let mut bytes = vec![];
        for (code, stat) in (74u8..).zip(stats) {
            bytes.push(code);
            bytes.extend(stat.to_be_bytes());
        }
        if let Some(respawn) = respawn {
            bytes.push(204);
            bytes.extend(respawn.to_be_bytes());
        }
        bytes.push(0);
        NpcType::decode(&bytes).unwrap()
    }
    #[test]
    fn cache_types_bind_their_stats_and_respawn_rate() {
        let npcs = BTreeMap::from([
            (2790, npc([1, 1, 1, 8, 1, 1], Some(45))),
            (3254, npc([19, 14, 18, 22, 1, 1], None)),
            (70_000, npc([1, 1, 1, 1, 1, 1], None)),
        ]);
        let bindings = from_cache(&npcs);
        assert_eq!(bindings.len(), 2, "ids above u16 cannot spawn");
        assert_eq!(
            bindings[&2790],
            Binding {
                stats: [1, 1, 1, 8, 1, 1],
                xp_multiplier: None,
                spawn_animation: None,
                respawn_ticks: 45,
            }
        );
        assert_eq!(bindings[&3254].stats, [19, 14, 18, 22, 1, 1]);
        assert_eq!(bindings[&3254].respawn_ticks, 100, "OpenRune respawnRate default");
    }
    #[test]
    fn missing_override_is_reported_once_with_its_path() {
        let npcs = BTreeMap::from([(1173, npc([1, 1, 1, 3, 1, 1], Some(25)))]);
        let missing = std::env::temp_dir().join("gielinor-no-such-npc-combat-directory");
        let loaded = load(&npcs, &BTreeMap::new(), Some(missing.clone()));
        assert_eq!(loaded.source, Override::Missing(missing.clone()));
        assert_eq!(loaded.bindings[&1173].stats[3], 3);
        assert_eq!(
            loaded.summary(),
            format!(
                "NPC combat data: 1 ids from cache (stats opcodes 74-79, respawn opcode 204); \
                 OSRS_NPC_COMBAT_DIRECTORY={} does not exist, override ignored",
                missing.display()
            )
        );
        let plain = load(&npcs, &BTreeMap::new(), None);
        assert_eq!(
            plain.summary(),
            "NPC combat data: 1 ids from cache (stats opcodes 74-79, respawn opcode 204)"
        );
    }
    #[test]
    fn override_directory_replaces_cache_bindings() {
        let directory = std::env::temp_dir().join(format!(
            "gielinor-npc-combat-override-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join("cow.json"),
            r#"[{"ids": [2790], "hitpoints": 9, "combat_xp_modifier": 1.5, "respawn_ticks": 7}]"#,
        )
        .unwrap();
        std::fs::write(directory.join("broken.json"), "{").unwrap();
        let npcs = BTreeMap::from([
            (2790, npc([1, 1, 1, 8, 1, 1], Some(45))),
            (1173, npc([1, 1, 1, 3, 1, 1], Some(25))),
        ]);
        let loaded = load(&npcs, &BTreeMap::new(), Some(directory.clone()));
        std::fs::remove_dir_all(&directory).unwrap();
        assert_eq!(loaded.bindings[&2790].stats, [1, 1, 1, 9, 1, 1]);
        assert_eq!(loaded.bindings[&2790].xp_multiplier, Some(1500));
        assert_eq!(loaded.bindings[&2790].respawn_ticks, 7);
        assert_eq!(loaded.bindings[&1173].respawn_ticks, 25);
        let Override::Loaded { ids, failures, .. } = &loaded.source else {
            panic!("override not loaded");
        };
        assert_eq!((*ids, failures.len()), (1, 1));
        assert!(loaded.summary().contains("1 ids overridden from"));
        assert!(loaded.summary().contains("(1 file failures, first: "));
    }
}
