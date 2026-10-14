use gielinor_world::{
    inventory::{Catalog, Inventory, Item, StackType},
    smithing_make::Request,
};

#[test]
fn original_make_loop_affordability_and_input_guards() {
    let fixture = include_str!("fixtures/smithing-make.expected");
    let mut cases = 0;
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('|').collect();
        assert_eq!(fields.len(), 8);
        let n = |i: usize| fields[i].parse::<i32>().unwrap();
        let mut inv = Inventory::new(2, StackType::Normal, false);
        for (slot, id, count) in [(0, 436, n(1)), (1, 438, n(2))] {
            if count != 0 {
                inv.restore_slot(slot, Some(Item { id, count, vars: 0 }))
                    .unwrap();
            }
        }
        let mut consumed = vec![];
        if n(0) & 1 != 0 {
            consumed.push((436, n(3)));
        }
        if n(0) & 2 != 0 {
            consumed.push((438, n(4)));
        }
        let request = Request {
            product: 2349,
            produced: 1,
            consumed,
            xp: 6.2,
            animation: 899,
            cycles: 5,
        };
        let catalog = Catalog::default();
        let actual = request
            .affordable_count(&inv, &catalog, n(5))
            .map(|n| n.to_string())
            .unwrap_or_else(|_| "ERR".into());
        assert_eq!(actual, fields[6], "{line}");
        assert_eq!(
            request.has_inputs(&inv, &catalog).to_string(),
            fields[7],
            "{line}"
        );
        cases += 1;
    }
    assert_eq!(cases, 12500);
}
