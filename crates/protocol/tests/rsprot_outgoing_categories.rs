use gielinor_protocol::{
    outgoing_category::{category, Category},
    tables,
};
#[test]
fn every_wire_opcode_matches_original_encoder_message_category() {
    let fixture = include_str!("fixtures/outgoing-categories.expected");
    for def in tables::SERVER {
        let row = fixture
            .lines()
            .find(|line| line.split_whitespace().nth(1) == Some(def.name))
            .expect("Missing original category");
        let fields: Vec<_> = row.split_whitespace().collect();
        assert_eq!(fields[0].parse::<u16>().unwrap(), def.opcode);
        let expected = match fields[2] {
            "HIGH_PRIORITY_PROT" => Some(Category::High),
            "LOW_PRIORITY_PROT" => Some(Category::Low),
            "UNSUPPORTED" => None,
            _ => panic!("Unknown original category"),
        };
        assert_eq!(category(def.opcode), expected, "{}", def.name);
    }
    assert_eq!(category(32767), None);
}
