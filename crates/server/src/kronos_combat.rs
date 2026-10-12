//! Runtime NPC stat/XP/spawn/respawn bind from the pinned Kotlin content script.
use crate::kronos_json::{self, Value};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
pub const DEFAULT_DIRECTORY: &str = "C:/Projects/OSRSRSPS/source-reference/kronos-osrs-184-master/kronos-osrs-184-master/Kronos-master/kronos-server/data/npcs/combat";
#[derive(Clone, Debug, PartialEq)]
pub struct Info {
    pub ids: Vec<i32>,
    pub stats: [i32; 6],
    pub xp_modifier: f64,
    pub spawn_animation: i32,
    pub respawn_ticks: i32,
}
#[derive(Default)]
pub struct Catalog {
    pub by_id: BTreeMap<i32, Info>,
    pub failures: Vec<String>,
}
pub fn strip_comments(source: &str) -> String {
    let b = source.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    let mut in_string = false;
    let mut escaped = false;
    while i < b.len() {
        let ch = b[i];
        if in_string {
            out.push(ch);
            if escaped {
                escaped = false;
            } else if ch == b'\\' {
                escaped = true;
            } else if ch == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if ch == b'"' {
            in_string = true;
            out.push(ch);
            i += 1;
            continue;
        }
        if ch == b'#' || (ch == b'/' && b.get(i + 1) == Some(&b'/')) {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if ch == b'/' && b.get(i + 1) == Some(&b'*') {
            i += 2;
            while i + 1 < b.len() && &b[i..i + 2] != b"*/" {
                i += 1;
            }
            i = (i + 2).min(b.len());
            continue;
        }
        out.push(ch);
        i += 1;
    }
    String::from_utf8(out).expect("comment removal preserves UTF-8")
}
/// Preserves the reference regex behavior, including matches inside quoted text.
pub fn quote_bare_tokens(source: &str) -> String {
    let b = source.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        if b[i] == b':' {
            let start = i;
            i += 1;
            while b.get(i).is_some_and(u8::is_ascii_whitespace) {
                i += 1;
            }
            let token = i;
            if b.get(i)
                .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
            {
                i += 1;
                while b
                    .get(i)
                    .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
                {
                    i += 1;
                }
                out.extend(&b[start..token]);
                let text = &b[token..i];
                if matches!(text, b"true" | b"false" | b"null") {
                    out.extend(text);
                } else {
                    out.push(b'"');
                    out.extend(text);
                    out.push(b'"');
                }
                continue;
            }
            out.extend(&b[start..i]);
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8(out).expect("token quoting preserves UTF-8")
}
fn number(value: &Value) -> Result<f64, &'static str> {
    match value {
        Value::Number(s) | Value::String(s) => {
            s.trim().parse().map_err(|_| "Invalid combat number")
        }
        Value::Null => Ok(0.0),
        _ => Err("Invalid combat number"),
    }
}
fn integer(value: &Value) -> Result<i32, &'static str> {
    let n = number(value)?;
    if !n.is_finite() || n < i32::MIN as f64 || n > i32::MAX as f64 {
        return Err("Combat integer outside i32");
    }
    Ok(n as i32)
}
pub fn parse(source: &str) -> Result<Vec<Info>, &'static str> {
    let text = quote_bare_tokens(&strip_comments(source));
    if text.trim().is_empty() {
        return Ok(vec![]);
    }
    let Value::Array(values) = kronos_json::parse(text.trim())? else {
        return Err("Combat JSON must be a list");
    };
    let mut infos = vec![];
    for value in values {
        let Value::Object(fields) = value else {
            return Err("Combat list entry must be an object");
        };
        // Jackson checks known fields even when the runtime bind does not consume them.
        for (key, value) in &fields {
            match key.as_str() {
                "hitpoints" | "aggressive_level" | "max_damage" | "slayer_level" | "attack"
                | "strength" | "defence" | "ranged" | "magic" | "stab_attack" | "slash_attack"
                | "crush_attack" | "magic_attack" | "ranged_attack" | "stab_defence"
                | "slash_defence" | "crush_defence" | "magic_defence" | "ranged_defence"
                | "attack_ticks" | "death_ticks" | "respawn_ticks" | "spawn_animation"
                | "attack_animation" | "defend_animation" | "death_animation" | "attack_sound"
                | "defend_sound" | "death_sound" | "random_drop_count" => {
                    integer(value)?;
                }
                "slayer_xp" | "combat_xp_modifier" => {
                    number(value)?;
                }
                "attack_style" => match value {
                    Value::String(s)
                        if [
                            "STAB",
                            "SLASH",
                            "CRUSH",
                            "RANGED",
                            "MAGIC",
                            "MAGICAL_RANGED",
                            "MAGICAL_MELEE",
                            "DRAGONFIRE",
                            "CANNON",
                        ]
                        .contains(&s.as_str()) =>
                    {
                        ()
                    }
                    Value::Number(_) if (0..=8).contains(&integer(value)?) => (),
                    _ => return Err("Invalid NPC attack style"),
                },
                "poison_immunity" | "venom_immunity" | "local_loot" => match value {
                    Value::Bool(v) => {
                        let _ = *v;
                    }
                    Value::Null => (),
                    Value::String(v) if v == "true" || v == "false" => (),
                    Value::Number(_) => {
                        number(value)?;
                    }
                    _ => return Err("Invalid combat boolean"),
                },
                "slayer_tasks" => {
                    if !matches!(value, Value::Null | Value::Array(_)) {
                        return Err("Invalid slayer tasks");
                    }
                }
                _ => (),
            }
        }
        let get = |key, default| fields.get(key).map(integer).unwrap_or(Ok(default));
        let ids = match fields.get("ids") {
            None => vec![],
            Some(Value::Array(ids)) => ids.iter().map(integer).collect::<Result<_, _>>()?,
            _ => return Err("Invalid combat ids"),
        };
        infos.push(Info {
            ids,
            stats: [
                get("attack", 1)?,
                get("defence", 1)?,
                get("strength", 1)?,
                get("hitpoints", 1)?,
                get("ranged", 1)?,
                get("magic", 1)?,
            ],
            xp_modifier: fields
                .get("combat_xp_modifier")
                .map(number)
                .unwrap_or(Ok(1.0))?,
            spawn_animation: get("spawn_animation", -1)?,
            respawn_ticks: get("respawn_ticks", 0)?,
        });
    }
    Ok(infos)
}
impl Catalog {
    pub fn bindings(
        &self,
        sequences: &BTreeMap<u32, gielinor_cache::sequences::SequenceType>,
    ) -> BTreeMap<u16, gielinor_world::npc_bind::Binding> {
        self.by_id
            .iter()
            .filter_map(|(&id, info)| {
                let id = u16::try_from(id).ok()?;
                let animation = u16::try_from(info.spawn_animation)
                    .ok()
                    .filter(|id| sequences.contains_key(&(*id as u32)));
                Some((
                    id,
                    gielinor_world::npc_bind::Binding {
                        stats: info.stats,
                        xp_multiplier: (info.xp_modifier * 1000.0).round_ties_even() as i32,
                        spawn_animation: animation,
                        respawn_ticks: info.respawn_ticks,
                    },
                ))
            })
            .collect()
    }
    pub fn load(directory: &Path) -> Self {
        fn files(path: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                if ty.is_dir() {
                    files(&entry.path(), output)?;
                } else if ty.is_file() && entry.file_name().to_string_lossy().ends_with(".json") {
                    output.push(entry.path());
                }
            }
            Ok(())
        }
        let mut catalog = Self::default();
        let mut paths = vec![];
        if let Err(e) = files(directory, &mut paths) {
            catalog
                .failures
                .push(format!("{}: {e}", directory.display()));
            return catalog;
        }
        paths.sort_by_key(|p| p.file_name().unwrap().to_string_lossy().to_lowercase());
        for path in paths {
            let parsed = std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|text| parse(&text).map_err(str::to_owned));
            match parsed {
                Ok(infos) => {
                    for info in infos {
                        for &id in &info.ids {
                            catalog.by_id.insert(id, info.clone());
                        }
                    }
                }
                Err(e) => catalog.failures.push(format!("{}: {e}", path.display())),
            }
        }
        catalog
    }
}
