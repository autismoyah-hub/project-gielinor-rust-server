//! Resolve content's symbolic references against the same immutable type inputs as Kotlin.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_world::food::{self, Edible};
use std::collections::BTreeMap;
/// Resolve both stat requirement pairs from enriched cache parameters. Level
/// parameters are signed Kotlin Ints; do not truncate them to a byte.
pub fn equipment(
    param_symbols: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::equipment::WearCatalog, String> {
    use gielinor_cache::definitions::Param;
    use gielinor_world::equipment::{WearCatalog, WearType};
    let mut pairs = Vec::new();
    for n in [1, 2] {
        let skill = *param_symbols
            .get(&format!("statreq{n}_skill"))
            .ok_or("Missing equipment skill parameter")? as u32;
        let level = *param_symbols
            .get(&format!("statreq{n}_level"))
            .ok_or("Missing equipment level parameter")? as u32;
        pairs.push((skill, level));
    }
    let mut catalog = WearCatalog::default();
    for (&id, ty) in &defs.items {
        let mut requirements = Vec::new();
        for &(skill, level) in &pairs {
            let Some(value) = ty.params.get(&skill) else {
                continue;
            };
            let Param::Integer(skill) = value else {
                return Err(format!("Item {id} has noninteger skill requirement"));
            };
            let skill =
                u8::try_from(*skill).map_err(|_| format!("Item {id} has invalid skill ID"))?;
            if !defs.stats.contains_key(&(skill as u32)) {
                return Err(format!("Item {id} references missing stat {skill}"));
            }
            let level = match ty.params.get(&level) {
                None => 0,
                Some(Param::Integer(level)) => *level,
                _ => return Err(format!("Item {id} has noninteger level requirement")),
            };
            requirements.push((skill, level));
        }
        let slot = |value: i8| usize::try_from(value).ok().filter(|slot| *slot < 14);
        catalog.types.insert(
            id as i32,
            WearType {
                primary: slot(ty.wear[0]),
                secondary: [slot(ty.wear[1]), slot(ty.wear[2])],
                requirements,
            },
        );
    }
    Ok(catalog)
}
/// Independently Wiki-pinned ordinary packs. Missing optional symbols are not
/// enabled; present profiles must have a valid operation and output/note pair.
pub fn item_packs(
    symbols: &Symbols,
    defs: &Definitions,
) -> Result<BTreeMap<i32, gielinor_world::item_packs::Rule>, String> {
    let mut rules = BTreeMap::new();
    for (pack_name, output_name, noted) in [
        ("pack_feather", "feather", false),
        ("pack_fishing_bait", "fishing_bait", false),
        ("pack_bucket", "bucket_empty", true),
        ("pack_jug_empty", "jug_empty", true),
        ("pack_vial_empty", "vial_empty", true),
        ("pack_vial_water", "vial_water", true),
        ("pack_compost", "bucket_compost", true),
        // Wiki plant-pot packs contain filled pots; do not copy the reference
        // enum's plantpot_empty symbol without effective-definition evidence.
        ("pack_plant_pot_compost", "plantpot_compost", true),
        ("pack_sack", "sack_empty", true),
        ("pack_basket", "basket_empty", true),
        ("pack_olive_oil", "oliveoil4", true),
        ("pack_airrune", "airrune", false),
        ("pack_waterrune", "waterrune", false),
        ("pack_earthrune", "earthrune", false),
        ("pack_firerune", "firerune", false),
        ("pack_mindrune", "mindrune", false),
        ("pack_chaosrune", "chaosrune", false),
    ] {
        let Some(&pack) = symbols.get(pack_name) else {
            continue;
        };
        let &output = symbols
            .get(output_name)
            .ok_or_else(|| format!("Missing pack output {output_name}"))?;
        let pack_type = defs
            .items
            .get(&(pack as u32))
            .ok_or_else(|| format!("Missing pack definition {pack_name}"))?;
        let output_type = defs
            .items
            .get(&(output as u32))
            .ok_or_else(|| format!("Missing pack output definition {output_name}"))?;
        if pack_type.cert.template != 0
            || pack_type.placeholder.template != 0
            || pack_type.inventory_ops[0]
                .as_deref()
                .is_none_or(|op| !op.eq_ignore_ascii_case("Open"))
            || output_type.cert.template != 0
            || output_type.placeholder.template != 0
            || output_type.stackable == noted
            || output == pack
        {
            return Err(format!("Invalid pack/output profile {pack_name}"));
        }
        if noted {
            let note_id = u32::try_from(output_type.cert.link).map_err(|_| "Invalid note ID")?;
            let note = defs
                .items
                .get(&note_id)
                .ok_or_else(|| format!("Missing pack output note {output_name}"))?;
            if note_id == 0 || note.cert.template == 0 || note.cert.link != output {
                return Err(format!("Invalid reciprocal note mapping {output_name}"));
            }
        }
        if rules
            .insert(
                pack,
                gielinor_world::item_packs::Rule {
                    output,
                    count: 100,
                    noted,
                },
            )
            .is_some()
        {
            return Err(format!("Duplicate pack binding {pack_name}"));
        }
    }
    Ok(rules)
}
pub fn foods(symbols: &Symbols, defs: &Definitions) -> Result<BTreeMap<i32, Edible>, String> {
    let resolve = |name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing food symbol {name}"))
    };
    let mut foods = BTreeMap::new();
    for food in food::catalog() {
        let id = resolve(food.symbol)?;
        let ty = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing food type {id}"))?;
        let replacement = food.replacement.map(resolve).transpose()?;
        if replacement.is_some_and(|id| !defs.items.contains_key(&(id as u32))) {
            return Err("Missing replacement type".into());
        }
        let message = food
            .message
            .unwrap_or_else(|| format!("You eat the {}.", ty.name.to_lowercase()));
        if foods
            .insert(
                id,
                Edible {
                    id,
                    heal: food.heal,
                    delay: food.delay,
                    replacement,
                    message,
                },
            )
            .is_some()
        {
            return Err(format!("Duplicate resolved food ID {id}"));
        }
    }
    Ok(foods)
}
/// GR-24's bounded cake/plain-pizza correction. Other foods retain the reference
/// profile; selected IDs must be unnoted, nonstackable, ordinary Eat definitions.
pub fn wiki_multipart_foods(
    symbols: &Symbols,
    defs: &Definitions,
    foods: &mut BTreeMap<i32, Edible>,
) -> Result<std::collections::BTreeSet<i32>, String> {
    let mut next = foods.clone();
    let mut selected = std::collections::BTreeSet::new();
    let rows = [
        ("cake", Some("partial_cake"), 4),
        ("partial_cake", Some("cake_slice"), 4),
        ("cake_slice", None, 4),
        ("plain_pizza", Some("half_plain_pizza"), 7),
        ("half_plain_pizza", None, 7),
    ];
    for (symbol, replacement, heal) in rows {
        let id = *symbols.get(symbol).ok_or("Missing multipart food symbol")?;
        let item = u32::try_from(id)
            .ok()
            .and_then(|id| defs.items.get(&id))
            .ok_or("Missing multipart food definition")?;
        if item.stackable
            || item.dummy == 1
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.inventory_ops[0].as_deref() != Some("Eat")
            || !selected.insert(id)
        {
            return Err("Invalid multipart food definition".into());
        }
        let food = next
            .get_mut(&id)
            .ok_or("Multipart food handler is absent")?;
        food.heal = heal;
        food.replacement = replacement
            .map(|name| {
                symbols
                    .get(name)
                    .copied()
                    .ok_or("Missing multipart remainder symbol")
            })
            .transpose()?;
    }
    *foods = next;
    Ok(selected)
}

#[cfg(test)]
mod multipart_tests {
    use super::*;

