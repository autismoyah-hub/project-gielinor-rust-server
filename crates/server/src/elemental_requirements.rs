//! Requirement subset of the existing Kotlin KronosElementalCatalog loader.
//! Only level and rune costs override casts; animation, XP and damage remain
//! those of ElementalSpells and MagicSpellRegistry.
use gielinor_cache::symbols::Symbols;
use gielinor_world::{runes::Requirement, spell_cast::Definition};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Override {
    pub level: i32,
    pub runes: Vec<RequirementValue>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequirementValue {
    pub item: i32,
    pub count: i32,
}
#[derive(Default)]
pub struct Catalog {
    pub spells: BTreeMap<String, Override>,
    pub failures: Vec<String>,
}
fn key(name: &str) -> Option<String> {
    for element in ["Wind", "Water", "Earth", "Fire"] {
        for tier in ["Strike", "Bolt", "Blast", "Wave", "Surge"] {
            if name == format!("{element}{tier}") {
                return Some(format!(
                    "{}_{}",
                    element.to_lowercase(),
                    tier.to_lowercase()
                ));
            }
        }
    }
    None
}
/// Kotlin's first balanced call search, including text in comments.
fn args<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let start = text.find(&format!("{name}("))? + name.len() + 1;
    let mut depth = 1;
    for (i, b) in text.as_bytes()[start..].iter().enumerate() {
        if *b == b'(' {
            depth += 1;
        } else if *b == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(&text[start..start + i]);
            }
        }
    }
    None
}
fn number<'a>(text: &'a str, name: &str, decimal: bool) -> Option<&'a str> {
    let prefix = format!("{name}(");
    text.match_indices(&prefix).find_map(|(start, _)| {
        let rest = &text[start + prefix.len()..];
        let end = rest.find(')')?;
        let n = &rest[..end];
        let mut parts = n.split('.');
        let whole = parts.next()?;
        if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if let Some(fraction) = parts.next() {
            if !decimal
                || fraction.is_empty()
                || !fraction.bytes().all(|b| b.is_ascii_digit())
                || parts.next().is_some()
            {
                return None;
            }
        }
        Some(n)
    })
}
pub fn modern_names(book: &str) -> Vec<String> {
    let Some(start) = book.find("MODERN(") else {
        return vec![];
    };
    let Some(end) = book.find("ANCIENT(").filter(|&end| end >= start) else {
        return vec![];
    };
    let body = &book[start..end];
    let mut result = vec![];
    for (i, _) in body.match_indices("new") {
        let rest = &body[i + 3..];
        if !rest.starts_with(char::is_whitespace) {
            continue;
        }
        let rest = rest.trim_start();
        let size = rest
            .bytes()
            .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
            .count();
        let name = &rest[..size];
        if key(name).is_some() && rest[size..].trim_start().starts_with('(') {
            result.push(name.to_owned());
        }
    }
    result
}
pub fn parse(
    book: &str,
    runes: &str,
    bodies: &BTreeMap<String, String>,
    symbols: &Symbols,
) -> Catalog {
    let mut result = Catalog::default();
    let mut known = BTreeSet::new();
    match (runes.find("enum Rune"), runes.find("private int id")) {
        (Some(start), Some(end)) if end >= start => {
            for line in runes[start..end].lines() {
                let line = line.trim_start();
                let len = line
                    .bytes()
                    .take_while(|b| b.is_ascii_alphanumeric() || *b == b'_')
                    .count();
                let name = &line[..len];
                if name.is_empty()
                    || name == "VALUES"
                    || !name.as_bytes()[0].is_ascii_uppercase()
                    || !name
                        .bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
                {
                    continue;
                }
                let mut tail = line[len..].trim_start();
                if tail.starts_with('(') {
                    let Some(end) = tail.find(')') else { continue };
                    tail = tail[end + 1..].trim_start();
                }
                if !tail.starts_with([',', ';']) {
                    continue;
                }
                known.insert(name.to_owned());
                if !symbols.contains_key(&format!("{}rune", name.to_lowercase())) {
                    result
                        .failures
                        .push(format!("{name} has no {}rune symbol", name.to_lowercase()));
                }
            }
        }
        _ => result.failures.push("Rune.java has no enum".into()),
    }
    let names = modern_names(book);
    if names.len() != 20 {
        result
            .failures
            .push(format!("modern book has {} elemental spells", names.len()));
    }
    for name in names {
        let Some(body) = bodies.get(&name) else {
            result.failures.push(format!("missing body {name}"));
            continue;
        };
        let fields = (
            number(body, "setLvlReq", false),
            number(body, "setBaseXp", true),
            number(body, "setMaxDamage", false),
            number(body, "setAutoCast", false),
            args(body, "setRunes"),
        );
        let (Some(level), Some(_), Some(_), Some(_), Some(costs)) = fields else {
            result.failures.push(format!("{name} is missing a setter"));
            continue;
        };
        let Ok(level) = level.parse() else {
            result.failures.push(format!("{name} level outside Int"));
            continue;
        };
        let mut required = vec![];
        for (i, _) in costs.match_indices("Rune.") {
            let rest = &costs[i + 5..];
            let len = rest.bytes().take_while(|b| b.is_ascii_uppercase()).count();
            if len == 0 {
                continue;
            }
            let rune = &rest[..len];
            let Some(count) = rest[len..]
                .strip_prefix(".toItem(")
                .and_then(|s| s.split_once(')').map(|(n, _)| n))
            else {
                continue;
            };
            if count.is_empty() || !count.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            if !known.contains(rune) {
                result
                    .failures
                    .push(format!("{name} uses unknown rune {rune}"));
                continue;
            }
            let Ok(count) = count.parse() else {
                result
                    .failures
                    .push(format!("{name} rune count outside Int"));
                continue;
            };
            required.push(RequirementValue {
                item: symbols
                    .get(&format!("{}rune", rune.to_lowercase()))
                    .copied()
                    .unwrap_or(-1),
                count,
            });
        }
        if required.is_empty() {
            result.failures.push(format!("{name} has no runes"));
            continue;
        }
        result.spells.insert(
            key(&name).unwrap(),
            Override {
                level,
                runes: required,
            },
        );
    }
    result
}
impl Catalog {
    pub fn apply(
        &self,
        internal_name: &str,
        spell: &mut Definition,
        known_item: impl Fn(i32) -> bool,
    ) {
        let name = internal_name
            .split_once('_')
            .map_or(internal_name, |(_, key)| key);
        let Some(row) = self.spells.get(name) else {
            return;
        };
        if row.runes.iter().any(|r| r.item < 0 || !known_item(r.item)) {
            return;
        }
        spell.level = row.level;
        spell.spell.requirements = row
            .runes
            .iter()
            .map(|r| Requirement {
                item: r.item,
                count: r.count,
                worn_slot: None,
            })
            .collect();
    }
}
/// Paths are supplied by the host to preserve isolated source fixtures.
pub fn load(directory: &Path, symbol_file: &Path) -> io::Result<Catalog> {
    let required = [
        directory.join("SpellBook.java"),
        directory.join("Spell.java"),
        directory.join("rune/Rune.java"),
        directory.join("rune/RuneRemoval.java"),
        directory.join("spells/modern/FireSpell.java"),
        symbol_file.to_owned(),
    ];
    let missing: Vec<_> = required
        .iter()
        .filter(|p| !p.exists())
        .map(|p| format!("missing {}", p.display()))
        .collect();
    if !missing.is_empty() {
        return Ok(Catalog {
            failures: missing,
            ..Default::default()
        });
    }
    let book = fs::read_to_string(&required[0])?;
    let runes = fs::read_to_string(&required[2])?;
    let mut symbols = Symbols::new();
    // Preserve the source loader's permissive tab/id parsing.
    let mut by_id = BTreeMap::new();
    for line in fs::read_to_string(symbol_file)?.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some((id, name)) = line.split_once('\t') {
            if let Ok(id) = id.parse::<i32>() {
                by_id.insert(id, name.to_owned());
            }
        }
    }
    for (id, name) in by_id {
        symbols.entry(name).or_insert(id);
    }
    let mut bodies = BTreeMap::new();
    for name in modern_names(&book) {
        let file = directory.join("spells/modern").join(format!("{name}.java"));
        if file.exists() {
            bodies.insert(name, fs::read_to_string(file)?);
        }
    }
    Ok(parse(&book, &runes, &bodies, &symbols))
}
