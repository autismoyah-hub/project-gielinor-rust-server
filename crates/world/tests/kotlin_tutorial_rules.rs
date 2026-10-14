use gielinor_world::{engine_queues::Kind, stats::Stats, tutorial_rules};

#[test]
fn original_tutorial_stat_handlers_unsigned_levels_and_current_completion() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-rules.expected"
    ))
    .unwrap();
    let mut count = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let completed = f[0].parse().unwrap();
        let kind = if f[1] == "1" {
            Kind::ChangeStat
        } else {
            Kind::AdvanceStat
        };
        let stat: u8 = f[2].parse().unwrap();
        let mut stats = Stats::default();
        stats.set_base(stat, f[3].parse::<u8>().unwrap() as i8);
        stats.set_current(stat, f[4].parse::<u8>().unwrap() as i8);
        stats.set_fine_xp(stat, 123456).unwrap();
        let effect = tutorial_rules::deliver(&mut stats, completed, kind, stat);
        let mut output = Vec::new();
        if effect.clear_death_queue {
            output.push("clear:1".to_owned());
        }
        if let Some(message) = effect.message {
            output.push(format!("mes:{message}"));
        }
        assert_eq!(stats.fine_xp(stat), f[5].parse::<i32>().unwrap(), "{row}");
        assert_eq!(stats.base(stat) as u8, f[6].parse::<u8>().unwrap(), "{row}");
        assert_eq!(
            stats.current(stat) as u8,
            f[7].parse::<u8>().unwrap(),
            "{row}"
        );
        assert_eq!(output.join("~"), f[8], "{row}");
        count += 1;
    }
    assert_eq!(count, 11776);
}
