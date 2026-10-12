use gielinor_world::ui::{Component, Events};
#[test]
fn original_component_event_ranges_and_last_write_precedence() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/ui.expected"
    ))
    .expect("Run scripts/ui-parity.ps1");
    let mut events = Events::default();
    let mut count = 0;
    for line in fixture.lines() {
        let words: Vec<_> = line.split_whitespace().collect();
        let num = |n: usize| words[n].parse::<i32>().unwrap();
        let component = Component::new(num(1) as u16, num(2) as u16);
        if num(0) == 0 {
            events.clear(component.interface());
        } else if num(0) < 5 {
            events
                .add(component, num(3), num(4), words[5].parse().unwrap())
                .unwrap();
        }
        let result = events.get(component, num(6));
        assert_eq!(result.is_some(), words[7] == "true", "{line}");
        assert_eq!(
            result.unwrap_or(0),
            words[8].parse::<u64>().unwrap(),
            "{line}"
        );
        count += 1;
    }
    assert_eq!(count, 4096);
}
