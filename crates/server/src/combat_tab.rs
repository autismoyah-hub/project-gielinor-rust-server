//! Source-resolved combat sidebar presentation and stance bindings.
use gielinor_cache::{
    enums::{EnumType, Value},
    game::Definitions,
    symbols::Symbols,
};
use gielinor_protocol::{
    client_script::{self, Argument},
    framing::Packet,
    varp,
};
use gielinor_world::{bank_actions::Binding, combat_stance, variables::Variables};
use std::collections::BTreeMap;
pub struct Presentation {
    pub interface: u16,
    pub category: Binding,
    pub level: Binding,
    pub decimal: Binding,
}
impl Presentation {
    pub fn tab(
        &self,
        vars: &mut Variables,
        weapon_category: Option<u8>,
        combat: i32,
        logged_in: bool,
    ) -> Result<Vec<Packet>, String> {
        let update = vars
            .set_bit(
                self.category.base,
                self.category.bit,
                combat_stance::category(weapon_category),
                logged_in,
            )
            .map_err(|e| format!("{e:?}"))?;
        let mut packets: Vec<_> = update
            .into_iter()
            .map(|u| varp::update(u.id, u.value))
            .collect();
        let script =
            client_script::run(5224, &[Argument::Int(combat)]).map_err(|e| format!("{e:?}"))?;
        packets.extend([script.clone(), script]);
        Ok(packets)
    }
    pub fn level(
        &self,
        vars: &mut Variables,
        whole: i32,
        decimal: i32,
        logged_in: bool,
    ) -> Result<Vec<Packet>, String> {
        let mut packets = Vec::new();
        for (binding, value) in [(self.level, whole), (self.decimal, decimal)] {
            if let Some(update) = vars
                .set_bit(binding.base, binding.bit, value, logged_in)
                .map_err(|e| format!("{e:?}"))?
            {
                packets.push(varp::update(update.id, update.value));
            }
        }
        Ok(packets)
    }
}
pub struct Config {
    pub presentation: Presentation,
    pub stance: combat_stance::Config,
    pub buttons: BTreeMap<u32, Option<u8>>,
}
pub fn resolve(
    interfaces: &Symbols,
    components: &Symbols,
    varps: &Symbols,
    bits: &Symbols,
    enums: &Symbols,
    categories: &Symbols,
    tables: &BTreeMap<u32, EnumType>,
    defs: &Definitions,
) -> Result<Config, String> {
    let symbol = |symbols: &Symbols, name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing combat tab symbol {name}"))
    };
    let binding_id = |id: u32| -> Result<Binding, String> {
        let bit = *defs.varbits.get(&id).ok_or("Missing combat varbit")?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing combat base varp")?;
        Ok(Binding { base, bit })
    };
    let binding = |name: &str| binding_id(symbol(bits, name)? as u32);
    let varp = |name: &str| {
        defs.varps
            .get(&(symbol(varps, name)? as u32))
            .copied()
            .ok_or("Missing combat varp".to_owned())
    };
    let table = tables
        .get(&(symbol(enums, "weapon_last_stance_varbits")? as u32))
        .ok_or("Missing saved stance enum")?;
    let mut saved = BTreeMap::new();
    for (&category, value) in &table.values {
        let Value::Int(id) = value else {
            return Err("Noninteger saved stance varbit".into());
        };
        if *id != -1 {
            saved.insert(category, binding_id(*id as u32)?);
        }
    }
    let bulwark = symbol(categories, "dinhs_bulwark")?;
    let weapons = defs
        .items
        .iter()
        .map(|(&id, item)| {
            (
                id as i32,
                combat_stance::Weapon {
                    category: item.weapon_category,
                    bulwark: item.category == bulwark,
                },
            )
        })
        .collect();
    let mut buttons = BTreeMap::new();
    for i in 0..4 {
        buttons.insert(
            symbol(components, &format!("combat_interface:{i}_back"))? as u32,
            Some(i),
        );
    }
    buttons.insert(symbol(components, "combat_interface:com_32")? as u32, None);
    Ok(Config {
        presentation: Presentation {
            interface: u16::try_from(symbol(interfaces, "combat_interface")?)
                .map_err(|_| "Invalid combat interface")?,
            category: binding("combat_weapon_category")?,
            level: binding("combatlevel_transmit")?,
            decimal: binding("combatlevel_decimal_transmit")?,
        },
        stance: combat_stance::Config {
            stance: varp("com_mode")?,
            melee: varp("com_stance")?,
            retaliate: varp("option_nodef")?,
            saved,
            styles: crate::bonuses::Styles::resolve(enums, tables)?.packed,
            weapons,
        },
        buttons,
    })
}
