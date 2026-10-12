#[test]
fn object_identity_and_hashed_key_match_kotlin_including_repeated_array_opcodes() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/item-identity.expected"
    ))
    .expect("Run scripts/item-identity-parity.ps1");
    assert_eq!(rows.lines().count(), 2048);
    for (case, line) in rows.lines().enumerate() {
        let f: Vec<_> = line.split('|').collect();
        let id = f[0].parse().unwrap();
        let bytes: Vec<_> = f[1]
            .as_bytes()
            .chunks_exact(2)
            .map(|v| u8::from_str_radix(std::str::from_utf8(v).unwrap(), 16).unwrap())
            .collect();
        let item = gielinor_cache::definitions::ItemType::decode(&bytes).unwrap();
        assert_eq!(
            gielinor_cache::item_identity::identity(id, &item).to_string(),
            f[2],
            "identity {case}"
        );
        assert_eq!(
            gielinor_cache::item_identity::hashed_type_hash(id, &item).to_string(),
            f[3],
            "hashed key {case}"
        );
    }
}
#[test]
fn small_object_keyed_hashmap_order_matches_kotlin_rune_repository() {
    let rows = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/item-identity.expected.order"
    ))
    .unwrap();
    assert_eq!(rows.lines().count(), 256);
    for line in rows.lines() {
        let (hashes, expected) = line.split_once('|').unwrap();
        let hashes: Vec<i32> = hashes.split(',').map(|s| s.parse().unwrap()).collect();
        let actual = gielinor_cache::jvm_hash::order(&hashes)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(actual, expected);
    }
}