    #[test]
    fn selected_food_binding_validates_every_stage_before_changing_catalog() {
        let symbols: Symbols = [
            ("cake", 1891),
            ("partial_cake", 1893),
            ("cake_slice", 1895),
            ("plain_pizza", 2289),
            ("half_plain_pizza", 2291),
        ]
        .into_iter()
        .map(|(name, id)| (name.into(), id))
        .collect();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut foods = BTreeMap::new();
        for &id in symbols.values() {
            let mut item = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            item.inventory_ops[0] = Some("Eat".into());
            defs.items.insert(id as u32, item);
            foods.insert(
                id,
                Edible {
                    id,
                    heal: 1,
                    delay: 3,
                    replacement: None,
                    message: "Fixture".into(),
                },
            );
        }
        defs.items.get_mut(&1895).unwrap().stackable = true;
        assert!(wiki_multipart_foods(&symbols, &defs, &mut foods).is_err());
        assert!(foods
            .values()
            .all(|food| food.heal == 1 && food.replacement.is_none()));
        defs.items.get_mut(&1895).unwrap().stackable = false;
        assert_eq!(
            wiki_multipart_foods(&symbols, &defs, &mut foods)
                .unwrap()
                .len(),
            5
        );
        for (id, replacement, heal) in [
            (1891, Some(1893), 4),
            (1893, Some(1895), 4),
            (1895, None, 4),
            (2289, Some(2291), 7),
            (2291, None, 7),
        ] {
            assert_eq!(
                (foods[&id].replacement, foods[&id].heal),
                (replacement, heal)
            );
        }
    }
}
pub fn banking(
    varbit_symbols: &Symbols,
    param_symbols: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::bank_actions::Config, String> {
    use gielinor_world::bank_actions::{Binding, Config};
    let binding = |name: &str| -> Result<Binding, String> {
        let id = *varbit_symbols
            .get(name)
            .ok_or_else(|| format!("Missing bank symbol {name}"))? as u32;
        let bit = *defs
            .varbits
            .get(&id)
            .ok_or_else(|| format!("Missing bank varbit {id}"))?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| format!("Missing bank base varp {}", bit.base))?;
        bit.extract(0).map_err(|e| format!("{e:?}"))?;
        Ok(Binding { base, bit })
    };
    let tabs: Vec<_> = (1..=9)
        .map(|i| binding(&format!("bank_tab_{i}")))
        .chain(std::iter::once(binding("bank_tab_main")))
        .collect::<Result<_, _>>()?;
    let no_bank = *param_symbols
        .get("no_bank")
        .ok_or("Missing no_bank parameter")? as u32;
    let mut restricted = std::collections::HashSet::new();
    for (&id, item) in &defs.items {
        match item.params.get(&no_bank) {
            Some(gielinor_cache::definitions::Param::Integer(value)) if *value != 0 => {
                restricted.insert(id as i32);
            }
            Some(gielinor_cache::definitions::Param::String(_)) => {
                return Err("Noninteger no_bank parameter".into())
            }
            _ => (),
        }
    }
    Ok(Config {
        sidebar: None,
        tabs: tabs.try_into().unwrap(),
        selected: binding("bank_currenttab")?,
        capacity: binding("bank_capacity")?,
        notes: binding("bank_withdrawnotes")?,
        placeholders: binding("bank_leaveplaceholders")?,
        quantity: binding("bank_quantity_type")?,
        last_quantity: binding("bank_requestedquantity")?,
        restricted,
    })
}
/// BankSettingsScript controls; identifiers come from the same cache symbols as Kotlin.
pub fn bank_sidebar(
    param_symbols: &Symbols,
    varp_symbols: &Symbols,
    content_symbols: &Symbols,
    parameter_types: &BTreeMap<u32, gielinor_cache::parameters::ParameterType>,
    defs: &Definitions,
) -> Result<gielinor_world::bank_sidebar::Sidebar, String> {
    use gielinor_cache::definitions::Param;
    use gielinor_world::bank_sidebar::{ItemOptions, Sidebar, Special};
    let param = |name: &str| {
        param_symbols
            .get(name)
            .map(|id| *id as u32)
            .ok_or_else(|| format!("Missing parameter {name}"))
    };
    let bit = param("bankside_extraop_conditional_bit")?;
    let condition = param("bankside_extraop_conditional_varbit")?;
    let flip = param("bankside_extraop_conditional_flip")?;
    let text = param("bankside_extraop")?;
    let wear_index = param("wear_op_index")?;
    let food = *content_symbols
        .get("food")
        .ok_or("Missing food content group")?;
    let potion = *content_symbols
        .get("potion")
        .ok_or("Missing potion content group")?;
    let outputs: Vec<_> = ["if1", "if2", "if3"]
        .into_iter()
        .map(|name| {
            let id = *varp_symbols
                .get(name)
                .ok_or_else(|| format!("Missing varp {name}"))? as u32;
            defs.varps
                .get(&id)
                .copied()
                .ok_or_else(|| format!("Missing varp {id}"))
        })
        .collect::<Result<_, _>>()?;
    let mut items = BTreeMap::new();
    for (&id, item) in &defs.items {
        let int = |key| -> Result<Option<i32>, String> {
            match item.params.get(&key) {
                Some(Param::Integer(value)) => Ok(Some(*value)),
                Some(_) => Err(format!("Noninteger parameter {key} on {id}")),
                None => Ok(parameter_types.get(&key).and_then(|p| p.default_int)),
            }
        };
        let special = if let Some(bit) = int(bit)? {
            let varbit = int(condition)?
                .ok_or_else(|| format!("Missing conditional bank varbit on {id}"))?
                as u32;
            Some(Special {
                bit,
                condition: *defs
                    .varbits
                    .get(&varbit)
                    .ok_or_else(|| format!("Unknown conditional varbit {varbit}"))?,
                flip: int(flip)?.unwrap_or(0) != 0,
            })
        } else {
            None
        };
        let extra_text = match item.params.get(&text) {
            Some(Param::String(value)) => value.as_str(),
            Some(_) => return Err(format!("Nonstring bank operation on {id}")),
            None => parameter_types
                .get(&text)
                .and_then(|p| p.default_string.as_deref())
                .unwrap_or(""),
        };
        // Kotlin Char.isWhitespace includes separators 1C..1F and excludes NEL.
        let blank = |text: &str| {
            text.chars()
                .all(|c| c.is_whitespace() && c != '\u{85}' || ('\u{1c}'..='\u{1f}').contains(&c))
        };
        let wear = if item.wear[0] != -1 {
            let index = int(wear_index)?.ok_or("Missing wear operation index default")?;
            index
                .checked_sub(1)
                .and_then(|i| usize::try_from(i).ok())
                .and_then(|i| item.inventory_ops.get(i))
                .and_then(Option::as_deref)
                .is_some_and(|text| !blank(text))
        } else {
            false
        };
        items.insert(
            id as i32,
            ItemOptions {
                special,
                custom_text: !blank(extra_text),
                wear,
                consumable: item.content_group == food || item.content_group == potion,
            },
        );
    }
    Ok(Sidebar {
        outputs: outputs.try_into().unwrap(),
        items,
    })
}

/// BankSettingsScript controls; identifiers come from the same cache symbols as Kotlin.
pub fn bank_controls(
    components: &gielinor_cache::symbols::Symbols,
    bits: &gielinor_cache::symbols::Symbols,
    defs: &Definitions,
) -> Result<std::collections::BTreeMap<u32, gielinor_world::bank_actions::Control>, String> {
    use gielinor_world::bank_actions::{Binding, Control};
    let binding = |name: &str| -> Result<Binding, String> {
        let id = *bits
            .get(name)
            .ok_or_else(|| format!("Missing bank symbol {name}"))? as u32;
        let bit = *defs
            .varbits
            .get(&id)
            .ok_or_else(|| format!("Missing bank varbit {id}"))?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing bank base varp")?;
        bit.extract(0).map_err(|e| format!("{e:?}"))?;
        Ok(Binding { base, bit })
    };
    let mut result = std::collections::BTreeMap::new();
    let mut insert = |name: &str, control| -> Result<(), String> {
        let name = format!("bankmain:{name}");
        let id = *components
            .get(&name)
            .ok_or_else(|| format!("Missing component {name}"))? as u32;
        if result.insert(id, control).is_some() {
            return Err("Duplicate bank control component".into());
        }
        Ok(())
    };
    for (component, bit, value) in [
        ("swap", "bank_insertmode", 0),
        ("insert", "bank_insertmode", 1),
        ("item", "bank_withdrawnotes", 0),
        ("note", "bank_withdrawnotes", 1),
        ("quantity1", "bank_quantity_type", 0),
        ("quantity5", "bank_quantity_type", 1),
        ("quantity10", "bank_quantity_type", 2),
        ("quantityall", "bank_quantity_type", 4),
        ("bank_filler_1", "bank_fillermode", 1),
        ("bank_filler_10", "bank_fillermode", 2),
        ("bank_filler_50", "bank_fillermode", 3),
        ("bank_filler_x", "bank_fillermode", 4),
        ("bank_filler_all", "bank_fillermode", 0),
    ] {
        insert(
            component,
            Control::Set {
                binding: binding(bit)?,
                value,
            },
        )?;
    }
    for (component, bit) in [
        ("placeholder", "bank_leaveplaceholders"),
        ("incinerator_toggle", "bank_showincinerator"),
        ("banktut_toggle", "bank_hidebanktut"),
        ("sideops_toggle", "bank_hidesideops"),
        ("depositinv_toggle", "bank_hidedepositinv"),
        ("depositworn_toggle", "bank_hidedepositworn"),
    ] {
        insert(
            component,
            Control::Toggle {
                binding: binding(bit)?,
                refresh_side: component == "sideops_toggle",
            },
        )?;
    }
    insert(
        "dropdown_content",
        Control::TabDisplay(binding("bank_tab_display")?),
    )?;
    insert("quantityx", Control::QuantityX)?;
    insert("tabs", Control::Tabs)?;
    insert("depositinv", Control::DepositAll { worn: false })?;
    insert("depositworn", Control::DepositAll { worn: true })?;
    Ok(result)
}

pub fn equipment_requirement_messages(
    defs: &Definitions,
    item: i32,
    missing: &[(u8, i32)],
    message_params: [u32; 2],
) -> Result<Vec<String>, String> {
    use gielinor_cache::definitions::Param;
    let ty = defs
        .items
        .get(&(item as u32))
        .ok_or("Unknown equipment item")?;
    let text = |id| match ty.params.get(&id) {
        None => Ok(None),
        Some(Param::String(value)) => Ok(Some(value.clone())),
        _ => Err("Nonstring equipment failure message".to_owned()),
    };
    if missing.is_empty() || missing.len() > 2 {
        return Err("Invalid equipment requirement count".into());
    }
    let first = text(message_params[0])?
        .unwrap_or_else(|| "You are not a high enough level to use this item.".into());
    let mut second = text(message_params[1])?.unwrap_or_else(|| {
        if missing.len() == 1 {
            "You need to have {skill1} level of {level1}.".into()
        } else {
            "You need to have {skill1} level of {level1} and {skill2} level of {level2}.".into()
        }
    });
    for (i, &(skill, level)) in missing.iter().enumerate() {
        let name = &defs
            .stats
            .get(&(skill as u32))
            .ok_or("Unknown requirement stat")?
            .name;
        let first = name
            .chars()
            .next()
            .ok_or("Empty requirement stat name")?
            .to_ascii_lowercase();
        let article = if "aeiou".contains(first) { "an" } else { "a" };
        second = second
            .replace(&format!("{{skill{}}}", i + 1), &format!("{article} {name}"))
            .replace(&format!("{{level{}}}", i + 1), &level.to_string());
    }
    Ok(vec![first, second])
}

