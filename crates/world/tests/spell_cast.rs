use gielinor_world::{
    autocast::Spell,
    bank_actions::Binding,
    inventory::{Catalog, Inventory, Item, StackType},
    magic,
    rune_consumption::ResultKind,
    runes::{self, Requirement},
    spell_cast::{attempt, Request, ResultKindCast},
    variables::{Transmit, Varbit, Variables, Varp},
};
fn binding(id: u16) -> Binding {
    Binding {
        base: Varp {
            id,
            permanent: true,
            transmit: Transmit::Different,
            bit_protect: false,
            client_code: -1,
        },
        bit: Varbit {
            base: id,
            lsb: 0,
            msb: 15,
        },
    }
}
fn config() -> magic::Config {
    magic::Config {
        catalog: runes::Catalog::default(),
        fake: [binding(0); 2],
        fountain: binding(0),
        blighted: binding(0),
        regular_pouches: Default::default(),
        divine_pouches: Default::default(),
        pouch_types: [binding(0); 4],
        pouch_counts: [(0, binding(0)); 4],
        lms_pouch: 999,
        packs: Default::default(),
        names: magic::Names::default(),
    }
}
fn request(spell: &Spell) -> Request<'_> {
    Request {
        spell,
        required_book: Some(0),
        active_book: 0,
        required_level: 1,
        visible_level: 1,
        sunfire_item: 2,
        logged: true,
    }
}
fn item(id: i32, count: i32) -> Option<Item> {
    Some(Item { id, count, vars: 0 })
}
#[test]
fn access_precedes_runes_and_never_mutates_on_failure() {
    let config = config();
    let spell = Spell {
        item: 100,
        requirements: vec![Requirement {
            item: 1,
            count: 1,
            worn_slot: None,
        }],
    };
    let mut inv = Inventory::new(28, StackType::Normal, false);
    let worn = Inventory::new(14, StackType::Normal, false);
    let mut vars = Variables::default();
    let items = Catalog::default();
    let mut req = request(&spell);
    req.active_book = 1;
    req.visible_level = 0;
    let result = attempt(&config, req, &mut inv, &worn, &mut vars, &items).unwrap();
    assert_eq!(result.kind, ResultKindCast::IncorrectSpellbook);
    assert_eq!(result.message, None);
    let mut req = request(&spell);
    req.visible_level = 0;
    let result = attempt(&config, req, &mut inv, &worn, &mut vars, &items).unwrap();
    assert_eq!(result.kind, ResultKindCast::MissingLevel);
    assert_eq!(
        result.message.as_deref(),
        Some("Your Magic level is not high enough for this spell.")
    );
    assert!(inv.slots().iter().all(Option::is_none));
}
#[test]
fn all_requirements_validate_before_consumption_and_substitute_marks_sunfire() {
    let mut config = config();
    config.catalog.substitutes.insert(1, vec![2]);
    config.names.missing.insert(3, "Missing catalyst.".into());
    let spell = Spell {
        item: 100,
        requirements: vec![
            Requirement {
                item: 1,
                count: 2,
                worn_slot: None,
            },
            Requirement {
                item: 3,
                count: 1,
                worn_slot: None,
            },
        ],
    };
    let mut inv = Inventory::new(28, StackType::Normal, false);
    inv.restore_slot(0, item(2, 5)).unwrap();
    let worn = Inventory::new(14, StackType::Normal, false);
    let mut vars = Variables::default();
    let mut items = Catalog::default();
    items.stackable.extend([2, 3]);
    let before = inv.clone();
    let result = attempt(&config, request(&spell), &mut inv, &worn, &mut vars, &items).unwrap();
    assert_eq!(result.kind, ResultKindCast::MissingRequirements);
    assert_eq!(result.message.as_deref(), Some("Missing catalyst."));
    assert_eq!(inv, before);
    inv.restore_slot(1, item(3, 1)).unwrap();
    let result = attempt(&config, request(&spell), &mut inv, &worn, &mut vars, &items).unwrap();
    assert!(result.success());
    assert!(result.used_sunfire());
    assert_eq!(inv.get(0).unwrap().count, 3);
    assert!(inv.get(1).is_none());
}
#[test]
fn unlimited_staff_and_original_pouch_only_transaction_failure() {
    let mut config = config();
    config.catalog.low.insert(1, [10].into_iter().collect());
    let spell = Spell {
        item: 100,
        requirements: vec![Requirement {
            item: 1,
            count: 2,
            worn_slot: None,
        }],
    };
    let mut inv = Inventory::new(28, StackType::Normal, false);
    let mut worn = Inventory::new(14, StackType::Normal, false);
    worn.restore_slot(3, item(10, 1)).unwrap();
    let mut vars = Variables::default();
    let items = Catalog::default();
    let result = attempt(&config, request(&spell), &mut inv, &worn, &mut vars, &items).unwrap();
    assert_eq!(
        result.kind,
        ResultKindCast::Consumption(ResultKind::AllUnlimited)
    );
    assert!(result.success());
    assert!(!result.used_sunfire());
    worn.restore_slot(3, None).unwrap();
    config.regular_pouches.insert(20);
    config.catalog.compact.insert(1, 1);
    config.pouch_types = [binding(1); 4];
    config.pouch_counts = [(2, binding(2)); 4];
    inv.restore_slot(0, item(20, 1)).unwrap();
    vars = Variables::restore([(1, 1), (2, 5)].into_iter().collect());
    let result = attempt(&config, request(&spell), &mut inv, &worn, &mut vars, &items).unwrap();
    assert_eq!(
        result.kind,
        ResultKindCast::Consumption(ResultKind::NotEnoughInventory)
    );
    assert!(!result.success());
    assert_eq!(vars.get(2), 5);
}
