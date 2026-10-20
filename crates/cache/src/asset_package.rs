//! Explicit, immutable, out-of-band original content. Does not modify stock JS5.
//! SHA-256 pins operator-admitted bytes; it is not a publisher signature.
use crate::{game::Definitions, invalid, locations::LocationType};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{self, Read},
    path::Path,
};

pub struct Namespace<'a> {
    pub models: &'a [u32],
    pub objects: &'a [u32],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "gielinor-asset-test-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            fs::write(path.join("registry.txt"), b"TOWER_ASSET_PACKAGE 1\nmodel 1251000 model-1251000.model c930cff3\nobject 65000 object-65000.obj d202ef8d\n").unwrap();
            fs::write(path.join("model-1251000.model"), b"model-bytes").unwrap();
            fs::write(path.join("object-65000.obj"), [0]).unwrap();
            Self(path)
        }
        fn load(&self) -> io::Result<AssetPackage> {
            AssetPackage::load(
                &self.0,
                identity(),
                Namespace {
                    models: &[1251000],
                    objects: &[65000],
                },
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert!(self.0.starts_with(std::env::temp_dir()));
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn identity() -> [u8; 32] {
        // Independently computed with Python hashlib and big-endian struct.pack.
        let hex = b"4a117728337b32b24e3f1bec0ff8b2616835bef49bf4066758d2a3331f9ece92";
        std::array::from_fn(|i| {
            u8::from_str_radix(std::str::from_utf8(&hex[2 * i..2 * i + 2]).unwrap(), 16).unwrap()
        })
    }
    fn definitions() -> Definitions {
        Definitions {
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            items: BTreeMap::new(),
            varbits: BTreeMap::new(),
            varps: BTreeMap::new(),
            stats: BTreeMap::new(),
        }
    }
    #[test]
    fn admitted_package_is_immutable_and_installation_refuses_stock_collisions_atomically() {
        let f = Fixture::new();
        let p = f.load().unwrap();
        assert_eq!(p.identity(), identity());
        fs::write(f.0.join("model-1251000.model"), b"changed-on-disk").unwrap();
        assert_eq!(p.model(1251000), Some(&b"model-bytes"[..]));
        assert!(f.load().is_err());
        let mut d = definitions();
        assert!(p.install(&mut d, &BTreeSet::from([1251000])).is_err());
        assert!(d.locations.is_empty());
        p.install(&mut d, &BTreeSet::new()).unwrap();
        assert_eq!(d.locations.len(), 1);
        assert!(p.install(&mut d, &BTreeSet::new()).is_err());
        assert_eq!(d.locations.len(), 1);
    }
    #[test]
    fn content_identity_namespace_crc_and_duplicate_rows_are_independent_rejections() {
        let f = Fixture::new();
        assert!(AssetPackage::load(
            &f.0,
            [0; 32],
            Namespace {
                models: &[1251000],
                objects: &[65000]
            }
        )
        .is_err());
        assert!(AssetPackage::load(
            &f.0,
            identity(),
            Namespace {
                models: &[1251000],
                objects: &[]
            }
        )
        .is_err());
        assert!(AssetPackage::load(
            &f.0,
            identity(),
            Namespace {
                models: &[1251000, 1251000],
                objects: &[65000]
            }
        )
        .is_err());
        let registry = fs::read_to_string(f.0.join("registry.txt")).unwrap();
        fs::write(
            f.0.join("registry.txt"),
            format!("{registry}object 65000 object-65000.obj d202ef8d\n"),
        )
        .unwrap();
        assert!(f.load().is_err());
        fs::write(
            f.0.join("registry.txt"),
            registry.replace("object-65000.obj", "../outside.obj"),
        )
        .unwrap();
        assert!(f.load().is_err());
    }

    #[test]
    fn a_crc_collision_does_not_satisfy_the_admitted_content_identity() {
        let f = Fixture::new();
        // Independent Python zlib control: distinct bytes, same original row CRC.
        let collision = b"other-model-bytes\xd0\xea\x4c\x53";
        assert_eq!(crate::crc32(collision), crate::crc32(b"model-bytes"));
        fs::write(f.0.join("model-1251000.model"), collision).unwrap();
        let error = match f.load() {
            Ok(_) => panic!("CRC collision admitted different content"),
            Err(error) => error,
        };
        assert_eq!(error.to_string(), "asset content identity");
    }
}
pub struct AssetPackage {
    identity: [u8; 32],
    models: BTreeMap<u32, Vec<u8>>,
    objects: BTreeMap<u32, LocationType>,
}
fn bounded(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(invalid("asset reparse point"));
        }
    }
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64 {
        return Err(invalid("asset file bounds"));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(invalid("asset file bounds"));
    }
    Ok(bytes)
}
impl AssetPackage {
    pub fn load(root: &Path, expected: [u8; 32], namespace: Namespace<'_>) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(root)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid("asset package root"));
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err(invalid("asset package reparse point"));
            }
        }
        let registry = bounded(&root.join("registry.txt"), 65536)?;
        let text = std::str::from_utf8(&registry).map_err(|_| invalid("asset registry text"))?;
        let mut lines = text.lines();
        if lines.next() != Some("TOWER_ASSET_PACKAGE 1") {
            return Err(invalid("asset registry version"));
        }
        let mut digest = Sha256::new();
        digest.update(b"GIELINOR_ASSET_PACKAGE_CONTENT_V1\0");
        digest.update((registry.len() as u64).to_be_bytes());
        digest.update(&registry);
        let allowed_models: BTreeSet<_> = namespace.models.iter().copied().collect();
        let allowed_objects: BTreeSet<_> = namespace.objects.iter().copied().collect();
        if allowed_models.len() != namespace.models.len()
            || allowed_objects.len() != namespace.objects.len()
        {
            return Err(invalid("duplicate namespace reservation"));
        }
        let mut package = Self {
            identity: expected,
            models: BTreeMap::new(),
            objects: BTreeMap::new(),
        };
        let mut total = 0usize;
        for line in lines {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() != 4
                || fields[2].is_empty()
                || fields[2].contains("..")
                || !fields[2]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'-')
            {
                return Err(invalid("asset registry path"));
            }
            let id: u32 = fields[1].parse().map_err(|_| invalid("asset ID"))?;
            let crc = u32::from_str_radix(fields[3], 16).map_err(|_| invalid("asset CRC"))?;
            match fields[0] {
                "model" if allowed_models.contains(&id) && !package.models.contains_key(&id) => {}
                "object"
                    if id <= 65535
                        && allowed_objects.contains(&id)
                        && !package.objects.contains_key(&id) => {}
                _ => return Err(invalid("asset kind, namespace or duplicate ID")),
            }
            let bytes = bounded(&root.join(fields[2]), 16 << 20)?;
            total = total
                .checked_add(bytes.len())
                .filter(|n| *n <= 256 << 20)
                .ok_or_else(|| invalid("asset package bounds"))?;
            if crate::crc32(&bytes) != crc {
                return Err(invalid("asset checksum"));
            }
            digest.update((bytes.len() as u64).to_be_bytes());
            digest.update(&bytes);
            if fields[0] == "model" {
                package.models.insert(id, bytes);
            } else {
                package.objects.insert(
                    id,
                    LocationType::decode(&bytes).map_err(|_| invalid("asset object encoding"))?,
                );
            }
        }
        if package.models.keys().copied().collect::<BTreeSet<_>>() != allowed_models
            || package.objects.keys().copied().collect::<BTreeSet<_>>() != allowed_objects
        {
            return Err(invalid("incomplete asset namespace"));
        }
        let identity: [u8; 32] = digest.finalize().into();
        if identity != expected {
            return Err(invalid("asset content identity"));
        }
        Ok(package)
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn model(&self, id: u32) -> Option<&[u8]> {
        self.models.get(&id).map(Vec::as_slice)
    }
    pub fn object(&self, id: u32) -> Option<&LocationType> {
        self.objects.get(&id)
    }
    /// Validate every collision first, then publish the complete definition set.
    pub fn install(
        &self,
        definitions: &mut Definitions,
        stock_models: &BTreeSet<u32>,
    ) -> io::Result<()> {
        if self.models.keys().any(|id| stock_models.contains(id))
            || self
                .objects
                .keys()
                .any(|id| definitions.locations.contains_key(id))
        {
            return Err(invalid("asset replaces stock ID"));
        }
        definitions
            .locations
            .extend(self.objects.iter().map(|(id, ty)| (*id, ty.clone())));
        Ok(())
    }
}

/// Immutable companion content admitted by an operator-pinned SHA-256.
pub fn read_pinned(path: &Path, expected: [u8; 32], limit: usize) -> io::Result<Vec<u8>> {
    let bytes = bounded(path, limit)?;
    let actual: [u8; 32] = Sha256::digest(&bytes).into();
    if actual != expected {
        return Err(invalid("companion content identity"));
    }
    Ok(bytes)
}