#[cfg(test)]
mod pack_binding_tests {
    use super::*;
    #[test]
    fn additional_noted_packs_select_filled_pots_and_reciprocal_notes() {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        symbols.insert("plantpot_empty".into(), 999);
        for (index, (pack_name, output_name)) in [
            ("pack_plant_pot_compost", "plantpot_compost"),
            ("pack_sack", "sack_empty"),
            ("pack_basket", "basket_empty"),
            ("pack_olive_oil", "oliveoil4"),
        ]
        .into_iter()
        .enumerate()
        {
            let pack = 100 + index as i32;
            let output = 200 + index as i32;
            let note = 300 + index as i32;
            symbols.insert(pack_name.into(), pack);
            symbols.insert(output_name.into(), output);
            let mut pack_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            pack_type.inventory_ops[0] = Some("Open".into());
            let mut output_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            output_type.cert.link = note;
            let mut note_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            note_type.cert.template = 799;
            note_type.cert.link = output;
            defs.items.insert(pack as u32, pack_type);
            defs.items.insert(output as u32, output_type);
            defs.items.insert(note as u32, note_type);
        }
        let rules = item_packs(&symbols, &defs).unwrap();
        assert_eq!(rules.len(), 4);
        for index in 0..4 {
            let rule = rules[&(100 + index)];
            assert_eq!(
                (rule.output, rule.count, rule.noted),
                (200 + index, 100, true)
            );
        }
        assert_ne!(rules[&100].output, symbols["plantpot_empty"]);
        defs.items.get_mut(&300).unwrap().cert.link = 999;
        assert!(item_packs(&symbols, &defs).is_err());
    }
    #[test]
    fn rune_profiles_resolve_symbols_and_reject_malformed_outputs() {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        // Synthetic identities ensure registration uses Names, not revision241 IDs.
        for (index, rune) in ["air", "water", "earth", "fire", "mind", "chaos"]
            .into_iter()
            .enumerate()
        {
            let pack = 10000 + index as i32;
            let output = 20000 + index as i32;
            symbols.insert(format!("pack_{rune}rune"), pack);
            symbols.insert(format!("{rune}rune"), output);
            let mut pack_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            pack_type.inventory_ops[0] = Some("Open".into());
            let mut output_type = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
            output_type.stackable = true;
            defs.items.insert(pack as u32, pack_type);
            defs.items.insert(output as u32, output_type);
        }
        let rules = item_packs(&symbols, &defs).unwrap();
        assert_eq!(rules.len(), 6);
        for index in 0..6 {
            let rule = rules[&(10000 + index)];
            assert_eq!(
                (rule.output, rule.count, rule.noted),
                (20000 + index, 100, false)
            );
        }
        defs.items.get_mut(&20005).unwrap().placeholder.template = 1;
        assert!(item_packs(&symbols, &defs).is_err());
        defs.items.get_mut(&20005).unwrap().placeholder.template = 0;
        symbols.remove("chaosrune");
        assert!(item_packs(&symbols, &defs).is_err());
    }
    #[test]
    fn pack_registration_requires_open_operation_and_real_stackable_output() {
        let symbols: Symbols = [("pack_feather".into(), 11881), ("feather".into(), 314)]
            .into_iter()
            .collect();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut pack = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        pack.inventory_ops[0] = Some("Open".into());
        let mut feather = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        feather.stackable = true;
        defs.items.insert(11881, pack);
        defs.items.insert(314, feather);
        assert_eq!(item_packs(&symbols, &defs).unwrap()[&11881].count, 100);
        defs.items.get_mut(&314).unwrap().stackable = false;
        assert!(item_packs(&symbols, &defs).is_err());
        defs.items.get_mut(&314).unwrap().stackable = true;
        defs.items.get_mut(&11881).unwrap().inventory_ops[0] = None;
        assert!(item_packs(&symbols, &defs).is_err());
        assert!(item_packs(&Symbols::new(), &defs).unwrap().is_empty());
        let symbols: Symbols = [("pack_bucket".into(), 22660), ("bucket_empty".into(), 1925)]
            .into_iter()
            .collect();
        let mut pack = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        pack.inventory_ops[0] = Some("Open".into());
        defs.items.insert(22660, pack);
        let mut bucket = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        bucket.cert.link = 1926;
        defs.items.insert(1925, bucket);
        assert!(item_packs(&symbols, &defs).is_err());
        let mut note = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        note.cert.template = 799;
        note.cert.link = 1925;
        defs.items.insert(1926, note);
        assert!(item_packs(&symbols, &defs).unwrap()[&22660].noted);
        defs.items.get_mut(&1926).unwrap().cert.link = 1935;
        assert!(item_packs(&symbols, &defs).is_err());
    }
}

