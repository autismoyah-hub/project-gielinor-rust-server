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
