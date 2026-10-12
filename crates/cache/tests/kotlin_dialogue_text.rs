fn bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
#[test]
fn original_font_decoder_and_dialogue_page_alignment() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/dialogue-text.expected"
    ))
    .expect("Run scripts/dialogue-text-parity.ps1");
    let mut count = 0;
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        let font = gielinor_cache::fonts::decode(&bytes(fields[0])).unwrap();
        let text = String::from_utf8(bytes(fields[1])).unwrap();
        let result = match font.pages(&text) {
            Ok(pages) => pages
                .into_iter()
                .map(|(text, count)| {
                    format!(
                        "{count},{},{}",
                        gielinor_world::dialogue_text::line_height(count),
                        text.as_bytes()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>()
                    )
                })
                .collect::<Vec<_>>()
                .join(";"),
            Err(_) => "error".into(),
        };
        assert_eq!(result, fields[2], "case {count}: {text:?}");
        count += 1;
    }
    assert_eq!(count, 384);
}