/// Wiki-pinned assembly and topping awards; target Names and atomic mutations.
pub fn assembly(
    symbols: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<Vec<gielinor_world::assembly::Recipe>, String> {
    let mut recipes = Vec::new();
    for (first, second, output, level, base_xp, members) in [
        ("pastry_dough", "piedish", "pie_shell", 1, 0, false),
        (
            "pie_shell",
            "redberries",
            "uncooked_redberry_pie",
            10,
            0,
            false,
        ),
        (
            "pie_shell",
            "cooked_meat",
            "uncooked_meat_pie",
            20,
            0,
            false,
        ),
        (
            "pie_shell",
            "cooked_chicken",
            "uncooked_meat_pie",
            20,
            0,
            false,
        ),
        (
            "pie_shell",
            "cooking_apple",
            "uncooked_apple_pie",
            30,
            0,
            false,
        ),
        (
            "pie_shell",
            "golovanova_top",
            "uncooked_botanical_pie",
            52,
            0,
            true,
        ),
        (
            "pie_shell",
            "fossil_sulliuscep_cap",
            "uncooked_mushroom_pie",
            60,
            0,
            true,
        ),
        (
            "pie_shell",
            "dragonfruit",
            "uncooked_dragonfruit_pie",
            73,
            0,
            true,
        ),
        (
            "pie_shell",
            "tomato",
            "unfinished_garden_pie_1",
            34,
            0,
            true,
        ),
        (
            "unfinished_garden_pie_1",
            "onion",
            "unfinished_garden_pie_2",
            34,
            0,
            true,
        ),
        (
            "unfinished_garden_pie_2",
            "cabbage",
            "uncooked_garden_pie",
            34,
            0,
            true,
        ),
        ("pie_shell", "trout", "unfinished_fish_pie_1", 47, 0, true),
        (
            "unfinished_fish_pie_1",
            "cod",
            "unfinished_fish_pie_2",
            47,
            0,
            true,
        ),
        (
            "unfinished_fish_pie_2",
            "potato",
            "uncooked_fish_pie",
            47,
            0,
            true,
        ),
        (
            "pie_shell",
            "raw_bear_meat",
            "unfinished_wild_pie_1",
            85,
            0,
            true,
        ),
        (
            "unfinished_wild_pie_1",
            "raw_chompy",
            "unfinished_wild_pie_2",
            85,
            0,
            true,
        ),
        (
            "unfinished_wild_pie_2",
            "raw_rabbit",
            "uncooked_wild_pie",
            85,
            0,
            true,
        ),
        (
            "pie_shell",
            "strawberry",
            "unfinished_summer_pie_1",
            95,
            0,
            true,
        ),
        (
            "unfinished_summer_pie_1",
            "watermelon",
            "unfinished_summer_pie_2",
            95,
            0,
            true,
        ),
        (
            "unfinished_summer_pie_2",
            "cooking_apple",
            "uncooked_summer_pie",
            95,
            0,
            true,
        ),
        ("pizza_base", "tomato", "incomplete_pizza", 35, 0, false),
        ("incomplete_pizza", "cheese", "uncooked_pizza", 35, 0, false),
        ("plain_pizza", "cooked_meat", "meat_pizza", 45, 26, false),
        ("plain_pizza", "cooked_chicken", "meat_pizza", 45, 26, false),
        ("plain_pizza", "anchovies", "anchovie_pizza", 55, 39, false),
        (
            "plain_pizza",
            "pineapple_chunks",
            "pineapple_pizza",
            65,
            45,
            true,
        ),
        (
            "plain_pizza",
            "pineapple_ring",
            "pineapple_pizza",
            65,
            45,
            true,
        ),
    ] {
        let resolve = |name: &str| -> Result<i32, String> {
            let id = *symbols
                .get(name)
                .ok_or_else(|| format!("Missing assembly symbol {name}"))?;
            let ty = defs
                .items
                .get(&(id as u32))
                .ok_or_else(|| format!("Missing assembly definition {name}"))?;
            if ty.cert.template != 0
                || ty.placeholder.template != 0
                || ty.stackable
                || ty.dummy == 1
            {
                return Err(format!("Invalid assembly profile {name}"));
            }
            Ok(id)
        };
        let first = resolve(first)?;
        let second = resolve(second)?;
        let output = resolve(output)?;
        if first == second || first == output || second == output {
            return Err("Aliased assembly ingredients/output".into());
        }
        recipes.push(gielinor_world::assembly::Recipe {
            first,
            second,
            output,
            skill,
            level,
            base_xp,
            members,
        });
    }
    Ok(recipes)
}

#[cfg(test)]
mod assembly_binding_tests {
    use super::*;
    #[test]
    fn assembly_profiles_require_real_unnoted_definitions() {
        let names = [
            "pastry_dough",
            "piedish",
            "pie_shell",
            "redberries",
            "uncooked_redberry_pie",
            "cooked_meat",
            "cooked_chicken",
            "cooking_apple",
            "uncooked_meat_pie",
            "uncooked_apple_pie",
            "golovanova_top",
            "uncooked_botanical_pie",
            "fossil_sulliuscep_cap",
            "uncooked_mushroom_pie",
            "dragonfruit",
            "uncooked_dragonfruit_pie",
            "onion",
            "cabbage",
            "unfinished_garden_pie_1",
            "unfinished_garden_pie_2",
            "uncooked_garden_pie",
            "trout",
            "cod",
            "potato",
            "unfinished_fish_pie_1",
            "unfinished_fish_pie_2",
            "uncooked_fish_pie",
            "raw_bear_meat",
            "raw_chompy",
            "raw_rabbit",
            "unfinished_wild_pie_1",
            "unfinished_wild_pie_2",
            "uncooked_wild_pie",
            "strawberry",
            "watermelon",
            "unfinished_summer_pie_1",
            "unfinished_summer_pie_2",
            "uncooked_summer_pie",
            "plain_pizza",
            "meat_pizza",
            "anchovies",
            "anchovie_pizza",
            "pineapple_chunks",
            "pineapple_ring",
            "pineapple_pizza",
            "pizza_base",
            "tomato",
            "incomplete_pizza",
            "cheese",
            "uncooked_pizza",
        ];
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (index, name) in names.iter().enumerate() {
            let id = index as i32 + 1;
            symbols.insert((*name).into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        let recipes = assembly(&symbols, &defs, 7).unwrap();
        assert_eq!(
            recipes.iter().map(|r| r.level).collect::<Vec<_>>(),
            [
                1, 10, 20, 20, 30, 52, 60, 73, 34, 34, 34, 47, 47, 47, 85, 85, 85, 95, 95, 95, 35,
                35, 45, 45, 55, 65, 65
            ]
        );
        for (ingredient, output, level, xp) in [
            ("cooked_meat", "meat_pizza", 45, 26),
            ("cooked_chicken", "meat_pizza", 45, 26),
            ("anchovies", "anchovie_pizza", 55, 39),
        ] {
            let recipe = recipes
                .iter()
                .find(|r| r.first == symbols["plain_pizza"] && r.second == symbols[ingredient])
                .unwrap();
            assert_eq!(
                (recipe.output, recipe.level, recipe.base_xp),
                (symbols[output], level, xp)
            );
        }
        assert!(
            recipes
                .iter()
                .filter(|r| r.first != symbols["plain_pizza"])
                .all(|r| r.base_xp == 0)
        );
        for input in ["pineapple_chunks", "pineapple_ring"] {
            let recipe = recipes
                .iter()
                .find(|r| r.first == symbols["plain_pizza"] && r.second == symbols[input])
                .unwrap();
            assert_eq!(
                (recipe.output, recipe.level, recipe.base_xp, recipe.members),
                (symbols["pineapple_pizza"], 65, 45, true)
            );
        }
        assert_eq!(recipes.iter().filter(|r| r.members).count(), 17);
        // Filling admission uses live boosts and membership; assembly gives no
        // baking XP. Unrelated input-gathering requirements are not pie gates.
        for (ingredient, output, level, member) in [
            ("cooked_meat", "uncooked_meat_pie", 20, false),
            ("cooked_chicken", "uncooked_meat_pie", 20, false),
            ("cooking_apple", "uncooked_apple_pie", 30, false),
            ("golovanova_top", "uncooked_botanical_pie", 52, true),
            ("fossil_sulliuscep_cap", "uncooked_mushroom_pie", 60, true),
            ("dragonfruit", "uncooked_dragonfruit_pie", 73, true),
        ] {
            use gielinor_world::{
                assembly::{Job, Outcome},
                inventory::{Catalog, Inventory, Item, StackType},
                stats::Stats,
            };
            let recipe = *recipes
                .iter()
                .find(|r| r.second == symbols[ingredient])
                .unwrap();
            assert_eq!(recipe.first, symbols["pie_shell"]);
            assert_eq!(recipe.output, symbols[output]);
            assert_eq!(recipe.members, member);
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.first), (1, recipe.second)] {
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
            let original = inv.clone();
            let mut stats = Stats::default();
            let job = Job {
                recipe,
                first_slot: 0,
                second_slot: 1,
            };
            if member {
                stats.set_current(7, level);
                let before = stats.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, false),
                    (Outcome::Members, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, before);
            }
            for low in [-1, level - 1] {
                stats.set_current(7, low);
                assert_eq!(
                    job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, member)
                        .0,
                    Outcome::Level
                );
                assert_eq!(inv, original);
            }
            stats.set_current(7, level);
            assert_eq!(stats.base(7), 1);
            let before = stats.clone();
            assert_eq!(
                job.complete_awarding(&mut inv, &Catalog::default(), &mut stats, 500, member)
                    .0,
                Outcome::Made
            );
            assert_eq!(
                inv.slots()
                    .iter()
                    .flatten()
                    .map(|i| (i.id, i.count))
                    .collect::<Vec<_>>(),
                vec![(symbols[output], 1)]
            );
            assert_eq!(stats.fine_xp(7), 0);
            assert_eq!(stats, before);
        }
        // Complete each three-stage chain from a full backpack. Every stage
        // independently rechecks membership/boost and commits no baking XP.
        for (ingredients, outputs, level) in [
            (
                ["raw_bear_meat", "raw_chompy", "raw_rabbit"],
                [
                    "unfinished_wild_pie_1",
                    "unfinished_wild_pie_2",
                    "uncooked_wild_pie",
                ],
                85,
            ),
            (
                ["strawberry", "watermelon", "cooking_apple"],
                [
                    "unfinished_summer_pie_1",
                    "unfinished_summer_pie_2",
                    "uncooked_summer_pie",
                ],
                95,
            ),
            (
                ["tomato", "onion", "cabbage"],
                [
                    "unfinished_garden_pie_1",
                    "unfinished_garden_pie_2",
                    "uncooked_garden_pie",
                ],
                34,
            ),
            (
                ["trout", "cod", "potato"],
                [
                    "unfinished_fish_pie_1",
                    "unfinished_fish_pie_2",
                    "uncooked_fish_pie",
                ],
                47,
            ),
        ] {
            use gielinor_world::{
                assembly::{Job, Outcome},
                inventory::{Catalog, Inventory, Item, StackType},
                stats::Stats,
            };
            let mut inv = Inventory::new(5, StackType::Normal, false);
            for (slot, id) in std::iter::once(symbols["pie_shell"])
                .chain(ingredients.iter().map(|n| symbols[*n]))
                .enumerate()
            {
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
            inv.restore_slot(
                4,
                Some(Item {
                    id: 995,
                    count: 1000,
                    vars: 0,
                }),
            )
            .unwrap();
            let coin = inv.get(4);
            let mut stats = Stats::default();
            let mut catalog = Catalog::default();
            catalog.stackable.insert(995);
            let mut base = symbols["pie_shell"];
            for stage in 0..3 {
                let recipe = *recipes
                    .iter()
                    .find(|r| r.first == base && r.second == symbols[ingredients[stage]])
                    .unwrap();
                assert_eq!(
                    (recipe.output, recipe.level, recipe.base_xp, recipe.members),
                    (symbols[outputs[stage]], level as u8, 0, true)
                );
                let job = Job {
                    recipe,
                    first_slot: 0,
                    second_slot: stage + 1,
                };
                stats.set_current(7, level);
                let original = inv.clone();
                let skills = stats.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, false),
                    (Outcome::Members, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, skills);
                stats.set_current(7, level - 1);
                let low = stats.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Level, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, low);
                stats.set_current(7, level);
                catalog.dummy.insert(recipe.output);
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Capacity, None)
                );
                assert_eq!(inv, original);
                assert_eq!(stats, skills);
                catalog.dummy.remove(&recipe.output);
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Made, None)
                );
                assert_eq!(stats, skills);
                assert_eq!(stats.base(7), 1);
                assert_eq!(
                    inv.get(0),
                    Some(Item {
                        id: recipe.output,
                        count: 1,
                        vars: 0
                    })
                );
                assert_eq!(inv.get(4), coin);
                for slot in 1..=stage + 1 {
                    assert!(inv.get(slot).is_none());
                }
                for slot in stage + 2..4 {
                    assert_eq!(inv.get(slot), original.get(slot));
                }
                let committed = inv.clone();
                assert_eq!(
                    job.complete_awarding(&mut inv, &catalog, &mut stats, 500, true),
                    (Outcome::Stale, None)
                );
                assert_eq!(inv, committed);
                assert_eq!(stats, skills);
                base = recipe.output;
            }
        }
        for (name, alias) in [
            ("pineapple_chunks", "plain_pizza"),
            ("pineapple_pizza", "plain_pizza"),
            ("pineapple_pizza", "pineapple_chunks"),
        ] {
            let original = symbols[name];
            let alias = symbols[alias];
            symbols.insert(name.into(), alias);
            assert!(assembly(&symbols, &defs, 7).is_err());
            symbols.insert(name.into(), original);
        }
        let output = symbols["pineapple_pizza"] as u32;
        defs.items.get_mut(&output).unwrap().dummy = 1;
        assert!(assembly(&symbols, &defs, 7).is_err());
        defs.items.get_mut(&output).unwrap().dummy = -1;
        assert!(assembly(&symbols, &defs, 7).is_ok());
        defs.items.get_mut(&1).unwrap().cert.template = 799;
        assert!(assembly(&symbols, &defs, 7).is_err());
        defs.items.get_mut(&1).unwrap().cert.template = 0;
        symbols.remove("tomato");
        assert!(assembly(&symbols, &defs, 7).is_err());
    }
}

