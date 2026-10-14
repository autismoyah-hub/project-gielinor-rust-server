use gielinor_world::{
    locations::Location,
    tutorial_gates::{Effect, Gate},
    tutorial_steps::Step,
    CoordGrid,
};

#[test]
fn original_gate_branches_text_destinations_and_call_order() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-gates.expected"
    ))
    .unwrap();
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let gate = match f[0] {
            "survivalEntry" => Gate::SurvivalEntry,
            "survivalExit" => Gate::SurvivalExit,
            "chefEntry" => Gate::ChefEntry,
            "chefExit" => Gate::ChefExit,
            "questEntry" => Gate::QuestEntry,
            "questLadder" => Gate::QuestLadder,
            "miningLadderUp" => Gate::MiningLadderUp,
            "miningExit" => Gate::MiningExit,
            "combatCage" => Gate::CombatCage,
            "combatLadder" => Gate::CombatLadder,
            "bankLadder" => Gate::BankLadder,
            "bankExit" => Gate::BankExit,
            "prayerDoor" => Gate::PrayerDoor,
            "prayerExit" => Gate::PrayerExit,
            _ => panic!("unknown gate"),
        };
        let loc = Location::new(
            CoordGrid::new(1703, 12526, 0).unwrap(),
            1,
            0,
            f[4].parse().unwrap(),
        )
        .unwrap();
        let source = if f[3] == "0" {
            loc.coords
        } else {
            loc.coords.translate(1, 0, 0).unwrap()
        };
        let effects = gate
            .decide(
                Step::from_var(f[1].parse().unwrap()),
                f[2].parse().unwrap(),
                source,
                loc,
            )
            .unwrap();
        let result = effects
            .into_iter()
            .map(|effect| match effect {
                Effect::Advance(step) => format!("advance:{}", step as i32),
                Effect::Deny(text) => format!(
                    "deny:{}",
                    text.as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                ),
                Effect::Teleport(to) => format!("teleport:{}:{}:{}", to.x(), to.z(), to.level()),
                Effect::Telejump(to) => format!("telejump:{}:{}:{}", to.x(), to.z(), to.level()),
            })
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(result, f[5], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 18816);
}
