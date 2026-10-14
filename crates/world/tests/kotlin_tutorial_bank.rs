use gielinor_world::{
    inventory::{Catalog, Insert, Inventory, StackType},
    tutorial_bank::{self, Booth, Config},
    tutorial_progression::{self, Effect},
    variables::{Transmit, Variables, Varp},
};
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
#[test]
fn unchanged_tutorial_booth_gates_gifts_output_order_and_progression() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/tutorial-bank.expected"
    ))
    .expect("Run scripts/tutorial-bank-parity.ps1");
    let varp = |id| Varp {
        id,
        permanent: true,
        transmit: Transmit::Always,
        bit_protect: false,
        client_code: -1,
    };
    let config = Config {
        progression: tutorial_progression::Config {
            step: varp(1),
            completed: varp(2),
        },
        bank: 10,
        poll: 11,
        coins: 995,
    };
    let mut catalog = Catalog::default();
    catalog.stackable.insert(995);
    let mut cases = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let n = |i: usize| f[i].parse::<i32>().unwrap();
        let booth = if n(0) == 0 { Booth::Bank } else { Booth::Poll };
        let layout = n(3);
        let mut bank = Inventory::new(usize::from(layout != 0), StackType::Always, false);
        if layout == 2 {
            bank.insert(&catalog, Insert::new(42, 1)).unwrap();
        }
        if layout >= 3 {
            bank.insert(
                &catalog,
                Insert::new(995, [1, 24, 25][(layout - 3) as usize]),
            )
            .unwrap();
        }
        let mut vars = Variables::restore([(1, n(1)), (2, n(2))].into());
        let mut output = vec![];
        if let Some(message) = config.denial(&vars, booth) {
            output.push(format!("message:{}", hex(message)));
        } else {
            let messages: &[&str] = if booth == Booth::Bank {
                config.grant(&mut bank, &catalog);
                output.push("open".into());
                &[tutorial_bank::BANK_MESSAGE]
            } else {
                &tutorial_bank::POLL_MESSAGES
            };
            for message in messages {
                output.push(format!("message:{}", hex(message)));
            }
            if let Some(next) = config.next(&vars, booth) {
                for effect in config.progression.set(&mut vars, next, None, true) {
                    output.push(match effect {
                        Effect::Variable(u) => format!("var:{}:{}", u.id, u.value),
                        Effect::Focus { step, .. } => format!("focus:{}", step as i32),
                        Effect::End => "end".into(),
                    });
                }
            }
        }
        assert_eq!((vars.get(1), vars.get(2)), (n(4), n(5)), "{row}");
        let encoded = bank
            .slots()
            .iter()
            .map(|i| i.map_or("_".into(), |i| format!("{}:{}:0", i.id, i.count)))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(encoded, f[6], "{row}");
        assert_eq!(output.join(","), f[7], "{row}");
        cases += 1;
    }
    assert_eq!(cases, 2016);
}