/// Ordinary bucket profile. Numeric60788 was independently attested in the
/// prepared239.1 cache (scene roots004), not imported from OpenRune241.
pub fn milking(
    obj: &Symbols,
    loc: &Symbols,
    seq: &Symbols,
    synth: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::milking::Config, String> {
    let resolve = |map: &Symbols, key: &str| {
        map.get(key)
            .copied()
            .ok_or_else(|| format!("Missing milk symbol {key}"))
    };
    let bucket = resolve(obj, "bucket_empty")?;
    let milk = resolve(obj, "bucket_milk")?;
    for id in [bucket, milk] {
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or("Missing milk item definition")?;
        if item.stackable || item.cert.template != 0 || item.placeholder.template != 0 {
            return Err("Invalid ordinary milk vessel".into());
        }
    }
    if bucket == milk {
        return Err("Milk input/output aliases".into());
    }
    let mut cows = BTreeMap::new();
    for (id, operation) in [
        (resolve(loc, "fat_cow")?, 1),
        (resolve(loc, "fairy_fat_cow")?, 2),
        (60788, 1),
    ] {
        let Some(def) = defs.locations.get(&(id as u32)) else {
            if id == 60788 {
                continue;
            }
            return Err("Missing cow definition".into());
        };
        if !def.name.eq_ignore_ascii_case("Dairy cow")
            || !def.multi.is_empty()
            || def.ops[operation as usize - 1].as_deref() != Some("Milk")
            || def.collision.width != 1
            || def.collision.length != 2
        {
            return Err(format!("Unattested cow profile {id}"));
        }
        cows.insert(id as u32, operation);
    }
    Ok(gielinor_world::milking::Config {
        cows,
        bucket,
        milk,
        animation: u16::try_from(resolve(seq, "milkit")?).map_err(|_| "Invalid milk animation")?,
        sound: u16::try_from(resolve(synth, "milk_cow")?).map_err(|_| "Invalid milk sound")?,
    })
}

#[cfg(test)]
mod milking_binding_tests {
    use super::*;
    #[test]
    fn milk_bindings_require_ordinary_vessels_and_exact_cow_operations() {
        let obj = [("bucket_empty".into(), 10), ("bucket_milk".into(), 20)].into();
        let loc = [("fat_cow".into(), 1), ("fairy_fat_cow".into(), 2)].into();
        let seq = [("milkit".into(), 2305)].into();
        let synth = [("milk_cow".into(), 372)].into();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for id in [10, 20] {
            defs.items.insert(
                id,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        for (id, op) in [(1, 0), (2, 1), (60788, 0)] {
            let mut ty = gielinor_cache::locations::LocationType::decode(&[0]).unwrap();
            ty.name = "Dairy cow".into();
            ty.collision.length = 2;
            ty.ops[op] = Some("Milk".into());
            defs.locations.insert(id, ty);
        }
        let config = milking(&obj, &loc, &seq, &synth, &defs).unwrap();
        assert_eq!(config.cows, [(1, 1), (2, 2), (60788, 1)].into());
        defs.locations.get_mut(&60788).unwrap().ops[0] = Some("Talk-to".into());
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_err());
        defs.locations.remove(&60788);
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_ok());
        defs.locations.get_mut(&1).unwrap().multi.push(2);
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_err());
        defs.locations.get_mut(&1).unwrap().multi.clear();
        defs.items.get_mut(&20).unwrap().cert.template = 1;
        assert!(milking(&obj, &loc, &seq, &synth, &defs).is_err());
    }
}

/// Narrow ordinary sink/well profiles attested in the prepared 239.1 cache.
/// Does not infer source behavior from a display name or map icon alone.
pub fn water_sources(
    obj: &Symbols,
    loc: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::water_sources::Config, String> {
    use gielinor_world::water_sources::{Config, Source, Vessel};
    let resolve = |symbols: &Symbols, name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing water-source symbol {name}"))
    };
    let mut config = Config::default();
    for (input, output, well) in [
        ("bucket_empty", "bucket_water", true),
        ("jug_empty", "jug_water", false),
        ("bowl_empty", "bowl_water", false),
        ("vial_empty", "vial_water", false),
    ] {
        let input = resolve(obj, input)?;
        let output = resolve(obj, output)?;
        if input == output {
            return Err("Water input/output aliases".into());
        }
        for id in [input, output] {
            let item = defs
                .items
                .get(&(id as u32))
                .ok_or("Missing water vessel definition")?;
            if item.stackable || item.cert.template != 0 || item.placeholder.template != 0 {
                return Err("Invalid ordinary water vessel".into());
            }
        }
        if config
            .vessels
            .insert(input, Vessel { output, well })
            .is_some()
        {
            return Err("Duplicate water vessel".into());
        }
    }
    for (symbol, kind, name, width, length) in [
        ("sink", Source::Sink, "Sink", 1, 2),
        ("sink2", Source::Sink, "Sink", 1, 2),
        ("desertwell", Source::Well, "Well", 1, 1),
        ("well", Source::Well, "Well", 2, 2),
    ] {
        let id = resolve(loc, symbol)? as u32;
        let def = defs
            .locations
            .get(&id)
            .ok_or("Missing water source definition")?;
        if def.name != name
            || !def.multi.is_empty()
            || def.collision.width != width
            || def.collision.length != length
            || def.ops.iter().any(Option::is_some)
        {
            return Err(format!("Unattested water source {symbol}"));
        }
        if config.sources.insert(id, kind).is_some() {
            return Err("Duplicate water source".into());
        }
    }
    Ok(config)
}

