//! Standard gameframe content rows; redirects are loaded from the same cache enums.
use gielinor_cache::{
    enums::{EnumType, Value},
    game::Definitions,
    symbols::Symbols,
};
use gielinor_world::{
    bank_actions::Binding,
    gameframe::{Frame, Frames},
    ui::Component,
};
use std::collections::BTreeMap;
pub fn standard(
    interfaces: &Symbols,
    components: &Symbols,
    enums: &Symbols,
    bits: &Symbols,
    tables: &BTreeMap<u32, EnumType>,
    defs: &Definitions,
) -> Result<Frames, String> {
    let symbol = |symbols: &Symbols, name: &str| {
        symbols
            .get(name)
            .copied()
            .ok_or_else(|| format!("Missing gameframe symbol {name}"))
    };
    let binding = |name: &str| -> Result<Binding, String> {
        let id = symbol(bits, name)? as u32;
        let bit = *defs.varbits.get(&id).ok_or("Missing gameframe varbit")?;
        let base = *defs
            .varps
            .get(&(bit.base as u32))
            .ok_or("Missing gameframe base varp")?;
        Ok(Binding { base, bit })
    };
    let mut overlays = Vec::new();
    for (interf, parent) in [
        ("chatbox", "chat_container"),
        ("buff_bar", "buff_bar"),
        ("stat_boosts_hud", "stat_boosts_hud"),
        ("pm_chat", "pm_container"),
        ("hpbar_hud", "hpbar_hud"),
        ("pvp_icons", "pvp_icons"),
        ("orbs", "orbs"),
        ("xp_drops", "xp_drops"),
        ("popout", "popout"),
        ("ehc_worldhop", "ehc_listener"),
        ("stats", "side1"),
        ("side_journal", "side2"),
        ("inventory", "side3"),
        ("wornitems", "side4"),
        ("prayerbook", "side5"),
        ("magic_spellbook", "side6"),
        ("friends", "side9"),
        ("account", "side8"),
        ("logout", "side10"),
        ("settings_side", "side11"),
        ("emote", "side12"),
        ("music", "side13"),
        ("side_channels", "side7"),
        ("combat_interface", "side0"),
    ] {
        let parent =
            Component(symbol(components, &format!("toplevel_osrs_stretch:{parent}"))? as u32);
        let id =
            u16::try_from(symbol(interfaces, interf)?).map_err(|_| "Invalid overlay interface")?;
        overlays.push((parent, id));
    }
    let mut ordered = Vec::new();
    for (mode, (name, redirect)) in [
        ("toplevel", "fixed_pane_redirect"),
        ("toplevel_osrs_stretch", "resizable_basic_pane_redirect"),
        ("toplevel_pre_eoc", "side_panels_resizable_pane_redirect"),
    ]
    .into_iter()
    .enumerate()
    {
        let id = symbol(enums, redirect)? as u32;
        let table = tables.get(&id).ok_or("Missing gameframe redirect enum")?;
        let mut mappings = BTreeMap::new();
        for (&key, value) in &table.values {
            let Value::Int(value) = value else {
                return Err("Noninteger component redirect".into());
            };
            if *value != -1 {
                mappings.insert(Component(key as u32), Component(*value as u32));
            }
        }
        ordered.push(Frame {
            top: u16::try_from(symbol(interfaces, name)?)
                .map_err(|_| "Invalid gameframe interface")?,
            overlays: overlays.clone(),
            mappings,
            client_mode: mode as i32,
            resizable: mode != 0,
            default: mode == 0,
            stones: mode == 2,
        });
    }
    let frames = Frames {
        ordered,
        top: binding("gameframe_toplevel")?,
        stones: binding("resizable_stone_arrangement")?,
    };
    frames.validate()?;
    Ok(frames)
}
