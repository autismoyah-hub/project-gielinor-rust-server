use gielinor_world::{
    dialogue::{Effect, Expression, Page, Speaker},
    inventory::{Catalog, Inventory, Item, StackType},
    tutorial_combat_tutor::{self as tutor, Config},
    tutorial_steps::Step,
};
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn decode(text: &str) -> Inventory {
    let slots: Vec<_> = text.split(',').collect();
    let mut inv = Inventory::new(slots.len(), StackType::Normal, false);
    for (slot, encoded) in slots.iter().enumerate() {
        if *encoded != "." {
            let (id, count) = encoded.split_once(':').unwrap();
            inv.restore_slot(
                slot,
                Some(Item {
                    id: id.parse().unwrap(),
                    count: count.parse().unwrap(),
                    vars: 0,
                }),
            )
            .unwrap();
        }
    }
    inv
}
fn encode(inv: &Inventory) -> String {
    inv.slots()
        .iter()
        .map(|item| {
            item.as_ref()
                .map_or(".".into(), |item| format!("{}:{}", item.id, item.count))
        })
        .collect::<Vec<_>>()
        .join(",")
}
#[test]
fn original_combat_instructor_dialogue_supplies_and_captured_equipment_branches() {
    let config = Config {
        step_varp: 5003,
        dagger: 1205,
        sword: 1277,
        shield: 1171,
        bow: 841,
        arrows: 882,
    };
    let mut catalog = Catalog::default();
    catalog.stackable.insert(882);
    let mut cases = 0;
    for row in include_str!("fixtures/tutorial-combat-tutor.expected").lines() {
        let f: Vec<_> = row.split('|').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let mut raw = n(0);
        let mut inv = decode(f[6]);
        let mut worn = decode(f[7]);
        let mut conversation = tutor::begin(config, Step::from_var(raw), &inv, &worn, &catalog);
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
                    conversation = tutor::choice(config, n(1)).unwrap();
                }
            }
            if name == "Before" {
                name = "After";
                if n(2) != -1 {
                    raw = n(2);
                }
                inv = decode(f[8]);
                worn = decode(f[9]);
            }
        }
        let Effect::CombatTutor { stage, .. } = conversation.after else {
            panic!("Unexpected effect")
        };
        let (next, messages) = tutor::finish(
            config,
            stage,
            Step::from_var(raw),
            &mut inv,
            &worn,
            &catalog,
        );
        for message in messages {
            output.push(format!("mes:{}", hex(message)));
        }
        if let Some((step, tab)) = next {
            raw = step as i32;
            output.push(format!("step:{raw}:37"));
            if let Some(tab) = tab {
                output.push(format!("cs:915:{tab}"));
            }
        }
        assert_eq!(raw, n(5), "{row}");
        assert_eq!(encode(&inv), f[10], "{row}");
        assert_eq!(encode(&worn), f[11], "{row}");
        assert_eq!(output.join(","), f[12], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 3960);
}
