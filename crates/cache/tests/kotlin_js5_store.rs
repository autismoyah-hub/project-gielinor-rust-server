use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn put24(bytes: &mut [u8], value: usize) {
    bytes.copy_from_slice(&[(value >> 16) as u8, (value >> 8) as u8, value as u8]);
}
fn disk(root: &Path, groups: &BTreeMap<(u8, u16), Vec<u8>>) -> io::Result<()> {
    fs::create_dir_all(root)?;
    let mut data = vec![0u8; 520];
    let mut indexes: BTreeMap<u8, Vec<u8>> = BTreeMap::new();
    for (&(archive, group), bytes) in groups {
        let sector = data.len() / 520;
        let index = indexes.entry(archive).or_default();
        index.resize(index.len().max((group as usize + 1) * 6), 0);
        let entry = &mut index[group as usize * 6..group as usize * 6 + 6];
        put24(&mut entry[..3], bytes.len());
        put24(&mut entry[3..], sector);
        // Empty entries retain a sector pointer, as in the reference Store.
        for (chunk, payload) in bytes.chunks(512).enumerate() {
            let mut block = [0; 520];
            block[..2].copy_from_slice(&group.to_be_bytes());
            block[2..4].copy_from_slice(&(chunk as u16).to_be_bytes());
            if (chunk + 1) * 512 < bytes.len() {
                put24(&mut block[4..7], sector + chunk + 1);
            }
            block[7] = archive;
            block[8..8 + payload.len()].copy_from_slice(payload);
            data.extend(block);
        }
    }
    fs::write(root.join("main_file_cache.dat2"), data)?;
    for (archive, bytes) in indexes {
        fs::write(root.join(format!("main_file_cache.idx{archive}")), bytes)?;
    }
    Ok(())
}
#[test]
fn actual_kotlin_js5_store_matches_master_crc_gaps_and_raw_group_trailers() {
    let fixture = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/js5-store.expected"
    ))
    .unwrap();
    assert_eq!(fixture.lines().count(), 128);
    let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../work/tmp");
    fs::create_dir_all(&parent).unwrap();
    let root = parent.join(format!(
        "js5-store-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    for (case, line) in fixture.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let mut groups = BTreeMap::new();
        for index in f[0].split(';') {
            let (id, bytes) = index.split_once(':').unwrap();
            groups.insert((255, id.parse().unwrap()), hex(bytes));
        }
        for group in f[1].split(';') {
            let p: Vec<_> = group.split(':').collect();
            groups.insert((p[0].parse().unwrap(), p[1].parse().unwrap()), hex(p[2]));
        }
        let path = root.join(case.to_string());
        disk(&path, &groups).unwrap();
        let mut store = gielinor_cache::js5::Store::open(&path, 65536).unwrap();
        assert_eq!(
            store.response(255, 255).unwrap(),
            hex(f[2]),
            "master {case}"
        );
        for (id, crc) in f[3].split(',').enumerate() {
            assert_eq!(store.checksums[id].to_string(), crc, "checksum {case}:{id}");
        }
        for group in f[4].split(';') {
            let p: Vec<_> = group.split(':').collect();
            assert_eq!(
                store
                    .response(p[0].parse().unwrap(), p[1].parse().unwrap())
                    .unwrap(),
                hex(p[2]),
                "group {case}"
            );
        }
    }
    // The path is constructed only under this crate's owned scratch directory.
    let canonical = root.canonicalize().unwrap();
    assert!(canonical.starts_with(parent.canonicalize().unwrap()));
    fs::remove_dir_all(canonical).unwrap();
}
