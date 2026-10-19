//! Ordinary spade Dig; actual quest/boss hook content registers independently.
use super::{Definitions, Symbols};
pub fn spade_tool(
    objects: &Symbols,
    sequences: &Symbols,
    sounds: &Symbols,
    defs: &Definitions,
) -> Result<gielinor_world::spade::Rule, String> {
    if objects.get("spade") != Some(&952)
        || sequences.get("human_dig") != Some(&830)
        || sounds.get("digspade").is_some_and(|&v| v != 1470)
        || defs.items.get(&952).is_none_or(|d| {
            d.name != "Spade"
                || d.members
                || d.stackable
                || d.dummy == 1
                || d.cert.template != 0
                || d.placeholder.template != 0
                || d.transform.template != 0
                || d.inventory_ops[0].as_deref() != Some("Dig")
                || d.inventory_ops[1..4].iter().any(Option::is_some)
        })
    {
        return Err("Unattested ordinary spade Dig".into());
    }
    // Prepared239 omits this alias. Source quest-pack gamevals explicitly maps
    // digspade1470; native asset presence/hash is pinned privately. No cache or
    // shared symbol is synthesized by this adapter.
    Ok(gielinor_world::spade::Rule {
        item: 952,
        animation: 830,
        sound: 1470,
        delay: 2,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spade_tool_exact_f2p_first_dig_and_scoped_missing_sound_alias_refuse_contradictions() {
        let o = [("spade".into(), 952)].into();
        let s = [("human_dig".into(), 830)].into();
        let mut sounds = Symbols::new();
        let mut d = Definitions {
            inventories: Default::default(),
            locations: Default::default(),
            npcs: Default::default(),
            items: Default::default(),
            varbits: Default::default(),
            varps: Default::default(),
            stats: Default::default(),
        };
        let mut t = gielinor_cache::definitions::ItemType::decode(&[0]).unwrap();
        t.name = "Spade".into();
        t.inventory_ops[0] = Some("Dig".into());
        d.items.insert(952, t);
        let r = spade_tool(&o, &s, &sounds, &d).unwrap();
        assert_eq!((r.item, r.animation, r.sound, r.delay), (952, 830, 1470, 2));
        sounds.insert("digspade".into(), 1470);
        assert!(spade_tool(&o, &s, &sounds, &d).is_ok());
        sounds.insert("digspade".into(), 1471);
        assert!(spade_tool(&o, &s, &sounds, &d).is_err());
        sounds.clear();
        d.items.get_mut(&952).unwrap().inventory_ops[0] = Some("Eat".into());
        assert!(spade_tool(&o, &s, &sounds, &d).is_err());
    }
}