#[cfg(test)]
mod water_binding_tests {
    use super::*;
    #[test]
    fn names_bind_separate_source_policies_and_reject_malformed_definitions() {
        let mut obj = Symbols::new();
        let mut loc = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (index, name) in [
            "bucket_empty",
            "bucket_water",
            "jug_empty",
            "jug_water",
            "bowl_empty",
            "bowl_water",
            "vial_empty",
            "vial_water",
        ]
        .into_iter()
        .enumerate()
        {
            let id = 100 + index as i32;
            obj.insert(name.into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        for (index, (symbol, name, width, length)) in [
            ("sink", "Sink", 1, 2),
            ("sink2", "Sink", 1, 2),
            ("desertwell", "Well", 1, 1),
            ("well", "Well", 2, 2),
        ]
        .into_iter()
        .enumerate()
        {
            let id = 300 + index as i32;
            loc.insert(symbol.into(), id);
            let mut raw = vec![2];
            raw.extend_from_slice(name.as_bytes());
            raw.extend_from_slice(&[0, 14, width, 15, length, 0]);
            defs.locations.insert(
                id as u32,
                gielinor_cache::locations::LocationType::decode(&raw).unwrap(),
            );
        }
        let config = water_sources(&obj, &loc, &defs).unwrap();
        assert_eq!(config.output(300, 102), Some(103));
        assert_eq!(config.output(303, 100), Some(101));
        assert_eq!(config.output(303, 102), None);
        defs.items.get_mut(&101).unwrap().cert.template = 799;
        assert!(water_sources(&obj, &loc, &defs).is_err());
        defs.items.get_mut(&101).unwrap().cert.template = 0;
        defs.locations.get_mut(&300).unwrap().name = "Decorative sink".into();
        assert!(water_sources(&obj, &loc, &defs).is_err());
    }
}

/// Independently Wiki-pinned recipes. Other crushing-table rows remain unassessed.
pub fn grinding(obj:&Symbols,seq:&Symbols,defs:&Definitions)->Result<Vec<gielinor_world::grinding::Recipe>,String>{
    let resolve=|symbols:&Symbols,name:&str|symbols.get(name).copied().ok_or_else(||format!("Missing grinding symbol {name}"));
    let animation=u16::try_from(resolve(seq,"human_herbing_grind")?).map_err(|_|"Invalid grinding sequence")?;
    let mut pairs=std::collections::BTreeSet::new();let mut recipes=Vec::new();
    for (tool,input,output,members) in [("knife","chocolate_bar","chocolate_dust",false),("pestle_and_mortar","chocolate_bar","chocolate_dust",true),("pestle_and_mortar","unicorn_horn","unicorn_horn_dust",true)] {
        let tool=resolve(obj,tool)?;let input=resolve(obj,input)?;let output=resolve(obj,output)?;
        if tool==input || input==output || tool==output || !pairs.insert((tool,input)) {return Err("Aliased grinding recipe".into());}
        for id in [tool,input,output]{
            let item=defs.items.get(&(id as u32)).ok_or("Missing grinding item definition")?;
            if item.stackable || item.cert.template!=0 || item.placeholder.template!=0 {return Err("Invalid ordinary grinding item".into());}
        }
        recipes.push(gielinor_world::grinding::Recipe{tool,input,output,members,animation:members.then_some(animation)});
    }
    Ok(recipes)
}

#[cfg(test)]
mod grinding_binding_tests {
    use super::*;
    #[test]
    fn named_recipes_distinguish_member_tool_and_validate_items() {
        let mut obj=Symbols::new();let seq=Symbols::from([("human_herbing_grind".into(),901)]);
        let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
        for (i,name) in ["knife","pestle_and_mortar","chocolate_bar","chocolate_dust","unicorn_horn","unicorn_horn_dust"].into_iter().enumerate(){
            obj.insert(name.into(),100+i as i32);defs.items.insert(100+i as u32,gielinor_cache::definitions::ItemType::decode(&[0]).unwrap());
        }
        let recipes=grinding(&obj,&seq,&defs).unwrap();assert_eq!(recipes.len(),3);
        assert_eq!((recipes[0].tool,recipes[0].input,recipes[0].output),(100,102,103));
        assert!(!recipes[0].members);assert_eq!(recipes[0].animation,None);
        assert!(recipes[1].members);assert_eq!(recipes[1].animation,Some(901));
        assert_eq!((recipes[2].tool,recipes[2].input,recipes[2].output),(101,104,105));
        defs.items.get_mut(&103).unwrap().cert.template=799;assert!(grinding(&obj,&seq,&defs).is_err());
        defs.items.get_mut(&103).unwrap().cert.template=0;obj.insert("chocolate_dust".into(),102);assert!(grinding(&obj,&seq,&defs).is_err());
    }
}

/// Ordinary dough inputs are bound independently of tutorial progression.
pub fn dough(
    symbols: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<gielinor_world::dough::Config, String> {
    let resolve = |name: &str| -> Result<i32, String> {
        let id = *symbols
            .get(name)
            .ok_or_else(|| format!("Missing dough symbol {name}"))?;
        let ty = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing dough definition {name}"))?;
        if ty.cert.template != 0 || ty.placeholder.template != 0 || ty.stackable {
            return Err(format!("Invalid ordinary dough item {name}"));
        }
        Ok(id)
    };
    let waters = [
        ("bucket_water", "bucket_empty"),
        ("jug_water", "jug_empty"),
        ("bowl_water", "bowl_empty"),
    ]
    .into_iter()
    .map(|(w, v)| Ok((resolve(w)?, resolve(v)?)))
    .collect::<Result<Vec<_>, String>>()?;
    let config = gielinor_world::dough::Config {
        skill,
        tutorial: None,
        flour: resolve("pot_flour")?,
        pot: resolve("pot_empty")?,
        waters,
        outputs: [
            gielinor_world::dough::Output {
                id: resolve("bread_dough")?,
                level: 1,
            },
            gielinor_world::dough::Output {
                id: resolve("pastry_dough")?,
                level: 1,
            },
            gielinor_world::dough::Output {
                id: resolve("pizza_base")?,
                level: 35,
            },
        ],
    };
    let ids: Vec<_> = std::iter::once(config.flour)
        .chain(std::iter::once(config.pot))
        .chain(config.waters.iter().flat_map(|&(a, b)| [a, b]))
        .chain(config.outputs.iter().map(|v| v.id))
        .collect();
    if ids.iter().enumerate().any(|(i, id)| ids[..i].contains(id)) {
        return Err("Dough symbol alias".into());
    }
    Ok(config)
}

#[cfg(test)]
mod dough_binding_tests {
    use super::*;
    #[test]
    fn ordinary_dough_has_nine_bound_choices_and_rejects_aliases_and_notes() {
        let names = [
            "pot_flour",
            "pot_empty",
            "bucket_water",
            "bucket_empty",
            "jug_water",
            "jug_empty",
            "bowl_water",
            "bowl_empty",
            "bread_dough",
            "pastry_dough",
            "pizza_base",
        ];
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (i, name) in names.iter().enumerate() {
            let id = i as i32 + 1;
            symbols.insert((*name).into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(&[0]).unwrap(),
            );
        }
        let config = dough(&symbols, &defs, 7).unwrap();
        assert_eq!(config.outputs.map(|o| o.level), [1, 1, 35]);
        for (water, vessel) in config.waters.iter().copied() {
            for index in 0..3 {
                let r = config.recipe(water, index).unwrap();
                assert_eq!(r.vessel, vessel);
            }
        }
        defs.items.get_mut(&1).unwrap().cert.template = 799;
        assert!(dough(&symbols, &defs, 7).is_err());
        defs.items.get_mut(&1).unwrap().cert.template = 0;
        symbols.insert("pot_empty".into(), 1);
        assert!(dough(&symbols, &defs, 7).is_err());
    }
}

/// Selected manual herb profiles; automatic continuation timing remains pending.
pub fn herb_cleaning(
    symbols: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<Vec<gielinor_world::herb_cleaning::Recipe>, String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut recipes = Vec::new();
    for (input, output, level, base_fine_xp) in [
        ("unidentified_guam", "guam_leaf", 3, 25),
        ("unidentified_marentill", "marentill", 5, 38),
        ("unidentified_tarromin", "tarromin", 11, 50),
        ("unidentified_harralander", "harralander", 20, 63),
        ("unidentified_ranarr", "ranarr_weed", 25, 75),
        ("unidentified_toadflax", "toadflax", 30, 80),
        ("unidentified_irit", "irit_leaf", 40, 88),
        ("unidentified_avantoe", "avantoe", 48, 100),
        ("unidentified_kwuarm", "kwuarm", 54, 113),
        ("unidentified_huasca", "huasca", 58, 118),
        ("unidentified_snapdragon", "snapdragon", 59, 118),
        ("unidentified_cadantine", "cadantine", 65, 125),
        ("unidentified_lantadyme", "lantadyme", 67, 131),
        ("unidentified_dwarf_weed", "dwarf_weed", 70, 138),
        ("unidentified_torstol", "torstol", 75, 150),
    ] {
        let mut ids = Vec::new();
        for (name, clean) in [(input, true), (output, false)] {
            let id = *symbols
                .get(name)
                .ok_or_else(|| format!("Missing herb symbol {name}"))?;
            let item = defs
                .items
                .get(&(id as u32))
                .ok_or_else(|| format!("Missing herb definition {name}"))?;
            if id < 0
                || !seen.insert(id)
                || item.stackable
                || item.cert.template != 0
                || item.placeholder.template != 0
                || item.dummy == 1
                || !item.members
            {
                return Err(format!("Invalid ordinary herb {name}"));
            }
            if clean && item.inventory_ops[0].as_deref() != Some("Clean") {
                return Err(format!("Missing Clean operation for {name}"));
            }
            if !clean
                && item
                    .inventory_ops
                    .iter()
                    .any(|op| op.as_deref() == Some("Clean"))
            {
                return Err(format!("Clean output still has Clean operation: {name}"));
            }
            ids.push(id);
        }
        recipes.push(gielinor_world::herb_cleaning::Recipe {
            input: ids[0],
            output: ids[1],
            skill,
            level,
            base_fine_xp,
        });
    }
    Ok(recipes)
}
/// Read only the attested Druidic Ritual progress state; never infer quest completion.
pub fn herb_cleaning_config(
    objects: &Symbols,
    varps: &Symbols,
    defs: &Definitions,
    skill: u8,
) -> Result<gielinor_world::herb_cleaning::Config, String> {
    let quest = *varps
        .get("druidquest")
        .ok_or("Missing Druidic Ritual symbol")?;
    let progress = defs
        .varps
        .get(&(quest as u32))
        .ok_or("Missing Druidic Ritual progress")?;
    if quest != 80 || progress.id != 80 || !progress.permanent {
        return Err("Druidic Ritual progress differs from attested239.1 binding".into());
    }
    Ok(gielinor_world::herb_cleaning::Config {
        recipes: herb_cleaning(objects, defs, skill)?
            .into_iter()
            .map(|r| (r.input, r))
            .collect(),
        quest_progress: progress.id,
        quest_complete: 4,
    })
}
#[cfg(test)]
mod herb_binding_tests {
    use super::*;
    fn fixture() -> (Symbols, Definitions) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (i, name) in [
            "unidentified_guam",
            "guam_leaf",
            "unidentified_marentill",
            "marentill",
            "unidentified_tarromin",
            "tarromin",
            "unidentified_harralander",
            "harralander",
            "unidentified_ranarr",
            "ranarr_weed",
            "unidentified_toadflax",
            "toadflax",
            "unidentified_irit",
            "irit_leaf",
            "unidentified_avantoe",
            "avantoe",
            "unidentified_kwuarm",
            "kwuarm",
            "unidentified_huasca",
            "huasca",
            "unidentified_snapdragon",
            "snapdragon",
            "unidentified_cadantine",
            "cadantine",
            "unidentified_lantadyme",
            "lantadyme",
            "unidentified_dwarf_weed",
            "dwarf_weed",
            "unidentified_torstol",
            "torstol",
        ]
        .into_iter()
        .enumerate()
        {
            let mut raw = vec![16];
            if i % 2 == 0 {
                raw.extend_from_slice(&[35, b'C', b'l', b'e', b'a', b'n', 0]);
            }
            raw.push(0);
            symbols.insert(name.into(), 100 + i as i32);
            defs.items.insert(
                100 + i as u32,
                gielinor_cache::definitions::ItemType::decode(&raw).unwrap(),
            );
        }
        (symbols, defs)
    }
    #[test]
    fn all_normal_herbs_reject_below_level_and_award_exact_fine_xp_at_threshold() {
        use gielinor_world::{
            herb_cleaning::{Job, Outcome},
            inventory::{Catalog, Inventory, Item, StackType},
            stats::Stats,
        };
        let (symbols, defs) = fixture();
        for recipe in herb_cleaning(&symbols, &defs, 15).unwrap() {
            let mut inv = Inventory::new(1, StackType::Normal, false);
            inv.restore_slot(
                0,
                Some(Item {
                    id: recipe.input,
                    count: 1,
                    vars: 0,
                }),
            )
            .unwrap();
            let mut stats = Stats::default();
            stats.set_fine_xp(15, 100_001).unwrap();
            stats.set_current(15, recipe.level as i8 - 1);
            let original = inv.clone();
            let job = Job { recipe, slot: 0 };
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, 500, true, true)
                    .0,
                Outcome::Level
            );
            assert_eq!(inv, original);
            assert_eq!(stats.fine_xp(15), 100_001);
            stats.set_current(15, recipe.level as i8);
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, 500, true, true)
                    .0,
                Outcome::Cleaned
            );
            assert_eq!(inv.get(0).unwrap().id, recipe.output);
            assert_eq!(
                stats.fine_xp(15),
                100_001 + i32::from(recipe.base_fine_xp) * 5,
                "herb{}",
                recipe.input
            );
        }
    }
    #[test]
    fn manual_herb_permission_binding_rejects_missing_or_different_quest_state() {
        let (objects, mut defs) = fixture();
        let mut symbols = Symbols::new();
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        symbols.insert("druidquest".into(), 80);
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        let varp = gielinor_world::variables::Varp {
            id: 80,
            permanent: true,
            transmit: gielinor_world::variables::Transmit::Always,
            bit_protect: false,
            client_code: -1,
        };
        defs.varps.insert(80, varp);
        let config = herb_cleaning_config(&objects, &symbols, &defs, 15).unwrap();
        assert_eq!(
            (
                config.quest_progress,
                config.quest_complete,
                config.recipes.len()
            ),
            (80, 4, 15)
        );
        defs.varps.get_mut(&80).unwrap().permanent = false;
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        defs.varps
            .insert(80, gielinor_world::variables::Varp { id: 81, ..varp });
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
        symbols.insert("druidquest".into(), 81);
        defs.varps
            .insert(81, gielinor_world::variables::Varp { id: 81, ..varp });
        assert!(herb_cleaning_config(&objects, &symbols, &defs, 15).is_err());
    }
    #[test]
    fn herb_profiles_require_member_unnoted_definitions_and_exact_clean_operations() {
        let (symbols, defs) = fixture();
        let recipes = herb_cleaning(&symbols, &defs, 15).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| (r.input, r.output, r.level, r.base_fine_xp))
                .collect::<Vec<_>>(),
            [
                (100, 101, 3, 25),
                (102, 103, 5, 38),
                (104, 105, 11, 50),
                (106, 107, 20, 63),
                (108, 109, 25, 75),
                (110, 111, 30, 80),
                (112, 113, 40, 88),
                (114, 115, 48, 100),
                (116, 117, 54, 113),
                (118, 119, 58, 118),
                (120, 121, 59, 118),
                (122, 123, 65, 125),
                (124, 125, 67, 131),
                (126, 127, 70, 138),
                (128, 129, 75, 150),
            ]
        );
        for mode in 0..8 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => defs.items.get_mut(&100).unwrap().inventory_ops[0] = None,
                1 => defs.items.get_mut(&101).unwrap().cert.template = 799,
                2 => defs.items.get_mut(&101).unwrap().placeholder.template = 14401,
                3 => defs.items.get_mut(&101).unwrap().dummy = 1,
                4 => defs.items.get_mut(&100).unwrap().stackable = true,
                5 => defs.items.get_mut(&100).unwrap().members = false,
                6 => {
                    symbols.insert("marentill".into(), 101);
                }
                _ => defs.items.get_mut(&101).unwrap().inventory_ops[0] = Some("Clean".into()),
            }
            assert!(herb_cleaning(&symbols, &defs, 15).is_err(), "mode{mode}");
        }
    }
}

