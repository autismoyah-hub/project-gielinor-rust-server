use gielinor_world::{
    tutorial_panels::{self, Action, Panel},
    tutorial_steps::Step,
};

#[test]
fn original_side_panel_handlers_preserve_refresh_and_completed_account_behavior() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-panels.expected"
    ))
    .unwrap();
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let panel = match f[0] {
            "Settings" => Panel::Settings,
            "Combat" => Panel::Combat,
            "Stats" => Panel::Stats,
            "QuestList" => Panel::Quests,
            "Inventory" => Panel::Inventory,
            "Worn" => Panel::Worn,
            "Prayer" => Panel::Prayer,
            "Magic" => Panel::Magic,
            "Account" => Panel::Account,
            _ => panic!("unknown panel"),
        };
        let raw: i32 = f[1].parse().unwrap();
        let action = tutorial_panels::opened(panel, Step::from_var(raw));
        let (decision, resulting_step) = match action {
            Action::None => ("none".to_string(), raw),
            Action::PointBackToGuide => ("guide".to_string(), raw),
            Action::Set(step) => (format!("set:{}", step as i32), step as i32),
        };
        assert_eq!(decision, f[3], "{row}");
        assert_eq!(resulting_step.to_string(), f[4], "{row}");
        assert_eq!(f[2], f[5], "handler must not change completed flag: {row}");
        cases += 1;
    }
    assert_eq!(cases, 1008);
}
