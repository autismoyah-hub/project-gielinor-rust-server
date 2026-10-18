//! Citizen (Man/Woman) coin-pouch conversion; pickpocketing is separate.
use super::{Definitions, Symbols};
use gielinor_cache::definitions::Param;
use gielinor_world::item_packs::{PouchRule, Rule};
use std::collections::BTreeMap;

pub fn coin_pouches(
    objects: &Symbols,
    params: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<i32, PouchRule>, String> {
    if objects.get("pickpocket_coin_pouch_citizen") != Some(&22521)
        || objects.get("coins") != Some(&995)
    {
        return Err("Missing or aliased coin-pouch objects".into());
    }
    let no_bank = u32::try_from(*params.get("no_bank").ok_or("Missing no_bank parameter")?)
        .map_err(|_| "Invalid no_bank parameter")?;
    let pouch = defs.items.get(&22521).ok_or("Missing citizen pouch")?;
    let coins = defs.items.get(&995).ok_or("Missing coin currency")?;
    for d in [pouch, coins] {
        if !d.stackable
            || d.dummy == 1
            || d.cert.template != 0
            || d.placeholder.template != 0
            || d.transform.template != 0
        {
            return Err("Invalid ordinary pouch/currency profile".into());
        }
    }
    if pouch.name != "Coin pouch"
        || !pouch.members
        || pouch.tradeable
        || pouch.stockmarket
        || pouch.inventory_ops[0].as_deref() != Some("Open-all")
        || pouch.inventory_ops[1].as_deref() != Some("Open")
        || !matches!(pouch.params.get(&no_bank), Some(Param::Integer(1)))
        || coins.name != "Coins"
        || coins.members
    {
        return Err("Unattested citizen pouch options/restrictions/currency".into());
    }
    Ok([(
        22521,
        PouchRule {
            rule: Rule {
                output: 995,
                count: 3,
                noted: false,
            },
            members: true,
        },
    )]
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_world::{
        inventory::{Catalog, Inventory, Item, StackType},
        item_packs::open_count,
    };
    #[test]
    fn citizen_coin_pouch_binds_exact_three_coin_openings_and_no_bank_metadata() {
        let objects = Symbols::from([
            ("pickpocket_coin_pouch_citizen".into(), 22521),
            ("coins".into(), 995),
        ]);
        let params = Symbols::from([("no_bank".into(), 59)]);
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut pouch = gielinor_cache::definitions::ItemType::decode(&[11, 16, 15, 0]).unwrap();
        pouch.name = "Coin pouch".into();
        pouch.inventory_ops[0] = Some("Open-all".into());
        pouch.inventory_ops[1] = Some("Open".into());
        pouch.params.insert(59, Param::Integer(1));
        defs.items.insert(22521, pouch);
        let mut coins = gielinor_cache::definitions::ItemType::decode(&[11, 0]).unwrap();
        coins.name = "Coins".into();
        defs.items.insert(995, coins);
        let rules = coin_pouches(&objects, &params, &defs).unwrap();
        let spec = rules[&22521];
        assert!(spec.members);
        assert_eq!(
            (spec.rule.output, spec.rule.count, spec.rule.noted),
            (995, 3, false)
        );
        let mut catalog = Catalog::default();
        catalog.stackable.extend([22521, 995]);
        let mut inv = Inventory::new(2, StackType::Normal, false);
        inv.restore_slot(
            0,
            Some(Item {
                id: 22521,
                count: 28,
                vars: 0,
            }),
        )
        .unwrap();
        inv.restore_slot(
            1,
            Some(Item {
                id: 995,
                count: 100,
                vars: 0,
            }),
        )
        .unwrap();
        open_count(&mut inv, &catalog, 0, 22521, spec.rule, 28).unwrap();
        assert!(inv.get(0).is_none());
        assert_eq!(inv.get(1).unwrap().count, 184);
        defs.items.get_mut(&22521).unwrap().params.remove(&59);
        assert!(coin_pouches(&objects, &params, &defs).is_err());
        defs.items
            .get_mut(&22521)
            .unwrap()
            .params
            .insert(59, Param::Integer(1));
        defs.items.get_mut(&22521).unwrap().tradeable = true;
        assert!(coin_pouches(&objects, &params, &defs).is_err());
    }
}