/// Fifteen ordinary water-based unfinished-potion profiles for the shared Make-X dialog.
/// Per-page Wiki evidence: openrune-unfinished-potions-profiles-001.json; no revision241 IDs reused.
pub fn unfinished_potions(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<Vec<gielinor_world::unfinished_potions::Recipe>, String> {
    let mut seen = std::collections::BTreeSet::new();
    let mut bind = |name: &str, members: bool| -> Result<i32, String> {
        let id = *objects
            .get(name)
            .ok_or_else(|| format!("Missing potion symbol {name}"))?;
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing potion item {name}"))?;
        if id < 0
            || !seen.insert(id)
            || item.stackable
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.dummy == 1
            || item.members != members
            || item
                .inventory_ops
                .iter()
                .any(|op| op.as_deref() == Some("Clean"))
        {
            return Err(format!(
                "Invalid ordinary potion ingredient or output {name}"
            ));
        }
        Ok(id)
    };
    let vessel = bind("vial_water", false)?;
    let mut recipes = Vec::new();
    for (herb, output, level) in [
        ("guam_leaf", "guamvial", 3),
        ("marentill", "marrentillvial", 5),
        ("tarromin", "tarrominvial", 12),
        ("harralander", "harralandervial", 22),
        ("ranarr_weed", "ranarrvial", 30),
        ("toadflax", "toadflaxvial", 34),
        ("irit_leaf", "iritvial", 45),
        ("avantoe", "avantoevial", 50),
        ("kwuarm", "kwuarmvial", 55),
        ("huasca", "huascavial", 58),
        ("snapdragon", "snapdragonvial", 63),
        ("cadantine", "cadantinevial", 66),
        ("lantadyme", "lantadymevial", 69),
        ("dwarf_weed", "dwarfweedvial", 72),
        ("torstol", "torstolvial", 78),
    ] {
        recipes.push(gielinor_world::unfinished_potions::Recipe {
            herb: bind(herb, true)?,
            vessel,
            output: bind(output, true)?,
            level,
        });
    }
    Ok(recipes)
}
#[cfg(test)]
mod unfinished_potion_binding_tests {
    use super::*;
    fn fixture() -> (Symbols, Definitions) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (name, id, member) in [
            ("guam_leaf", 249, true),
            ("vial_water", 227, false),
            ("guamvial", 91, true),
            ("marentill", 251, true),
            ("marrentillvial", 93, true),
            ("tarromin", 253, true),
            ("tarrominvial", 95, true),
            ("harralander", 255, true),
            ("harralandervial", 97, true),
            ("ranarr_weed", 257, true),
            ("ranarrvial", 99, true),
            ("toadflax", 2998, true),
            ("toadflaxvial", 3002, true),
            ("irit_leaf", 259, true),
            ("iritvial", 101, true),
            ("avantoe", 261, true),
            ("avantoevial", 103, true),
            ("kwuarm", 263, true),
            ("kwuarmvial", 105, true),
            ("huasca", 30097, true),
            ("huascavial", 30100, true),
            ("snapdragon", 3000, true),
            ("snapdragonvial", 3004, true),
            ("cadantine", 265, true),
            ("cadantinevial", 107, true),
            ("lantadyme", 2481, true),
            ("lantadymevial", 2483, true),
            ("dwarf_weed", 267, true),
            ("dwarfweedvial", 109, true),
            ("torstol", 269, true),
            ("torstolvial", 111, true),
        ] {
            symbols.insert(name.into(), id);
            defs.items.insert(
                id as u32,
                gielinor_cache::definitions::ItemType::decode(if member { &[16, 0] } else { &[0] })
                    .unwrap(),
            );
        }
        (symbols, defs)
    }
    #[test]
    fn named_profiles_have_zero_reward_at_boosted_threshold_and_reject_invalid_definitions() {
        use gielinor_world::{
            inventory::{Catalog, Inventory, Item, StackType},
            stats::Stats,
            unfinished_potions::{Job, Outcome},
        };
        let (symbols, defs) = fixture();
        let recipes = unfinished_potions(&symbols, &defs).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| (r.herb, r.vessel, r.output, r.level))
                .collect::<Vec<_>>(),
            [
                (249, 227, 91, 3),
                (251, 227, 93, 5),
                (253, 227, 95, 12),
                (255, 227, 97, 22),
                (257, 227, 99, 30),
                (2998, 227, 3002, 34),
                (259, 227, 101, 45),
                (261, 227, 103, 50),
                (263, 227, 105, 55),
                (30097, 227, 30100, 58),
                (3000, 227, 3004, 63),
                (265, 227, 107, 66),
                (2481, 227, 2483, 69),
                (267, 227, 109, 72),
                (269, 227, 111, 78)
            ]
        );
        for recipe in recipes {
            let mut inv = Inventory::new(2, StackType::Normal, false);
            for (slot, id) in [(0, recipe.herb), (1, recipe.vessel)] {
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
            let mut stats = Stats::default();
            stats.set_fine_xp(15, 3881).unwrap();
            stats.set_current(15, recipe.level as i8 - 1);
            let job = Job {
                recipe,
                herb_slot: 0,
                vessel_slot: 1,
            };
            let original = inv.clone();
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, true, true),
                Outcome::Level
            );
            assert_eq!(inv, original);
            stats.set_current(15, recipe.level as i8);
            let unchanged = stats.clone();
            assert_eq!(
                job.complete(&mut inv, &Catalog::default(), &mut stats, true, true),
                Outcome::Made
            );
            assert_eq!(inv.get(0).unwrap().id, recipe.output);
            assert_eq!(stats, unchanged);
        }
        for mode in 0..9 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => {
                    symbols.remove("guamvial");
                }
                1 => {
                    defs.items.remove(&91);
                }
                2 => defs.items.get_mut(&91).unwrap().stackable = true,
                3 => defs.items.get_mut(&249).unwrap().cert.template = 799,
                4 => defs.items.get_mut(&93).unwrap().placeholder.template = 14401,
                5 => defs.items.get_mut(&91).unwrap().dummy = 1,
                6 => defs.items.get_mut(&227).unwrap().members = true,
                7 => defs.items.get_mut(&249).unwrap().inventory_ops[0] = Some("Clean".into()),
                _ => {
                    symbols.insert("marrentillvial".into(), 91);
                }
            }
            assert!(unfinished_potions(&symbols, &defs).is_err(), "mode{mode}");
        }
    }
}

