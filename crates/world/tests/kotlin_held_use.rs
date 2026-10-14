use gielinor_world::{
    held_use::{Registry, Scope, Type},
    inventory::{Inventory, Item, StackType},
};
#[test]
fn original_held_use_handler_precedence_orientation_and_slot_verification() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/held-use.expected"
    ))
    .unwrap();
    let types = [
        (
            10,
            Type {
                id: 10,
                content: 100,
            },
        ),
        (
            20,
            Type {
                id: 20,
                content: 200,
            },
        ),
    ]
    .into();
    let mut count = 0;
    for row in fixture.lines() {
        let f: Vec<_> = row.split('|').collect();
        let mask: u32 = f[0].parse().unwrap();
        let reverse: usize = f[1].parse().unwrap();
        let invalid: i32 = f[2].parse().unwrap();
        let mut registry = Registry::default();
        for direction in 0..2 {
            let (a, b, ac, bc) = if direction == 0 {
                (10, 20, 100, 200)
            } else {
                (20, 10, 200, 100)
            };
            for (scope, first, second) in [
                (Scope::Types, a, b),
                (Scope::ContentType, ac, b),
                (Scope::Contents, ac, bc),
                (Scope::DefaultType, a, 0),
                (Scope::DefaultContent, ac, 0),
            ] {
                if mask & (1 << (direction * 5 + scope as u32)) != 0 {
                    registry.register(scope, first, second, ()).unwrap();
                }
            }
        }
        let mut inv = Inventory::new(4, StackType::Normal, false);
        for (slot, id) in [(0, 10), (1, 20)] {
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
        let selected = reverse;
        let target = if invalid == 1 { selected } else { 1 - reverse };
        if invalid == 2 {
            inv.restore_slot(selected, None).unwrap();
        }
        if invalid == 3 {
            inv.restore_slot(
                target,
                Some(Item {
                    id: 99,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
        }
        let output = match registry.resolve(
            &inv,
            &types,
            selected as i32,
            if reverse == 0 { 10 } else { 20 },
            target as i32,
            if reverse == 0 { 20 } else { 10 },
        ) {
            Ok(Some(found)) => format!(
                "publish:{}:{}:{}:{}:{}",
                found.scope as usize,
                found.first.ty.id,
                found.first.slot,
                found.second.ty.id,
                found.second.slot
            ),
            Ok(None) => "message:Nothing interesting happens.".into(),
            Err(_) => "resend:0".into(),
        };
        assert_eq!(output, f[3], "{row}");
        count += 1;
    }
    assert_eq!(count, 4608);
}
