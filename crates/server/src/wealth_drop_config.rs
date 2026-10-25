//! Revision239 wearable wealth variants. Loot context only; no charges,
//! autopickup, quest state or world-membership behavior is added.
use gielinor_cache::{game::Definitions, symbols::Symbols};
use gielinor_world::npc_loot::FINGER_SLOT;
use std::collections::BTreeSet;

pub const RINGS: &[(&str, i32, &str)] = &[
    ("ring_of_wealth", 2572, "Ring of wealth"),
    ("ring_of_wealth_i", 12785, "Ring of wealth (i)"),
    ("ring_of_wealth_5", 11980, "Ring of wealth (5)"),
    ("ring_of_wealth_4", 11982, "Ring of wealth (4)"),
    ("ring_of_wealth_3", 11984, "Ring of wealth (3)"),
    ("ring_of_wealth_2", 11986, "Ring of wealth (2)"),
    ("ring_of_wealth_1", 11988, "Ring of wealth (1)"),
    ("ring_of_wealth_i5", 20786, "Ring of wealth (i5)"),
    ("ring_of_wealth_i4", 20787, "Ring of wealth (i4)"),
    ("ring_of_wealth_i3", 20788, "Ring of wealth (i3)"),
    ("ring_of_wealth_i2", 20789, "Ring of wealth (i2)"),
    ("ring_of_wealth_i1", 20790, "Ring of wealth (i1)"),
];

pub fn resolve(objects: &Symbols, defs: &Definitions) -> Result<BTreeSet<i32>, String> {
    let mut rings = BTreeSet::new();
    for &(symbol, id, name) in RINGS {
        let item = defs.items.get(&(id as u32)).ok_or_else(|| format!("Missing wealth ring {symbol}"))?;
        if objects.get(symbol) != Some(&id) || item.name != name
            || item.wear != [FINGER_SLOT as i8, -1, -1]
            || item.stackable || item.dummy == 1 || item.cert.template != 0
            || item.placeholder.template != 0 || item.transform.template != 0
            || !rings.insert(id)
        {
            return Err(format!("Unattested wealth ring symbol/identity/finger slot: {symbol}"));
        }
    }
    Ok(rings)
}
