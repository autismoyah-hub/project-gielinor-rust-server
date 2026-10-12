//! Read-only `.sym` mappings matching Kotlin NameLoader and ordered directory overlays.
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::Path,
};
pub type Symbols = BTreeMap<String, i32>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidId(String),
    DuplicateId(i32),
    InvalidChild(String),
    UnknownParent(String),
}
fn integer(text: &str) -> Result<i32, Error> {
    // Rust integer parsing, like Kotlin toIntOrNull, rejects surrounding whitespace.
    text.parse().map_err(|_| Error::InvalidId(text.into()))
}
pub fn parse(text: &str, parents: Option<&Symbols>) -> Result<Symbols, Error> {
    let mut values = Symbols::new();
    let mut ids = BTreeSet::new();
    for line in text.lines() {
        if line.starts_with('#') {
            continue;
        }
        let Some((id, name)) = line.split_once('\t') else {
            continue;
        };
        let id = if let Some(parents) = parents {
            let (parent, child) = id
                .split_once(':')
                .ok_or_else(|| Error::InvalidChild(id.into()))?;
            let parent = *parents
                .get(parent)
                .ok_or_else(|| Error::UnknownParent(parent.into()))?;
            (parent.wrapping_shl(16)) | integer(child)?
        } else {
            integer(id)?
        };
        if !ids.insert(id) {
            return Err(Error::DuplicateId(id));
        }
        // Original loader permits duplicate names and retains the last mapping.
        values.insert(name.into(), id);
    }
    Ok(values)
}
/// The caller supplies directory precedence; missing files contribute no names.
pub fn load(directories: &[&Path], kind: &str, parents: Option<&Symbols>) -> io::Result<Symbols> {
    if kind.is_empty() || !kind.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid symbol kind",
        ));
    }
    let mut result = Symbols::new();
    for directory in directories {
        let path = directory.join(format!("{kind}.sym"));
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        };
        result.extend(parse(&text, parents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {e:?}", path.display()),
            )
        })?);
    }
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_name_collision_and_parent_packing_semantics() {
        let plain = parse(
            "# note\nignored\n1\tone\n2\tone\n-1\tnull\n+3\tthree\r\n",
            None,
        )
        .unwrap();
        assert_eq!(plain["one"], 2);
        assert_eq!(plain["null"], -1);
        assert_eq!(plain["three"], 3);
        assert_eq!(parse("1\tone\n1\ttwo", None), Err(Error::DuplicateId(1)));
        assert!(parse(" 1\tone", None).is_err());
        let parents = BTreeMap::from([("inventory".into(), 149)]);
        assert_eq!(
            parse("inventory:4\titems", Some(&parents)).unwrap()["items"],
            (149 << 16) | 4
        );
        // No child mask/range clamp is applied by NameLoader.
        assert_eq!(
            parse("inventory:-1\tall", Some(&parents)).unwrap()["all"],
            -1
        );
        assert!(parse("unknown:4\titems", Some(&parents)).is_err());
    }
}