/// Three ordinary finished-potion profiles; runtime and equipment modifiers are separate.
/// Pinned Wiki/239.1 evidence: work/openrune-finished-potions-readiness-001.json.
pub fn finished_potions(
    objects: &Symbols,
    defs: &Definitions,
) -> Result<Vec<gielinor_world::finished_potions::Recipe>, String> {
    let mut seen = std::collections::BTreeMap::<i32, String>::new();
    let mut bind = |name: &str, members: bool, drink: bool| -> Result<i32, String> {
        let id = *objects
            .get(name)
            .ok_or_else(|| format!("Missing finished-potion symbol {name}"))?;
        let item = defs
            .items
            .get(&(id as u32))
            .ok_or_else(|| format!("Missing finished-potion definition {name}"))?;
        if id < 0
            || seen.insert(id, name.to_owned()).is_some_and(|previous| previous != name)
            || item.stackable
            || item.cert.template != 0
            || item.placeholder.template != 0
            || item.dummy == 1
            || item.members != members
            || (item.inventory_ops[0].as_deref() == Some("Drink")) != drink
            || item
                .inventory_ops
                .iter()
                .any(|op| op.as_deref() == Some("Clean"))
        {
            return Err(format!("Invalid ordinary finished-potion item {name}"));
        }
        Ok(id)
    };
    let mut result = Vec::new();
    for (unf, secondary, output, secondary_member, output_member, level, boostable, base_fine_xp) in [
        (
            "guamvial",
            "eye_of_newt",
            "3dose1attack",
            false,
            true,
            3,
            false,
            250,
        ),
        (
            "marrentillvial",
            "unicorn_horn_dust",
            "3doseantipoison",
            true,
            false,
            5,
            true,
            375,
        ),
        (
            "tarrominvial",
            "limpwurt_root",
            "3dose1strength",
            false,
            false,
            12,
            true,
            500,
        ),
        ("harralandervial", "red_spiders_eggs", "3dosestatrestore", false, true, 22, true, 625),
        ("harralandervial", "chocolate_dust", "3dose1energy", false, false, 26, true, 675),
        ("ranarrvial", "white_berries", "3dose1defense", true, true, 30, true, 750),
        ("toadflaxvial", "toads_legs", "3dose1agility", true, true, 34, true, 800),
        ("harralandervial", "ground_desert_goat_horn", "3dosecombat", true, true, 36, true, 840),
        ("ranarrvial", "snape_grass", "3doseprayerrestore", true, true, 38, true, 875),
        ("iritvial", "eye_of_newt", "3dose2attack", false, true, 45, true, 1000),
        ("iritvial", "unicorn_horn_dust", "3dose2antipoison", true, true, 48, true, 1063),
        ("avantoevial", "snape_grass", "3dosefisherspotion", true, true, 50, true, 1125),
        ("avantoevial", "mortmyremushroom", "3dose2energy", true, true, 52, true, 1175),
        ("avantoevial", "huntingbeast_sabreteeth_dust", "3dosehunting", true, true, 53, true, 1200),
        ("kwuarmvial", "limpwurt_root", "3dose2strength", false, true, 55, true, 1250),
    ] {
        result.push(gielinor_world::finished_potions::Recipe {
            unfinished: bind(unf, true, false)?,
            secondary: bind(secondary, secondary_member, false)?,
            output: bind(output, output_member, true)?,
            level,
            boostable,
            base_fine_xp,
        });
    }
    Ok(result)
}
#[cfg(test)]
mod finished_potion_binding_tests {
    use super::*;
    fn fixture() -> (Symbols, Definitions) {
        let mut symbols = Symbols::new();
        let mut defs = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        for (name, id, member, drink) in [
            ("guamvial", 91, true, false),
            ("eye_of_newt", 221, false, false),
            ("3dose1attack", 121, true, true),
            ("marrentillvial", 93, true, false),
            ("unicorn_horn_dust", 235, true, false),
            ("3doseantipoison", 175, false, true),
            ("tarrominvial", 95, true, false),
            ("limpwurt_root", 225, false, false),
            ("3dose1strength", 115, false, true),
            ("iritvial", 101, true, false),
            ("3dose2attack", 145, true, true),
            ("3dose2antipoison", 181, true, true),
            ("avantoevial", 103, true, false),
            ("3dosefisherspotion", 151, true, true),
            ("mortmyremushroom", 2970, true, false),
            ("3dose2energy", 3018, true, true),
            ("huntingbeast_sabreteeth_dust", 10111, true, false),
            ("3dosehunting", 10000, true, true),
            ("kwuarmvial", 105, true, false),
            ("3dose2strength", 157, true, true),
            ("harralandervial", 97, true, false),
            ("red_spiders_eggs", 223, false, false),
            ("3dosestatrestore", 127, true, true),
            ("chocolate_dust", 1975, false, false),
            ("3dose1energy", 3010, false, true),
            ("ranarrvial", 99, true, false),
            ("white_berries", 239, true, false),
            ("3dose1defense", 133, true, true),
            ("toadflaxvial", 3002, true, false),
            ("toads_legs", 2152, true, false),
            ("3dose1agility", 3034, true, true),
            ("ground_desert_goat_horn", 9736, true, false),
            ("3dosecombat", 9741, true, true),
            ("snape_grass", 231, true, false),
            ("3doseprayerrestore", 139, true, true),
        ] {
            symbols.insert(name.into(), id);
            let mut item =
                gielinor_cache::definitions::ItemType::decode(if member { &[16, 0] } else { &[0] })
                    .unwrap();
            if drink {
                item.inventory_ops[0] = Some("Drink".into());
            }
            defs.items.insert(id as u32, item);
        }
        (symbols, defs)
    }
    #[test]
    fn exact_profile_awards_and_target_item_membership_are_independent_of_recipe_membership() {
        let (symbols, defs) = fixture();
        let recipes = finished_potions(&symbols, &defs).unwrap();
        assert_eq!(
            recipes
                .iter()
                .map(|r| (
                    r.unfinished,
                    r.secondary,
                    r.output,
                    r.level,
                    r.boostable,
                    r.base_fine_xp
                ))
                .collect::<Vec<_>>(),
            [
                (91, 221, 121, 3, false, 250),
                (93, 235, 175, 5, true, 375),
                (95, 225, 115, 12, true, 500),
                (97, 223, 127, 22, true, 625),
                (97, 1975, 3010, 26, true, 675),
                (99, 239, 133, 30, true, 750),
                (3002, 2152, 3034, 34, true, 800),
                (97, 9736, 9741, 36, true, 840),
                (99, 231, 139, 38, true, 875),
                (101, 221, 145, 45, true, 1000),
                (101, 235, 181, 48, true, 1063),
                (103, 231, 151, 50, true, 1125),
                (103, 2970, 3018, 52, true, 1175),
                (103, 10111, 10000, 53, true, 1200),
                (105, 225, 157, 55, true, 1250),
            ]
        );
        assert!(!defs.items[&175].members && !defs.items[&115].members);
        for mode in 0..10 {
            let (mut symbols, mut defs) = fixture();
            match mode {
                0 => {
                    symbols.remove("guamvial");
                }
                1 => {
                    defs.items.remove(&121);
                }
                2 => defs.items.get_mut(&221).unwrap().stackable = true,
                3 => defs.items.get_mut(&93).unwrap().cert.template = 799,
                4 => defs.items.get_mut(&175).unwrap().placeholder.template = 14401,
                5 => defs.items.get_mut(&115).unwrap().dummy = 1,
                6 => defs.items.get_mut(&175).unwrap().members = true,
                7 => defs.items.get_mut(&121).unwrap().inventory_ops[0] = None,
                8 => defs.items.get_mut(&235).unwrap().inventory_ops[0] = Some("Clean".into()),
                _ => {
                    symbols.insert("3doseantipoison".into(), 121);
                }
            }
            assert!(finished_potions(&symbols, &defs).is_err(), "mode{mode}");
        }
    }
}

/// Two target-revision cosmetic variants of prescription goggles.
pub fn finished_potion_goggles(objects: &Symbols, defs: &Definitions) -> Result<[i32; 2], String> {
    let mut result = [0; 2];
    for (index, name) in ["mm_alchemist_hat", "mm_alchemist_hat_alt"].into_iter().enumerate() {
        let id = *objects.get(name).ok_or_else(|| format!("Missing goggles symbol {name}"))?;
        let item = defs.items.get(&(id as u32)).ok_or_else(|| format!("Missing goggles definition {name}"))?;
        if id < 0 || !item.members || item.stackable || item.cert.template != 0 || item.placeholder.template != 0
            || item.dummy == 1 || item.wear[0] != 0 || item.inventory_ops[1].as_deref() != Some("Wear") {
            return Err(format!("Invalid target goggles definition {name}"));
        }
        result[index] = id;
    }
    if result[0] == result[1] { return Err("Goggles variants alias".into()); }
    Ok(result)
}
#[cfg(test)]
mod finished_goggles_binding_tests {
    use super::*;
    #[test]
    fn goggles_require_distinct_ordinary_member_head_equipment() {
        for invalid in 0..10 {
            let mut names=Symbols::new();
            let mut defs=Definitions{inventories:Default::default(),locations:Default::default(),npcs:Default::default(),items:Default::default(),varbits:Default::default(),varps:Default::default(),stats:Default::default()};
            for (name,id) in [("mm_alchemist_hat",29974),("mm_alchemist_hat_alt",29976)] {
                names.insert(name.into(),id);
                let mut item=gielinor_cache::definitions::ItemType::decode(&[16,13,0,0]).unwrap();
                item.inventory_ops[1]=Some("Wear".into());defs.items.insert(id as u32,item);
            }
            match invalid {
                0=>{assert_eq!(finished_potion_goggles(&names,&defs).unwrap(),[29974,29976]);continue;}
                1=>{names.remove("mm_alchemist_hat");}
                2=>{names.insert("mm_alchemist_hat_alt".into(),29974);}
                3=>defs.items.get_mut(&29974).unwrap().members=false,
                4=>defs.items.get_mut(&29974).unwrap().stackable=true,
                5=>defs.items.get_mut(&29974).unwrap().cert.template=799,
                6=>defs.items.get_mut(&29974).unwrap().placeholder.template=14401,
                7=>defs.items.get_mut(&29974).unwrap().dummy=1,
                8=>defs.items.get_mut(&29974).unwrap().wear[0]=1,
                _=>defs.items.get_mut(&29974).unwrap().inventory_ops[1]=None,
            }
            assert!(finished_potion_goggles(&names,&defs).is_err(),"case{invalid}");
        }
    }
}
