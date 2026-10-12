//! Cache-resolved generic held-item drop, destroy and release metadata.
use gielinor_cache::{
    definitions::Param, game::Definitions, parameters::ParameterType, symbols::Symbols,
};
use gielinor_world::{
    bank_actions::Binding,
    drop_actions::{Config, Kind, Rule},
    ui::Component,
};
use std::collections::BTreeMap;
#[derive(Clone, Copy)]
pub struct Presentation {
    pub parent: Component,
    pub unclamp: u32,
    pub object_interface: u16,
    pub object_button: Component,
    pub object_model: Component,
    pub object_text: Component,
    pub choice_interface: u16,
    pub choice_button: Component,
    pub destroy_interface: u16,
    pub destroy_button: Component,
}
pub fn resolve(
    params: &Symbols,
    varbits: &Symbols,
    synths: &Symbols,
    parameter_types: &BTreeMap<u32, ParameterType>,
    defs: &Definitions,
) -> Result<Config, String> {
    let symbol = |map: &Symbols, name: &str| {
        map.get(name)
            .copied()
            .ok_or_else(|| format!("Missing drop symbol {name}"))
    };
    let binding = |name: &str| -> Result<Binding, String> {
        let id = symbol(varbits, name)? as u32;
        let bit = *defs
            .varbits
            .get(&id)
            .ok_or_else(|| format!("Missing drop varbit {id}"))?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or_else(|| format!("Missing drop base {}", bit.base))?;
        Ok(Binding { base, bit })
    };
    let title = symbol(params, "destroy_note_title")? as u32;
    let desc = symbol(params, "destroy_note_desc")? as u32;
    let release_title = symbol(params, "release_note_title")? as u32;
    let release_message = symbol(params, "release_note_message")? as u32;
    let mut rules = BTreeMap::new();
    for (&id, ty) in &defs.items {
        let kind = match ty.inventory_ops[4].as_deref() {
            None | Some("Drop") => Kind::Drop,
            Some("Destroy") => Kind::Destroy,
            Some("Release") => Kind::Release,
            _ => continue,
        };
        let string = |key| -> Result<Option<String>, String> {
            match ty.params.get(&key) {
                Some(Param::String(v)) => Ok(Some(v.clone())),
                Some(_) => Err(format!("Invalid drop string parameter {key} on {id}")),
                None => Ok(parameter_types
                    .get(&key)
                    .and_then(|p| p.default_string.clone())),
            }
        };
        let market = if ty.cert.template != 0 {
            defs.items
                .get(&(ty.cert.link as u32))
                .ok_or_else(|| format!("Missing unnoted price for {id}"))?
        } else {
            ty
        };
        rules.insert(
            id as i32,
            Rule {
                kind,
                name: ty.name.clone(),
                market_price: market.cost,
                destroy_title: string(title)?.unwrap_or_default(),
                destroy_description: string(desc)?.unwrap_or_default(),
                release_title: string(release_title)?,
                release_message: string(release_message)?,
            },
        );
    }
    Ok(Config {
        rules,
        warning_enabled: binding("option_dropwarning_on")?,
        warning_threshold: binding("option_dropwarning_value")?,
        sound: u16::try_from(symbol(synths, "put_down")?).map_err(|_| "Invalid drop synth")?,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use gielinor_cache::definitions::ItemType;
    use gielinor_world::variables::{Transmit, Varbit, Varp};
    #[test]
    fn default_market_prices_follow_only_actual_note_templates() {
        let mut plain = ItemType::decode(&[0]).unwrap();
        plain.cost = 12345;
        let mut note = plain.clone();
        note.cost = 1;
        note.cert.link = 1;
        note.cert.template = 799;
        let mut custom = plain.clone();
        custom.inventory_ops[4] = Some("Uncharge".into());
        let mut destroy = plain.clone();
        destroy.inventory_ops[4] = Some("Destroy".into());
        let defs = Definitions {
            items: [(1, plain), (2, note), (3, custom), (4, destroy)].into(),
            inventories: BTreeMap::new(),
            locations: BTreeMap::new(),
            npcs: BTreeMap::new(),
            stats: BTreeMap::new(),
            varps: [(
                0,
                Varp {
                    id: 0,
                    permanent: true,
                    transmit: Transmit::Always,
                    bit_protect: false,
                    client_code: -1,
                },
            )]
            .into(),
            varbits: [
                (
                    0,
                    Varbit {
                        base: 0,
                        lsb: 0,
                        msb: 0,
                    },
                ),
                (
                    1,
                    Varbit {
                        base: 0,
                        lsb: 1,
                        msb: 20,
                    },
                ),
            ]
            .into(),
        };
        let params: Symbols = [
            ("destroy_note_title".into(), 0),
            ("destroy_note_desc".into(), 1),
            ("release_note_title".into(), 2),
            ("release_note_message".into(), 3),
        ]
        .into();
        let vars: Symbols = [
            ("option_dropwarning_on".into(), 0),
            ("option_dropwarning_value".into(), 1),
        ]
        .into();
        let synth: Symbols = [("put_down".into(), 2739)].into();
        let mut title = ParameterType::decode(&[0]).unwrap();
        title.default_string = Some("Are you sure?".into());
        let config = resolve(&params, &vars, &synth, &[(0, title)].into(), &defs).unwrap();
        assert_eq!(config.rules[&1].market_price, 12345);
        assert_eq!(config.rules[&2].market_price, 12345);
        assert!(!config.rules.contains_key(&3));
        assert_eq!(config.rules[&4].kind, Kind::Destroy);
        assert_eq!(config.rules[&4].destroy_title, "Are you sure?");
        assert_eq!(config.rules[&1].release_title, None);
    }
}
