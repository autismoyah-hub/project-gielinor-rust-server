use gielinor_world::{
    dialogue::{Page, Speaker},
    inventory::{Catalog, Inventory, Item, StackType},
    tutorial_magic_tutor::{self as tutor, Action, Config, Stage},
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

fn apply(
    action: Action,
    config: Config,
    raw: &mut i32,
    inv: &mut Inventory,
    catalog: &Catalog,
    out: &mut Vec<String>,
) {
    if matches!(action, Action::Runes | Action::RunesAndStep) {
        tutor::replenish(config, inv, catalog);
    }
    let step = match action {
        Action::Step(step) => Some(step),
        Action::RunesAndStep => Some(Step::MagicRunes),
        Action::Leave => {
            out.push("leave".into());
            None
        }
        _ => None,
    };
    if let Some(step) = step {
        *raw = step as i32;
        out.push(format!("step:{raw}:37"));
        if matches!(step, Step::MagicTab | Step::HomeTele) {
            out.push("cs:915:6".into());
        }
    }
}
#[test]
fn unchanged_magic_instructor_suspensions_choices_and_inventory_mutations() {
    let config = Config {
        step_varp: 5003,
        air: 556,
        mind: 558,
    };
    let mut catalog = Catalog::default();
    catalog.stackable.extend([556, 558]);
    let mut cases = 0;
    for row in include_str!("fixtures/tutorial-magic-tutor.expected").lines() {
        let f: Vec<_> = row.split('|').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let mut raw = n(0);
        let mut inv = decode(f[8]);
        let mut stage = tutor::begin(Step::from_var(raw));
        let mut output = vec![];
        let suspended = stage != Stage::Waiting || tutor::plenty(config, &inv, &catalog);
        assert_eq!(suspended, f[6] == "true", "{row}");
        if !suspended {
            tutor::replenish(config, &mut inv, &catalog);
            stage = Stage::Done;
        }
        let mut first = true;
        let mut choices = 0;
        while stage != Stage::Done {
            let mut conversation = tutor::conversation(config, stage);
            let mut selected = None;
            while let Some(page) = conversation.pages.pop_front() {
                match page {
                    Page::Chat { speaker, text, .. } | Page::NamedChat { speaker, text, .. } => {
                        output.push(format!(
                            "chat:{}:happy:{}",
                            if speaker == Speaker::Npc {
                                "Npc"
                            } else {
                                "Player"
                            },
                            hex(text)
                        ))
                    }
                    Page::Choice(options) | Page::TitledChoice { options, .. } => {
                        let title = match page {
                            Page::TitledChoice { title, .. } => title,
                            _ => "Select an option",
                        };
                        output.push(format!(
                            "choice:{}:{}",
                            hex(title),
                            options.iter().map(|s| hex(s)).collect::<Vec<_>>().join(":")
                        ));
                        selected = Some(tutor::choice(stage, n(1 + choices)).unwrap());
                        choices += 1;
                    }
                }
                if first {
                    first = false;
                    if f[5] == "true" {
                        inv = decode(f[9]);
                        raw = 0;
                    }
                }
            }
            let (next, action) = selected.unwrap_or_else(|| tutor::continued(stage));
            apply(action, config, &mut raw, &mut inv, &catalog, &mut output);
            stage = next;
        }
        assert_eq!(raw, n(7), "{row}");
        assert_eq!(encode(&inv), f[10], "{row}");
        assert_eq!(output.join(","), f[11], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 8064);
}
