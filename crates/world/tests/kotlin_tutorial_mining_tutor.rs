use gielinor_world::{
    dialogue::{Effect, Expression, Page, Speaker},
    inventory::{Catalog, Inventory, Item, StackType},
    tutorial_mining_tutor::{self as tutor, Config},
    tutorial_steps::Step,
};
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn original_mining_tutor_branches_inventory_and_name_timing() {
    let config = Config {
        step_varp: 5003,
        pickaxe: 1265,
        hammer: 2347,
    };
    let catalog = Catalog::default();
    let mut cases = 0;
    for row in include_str!("fixtures/tutorial-mining-tutor.expected").lines() {
        let f: Vec<_> = row.split('|').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let initial = n(0);
        let choice = n(1);
        let held = n(3);
        let free = n(4);
        let mut inv = Inventory::new(28, StackType::Normal, false);
        for slot in 0..(28 - free as usize) {
            let id = if slot == 0 && held & 1 != 0 {
                1265
            } else if slot == 1 && held & 2 != 0 {
                2347
            } else {
                995
            };
            inv.restore_slot(
                slot,
                Some(Item {
                    id,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        if Step::from_var(initial) >= Step::MineGate {
            tutor::prepare_recap(config, &mut inv, &catalog);
        }
        let mut conversation = tutor::begin(config, Step::from_var(initial));
        let mut raw = initial;
        let mut name = "Before";
        let mut output = vec![];
        while let Some(page) = conversation.pages.pop_front() {
            match page {
                Page::Chat {
                    speaker,
                    expression,
                    text,
                }
                | Page::NamedChat {
                    speaker,
                    expression,
                    text,
                } => output.push(format!(
                    "chat:{}:{}:{}",
                    if speaker == Speaker::Npc {
                        "Npc"
                    } else {
                        "Player"
                    },
                    if expression == Expression::Quiz {
                        "quiz"
                    } else {
                        "happy"
                    },
                    hex(&text.replace("${player.displayName}", name))
                )),
                Page::Choice(options) | Page::TitledChoice { options, .. } => {
                    output.push(format!(
                        "choice:{}",
                        options.iter().map(|s| hex(s)).collect::<Vec<_>>().join(":")
                    ));
                    conversation = tutor::choice(config, choice).unwrap();
                }
            }
            if name == "Before" {
                name = "After";
                if n(2) != -1 {
                    raw = n(2);
                }
            }
        }
        let Effect::MiningTutor { stage, .. } = conversation.after else {
            panic!("Unexpected effect")
        };
        let (step, message) = tutor::finish(config, stage, Step::from_var(raw), &mut inv, &catalog);
        if let Some(message) = message {
            output.push(format!("mes:{}", hex(message)));
        }
        if let Some(step) = step {
            raw = step as i32;
            output.push(format!("step:{raw}:37"));
        }
        assert_eq!(raw, n(5), "{row}");
        for (id, column) in [(1265, 6), (2347, 7)] {
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .filter(|item| item.id == id)
                    .map(|item| item.count)
                    .sum::<i32>(),
                n(column),
                "{row}"
            );
        }
        assert_eq!(
            inv.slots().iter().filter(|s| s.is_none()).count() as i32,
            n(8),
            "{row}"
        );
        assert_eq!(output.join(","), f[9], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 5376);
}
